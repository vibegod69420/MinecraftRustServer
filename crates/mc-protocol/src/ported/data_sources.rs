use std::collections::BTreeMap;
use std::sync::LazyLock;

/// Minimal placeholder for `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapCodec {
    pub id: String,
}

impl MapCodec {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// Minimal placeholder for `net.minecraft.util.ExtraCodecs.LateBoundIdMapper`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LateBoundIdMapper {
    entries: BTreeMap<String, MapCodec>,
}

impl LateBoundIdMapper {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn put(&mut self, key: impl Into<String>, value: MapCodec) {
        self.entries.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&MapCodec> {
        self.entries.get(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Minimal placeholder for the codec produced by
/// `ComponentSerialization.createLegacyComponentMatcher(...)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSourceCodec {
    pub source_key: &'static str,
}

/// Holds the static state corresponding to `net.minecraft.network.chat.contents.data.DataSources`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSourcesState {
    pub id_mapper: LateBoundIdMapper,
    pub codec: DataSourceCodec,
}

// JAVA: net.minecraft.network.chat.contents.data.DataSources#<clinit>
pub fn init_data_sources() -> DataSourcesState {
    let mut id_mapper = LateBoundIdMapper::new();
    id_mapper.put("entity", MapCodec::new("entity"));
    id_mapper.put("block", MapCodec::new("block"));
    id_mapper.put("storage", MapCodec::new("storage"));

    DataSourcesState {
        id_mapper,
        codec: DataSourceCodec { source_key: "source" },
    }
}

pub static ID_MAPPER: LazyLock<LateBoundIdMapper> =
    LazyLock::new(|| init_data_sources().id_mapper);

pub static CODEC: LazyLock<DataSourceCodec> = LazyLock::new(|| init_data_sources().codec);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_all_data_sources() {
        let state = init_data_sources();
        assert_eq!(state.id_mapper.len(), 3);
        assert_eq!(state.id_mapper.get("entity"), Some(&MapCodec::new("entity")));
        assert_eq!(state.id_mapper.get("block"), Some(&MapCodec::new("block")));
        assert_eq!(
            state.id_mapper.get("storage"),
            Some(&MapCodec::new("storage"))
        );
        assert_eq!(state.codec.source_key, "source");
    }

    #[test]
    fn missing_and_empty_keys_return_none() {
        let state = init_data_sources();
        assert!(state.id_mapper.get("missing").is_none());
        assert!(state.id_mapper.get("").is_none());
        assert!(!state.id_mapper.is_empty());
    }

    #[test]
    fn put_overwrites_existing_entry() {
        let mut mapper = LateBoundIdMapper::new();
        mapper.put("entity", MapCodec::new("old"));
        mapper.put("entity", MapCodec::new("new"));
        assert_eq!(mapper.len(), 1);
        assert_eq!(mapper.get("entity"), Some(&MapCodec::new("new")));
    }

    #[test]
    fn java_int_semantics_overflow_and_negative_inputs() {
        let base: i32 = 3; // number of registered sources
        let overflowed = base.wrapping_add(i32::MAX);
        assert_eq!(overflowed, i32::MIN + 2);

        let negative: i32 = -1;
        let wrapped_sub = negative.wrapping_sub(2);
        assert_eq!(wrapped_sub, -3);

        let masked_shift = 1i32.wrapping_shl(33 & 31);
        assert_eq!(masked_shift, 2);

        let negative_masked_shift = (-2i32).wrapping_shl(65 & 31);
        assert_eq!(negative_masked_shift, -4);
    }

    #[test]
    fn float_bits_preserve_java_style_representation() {
        let neg_one_bits = (-1.0f32).to_bits();
        assert_eq!(neg_one_bits, 0xBF80_0000);

        let neg_zero_bits = (-0.0f32).to_bits();
        assert_eq!(neg_zero_bits, 0x8000_0000);
        assert_ne!(neg_zero_bits, 0.0f32.to_bits());

        let nan_bits = f32::NAN.to_bits();
        assert_eq!(nan_bits & 0x7F80_0000, 0x7F80_0000);
        assert_ne!(nan_bits & 0x007F_FFFF, 0);

        let roundtrip = f32::from_bits(0x3F80_0000);
        assert_eq!(roundtrip.to_bits(), 1.0f32.to_bits());
    }
}
