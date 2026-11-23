use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

struct ZipEntry {
    name: String,
    data: Vec<u8>,
}

pub struct FileZipper {
    output_file: PathBuf,
    temp_file: PathBuf,
    entries: Vec<ZipEntry>,
    closed: bool,
}

impl FileZipper {
    // JAVA: net.minecraft.util.FileZipper#<init>
    pub fn new(output_file: PathBuf) -> Self {
        let file_name = output_file
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let temp_file = output_file.with_file_name(format!("{}_tmp", file_name));
        Self {
            output_file,
            temp_file,
            entries: Vec::new(),
            closed: false,
        }
    }

    // JAVA: net.minecraft.util.FileZipper#add(Path,String)
    pub fn add_content(&mut self, path: &Path, content: &str) {
        self.entries.push(ZipEntry {
            name: path_to_zip_name(path),
            data: content.as_bytes().to_vec(),
        });
    }

    // JAVA: net.minecraft.util.FileZipper#add(Path,File)
    pub fn add_file(&mut self, path_in_zip: &Path, disk_file: &Path) -> io::Result<()> {
        let data = fs::read(disk_file)?;
        self.entries.push(ZipEntry {
            name: path_to_zip_name(path_in_zip),
            data,
        });
        Ok(())
    }

    // JAVA: net.minecraft.util.FileZipper#add(Path)
    pub fn add_path(&mut self, source: &Path) -> io::Result<()> {
        if source.is_file() {
            let parent = source.parent().unwrap_or_else(|| Path::new(""));
            let rel = source.strip_prefix(parent).unwrap_or(source);
            let data = fs::read(source)?;
            self.entries.push(ZipEntry {
                name: path_to_zip_name(rel),
                data,
            });
        } else if source.is_dir() {
            let mut files = walk_files(source)?;
            files.sort();
            for file in files {
                let rel = file.strip_prefix(source).unwrap_or(&file);
                let data = fs::read(&file)?;
                self.entries.push(ZipEntry {
                    name: path_to_zip_name(rel),
                    data,
                });
            }
        }
        Ok(())
    }

    // JAVA: net.minecraft.util.FileZipper#close
    pub fn close(&mut self) -> io::Result<()> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        let data = write_zip(&self.entries)?;
        if let Some(parent) = self.temp_file.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let mut f = fs::File::create(&self.temp_file)?;
        f.write_all(&data)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&self.temp_file, &self.output_file)?;
        eprintln!("Compressed to {}", self.output_file.display());
        Ok(())
    }
}

impl Drop for FileZipper {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn path_to_zip_name(path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for comp in path.components() {
        match comp {
            std::path::Component::Normal(s) => parts.push(s.to_string_lossy().to_string()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => parts.push("..".to_string()),
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {}
        }
    }
    parts.join("/")
}

fn walk_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                out.extend(walk_files(&p)?);
            } else if p.is_file() {
                out.push(p);
            }
        }
    }
    Ok(out)
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 { 0xEDB8_8320u32 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    crc ^ 0xFFFF_FFFF
}

