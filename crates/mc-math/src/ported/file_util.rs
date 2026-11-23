use std::io;
use std::path::{Component, Path, PathBuf};

/// Result type matching Java's `com.mojang.serialization.DataResult` usage in this class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataResult<T> {
    Success(T),
    Error(String),
}

impl<T> DataResult<T> {
    pub fn success(value: T) -> Self {
        DataResult::Success(value)
    }

    pub fn error(message: String) -> Self {
        DataResult::Error(message)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, DataResult::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, DataResult::Error(_))
    }

    pub fn result(&self) -> Option<&T> {
        match self {
            DataResult::Success(v) => Some(v),
            DataResult::Error(_) => None,
        }
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            DataResult::Success(_) => None,
            DataResult::Error(msg) => Some(msg),
        }
    }
}

/// Minimal mirror of `net.minecraft.SharedConstants.ILLEGAL_FILE_CHARACTERS`.
/// Java 1.21.11 defines:
/// `ILLEGAL_FILE_CHARACTERS = new char[]{'/', '\n', '\r', '\t', '\0', '\f', '`', '?', '*', '\\', '<', '>', '|', '"', ':'}`
/// We keep it local to make the module self-contained.
pub const ILLEGAL_FILE_CHARACTERS: [char; 15] = [
    '/', '\n', '\r', '\t', '\0', '\u{000C}', '`', '?', '*', '\\', '<', '>', '|', '"', ':',
];

// JAVA: Pattern.compile("(<name>.*) \\((<count>\\d*)\\)", 66)
// Flags 66 = CASE_INSENSITIVE(2) | DOTALL(32)
const MAX_FILE_NAME: usize = 255;

// JAVA: Pattern.compile(".*\\.|(?:COM|CLOCK\\$|CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\\..*)?", 2)
// Flag 2 = CASE_INSENSITIVE
fn is_reserved_windows_filename(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with('.') {
        return true;
    }

    let (base, has_ext) = match lower.find('.') {
        Some(idx) => (&lower[..idx], true),
        None => (lower.as_str(), false),
    };

    let is_reserved_base = matches!(base, "com" | "clock$" | "con" | "prn" | "aux" | "nul")
        || (base.len() == 4 && {
            let bytes = base.as_bytes();
            (bytes.starts_with(b"com") || bytes.starts_with(b"lpt"))
                && (bytes[3] as char).is_ascii_digit()
                && bytes[3] != b'0'
        });

    if !is_reserved_base {
        return false;
    }

    // Pattern allows optional extension: (?:\..*)?
    // So reserved base alone or reserved base with any extension matches.
    let _ = has_ext;
    true
}

// JAVA: Pattern.compile("[-._a-z0-9]+")
fn is_strict_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.' || b == b'_')
}

fn parse_copy_counter(name: &str) -> Option<(String, i32)> {
    // JAVA: Pattern.compile("(<name>.*) \\((<count>\\d*)\\)", 66) where flags are CASE_INSENSITIVE|DOTALL.
    // DOTALL makes '.' match newlines; name is greedy in Java regex but the whole pattern must match.
    // We emulate the exact shape: ^(?s:(.*)) \((\d*)\)$ with ASCII case-insensitive irrelevant here.
    let suffix = " (";
    let close = ')';
    if !name.ends_with(close) {
        return None;
    }
    let open_idx = name.rfind(suffix)?;
    let count_str = &name[open_idx + 2..name.len() - 1];
    // \d* allows empty? Java group is \d*; Integer.parseInt("") would throw, but Java's matcher would still match.
    // In practice Minecraft names have digits; we mirror Java behavior: parseInt on the captured digits.
    if !count_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if count_str.is_empty() {
        // Java would match but Integer.parseInt("") throws NumberFormatException.
        // Keep behavior explicit: treat as no match to avoid panic, but this case is pathological.
        return None;
    }
    let count = count_str.parse::<i32>().ok()?;
    let base = name[..open_idx].to_string();
    Some((base, count))
}

