use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashCode {
    bytes: Vec<u8>,
}

impl HashCode {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Display for HashCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.bytes {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

pub trait Hasher {
    fn update(&mut self, data: &[u8]);
    fn finish(self) -> HashCode;
}

pub trait HashFunction {
    type Hasher: Hasher;
    fn new_hasher(&self) -> Self::Hasher;
}

pub trait DownloadProgressListener {
    // JAVA: net.minecraft.util.HttpUtil$DownloadProgressListener#requestStart
    fn request_start(&mut self);
    // JAVA: net.minecraft.util.HttpUtil$DownloadProgressListener#downloadStart
    fn download_start(&mut self, size: Option<i64>);
    // JAVA: net.minecraft.util.HttpUtil$DownloadProgressListener#downloadedBytes
    fn downloaded_bytes(&mut self, bytes: i64);
    // JAVA: net.minecraft.util.HttpUtil$DownloadProgressListener#requestFinished
    fn request_finished(&mut self, success: bool);
}

pub struct HttpResponse {
    pub content_length: Option<i64>,
    pub body: Box<dyn Read>,
}

pub trait HttpConnector {
    fn open(&self, url: &str, headers: &HashMap<String, String>) -> io::Result<HttpResponse>;
}

#[derive(Debug)]
pub enum DownloadError {
    UncheckedIo(String, io::Error),
    IllegalState(String, Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadError::UncheckedIo(msg, e) => write!(f, "{}: {}", msg, e),
            DownloadError::IllegalState(msg, e) => write!(f, "{}: {}", msg, e),
        }
    }
}

impl std::error::Error for DownloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DownloadError::UncheckedIo(_, e) => Some(e),
            DownloadError::IllegalState(_, e) => Some(e.as_ref()),
        }
    }
}

fn optional_long_to_string(opt: Option<i64>) -> String {
    match opt {
        Some(v) => format!("OptionalLong[{}]", v),
        None => "OptionalLong.empty".to_string(),
    }
}

fn create_temp_file(dir: &Path, prefix: &str, suffix: &str) -> io::Result<PathBuf> {
    let pid = std::process::id();
    for i in 0u64..100_000u64 {
        let name = format!("{}{}-{}{}", prefix, pid, i, suffix);
        let p = dir.join(name);
        match fs::OpenOptions::new().write(true).create_new(true).open(&p) {
            Ok(_) => return Ok(p),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "failed to create temp file",
    ))
}

// JAVA: net.minecraft.util.HttpUtil#updateModificationTime
fn update_modification_time(path: &Path) {
    let result = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .and_then(|f| f.set_modified(SystemTime::now()));
    if result.is_err() {
        // LOGGER.warn("Failed to update modification time of {}", path, e);
    }
}

// JAVA: net.minecraft.util.HttpUtil#hashFile
fn hash_file<F: HashFunction + ?Sized>(path: &Path, hash_fn: &F) -> io::Result<HashCode> {
    let mut hasher = hash_fn.new_hasher();
    let mut file = File::open(path)?;
    let mut buf = [0u8; 8196];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finish())
}

// JAVA: net.minecraft.util.HttpUtil#checkExistingFile
fn check_existing_file<F: HashFunction + ?Sized>(
    path: &Path,
    hash_fn: &F,
    expected: &HashCode,
) -> io::Result<bool> {
    if path.exists() {
        let actual = hash_file(path, hash_fn)?;
        if &actual == expected {
            return Ok(true);
        }
        // LOGGER.warn("Mismatched hash of file {}, expected {} but found {}", path, expected, actual);
    }
    Ok(false)
}

// JAVA: net.minecraft.util.HttpUtil#cachedFilePath
fn cached_file_path(dir: &Path, hash: &HashCode) -> PathBuf {
    dir.join(hash.to_string())
}

// JAVA: net.minecraft.util.HttpUtil#downloadAndHash
fn download_and_hash<F, L>(
    hash_fn: &F,
    max_size: i32,
    listener: &mut L,
    input: &mut dyn Read,
    path: &Path,
) -> io::Result<HashCode>
where
    F: HashFunction + ?Sized,
    L: DownloadProgressListener + ?Sized,
{
    let mut output = File::create(path)?;
    let mut hasher = hash_fn.new_hasher();
    let mut buf = [0u8; 8196];
    let mut total: i64 = 0i64;
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let n_i32 = n as i32;
        total += n_i32 as i64;
        listener.downloaded_bytes(total);
        if total > max_size as i64 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Filesize was bigger than maximum allowed (got >= {}, limit was {})",
                    total, max_size
                ),
            ));
        }
        // Thread.interrupted() check omitted: no direct std equivalent.
        output.write_all(&buf[..n])?;
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finish())
}

