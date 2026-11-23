use std::io;
use std::path::PathBuf;

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn is_windows_drive(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

fn strip_file_scheme(rest_after_file_colon: &str) -> String {
    let p = if let Some(s) = rest_after_file_colon.strip_prefix("//") {
        if let Some(s2) = s.strip_prefix('/') {
            // file:///path
            s2.to_string()
        } else {
            // file://host/path
            match s.find('/') {
                Some(idx) => s[idx + 1..].to_string(),
                None => String::new(),
            }
        }
    } else {
        rest_after_file_colon.to_string()
    };

    let bytes = p.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic() && bytes[2] == b':' {
        return p[1..].to_string();
    }
    if is_windows_drive(&p) {
        return p;
    }
    if p.starts_with('/') {
        p
    } else if p.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", p)
    }
}

fn try_get_path(uri: &str) -> io::Result<PathBuf> {
    if let Some(rest) = uri.strip_prefix("file:") {
        let path = strip_file_scheme(rest);
        let decoded = percent_decode(&path);
        return Ok(PathBuf::from(decoded));
    }

    // Non-file URI schemes (e.g. jar:) cannot be resolved by std path APIs.
    // Map to NotFound to mirror Java's FileSystemNotFoundException path.
    if uri.contains(':') && !is_windows_drive(uri) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "filesystem not found for URI scheme",
        ));
    }

    Ok(PathBuf::from(uri))
}

// JAVA: net.minecraft.util.FileSystemUtil#safeGetPath
pub fn safe_get_path(uri: &str) -> io::Result<PathBuf> {
    match try_get_path(uri) {
        Ok(p) => return Ok(p),
        Err(e) => {
            // FileSystemNotFoundException is ignored; other throwables are logged.
            if e.kind() != io::ErrorKind::NotFound {
                eprintln!("Unable to get path for: {}", uri);
            }
        }
    }

    // Java: FileSystems.newFileSystem(uri, Collections.emptyMap())
    // std has no jar/zip filesystem mounting; no-op in this self-contained port.

    try_get_path(uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uri_decodes_unix_path() {
        let p = safe_get_path("file:///tmp/test.txt").unwrap();
        let s = p.to_string_lossy().replace('\\', "/");
        assert!(s.ends_with("/tmp/test.txt") || s.ends_with("tmp/test.txt"));
    }

    #[test]
    fn file_uri_percent_decodes() {
        let p = safe_get_path("file:///tmp/my%20file.txt").unwrap();
        let s = p.to_string_lossy();
        assert!(s.contains("my file.txt"));
    }

    #[test]
    fn plain_path_fallback() {
        let p = safe_get_path("/tmp/plain.txt").unwrap();
        assert_eq!(p, PathBuf::from("/tmp/plain.txt"));
    }

    #[test]
    fn windows_drive_file_uri() {
        let p = safe_get_path("file:///C:/Users/test/file.txt").unwrap();
        let s = p.to_string_lossy().replace('\\', "/");
        assert!(s.contains("C:/Users/test/file.txt"));
    }

    #[test]
    fn unsupported_scheme_returns_error() {
        let err = safe_get_path("jar:file:///tmp/test.jar!/entry").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