// JAVA: net.minecraft.util.FileUtil#sanitizeName
pub fn sanitize_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    for ch in name.chars() {
        if ILLEGAL_FILE_CHARACTERS.contains(&ch) {
            result.push('_');
        } else {
            result.push(ch);
        }
    }
    // JAVA: replaceAll("[./\"]", "_")
    let mut out = String::with_capacity(result.len());
    for ch in result.chars() {
        if ch == '.' || ch == '/' || ch == '"' {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    out
}

// JAVA: net.minecraft.util.FileUtil#isPathPartPortable
pub fn is_path_part_portable(name: &str) -> bool {
    !is_reserved_windows_filename(name)
}

// JAVA: net.minecraft.util.FileUtil#findAvailableName
pub fn find_available_name(dir: &Path, name: &str, extension: &str) -> io::Result<String> {
    let mut base_name = sanitize_name(name);
    if !is_path_part_portable(&base_name) {
        base_name = format!("_{}_", base_name);
    }

    let mut count: i32 = 0;
    if let Some((parsed_name, parsed_count)) = parse_copy_counter(&base_name) {
        base_name = parsed_name;
        count = parsed_count;
    }

    let ext_len = extension.chars().count();
    if base_name.chars().count() > MAX_FILE_NAME.saturating_sub(ext_len) {
        let limit = MAX_FILE_NAME.saturating_sub(ext_len);
        base_name = base_name.chars().take(limit).collect();
    }

    loop {
        let mut candidate = base_name.clone();
        if count != 0 {
            let suffix = format!(" ({})", count);
            let suffix_len = suffix.chars().count();
            let limit = MAX_FILE_NAME.saturating_sub(suffix_len);
            if candidate.chars().count() > limit {
                candidate = candidate.chars().take(limit).collect();
            }
            candidate.push_str(&suffix);
        }

        candidate.push_str(extension);
        let candidate_path = dir.join(&candidate);

        match std::fs::create_dir(&candidate_path) {
            Ok(_) => {
                // JAVA: Files.deleteIfExists($$9); return $$0.relativize($$9).toString();
                let _ = std::fs::remove_dir(&candidate_path);
                // Java relativize returns a path string using the path separator of the filesystem in Path.toString(),
                // but the returned value is used as a name/path string; preserve the relative path as produced by Path.
                let rel = candidate_path
                    .strip_prefix(dir)
                    .unwrap_or(&candidate_path)
                    .to_string_lossy()
                    .into_owned();
                return Ok(rel);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                count = count.wrapping_add(1);
            }
            Err(e) => return Err(e),
        }
    }
}

// JAVA: net.minecraft.util.FileUtil#isPathNormalized
pub fn is_path_normalized(path: &Path) -> bool {
    // Java Path.normalize() removes redundant name elements; equals compares the path.
    // Rust Path components normalization is similar for lexical normalization; we emulate Java's lexical check.
    let normalized = normalize_lexical_path(path);
    normalized == path
}

fn normalize_lexical_path(path: &Path) -> PathBuf {
    let mut components: Vec<Component<'_>> = Vec::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                match components.last() {
                    Some(Component::Normal(_)) => {
                        components.pop();
                    }
                    Some(Component::ParentDir) | None => {
                        // Preserve leading .. like Java normalize behavior for relative paths.
                        if !path.is_absolute() {
                            components.push(comp);
                        }
                    }
                    Some(Component::RootDir) | Some(Component::Prefix(_)) => {
                        // Cannot go above root; Java normalize keeps root.
                    }
                    Some(Component::CurDir) => unreachable!(),
                }
            }
            _ => components.push(comp),
        }
    }
    let mut out = PathBuf::new();
    for c in components {
        out.push(c.as_os_str());
    }
    if out.as_os_str().is_empty() && !path.is_absolute() {
        // Java normalize("") behavior is path-dependent; empty relative path normalizes to empty.
        PathBuf::new()
    } else {
        out
    }
}

// JAVA: net.minecraft.util.FileUtil#isPathPortable
pub fn is_path_portable(path: &Path) -> bool {
    for component in path.components() {
        if let Component::Normal(os) = component {
            let s = os.to_string_lossy();
            if !is_path_part_portable(&s) {
                return false;
            }
        } else {
            // Java iterates over Path elements (name elements). Root/prefix are not name elements in the same way;
            // only normal segments matter. Non-normal segments like "." ".." should be checked as strings if iterated,
            // but Path iterator in Java yields name elements; CurDir/ParentDir are unusual in resolved paths.
            let s = component.as_os_str().to_string_lossy();
            if s == "." || s == ".." {
                if !is_path_part_portable(&s) {
                    return false;
                }
            }
        }
    }
    true
}

