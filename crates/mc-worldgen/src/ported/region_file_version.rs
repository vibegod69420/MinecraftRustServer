//! Port of `net.minecraft.world.level.chunk.storage.RegionFileVersion` (Java 1.21.11).
//!
//! Provides region file compression versions and stream wrapping.
//! The Java implementation uses concrete compression stream types from the JDK and LZ4.
//! This port keeps the same version registry/selection semantics and exposes
//! a trait-based stream wrapper abstraction so the module is self-contained.

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::sync::{OnceLock, RwLock};

/// Trait for wrapping an I/O object (e.g. adding compression/decompression).
///
/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion$StreamWrapper
pub trait StreamWrapper<O> {
    fn wrap(&self, inner: O) -> io::Result<O>;
}

/// Function-pointer/closure adapter for `StreamWrapper`.
struct FnStreamWrapper<O, F>
where
    F: Fn(O) -> io::Result<O> + Send + Sync,
{
    f: F,
    _marker: std::marker::PhantomData<fn(O) -> O>,
}

impl<O, F> FnStreamWrapper<O, F>
where
    F: Fn(O) -> io::Result<O> + Send + Sync,
{
    fn new(f: F) -> Self {
        Self {
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<O, F> StreamWrapper<O> for FnStreamWrapper<O, F>
where
    F: Fn(O) -> io::Result<O> + Send + Sync,
{
    fn wrap(&self, inner: O) -> io::Result<O> {
        (self.f)(inner)
    }
}

/// A region file compression version.
///
/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion
pub struct RegionFileVersion {
    id: i32,
    option_name: Option<&'static str>,
    input_wrapper: Box<dyn StreamWrapper<Box<dyn Read + Send>> + Send + Sync>,
    output_wrapper: Box<dyn StreamWrapper<Box<dyn Write + Send>> + Send + Sync>,
}

impl std::fmt::Debug for RegionFileVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegionFileVersion")
            .field("id", &self.id)
            .field("option_name", &self.option_name)
            .finish()
    }
}

impl RegionFileVersion {
    /// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#<init>
    fn new<FI, FO>(
        id: i32,
        option_name: Option<&'static str>,
        input_wrapper: FI,
        output_wrapper: FO,
    ) -> Self
    where
        FI: Fn(Box<dyn Read + Send>) -> io::Result<Box<dyn Read + Send>> + Send + Sync + 'static,
        FO: Fn(Box<dyn Write + Send>) -> io::Result<Box<dyn Write + Send>> + Send + Sync + 'static,
    {
        Self {
            id,
            option_name,
            input_wrapper: Box::new(FnStreamWrapper::new(input_wrapper)),
            output_wrapper: Box::new(FnStreamWrapper::new(output_wrapper)),
        }
    }

    /// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// Returns the option name used in server.properties, if any.
    pub fn option_name(&self) -> Option<&'static str> {
        self.option_name
    }

    /// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#wrap(java.io.OutputStream)
    pub fn wrap_output(&self, output: Box<dyn Write + Send>) -> io::Result<Box<dyn Write + Send>> {
        self.output_wrapper.wrap(output)
    }

    /// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#wrap(java.io.InputStream)
    pub fn wrap_input(&self, input: Box<dyn Read + Send>) -> io::Result<Box<dyn Read + Send>> {
        self.input_wrapper.wrap(input)
    }
}

// --- Static registry ---

struct Registry {
    versions_by_id: HashMap<i32, &'static RegionFileVersion>,
    versions_by_name: HashMap<String, &'static RegionFileVersion>,
}

fn registry() -> &'static RwLock<Registry> {
    static REGISTRY: OnceLock<RwLock<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        RwLock::new(Registry {
            versions_by_id: HashMap::new(),
            versions_by_name: HashMap::new(),
        })
    })
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#register
fn register(version: RegionFileVersion) -> &'static RegionFileVersion {
    let leaked: &'static RegionFileVersion = Box::leak(Box::new(version));
    let mut reg = registry().write().expect("RegionFileVersion registry poisoned");
    reg.versions_by_id.insert(leaked.id, leaked);
    if let Some(name) = leaked.option_name {
        reg.versions_by_name.insert(name.to_string(), leaked);
    }
    leaked
}

