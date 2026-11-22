/// Port of `net.minecraft.world.level.levelgen.structure.TerrainAdjustment`.
///
/// The Java `CODEC` field (`StringRepresentable.fromEnum(...)`) is not represented
/// here because this module is self-contained and no codec dependency is available;
/// `get_serialized_name` and `by_serialized_name` provide the core string mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainAdjustment {
    None,
    Bury,
    BeardThin,
    BeardBox,
    Encapsulate,
}

impl TerrainAdjustment {
    pub const VALUES: [TerrainAdjustment; 5] = [
        Self::None,
        Self::Bury,
        Self::BeardThin,
        Self::BeardBox,
        Self::Encapsulate,
    ];

    // JAVA: net.minecraft.world.level.levelgen.structure.TerrainAdjustment#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Bury => "bury",
            Self::BeardThin => "beard_thin",
            Self::BeardBox => "beard_box",
            Self::Encapsulate => "encapsulate",
        }
    }

    /// Lookup by serialized name, mirroring `StringRepresentable.fromEnum` behavior.
    pub fn by_serialized_name(name: &str) -> Option<Self> {
        match name {
            "none" => Some(Self::None),
            "bury" => Some(Self::Bury),
            "beard_thin" => Some(Self::BeardThin),
            "beard_box" => Some(Self::BeardBox),
            "encapsulate" => Some(Self::Encapsulate),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TerrainAdjustment;

    #[test]
    fn serialized_names_match_java() {
        assert_eq!(TerrainAdjustment::None.get_serialized_name(), "none");
        assert_eq!(TerrainAdjustment::Bury.get_serialized_name(), "bury");
        assert_eq!(
            TerrainAdjustment::BeardThin.get_serialized_name(),
            "beard_thin"
        );
        assert_eq!(
            TerrainAdjustment::BeardBox.get_serialized_name(),
            "beard_box"
        );
        assert_eq!(
            TerrainAdjustment::Encapsulate.get_serialized_name(),
            "encapsulate"
        );
    }

    #[test]
    fn round_trip_all_values() {
        for &value in &TerrainAdjustment::VALUES {
            let name = value.get_serialized_name();
            let parsed = TerrainAdjustment::by_serialized_name(name);
            assert_eq!(parsed, Some(value));
        }
    }

    #[test]
    fn unknown_and_edge_case_names_return_none() {
        assert_eq!(TerrainAdjustment::by_serialized_name(""), None);
        assert_eq!(TerrainAdjustment::by_serialized_name("NONE"), None);
        assert_eq!(TerrainAdjustment::by_serialized_name("beard_thin "), None);
        assert_eq!(TerrainAdjustment::by_serialized_name("unknown"), None);
        assert_eq!(TerrainAdjustment::by_serialized_name("bury\0"), None);
    }

    #[test]
    fn values_are_distinct_and_complete() {
        let values = TerrainAdjustment::VALUES;
        assert_eq!(values.len(), 5);
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                assert_ne!(values[i], values[j]);
                assert_ne!(
                    values[i].get_serialized_name(),
                    values[j].get_serialized_name()
                );
            }
        }
    }

    #[test]
    fn negative_and_overflow_like_string_inputs_are_rejected() {
        // Edge-case inputs that might appear from malformed numeric/string conversions.
        assert_eq!(TerrainAdjustment::by_serialized_name("-1"), None);
        assert_eq!(
            TerrainAdjustment::by_serialized_name("2147483648"),
            None
        );
        assert_eq!(
            TerrainAdjustment::by_serialized_name(&i64::MIN.to_string()),
            None
        );
    }
}