// JAVA: net.minecraft.util.FileUtil#createPathToResource
pub fn create_path_to_resource(dir: &Path, resource_name: &str, extension: &str) -> PathBuf {
    let full = format!("{}{}", resource_name, extension);
    let resource_path = PathBuf::from(&full);
    // JAVA: if ($$4.endsWith($$2)) throw new InvalidPathException($$3, "empty resource name");
    // Java Path.endsWith(String) checks path segment ending; for extension like ".json", a path ending with that
    // segment means the resource name part was empty in this construction.
    let ends_with_ext = resource_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n == extension)
        .unwrap_or(false)
        || full == extension;
    if ends_with_ext {
        // Java throws InvalidPathException. In Rust we panic with a clear message matching the intent;
        // callers in MC catch/handle at higher levels. Keep it explicit.
        panic!("InvalidPathException: {}: empty resource name", full);
    }
    dir.join(resource_path)
}

// JAVA: net.minecraft.util.FileUtil#getFullResourcePath
pub fn get_full_resource_path(path: &str) -> String {
    // Emulate Apache Commons IO FilenameUtils.getFullPath + replace(File.separator, "/")
    let normalized_seps = path.replace('\\', "/");
    match normalized_seps.rfind('/') {
        Some(idx) => normalized_seps[..=idx].to_string(),
        None => String::new(),
    }
}

// JAVA: net.minecraft.util.FileUtil#normalizeResourcePath
pub fn normalize_resource_path(path: &str) -> String {
    // Emulate Apache Commons IO FilenameUtils.normalize + replace(File.separator, "/")
    // FilenameUtils.normalize returns null for invalid paths; Java then would NPE on .replace, but MC usage expects valid paths.
    // We return a best-effort normalized string; for clearly invalid prefix traversal on absolute/relative, return empty-like behavior is avoided.
    normalize_filename_utils_like(path).unwrap_or_else(|| path.replace('\\', "/").replace("//", "/"))
}

fn normalize_filename_utils_like(path: &str) -> Option<String> {
    let sep = '/';
    let input = path.replace('\\', "/");
    if input.is_empty() {
        return Some(input);
    }
    let prefix_len = get_prefix_len(&input);
    if prefix_len < 0 {
        return None;
    }
    let mut out: Vec<char> = input.chars().collect();
    let size = out.len();
    let mut i = prefix_len as usize;
    // add trailing separator if needed? FilenameUtils does normalization and preserves semantics.
    // We'll use a segment stack approach on the remainder.
    let prefix: String = out[..i].iter().collect();
    let rest: String = out[i..].iter().collect();
    let mut stack: Vec<&str> = Vec::new();
    let is_absolute = prefix.starts_with('/') || prefix.contains(':');
    let parts: Vec<&str> = rest.split('/').collect();
    let had_trailing_sep = rest.ends_with('/') && rest.len() > 1;

    for part in parts {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if !stack.is_empty() && stack.last() != Some(&"..") {
                stack.pop();
            } else if !is_absolute {
                stack.push("..");
            } else {
                // trying to go above root
                return None;
            }
        } else {
            stack.push(part);
        }
    }

    let mut result = prefix;
    if !result.is_empty() && !result.ends_with('/') && !stack.is_empty() {
        // prefix like "C:" needs separator? FilenameUtils keeps prefix form.
        // For unix absolute "/", prefix already ends with '/'.
    }
    for (idx, part) in stack.iter().enumerate() {
        if idx > 0 || (result.ends_with('/') == false && !result.is_empty()) {
            if !result.is_empty() && !result.ends_with('/') {
                result.push(sep);
            }
        }
        if result.is_empty() && is_absolute && !result.starts_with('/') {
            // keep as-is
        }
        result.push_str(part);
    }
    if had_trailing_sep && !result.ends_with('/') {
        result.push(sep);
    }
    if result.is_empty() {
        // FilenameUtils.normalize can return null for some cases; but empty relative may become ""
        return Some(String::new());
    }
    Some(result.replace(FileUtilSeparator::SEP, "/"))
}