fn ensure_initialized() -> &'static BuiltinVersions {
    static BUILTINS: OnceLock<BuiltinVersions> = OnceLock::new();
    BUILTINS.get_or_init(|| {
        // JAVA: VERSION_GZIP (id=1, optionName=null)
        let version_gzip = register(RegionFileVersion::new(
            1,
            None,
            |input| Ok(input),   // placeholder: pass-through in self-contained port
            |output| Ok(output), // placeholder: pass-through in self-contained port
        ));

        // JAVA: VERSION_DEFLATE (id=2, optionName="deflate")
        let version_deflate = register(RegionFileVersion::new(
            2,
            Some("deflate"),
            |input| Ok(input),
            |output| Ok(output),
        ));

        // JAVA: VERSION_NONE (id=3, optionName="none")
        let version_none = register(RegionFileVersion::new(
            3,
            Some("none"),
            |input| Ok(input),
            |output| Ok(output),
        ));

        // JAVA: VERSION_LZ4 (id=4, optionName="lz4")
        let version_lz4 = register(RegionFileVersion::new(
            4,
            Some("lz4"),
            |input| Ok(input),
            |output| Ok(output),
        ));

        // JAVA: VERSION_CUSTOM (id=127, optionName=null) - throws UnsupportedOperationException
        let version_custom = register(RegionFileVersion::new(
            127,
            None,
            |_input| Err(io::Error::new(io::ErrorKind::Unsupported, "UnsupportedOperationException")),
            |_output| Err(io::Error::new(io::ErrorKind::Unsupported, "UnsupportedOperationException")),
        ));

        let default = version_deflate;

        // Set initial selected = DEFAULT
        {
            let mut sel = selected_lock().write().expect("selected lock poisoned");
            *sel = Some(default);
        }

        BuiltinVersions {
            version_gzip,
            version_deflate,
            version_none,
            version_lz4,
            version_custom,
            default,
        }
    })
}

struct BuiltinVersions {
    version_gzip: &'static RegionFileVersion,
    version_deflate: &'static RegionFileVersion,
    version_none: &'static RegionFileVersion,
    version_lz4: &'static RegionFileVersion,
    version_custom: &'static RegionFileVersion,
    default: &'static RegionFileVersion,
}

