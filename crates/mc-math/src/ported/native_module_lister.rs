use std::cmp::Ordering;
use std::fmt;

/// Minimal crash-report category abstraction for self-contained porting.
/// Mirrors the `setDetail(String, Supplier<String>)` shape used by Java.
pub struct CrashReportCategory {
    details: Vec<(String, String)>,
}

impl CrashReportCategory {
    pub fn new() -> Self {
        Self {
            details: Vec::new(),
        }
    }

    pub fn set_detail<F>(&mut self, key: &str, supplier: F)
    where
        F: FnOnce() -> String,
    {
        self.details.push((key.to_string(), supplier()));
    }

    pub fn get_detail(&self, key: &str) -> Option<&str> {
        self.details
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn details(&self) -> &[(String, String)] {
        &self.details
    }
}

impl Default for CrashReportCategory {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NativeModuleLister;

impl NativeModuleLister {
    const LANG_MASK: i32 = 65535;
    const DEFAULT_LANG: i32 = 1033;
    const CODEPAGE_MASK: i32 = -65536i32;
    const DEFAULT_CODEPAGE: i32 = 78643200;

    // JAVA: net.minecraft.util.NativeModuleLister#listModules
    pub fn list_modules() -> Vec<NativeModuleInfo> {
        // Platform-specific JNA/Win32 enumeration is not available in this
        // self-contained Rust port. Java returns an empty immutable list on
        // non-Windows; we preserve the same observable empty-list behavior.
        Vec::new()
    }

    // JAVA: net.minecraft.util.NativeModuleLister#tryGetVersion
    #[allow(dead_code)]
    fn try_get_version(_path: &str) -> Option<NativeModuleVersion> {
        // Java implementation depends on Windows/JNA APIs (Kernel32, Version).
        // Kept as a faithful structural stub for the non-Windows/self-contained case.
        None
    }

    // JAVA: net.minecraft.util.NativeModuleLister#langTableKey
    fn lang_table_key(key: &str, lang: i32, codepage: i32) -> String {
        format!("\\StringFileInfo\\{:04x}{:04x}\\{}", lang, codepage, key)
    }

    // JAVA: net.minecraft.util.NativeModuleLister#findLangAndCodepage
    fn find_lang_and_codepage(values: &[i32]) -> Option<i32> {
        let mut selected: Option<i32> = None;

        for &value in values {
            if (value & Self::CODEPAGE_MASK) == Self::DEFAULT_CODEPAGE
                && (value & Self::LANG_MASK) == Self::DEFAULT_LANG
            {
                return Some(value);
            }
            selected = Some(value);
        }

        selected
    }

    // JAVA: net.minecraft.util.NativeModuleLister#addCrashSection
    pub fn add_crash_section(category: &mut CrashReportCategory) {
        let detail = {
            let mut modules = Self::list_modules();
            modules.sort_by(|a, b| match a.name.cmp(&b.name) {
                Ordering::Equal => Ordering::Equal,
                other => other,
            });
            modules
                .iter()
                .map(|m| format!("\n\t\t{}", m))
                .collect::<Vec<String>>()
                .join("")
        };
        category.set_detail("Modules", || detail);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeModuleInfo {
    pub name: String,
    pub version: Option<NativeModuleVersion>,
}

impl NativeModuleInfo {
    // JAVA: net.minecraft.util.NativeModuleLister$NativeModuleInfo#<init>
    pub fn new(name: String, version: Option<NativeModuleVersion>) -> Self {
        Self { name, version }
    }
}

impl fmt::Display for NativeModuleInfo {
    // JAVA: net.minecraft.util.NativeModuleLister$NativeModuleInfo#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.version {
            Some(version) => write!(f, "{}:{}", self.name, version),
            None => write!(f, "{}", self.name),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeModuleVersion {
    pub description: String,
    pub version: String,
    pub company: String,
}

impl NativeModuleVersion {
    // JAVA: net.minecraft.util.NativeModuleLister$NativeModuleVersion#<init>
    pub fn new(description: String, version: String, company: String) -> Self {
        Self {
            description,
            version,
            company,
        }
    }
}

impl fmt::Display for NativeModuleVersion {
    // JAVA: net.minecraft.util.NativeModuleLister$NativeModuleVersion#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.description, self.version, self.company)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_table_key_formats_hex_like_java() {
        let key = NativeModuleLister::lang_table_key("FileDescription", 1033, 1200);
        assert_eq!(key, "\\StringFileInfo\\040904b0\\FileDescription");

        let key2 = NativeModuleLister::lang_table_key("CompanyName", 0, 0);
        assert_eq!(key2, "\\StringFileInfo\\00000000\\CompanyName");
    }

    #[test]
    fn find_lang_and_codepage_prefers_default_pair() {
        let default_pair = (NativeModuleLister::DEFAULT_CODEPAGE as u32
            | NativeModuleLister::DEFAULT_LANG as u32) as i32;
        let other = 0x00000409_i32;
        let selected = NativeModuleLister::find_lang_and_codepage(&[other, default_pair, 0x1234]);
        assert_eq!(selected, Some(default_pair));
    }

    #[test]
    fn find_lang_and_codepage_falls_back_to_last_seen() {
        let first = 0x00000409_i32;
        let second = 0x0000040c_i32;
        let selected = NativeModuleLister::find_lang_and_codepage(&[first, second]);
        assert_eq!(selected, Some(second));
        assert_eq!(NativeModuleLister::find_lang_and_codepage(&[]), None);
    }

    #[test]
    fn native_module_info_to_string_matches_java() {
        let info_no_ver = NativeModuleInfo::new("kernel32.dll".to_string(), None);
        assert_eq!(info_no_ver.to_string(), "kernel32.dll");

        let ver = NativeModuleVersion::new(
            "Windows Kernel".to_string(),
            "10.0.0.1".to_string(),
            "Microsoft".to_string(),
        );
        let info = NativeModuleInfo::new("kernel32.dll".to_string(), Some(ver.clone()));
        assert_eq!(info.to_string(), "kernel32.dll:Windows Kernel:10.0.0.1:Microsoft");
        assert_eq!(ver.to_string(), "Windows Kernel:10.0.0.1:Microsoft");
    }

    #[test]
    fn add_crash_section_sets_modules_detail() {
        let mut category = CrashReportCategory::new();
        NativeModuleLister::add_crash_section(&mut category);
        let detail = category.get_detail("Modules").expect("Modules detail should exist");
        // list_modules() is empty in this self-contained port, so detail is empty string.
        assert_eq!(detail, "");
    }

    #[test]
    fn list_modules_is_empty_in_self_contained_port() {
        let modules = NativeModuleLister::list_modules();
        assert!(modules.is_empty());
    }
}
