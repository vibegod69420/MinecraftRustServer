use std::marker::PhantomData;

/// Minimal placeholder for `net.minecraft.world.level.Level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Level;

/// Minimal placeholder for `net.minecraft.resources.ResourceKey<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey<T> {
    pub location: String,
    _marker: PhantomData<T>,
}

impl<T> ResourceKey<T> {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            _marker: PhantomData,
        }
    }
}

/// Port of `net.minecraft.world.level.chunk.storage.RegionStorageInfo`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionStorageInfo {
    pub level: String,
    pub dimension: ResourceKey<Level>,
    pub type_name: String,
}

impl RegionStorageInfo {
    pub fn new(level: impl Into<String>, dimension: ResourceKey<Level>, type_name: impl Into<String>) -> Self {
        Self {
            level: level.into(),
            dimension,
            type_name: type_name.into(),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.storage.RegionStorageInfo#withTypeSuffix
    pub fn with_type_suffix(&self, suffix: &str) -> Self {
        Self {
            level: self.level.clone(),
            dimension: self.dimension.clone(),
            type_name: format!("{}{}", self.type_name, suffix),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_info() -> RegionStorageInfo {
        RegionStorageInfo::new(
            "world",
            ResourceKey::<Level>::new("minecraft:overworld"),
            "region",
        )
    }

    #[test]
    fn with_type_suffix_appends_suffix() {
        let info = sample_info();
        let suffixed = info.with_type_suffix("_old");
        assert_eq!(suffixed.level, "world");
        assert_eq!(suffixed.dimension, ResourceKey::<Level>::new("minecraft:overworld"));
        assert_eq!(suffixed.type_name, "region_old");
    }

    #[test]
    fn with_type_suffix_empty_string_preserves_type() {
        let info = sample_info();
        let suffixed = info.with_type_suffix("");
        assert_eq!(suffixed.type_name, "region");
        assert_eq!(suffixed, info);
    }

    #[test]
    fn with_type_suffix_does_not_mutate_original() {
        let info = sample_info();
        let _ = info.with_type_suffix("_copy");
        assert_eq!(info.type_name, "region");
        assert_eq!(info.level, "world");
    }

    #[test]
    fn equality_and_clone() {
        let a = sample_info();
        let b = a.clone();
        assert_eq!(a, b);
        let c = a.with_type_suffix("_different");
        assert_ne!(a, c);
    }

    #[test]
    fn handles_empty_fields_and_negative_like_strings() {
        let info = RegionStorageInfo::new("", ResourceKey::<Level>::new(""), "");
        let suffixed = info.with_type_suffix("-1");
        assert_eq!(suffixed.level, "");
        assert_eq!(suffixed.dimension.location, "");
        assert_eq!(suffixed.type_name, "-1");
    }
}