// JAVA: net.minecraft.util.HttpUtil#downloadFile
pub fn download_file<C, F, L>(
    dir: &Path,
    url: &str,
    headers: &HashMap<String, String>,
    hash_fn: &F,
    expected_hash: Option<&HashCode>,
    max_size: i32,
    connector: &C,
    listener: &mut L,
) -> Result<PathBuf, DownloadError>
where
    C: HttpConnector + ?Sized,
    F: HashFunction + ?Sized,
    L: DownloadProgressListener + ?Sized,
{
    listener.request_start();

    let cached_path: Option<PathBuf> = if let Some(expected) = expected_hash {
        let p = cached_file_path(dir, expected);
        match check_existing_file(&p, hash_fn, expected) {
            Ok(true) => {
                // LOGGER.info("Returning cached file since actual hash matches requested");
                listener.request_finished(true);
                update_modification_time(&p);
                return Ok(p);
            }
            Ok(false) => {}
            Err(_e) => {
                // LOGGER.warn("Failed to check cached file {}", p, e);
            }
        }
        // LOGGER.warn("Existing file {} not found or had mismatched hash", p);
        match fs::remove_file(&p) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                listener.request_finished(false);
                return Err(DownloadError::UncheckedIo(
                    format!("Failed to remove existing file {}", p.display()),
                    e,
                ));
            }
        }
        Some(p)
    } else {
        None
    };

    let result: Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> = (|| {
        let mut response = connector.open(url, headers)?;
        let content_length = response.content_length;
        let size_opt: Option<i64> = match content_length {
            Some(-1i64) => None,
            Some(len) => Some(len),
            None => None,
        };
        fs::create_dir_all(dir)?;
        listener.download_start(size_opt);
        if let Some(size) = size_opt {
            if size > max_size as i64 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Filesize is bigger than maximum allowed (file is {}, limit is {})",
                        optional_long_to_string(size_opt),
                        max_size
                    ),
                )
                .into());
            }
        }

        if let Some(ref cp) = cached_path {
            let actual = download_and_hash(hash_fn, max_size, listener, &mut *response.body, cp)?;
            let expected = expected_hash.unwrap();
            if &actual != expected {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Hash of downloaded file ({}) did not match requested ({})",
                        actual, expected
                    ),
                )
                .into());
            }
            listener.request_finished(true);
            Ok(cp.clone())
        } else {
            let tmp = create_temp_file(dir, "download", ".tmp")?;
            let res: Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> = (|| {
                let actual =
                    download_and_hash(hash_fn, max_size, listener, &mut *response.body, &tmp)?;
                let target = cached_file_path(dir, &actual);
                let exists = check_existing_file(&target, hash_fn, &actual)?;
                if !exists {
                    fs::rename(&tmp, &target).or_else(|_| {
                        fs::copy(&tmp, &target).and_then(|_| fs::remove_file(&tmp))
                    })?;
                } else {
                    update_modification_time(&target);
                }
                listener.request_finished(true);
                Ok(target)
            })();
            let _ = fs::remove_file(&tmp);
            res
        }
    })();

    match result {
        Ok(p) => Ok(p),
        Err(e) => {
            // LOGGER.error("HTTP response error..." / "Failed to read response from server")
            listener.request_finished(false);
            Err(DownloadError::IllegalState(
                format!("Failed to download file {}", url),
                e,
            ))
        }
    }
}

// JAVA: net.minecraft.util.HttpUtil#getAvailablePort
pub fn get_available_port() -> i32 {
    match TcpListener::bind("0.0.0.0:0") {
        Ok(listener) => match listener.local_addr() {
            Ok(addr) => addr.port() as i32,
            Err(_) => 25564,
        },
        Err(_) => 25564,
    }
}

