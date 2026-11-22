/// Minimal self-contained placeholder for `net.minecraft.util.random.WeightedList<T>`.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedList<T> {
    pub items: Vec<T>,
}

/// Minimal self-contained placeholder for
/// `net.minecraft.world.level.biome.MobSpawnSettings.SpawnerData`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnerData;

/// Port of `net.minecraft.world.level.levelgen.structure.StructureSpawnOverride`.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureSpawnOverride<S = SpawnerData> {
    bounding_box: BoundingBoxType,
    spawns: WeightedList<S>,
}

impl<S> StructureSpawnOverride<S> {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructureSpawnOverride#<init>
    pub fn new(bounding_box: BoundingBoxType, spawns: WeightedList<S>) -> Self {
        Self {
            bounding_box,
            spawns,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureSpawnOverride#boundingBox
    pub fn bounding_box(&self) -> BoundingBoxType {
        self.bounding_box
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureSpawnOverride#spawns
    pub fn spawns(&self) -> &WeightedList<S> {
        &self.spawns
    }
}

/// Port of `net.minecraft.world.level.levelgen.structure.StructureSpawnOverride$BoundingBoxType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundingBoxType {
    Piece,
    Structure,
}

impl BoundingBoxType {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructureSpawnOverride$BoundingBoxType#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Self::Piece => "piece",
            Self::Structure => "full",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BoundingBoxType, SpawnerData, StructureSpawnOverride, WeightedList};

    #[test]
    fn bounding_box_type_serialized_names() {
        assert_eq!(BoundingBoxType::Piece.get_serialized_name(), "piece");
        assert_eq!(BoundingBoxType::Structure.get_serialized_name(), "full");
    }

    #[test]
    fn bounding_box_type_copy_and_eq() {
        let a = BoundingBoxType::Piece;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(BoundingBoxType::Piece, BoundingBoxType::Structure);
    }

    #[test]
    fn spawn_override_accessors() {
        let spawns = WeightedList {
            items: vec![SpawnerData, SpawnerData],
        };
        let spawn_override =
            StructureSpawnOverride::new(BoundingBoxType::Structure, spawns.clone());
        assert_eq!(spawn_override.bounding_box(), BoundingBoxType::Structure);
        assert_eq!(spawn_override.spawns(), &spawns);
        assert_eq!(spawn_override.spawns().items.len(), 2);
    }

    #[test]
    fn spawn_override_empty_spawns() {
        let spawns: WeightedList<SpawnerData> = WeightedList { items: Vec::new() };
        let spawn_override = StructureSpawnOverride::new(BoundingBoxType::Piece, spawns);
        assert_eq!(spawn_override.bounding_box(), BoundingBoxType::Piece);
        assert!(spawn_override.spawns().items.is_empty());
    }

    #[test]
    fn spawn_override_generic_edge_values() {
        #[derive(Debug, Clone, PartialEq)]
        struct Dummy {
            weight: i32,
            chance: f32,
        }

        let spawns = WeightedList {
            items: vec![
                Dummy {
                    weight: i32::MIN,
                    chance: -0.0_f32,
                },
                Dummy {
                    weight: i32::MAX,
                    chance: f32::NAN,
                },
            ],
        };
        let spawn_override = StructureSpawnOverride::new(BoundingBoxType::Piece, spawns);

        assert_eq!(spawn_override.spawns().items[0].weight, i32::MIN);
        assert_eq!(spawn_override.spawns().items[1].weight, i32::MAX);
        assert_eq!(
            spawn_override.spawns().items[0].chance.to_bits(),
            (-0.0_f32).to_bits()
        );
        assert!(spawn_override.spawns().items[1].chance.is_nan());
    }
}