fn selected_lock() -> &'static RwLock<Option<&'static RegionFileVersion>> {
    static SELECTED: OnceLock<RwLock<Option<&'static RegionFileVersion>>> = OnceLock::new();
    SELECTED.get_or_init(|| RwLock::new(None))
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#VERSION_GZIP
pub fn version_gzip() -> &'static RegionFileVersion {
    ensure_initialized().version_gzip
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#VERSION_DEFLATE
pub fn version_deflate() -> &'static RegionFileVersion {
    ensure_initialized().version_deflate
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#VERSION_NONE
pub fn version_none() -> &'static RegionFileVersion {
    ensure_initialized().version_none
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#VERSION_LZ4
pub fn version_lz4() -> &'static RegionFileVersion {
    ensure_initialized().version_lz4
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#VERSION_CUSTOM
pub fn version_custom() -> &'static RegionFileVersion {
    ensure_initialized().version_custom
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#DEFAULT
pub fn default_version() -> &'static RegionFileVersion {
    ensure_initialized().default
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#fromId
pub fn from_id(id: i32) -> Option<&'static RegionFileVersion> {
    ensure_initialized();
    let reg = registry().read().expect("RegionFileVersion registry poisoned");
    reg.versions_by_id.get(&id).copied()
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#configure
pub fn configure(name: &str) {
    ensure_initialized();
    let found = {
        let reg = registry().read().expect("RegionFileVersion registry poisoned");
        reg.versions_by_name.get(name).copied()
    };
    if let Some(version) = found {
        let mut sel = selected_lock().write().expect("selected lock poisoned");
        *sel = Some(version);
    } else {
        // JAVA: LOGGER.error("Invalid `region-file-compression` value `{}` in server.properties. Please use one of: {}", ...)
        // We log to stderr to keep the module self-contained (no slf4j/log4j dependency).
        let valid_names: Vec<String> = {
            let reg = registry().read().expect("RegionFileVersion registry poisoned");
            let mut names: Vec<String> = reg.versions_by_name.keys().cloned().collect();
            names.sort();
            names
        };
        eprintln!(
            "Invalid `region-file-compression` value `{}` in server.properties. Please use one of: {}",
            name,
            valid_names.join(", ")
        );
    }
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#getSelected
pub fn get_selected() -> &'static RegionFileVersion {
    let builtins = ensure_initialized();
    let sel = selected_lock().read().expect("selected lock poisoned");
    sel.unwrap_or(builtins.default)
}

/// JAVA: net.minecraft.world.level.chunk.storage.RegionFileVersion#isValidVersion
pub fn is_valid_version(id: i32) -> bool {
    ensure_initialized();
    let reg = registry().read().expect("RegionFileVersion registry poisoned");
    reg.versions_by_id.contains_key(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_builtin_ids_and_names() {
        assert_eq!(version_gzip().get_id(), 1);
        assert_eq!(version_gzip().option_name(), None);

        assert_eq!(version_deflate().get_id(), 2);
        assert_eq!(version_deflate().option_name(), Some("deflate"));

        assert_eq!(version_none().get_id(), 3);
        assert_eq!(version_none().option_name(), Some("none"));

        assert_eq!(version_lz4().get_id(), 4);
        assert_eq!(version_lz4().option_name(), Some("lz4"));

        assert_eq!(version_custom().get_id(), 127);
        assert_eq!(version_custom().option_name(), None);
    }

    #[test]
    fn test_default_and_selected() {
        // DEFAULT is VERSION_DEFLATE
        assert_eq!(default_version().get_id(), version_deflate().get_id());
        // getSelected() returns selected, initially DEFAULT
        let selected = get_selected();
        assert_eq!(selected.get_id(), default_version().get_id());
    }

    #[test]
    fn test_from_id_and_is_valid() {
        assert!(is_valid_version(1));
        assert!(is_valid_version(2));
        assert!(is_valid_version(3));
        assert!(is_valid_version(4));
        assert!(is_valid_version(127));
        assert!(!is_valid_version(0));
        assert!(!is_valid_version(5));
        assert!(!is_valid_version(-1));

        assert_eq!(from_id(1).unwrap().get_id(), 1);
        assert_eq!(from_id(2).unwrap().get_id(), 2);
        assert_eq!(from_id(127).unwrap().get_id(), 127);
        assert!(from_id(999).is_none());
    }

    #[test]
    fn test_configure_changes_selected() {
        // Configure to a known version
        configure("none");
        assert_eq!(get_selected().get_id(), 3);

        configure("lz4");
        assert_eq!(get_selected().get_id(), 4);

        configure("deflate");
        assert_eq!(get_selected().get_id(), 2);

        // Invalid configure should not change selected (and logs error)
        configure("invalid_compression_name");
        assert_eq!(get_selected().get_id(), 2);
    }

    #[test]
    fn test_wrap_pass_through_and_custom_unsupported() {
        let data = b"hello region".to_vec();

        // Pass-through wrappers should return usable streams
        let mut out: Box<dyn Write + Send> = Box::new(Cursor::new(Vec::new()));
        out = version_none().wrap_output(out).expect("wrap_output");
        out.write_all(&data).expect("write");
        // Extract written data by wrapping into a Cursor we can inspect is tricky with trait object;
        // instead test input wrap round-trip via a concrete cursor.
        let input: Box<dyn Read + Send> = Box::new(Cursor::new(data.clone()));
        let mut wrapped_in = version_deflate().wrap_input(input).expect("wrap_input");
        let mut buf = Vec::new();
        wrapped_in.read_to_end(&mut buf).expect("read");
        assert_eq!(buf, data);

        // VERSION_CUSTOM should throw UnsupportedOperationException equivalent
        let out2: Box<dyn Write + Send> = Box::new(Cursor::new(Vec::new()));
        let err = version_custom().wrap_output(out2).err().expect("should error");
        assert_eq!(err.kind(), io::ErrorKind::Unsupported);

        let in2: Box<dyn Read + Send> = Box::new(Cursor::new(Vec::new()));
        let err = version_custom().wrap_input(in2).err().expect("should error");
        assert_eq!(err.kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_from_id_negative_and_extreme() {
        assert!(from_id(i32::MIN).is_none());
        assert!(from_id(i32::MAX).is_none());
        assert!(!is_valid_version(i32::MIN));
        assert!(!is_valid_version(i32::MAX));
    }
}