fn write_u16_le(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_u32_le(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_zip(entries: &[ZipEntry]) -> io::Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    let mut offsets: Vec<u32> = Vec::with_capacity(entries.len());

    for e in entries {
        let name_bytes = e.name.as_bytes();
        let crc = crc32(&e.data);
        let size = e.data.len() as u32;
        let offset = out.len() as u32;
        offsets.push(offset);

        write_u32_le(&mut out, 0x0403_4B50);
        write_u16_le(&mut out, 20);
        write_u16_le(&mut out, 0);
        write_u16_le(&mut out, 0);
        write_u16_le(&mut out, 0);
        write_u16_le(&mut out, 0);
        write_u32_le(&mut out, crc);
        write_u32_le(&mut out, size);
        write_u32_le(&mut out, size);
        write_u16_le(&mut out, name_bytes.len() as u16);
        write_u16_le(&mut out, 0);
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(&e.data);
    }

    let cd_offset = out.len() as u32;
    let mut central: Vec<u8> = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        let name_bytes = e.name.as_bytes();
        let crc = crc32(&e.data);
        let size = e.data.len() as u32;

        write_u32_le(&mut central, 0x0201_4B50);
        write_u16_le(&mut central, 20);
        write_u16_le(&mut central, 20);
        write_u16_le(&mut central, 0);
        write_u16_le(&mut central, 0);
        write_u16_le(&mut central, 0);
        write_u16_le(&mut central, 0);
        write_u32_le(&mut central, crc);
        write_u32_le(&mut central, size);
        write_u32_le(&mut central, size);
        write_u16_le(&mut central, name_bytes.len() as u16);
        write_u16_le(&mut central, 0);
        write_u16_le(&mut central, 0);
        write_u16_le(&mut central, 0);
        write_u16_le(&mut central, 0);
        write_u32_le(&mut central, 0);
        write_u32_le(&mut central, offsets[i]);
        central.extend_from_slice(name_bytes);
    }

    let cd_size = central.len() as u32;
    out.extend_from_slice(&central);

    write_u32_le(&mut out, 0x0605_4B50);
    write_u16_le(&mut out, 0);
    write_u16_le(&mut out, 0);
    write_u16_le(&mut out, entries.len() as u16);
    write_u16_le(&mut out, entries.len() as u16);
    write_u32_le(&mut out, cd_size);
    write_u32_le(&mut out, cd_offset);
    write_u16_le(&mut out, 0);

    Ok(out)
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
            "mc_math_zipper_{}_{}_{}",
            name,
            std::process::id(),
            id
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn add_content_and_close_creates_zip() {
        let dir = temp_dir("content");
        let out = dir.join("test.zip");
        {
            let mut z = FileZipper::new(out.clone());
            z.add_content(Path::new("hello.txt"), "hello");
            z.close().unwrap();
        }
        let bytes = fs::read(&out).unwrap();
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
        assert!(bytes.windows(9).any(|w| w == b"hello.txt"));
        assert!(bytes.windows(5).any(|w| w == b"hello"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn add_file_copies_disk_file() {
        let dir = temp_dir("add_file");
        let src = dir.join("src.txt");
        fs::write(&src, "data123").unwrap();
        let out = dir.join("out.zip");
        {
            let mut z = FileZipper::new(out.clone());
            z.add_file(Path::new("inside/src.txt"), &src).unwrap();
            z.close().unwrap();
        }
        let bytes = fs::read(&out).unwrap();
        assert!(bytes.windows(7).any(|w| w == b"data123"));
        assert!(bytes.windows(14).any(|w| w == b"inside/src.txt"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn add_path_directory_walk() {
        let dir = temp_dir("walk");
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("a.txt"), "A").unwrap();
        fs::write(sub.join("b.txt"), "B").unwrap();
        let out = dir.join("walk.zip");
        {
            let mut z = FileZipper::new(out.clone());
            z.add_path(&sub).unwrap();
            z.close().unwrap();
        }
        let bytes = fs::read(&out).unwrap();
        assert!(bytes.windows(5).any(|w| w == b"a.txt"));
        assert!(bytes.windows(5).any(|w| w == b"b.txt"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn close_moves_temp_to_output() {
        let dir = temp_dir("move");
        let out = dir.join("final.zip");
        let mut z = FileZipper::new(out.clone());
        z.add_content(Path::new("x.txt"), "x");
        z.close().unwrap();
        assert!(out.exists());
        let temp = dir.join("final.zip_tmp");
        assert!(!temp.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_zip_is_valid_eocd() {
        let dir = temp_dir("empty");
        let out = dir.join("empty.zip");
        {
            let mut z = FileZipper::new(out.clone());
            z.close().unwrap();
        }
        let bytes = fs::read(&out).unwrap();
        assert!(bytes.windows(4).any(|w| w == [0x50, 0x4B, 0x05, 0x06]));
        let _ = fs::remove_dir_all(&dir);
    }
}