struct FileUtilSeparator;
impl FileUtilSeparator {
    const SEP: char = '/';
}

fn get_prefix_len(path: &str) -> i32 {
    let bytes = path.as_bytes();
    let len = bytes.len() as i32;
    if len == 0 {
        return 0;
    }
    let ch0 = bytes[0] as char;
    if ch0 == '/' {
        if len > 1 && bytes[1] as char == '/' {
            // UNC-like prefix; simplified handling
            let mut pos = 2;
            let mut seen = 0;
            while pos < bytes.len() {
                if bytes[pos] as char == '/' {
                    seen += 1;
                    pos += 1;
                    if seen == 2 {
                        return pos as i32;
                    }
                } else {
                    pos += 1;
                }
            }
            return len;
        }
        return 1;
    }
    if ch0.is_ascii_alphabetic() && len > 1 && bytes[1] as char == ':' {
        if len > 2 && bytes[2] as char == '/' {
            return 3;
        }
        return 2;
    }
    0
}

// JAVA: net.minecraft.util.FileUtil#decomposePath
pub fn decompose_path(path: &str) -> DataResult<Vec<String>> {
    let slash_idx = path.find('/');
    if slash_idx.is_none() {
        return match path {
            "" | "." | ".." => DataResult::error(format!("Invalid path '{}'", path)),
            _ => {
                if !contains_allowed_characters_only(path) {
                    DataResult::error(format!("Invalid path '{}'", path))
                } else {
                    DataResult::success(vec![path.to_string()])
                }
            }
        };
    }

    let mut result: Vec<String> = Vec::new();
    let mut start: usize = 0;
    let mut current_slash = slash_idx.unwrap();
    let mut last_segment = false;

    loop {
        let segment = &path[start..current_slash];
        match segment {
            "" | "." | ".." => {
                return DataResult::error(format!(
                    "Invalid segment '{}' in path '{}'",
                    segment, path
                ));
            }
            _ => {}
        }
        if !contains_allowed_characters_only(segment) {
            return DataResult::error(format!(
                "Invalid segment '{}' in path '{}'",
                segment, path
            ));
        }
        result.push(segment.to_string());
        if last_segment {
            return DataResult::success(result);
        }
        start = current_slash + 1;
        match path[start..].find('/') {
            Some(rel) => {
                current_slash = start + rel;
            }
            None => {
                current_slash = path.len();
                last_segment = true;
            }
        }
    }
}

// JAVA: net.minecraft.util.FileUtil#resolvePath
pub fn resolve_path(base: &Path, parts: &[String]) -> PathBuf {
    let size = parts.len();
    match size {
        0 => base.to_path_buf(),
        1 => base.join(&parts[0]),
        _ => {
            let mut rest: Vec<&str> = Vec::with_capacity(size - 1);
            for i in 1..size {
                rest.push(&parts[i]);
            }
            // JAVA: $$0.resolve($$0.getFileSystem().getPath($$1.get(0), $$3));
            let mut subpath = PathBuf::from(&parts[0]);
            for p in rest {
                subpath.push(p);
            }
            base.join(subpath)
        }
    }
}

// JAVA: net.minecraft.util.FileUtil#containsAllowedCharactersOnly
fn contains_allowed_characters_only(s: &str) -> bool {
    is_strict_path_segment(s)
}

// JAVA: net.minecraft.util.FileUtil#isValidPathSegment
pub fn is_valid_path_segment(segment: &str) -> bool {
    segment != ".." && segment != "." && contains_allowed_characters_only(segment)
}

// JAVA: net.minecraft.util.FileUtil#validatePath
pub fn validate_path(parts: &[&str]) {
    if parts.is_empty() {
        panic!("IllegalArgumentException: Path must have at least one element");
    }
    for part in parts {
        if !is_valid_path_segment(part) {
            panic!(
                "IllegalArgumentException: Illegal segment {} in path {:?}",
                part, parts
            );
        }
    }
}

