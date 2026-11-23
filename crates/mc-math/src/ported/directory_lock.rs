use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const LOCK_FILE: &str = "session.lock";

const DUMMY: &[u8] = "☃".as_bytes();

#[derive(Debug)]
pub struct DirectoryLock {
    lock_file: Option<File>,
    locked: bool,
}

impl DirectoryLock {
    // JAVA: net.minecraft.util.DirectoryLock#create
    pub fn create(path: &Path) -> io::Result<Self> {
        let lock_path = path.join(LOCK_FILE);
        fs::create_dir_all(path)?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)?;

        let res = (|| -> io::Result<()> {
            file.write_all(DUMMY)?;
            file.sync_all()?;
            match file.try_lock() {
                Ok(()) => Ok(()),
                Err(std::fs::TryLockError::WouldBlock) => {
                    Err(LockException::already_locked(&lock_path).into())
                }
                Err(std::fs::TryLockError::Error(e)) => Err(e),
            }
        })();

        match res {
            Ok(()) => Ok(Self {
                lock_file: Some(file),
                locked: true,
            }),
            Err(e) => {
                drop(file);
                Err(e)
            }
        }
    }

    // JAVA: net.minecraft.util.DirectoryLock#close
    pub fn close(&mut self) -> io::Result<()> {
        let mut release_err: Option<io::Error> = None;
        if self.locked {
            if let Some(f) = self.lock_file.as_ref() {
                if let Err(e) = f.unlock() {
                    release_err = Some(e);
                }
            }
            self.locked = false;
        }

        // Close the file (drop). Java closes in finally; std File drop does not report errors.
        self.lock_file.take();

        match release_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    // JAVA: net.minecraft.util.DirectoryLock#isValid
    pub fn is_valid(&self) -> bool {
        self.locked && self.lock_file.is_some()
    }

    // JAVA: net.minecraft.util.DirectoryLock#isLocked
    pub fn is_locked(path: &Path) -> io::Result<bool> {
        let lock_path = path.join(LOCK_FILE);
        match OpenOptions::new().write(true).open(&lock_path) {
            Ok(file) => {
                let locked = match file.try_lock() {
                    Ok(()) => {
                        let _ = file.unlock();
                        false
                    }
                    Err(std::fs::TryLockError::WouldBlock) => true,
                    Err(std::fs::TryLockError::Error(e)) => return Err(e),
                };
                drop(file);
                Ok(locked)
            }
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
}

impl Drop for DirectoryLock {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[derive(Debug)]
pub struct LockException {
    path: PathBuf,
    message: String,
}

impl LockException {
    fn absolute_path_string(path: &Path) -> String {
        std::path::absolute(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .display()
            .to_string()
    }

    // JAVA: net.minecraft.util.DirectoryLock$LockException#<init>
    fn new(path: &Path, message: &str) -> Self {
        Self {
            path: path.to_path_buf(),
            message: message.to_string(),
        }
    }

    // JAVA: net.minecraft.util.DirectoryLock$LockException#alreadyLocked
    pub fn already_locked(path: &Path) -> Self {
        Self::new(
            path,
            "already locked (possibly by other Minecraft instance?)",
        )
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for LockException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            Self::absolute_path_string(&self.path),
            self.message
        )
    }
}

impl std::error::Error for LockException {}

impl From<LockException> for io::Error {
    fn from(e: LockException) -> Self {
        io::Error::other(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut p = std::env::temp_dir();
        p.push(format!(
            "mc_math_dirlock_{}_{}_{}",
            name,
            std::process::id(),
            id
        ));
        let _ = fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn create_writes_lock_file_and_is_valid() {
        let dir = temp_dir("create");
        let mut lock = DirectoryLock::create(&dir).expect("create should succeed");
        assert!(lock.is_valid());
        assert!(dir.join(LOCK_FILE).exists());

        let bytes = fs::read(dir.join(LOCK_FILE)).expect("lock file should be readable");
        assert_eq!(&bytes[..DUMMY.len()], DUMMY);

        lock.close().expect("close should succeed");
        assert!(!lock.is_valid());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn double_lock_fails_with_already_locked() {
        let dir = temp_dir("double");
        let _first = DirectoryLock::create(&dir).expect("first create should succeed");
        let err = DirectoryLock::create(&dir).expect_err("second lock should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("already locked"),
            "unexpected error message: {msg}"
        );
        drop(_first);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn is_locked_reflects_lock_state() {
        let dir = temp_dir("state");
        fs::create_dir_all(&dir).expect("create dir should succeed");
        assert!(!DirectoryLock::is_locked(&dir).expect("is_locked should succeed"));

        let mut lock = DirectoryLock::create(&dir).expect("create should succeed");
        assert!(DirectoryLock::is_locked(&dir).expect("is_locked should succeed"));
        assert!(lock.is_valid());

        lock.close().expect("close should succeed");
        assert!(!lock.is_valid());
        assert!(!DirectoryLock::is_locked(&dir).expect("is_locked should succeed"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn is_locked_nonexistent_returns_false() {
        let dir = temp_dir("nonexistent");
        let _ = fs::remove_dir_all(&dir);
        assert!(!DirectoryLock::is_locked(&dir).expect("is_locked should succeed"));
    }

    #[test]
    fn create_creates_parent_directories() {
        let base = temp_dir("parents");
        let dir = base.join("a").join("b");
        let _ = fs::remove_dir_all(&base);

        let lock = DirectoryLock::create(&dir).expect("create should succeed");
        assert!(dir.join(LOCK_FILE).exists());
        assert!(lock.is_valid());
        drop(lock);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn close_is_idempotent() {
        let dir = temp_dir("idempotent");
        let mut lock = DirectoryLock::create(&dir).expect("create should succeed");
        lock.close().expect("first close should succeed");
        lock.close().expect("second close should succeed");
        assert!(!lock.is_valid());
        let _ = fs::remove_dir_all(&dir);
    }
}
