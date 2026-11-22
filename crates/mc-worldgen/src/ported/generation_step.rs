pub trait StringRepresentable {
    fn get_serialized_name(&self) -> &str;
}

pub struct GenerationStep;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Decoration {
    RawGeneration,
    Lakes,
    LocalModifications,
    UndergroundStructures,
    SurfaceStructures,
    Strongholds,
    UndergroundOres,
    UndergroundDecoration,
    FluidSprings,
    VegetalDecoration,
    TopLayerModification,
}

impl Decoration {
    pub const VALUES: [Decoration; 11] = [
        Decoration::RawGeneration,
        Decoration::Lakes,
        Decoration::LocalModifications,
        Decoration::UndergroundStructures,
        Decoration::SurfaceStructures,
        Decoration::Strongholds,
        Decoration::UndergroundOres,
        Decoration::UndergroundDecoration,
        Decoration::FluidSprings,
        Decoration::VegetalDecoration,
        Decoration::TopLayerModification,
    ];

    // JAVA: net.minecraft.world.level.levelgen.GenerationStep$Decoration#getName
    pub fn get_name(&self) -> &str {
        match self {
            Decoration::RawGeneration => "raw_generation",
            Decoration::Lakes => "lakes",
            Decoration::LocalModifications => "local_modifications",
            Decoration::UndergroundStructures => "underground_structures",
            Decoration::SurfaceStructures => "surface_structures",
            Decoration::Strongholds => "strongholds",
            Decoration::UndergroundOres => "underground_ores",
            Decoration::UndergroundDecoration => "underground_decoration",
            Decoration::FluidSprings => "fluid_springs",
            Decoration::VegetalDecoration => "vegetal_decoration",
            Decoration::TopLayerModification => "top_layer_modification",
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.GenerationStep$Decoration#getSerializedName
    pub fn get_serialized_name_value(&self) -> &str {
        self.get_name()
    }
}

impl StringRepresentable for Decoration {
    fn get_serialized_name(&self) -> &str {
        self.get_serialized_name_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_java_serialized_names() {
        let expected = [
            (Decoration::RawGeneration, "raw_generation"),
            (Decoration::Lakes, "lakes"),
            (Decoration::LocalModifications, "local_modifications"),
            (Decoration::UndergroundStructures, "underground_structures"),
            (Decoration::SurfaceStructures, "surface_structures"),
            (Decoration::Strongholds, "strongholds"),
            (Decoration::UndergroundOres, "underground_ores"),
            (Decoration::UndergroundDecoration, "underground_decoration"),
            (Decoration::FluidSprings, "fluid_springs"),
            (Decoration::VegetalDecoration, "vegetal_decoration"),
            (Decoration::TopLayerModification, "top_layer_modification"),
        ];
        for (dec, name) in expected {
            assert_eq!(dec.get_name(), name);
            assert_eq!(dec.get_serialized_name_value(), name);
            assert_eq!(StringRepresentable::get_serialized_name(&dec), name);
        }
    }

    #[test]
    fn values_contains_all_variants_in_order() {
        assert_eq!(Decoration::VALUES.len(), 11);
        assert_eq!(Decoration::VALUES[0], Decoration::RawGeneration);
        assert_eq!(Decoration::VALUES[10], Decoration::TopLayerModification);
    }

    #[test]
    fn serialized_name_equals_name_for_all_values() {
        for d in Decoration::VALUES {
            assert_eq!(d.get_name(), d.get_serialized_name_value());
        }
    }

    #[test]
    fn enum_is_copy_and_eq() {
        let a = Decoration::Lakes;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(Decoration::Lakes, Decoration::Strongholds);
    }
}