// JAVA: net.minecraft.util.FileUtil#createDirectoriesSafe
pub fn create_directories_safe(path: &Path) -> io::Result<()> {
    // JAVA: Files.createDirectories(Files.exists($$0) ? $$0.toRealPath() : $$0);
    let target = if path.exists() {
        std::fs::canonicalize(path)?
    } else {
        path.to_path_buf()
    };
    std::fs::create_dir_all(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        p.push(format!("mc_file_util_{}_{}_{}", name, std::process::id(), nanos));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn sanitize_name_replaces_illegal_and_dot_slash_quote() {
        // JAVA: sanitizeName replaces ILLEGAL_FILE_CHARACTERS with '_' then replaces [./"] with '_'
        assert_eq!(sanitize_name("a/b\\c:d"), "a_b_c_d");
        assert_eq!(sanitize_name("name.with\"quotes"), "name_with_quotes");
        assert_eq!(sanitize_name("a\nb\tc\0d"), "a_b_c_d");
        assert_eq!(sanitize_name("safe-name_1"), "safe-name_1");
    }

    #[test]
    fn path_part_portable_rejects_reserved_windows_names() {
        // JAVA: RESERVED_WINDOWS_FILENAMES is case-insensitive and matches reserved device names
        assert!(!is_path_part_portable("CON"));
        assert!(!is_path_part_portable("con.txt"));
        assert!(!is_path_part_portable("LPT1"));
        assert!(!is_path_part_portable("com9.png"));
        assert!(!is_path_part_portable("bad."));
        assert!(is_path_part_portable("normal"));
        assert!(is_path_part_portable("CONSOLE"));
    }

    #[test]
    fn decompose_path_validates_segments() {
        // JAVA: decomposePath returns DataResult.error for invalid segments/paths
        assert_eq!(
            decompose_path("foo/bar"),
            DataResult::success(vec!["foo".to_string(), "bar".to_string()])
        );
        assert_eq!(
            decompose_path("single"),
            DataResult::success(vec!["single".to_string()])
        );
        assert!(decompose_path("").is_error());
        assert!(decompose_path(".").is_error());
        assert!(decompose_path("..").is_error());
        assert!(decompose_path("foo/../bar").is_error());
        assert!(decompose_path("foo//bar").is_error());
        assert!(decompose_path("foo/bar!").is_error());
    }

    #[test]
    fn resolve_and_validate_path_behave_like_java() {
        let base = PathBuf::from("/tmp/base");
        assert_eq!(resolve_path(&base, &[]), base);
        assert_eq!(
            resolve_path(&base, &["a".to_string()]),
            base.join("a")
        );
        assert_eq!(
            resolve_path(&base, &["a".to_string(), "b".to_string(), "c".to_string()]),
            base.join("a").join("b").join("c")
        );

        assert!(is_valid_path_segment("abc-._123"));
        assert!(!is_valid_path_segment("."));
        assert!(!is_valid_path_segment(".."));
        assert!(!is_valid_path_segment("bad/slash"));

        let ok = std::panic::catch_unwind(|| validate_path(&["a", "b_c-1"]));
        assert!(ok.is_ok());
        let err = std::panic::catch_unwind(|| validate_path(&["a", ".."]));
        assert!(err.is_err());
    }

    #[test]
    fn find_available_name_handles_conflicts_and_counter() {
        let dir = temp_dir("find_available");
        // Pre-create a conflicting directory to force counter increment.
        fs::create_dir(dir.join("test.json")).unwrap();

        let name = find_available_name(&dir, "test", ".json").unwrap();
        // JAVA: should return a relative name that was available; "test.json" exists so it becomes "test (1).json"
        assert_eq!(name, "test (1).json");
        assert!(!dir.join("test (1).json").exists());

        let name2 = find_available_name(&dir, "bad/name", ".txt").unwrap();
        assert!(is_path_part_portable(&sanitize_name("bad/name")));
        assert!(name2.ends_with(".txt"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn normalize_and_full_resource_path_replace_separators() {
        // JAVA: getFullResourcePath uses FilenameUtils.getFullPath and replaces File.separator with "/"
        assert_eq!(get_full_resource_path("foo/bar/baz.json"), "foo/bar/");
        assert_eq!(get_full_resource_path("baz.json"), "");
        // normalizeResourcePath uses FilenameUtils.normalize and replaces separators with "/"
        let normalized = normalize_resource_path("foo/bar/../baz.json");
        assert_eq!(normalized, "foo/baz.json");
    }
}