// JAVA: net.minecraft.util.HttpUtil#isPortAvailable
pub fn is_port_available(port: i32) -> bool {
    if !(0..=65535).contains(&port) {
        return false;
    }
    match TcpListener::bind(("0.0.0.0", port as u16)) {
        Ok(listener) => match listener.local_addr() {
            Ok(addr) => addr.port() as i32 == port,
            Err(_) => false,
        },
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io;
    use std::path::Path;

    #[derive(Clone)]
    struct MockHashFn;

    struct MockHasher(Vec<u8>);

    impl Hasher for MockHasher {
        fn update(&mut self, data: &[u8]) {
            self.0.extend_from_slice(data);
        }

        fn finish(self) -> HashCode {
            let mut hash = [0u8; 4];
            for (i, b) in self.0.iter().enumerate() {
                hash[i % 4] ^= *b;
            }
            HashCode::new(hash.to_vec())
        }
    }

    impl HashFunction for MockHashFn {
        type Hasher = MockHasher;
        fn new_hasher(&self) -> Self::Hasher {
            MockHasher(Vec::new())
        }
    }

    struct MockConnector {
        data: Vec<u8>,
        content_length: Option<i64>,
    }

    impl HttpConnector for MockConnector {
        fn open(&self, _url: &str, _headers: &HashMap<String, String>) -> io::Result<HttpResponse> {
            Ok(HttpResponse {
                content_length: self.content_length,
                body: Box::new(io::Cursor::new(self.data.clone())),
            })
        }
    }

    #[derive(Default)]
    struct MockListener {
        starts: i32,
        download_starts: Vec<Option<i64>>,
        downloaded: Vec<i64>,
        finishes: Vec<bool>,
    }

    impl DownloadProgressListener for MockListener {
        fn request_start(&mut self) {
            self.starts += 1;
        }

        fn download_start(&mut self, size: Option<i64>) {
            self.download_starts.push(size);
        }

        fn downloaded_bytes(&mut self, bytes: i64) {
            self.downloaded.push(bytes);
        }

        fn request_finished(&mut self, success: bool) {
            self.finishes.push(success);
        }
    }

    fn test_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "mc_http_{}_{}_{}",
            name,
            std::process::id(),
            nanos
        ));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn test_is_port_available_negative_and_overflow() {
        assert!(!is_port_available(-1));
        assert!(!is_port_available(-100));
        assert!(!is_port_available(65536));
        assert!(!is_port_available(i32::MAX));
    }

    #[test]
    fn test_get_available_port_valid_or_fallback() {
        let port = get_available_port();
        assert!(port == 25564 || (port > 0 && port <= 65535));
    }

    #[test]
    fn test_hash_code_display_and_cached_path() {
        let hash = HashCode::new(vec![0x0a, 0xff, 0x00, 0x1b]);
        assert_eq!(hash.to_string(), "0aff001b");
        let dir = Path::new("/tmp/cache");
        let p = cached_file_path(dir, &hash);
        assert_eq!(p, dir.join("0aff001b"));
    }

    #[test]
    fn test_download_file_success_no_expected_hash() {
        let dir = test_dir("success");
        let connector = MockConnector {
            data: b"hello world".to_vec(),
            content_length: Some(11),
        };
        let hash_fn = MockHashFn;
        let mut listener = MockListener::default();
        let headers = HashMap::new();
        let result = download_file(
            &dir,
            "http://example.com/file",
            &headers,
            &hash_fn,
            None,
            1024,
            &connector,
            &mut listener,
        );
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
        let content = fs::read(&path).unwrap();
        assert_eq!(content, b"hello world");
        assert_eq!(listener.starts, 1);
        assert_eq!(listener.finishes, vec![true]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_download_file_size_limit_and_hash_mismatch() {
        let dir = test_dir("limits");
        let headers = HashMap::new();
        let hash_fn = MockHashFn;

        // Content-length exceeds max_size before download
        let connector = MockConnector {
            data: b"hello world".to_vec(),
            content_length: Some(11),
        };
        let mut listener = MockListener::default();
        let result = download_file(
            &dir,
            "http://example.com/file",
            &headers,
            &hash_fn,
            None,
            5,
            &connector,
            &mut listener,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            DownloadError::IllegalState(msg, _) => {
                assert!(msg.contains("Failed to download file"))
            }
            other => panic!("unexpected error: {:?}", other),
        }
        assert_eq!(listener.finishes, vec![false]);

        // Hash mismatch when expected hash is provided
        let connector2 = MockConnector {
            data: b"wrong content".to_vec(),
            content_length: Some(13),
        };
        let mut listener2 = MockListener::default();
        let expected = HashCode::new(vec![0, 0, 0, 0]);
        let result2 = download_file(
            &dir,
            "http://example.com/file2",
            &headers,
            &hash_fn,
            Some(&expected),
            1024,
            &connector2,
            &mut listener2,
        );
        assert!(result2.is_err());
        assert_eq!(listener2.finishes, vec![false]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_download_file_cached_and_stream_limit() {
        let dir = test_dir("cache_stream");
        let headers = HashMap::new();
        let hash_fn = MockHashFn;

        // Cached hit
        let data = b"cached content".to_vec();
        let mut hasher = hash_fn.new_hasher();
        hasher.update(&data);
        let expected = hasher.finish();
        let cached = cached_file_path(&dir, &expected);
        fs::write(&cached, &data).unwrap();

        let connector = MockConnector {
            data: b"should not be used".to_vec(),
            content_length: Some(18),
        };
        let mut listener = MockListener::default();
        let result = download_file(
            &dir,
            "http://example.com/file",
            &headers,
            &hash_fn,
            Some(&expected),
            1024,
            &connector,
            &mut listener,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), cached);
        assert_eq!(listener.starts, 1);
        assert_eq!(listener.finishes, vec![true]);
        assert!(listener.download_starts.is_empty());

        // Stream size limit exceeded during download (unknown content length)
        let dir2 = test_dir("stream_limit");
        let connector2 = MockConnector {
            data: vec![1u8; 100],
            content_length: None,
        };
        let mut listener2 = MockListener::default();
        let result2 = download_file(
            &dir2,
            "http://example.com/file",
            &headers,
            &hash_fn,
            None,
            50,
            &connector2,
            &mut listener2,
        );
        assert!(result2.is_err());
        assert_eq!(listener2.finishes, vec![false]);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&dir2);
    }
}
