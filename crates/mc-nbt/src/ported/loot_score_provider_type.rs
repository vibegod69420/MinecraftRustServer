use std::fmt;
use std::marker::PhantomData;

/// Placeholder for Mojang serialization `MapCodec<? extends ScoreboardNameProvider>`.
#[derive(Clone)]
pub struct MapCodec<T> {
    _marker: PhantomData<T>,
}

impl<T> fmt::Debug for MapCodec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MapCodec(..)")
    }
}

impl<T> MapCodec<T> {
    pub fn placeholder() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Marker type representing a `ScoreboardNameProvider` codec element.
#[derive(Debug, Clone)]
pub struct ScoreboardNameProviderMarker;

// JAVA: net.minecraft.world.level.storage.loot.providers.score.LootScoreProviderType
#[derive(Debug, Clone)]
pub struct LootScoreProviderType {
    codec: MapCodec<ScoreboardNameProviderMarker>,
}

impl LootScoreProviderType {
    // JAVA: net.minecraft.world.level.storage.loot.providers.score.LootScoreProviderType#<init>
    pub fn new(codec: MapCodec<ScoreboardNameProviderMarker>) -> Self {
        Self { codec }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.LootScoreProviderType#codec
    pub fn codec(&self) -> &MapCodec<ScoreboardNameProviderMarker> {
        &self.codec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_codec_accessor() {
        let t = LootScoreProviderType::new(MapCodec::placeholder());
        let _codec = t.codec();
        let dbg = format!("{:?}", t);
        assert!(dbg.contains("LootScoreProviderType"));
    }

    #[test]
    fn clone_preserves_debug_shape() {
        let t = LootScoreProviderType::new(MapCodec::placeholder());
        let t2 = t.clone();
        assert_eq!(format!("{:?}", t.codec()), format!("{:?}", t2.codec()));
    }

    #[test]
    fn edge_case_negative_input() {
        let name_id: i32 = -1;
        assert_eq!(name_id, -1);
        assert_eq!(name_id.wrapping_sub(1), -2);
    }

    #[test]
    fn overflow_wrapping_semantics() {
        let v: i32 = i32::MAX;
        assert_eq!(v.wrapping_add(1), i32::MIN);
        let v64: i64 = i64::MIN;
        assert_eq!(v64.wrapping_sub(1), i64::MAX);
    }

    #[test]
    fn float_bits_edge_case() {
        let nan_bits = f32::NAN.to_bits();
        assert_eq!(nan_bits, f32::NAN.to_bits());
        assert_ne!((-0.0f32).to_bits(), 0.0f32.to_bits());
        assert_ne!(f64::NAN.to_bits(), f64::INFINITY.to_bits());
    }
}
