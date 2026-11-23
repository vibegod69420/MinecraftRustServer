//! Port of `net.minecraft.world.level.levelgen.structure.StructureType`.

use std::marker::PhantomData;
use std::sync::{LazyLock, Mutex};

/// Minimal placeholder for `net.minecraft.world.level.levelgen.structure.Structure`.
pub trait Structure: 'static {}

/// Minimal placeholder for `com.mojang.serialization.MapCodec`.
#[derive(Debug)]
pub struct MapCodec<S: ?Sized> {
    _marker: PhantomData<fn() -> S>,
}

impl<S: ?Sized> MapCodec<S> {
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<S: ?Sized> Clone for MapCodec<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: ?Sized> Copy for MapCodec<S> {}

impl<S: ?Sized> Default for MapCodec<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Port of `net.minecraft.world.level.levelgen.structure.StructureType`.
#[derive(Debug, Clone, Copy)]
pub struct StructureType<S: Structure> {
    name: &'static str,
    codec: MapCodec<S>,
}

impl<S: Structure> StructureType<S> {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructureType#codec
    pub fn codec(&self) -> &MapCodec<S> {
        &self.codec
    }

    /// Returns the registered identifier for this structure type.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

static STRUCTURE_TYPE_REGISTRY: LazyLock<Mutex<Vec<&'static str>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

// JAVA: net.minecraft.world.level.levelgen.structure.StructureType#register
fn register<S: Structure>(name: &'static str, codec: MapCodec<S>) -> StructureType<S> {
    // Faithful stub for: Registry.register(BuiltInRegistries.STRUCTURE_TYPE, name, () -> codec)
    STRUCTURE_TYPE_REGISTRY
        .lock()
        .expect("structure type registry lock poisoned")
        .push(name);
    StructureType { name, codec }
}

/// Returns a snapshot of registered structure type names in registration order.
pub fn registered_structure_type_names() -> Vec<&'static str> {
    STRUCTURE_TYPE_REGISTRY
        .lock()
        .expect("structure type registry lock poisoned")
        .clone()
}

macro_rules! structure_stubs {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Default)]
            pub struct $name;
            impl Structure for $name {}
            impl $name {
                pub const CODEC: MapCodec<Self> = MapCodec::new();
            }
        )*
    };
}

structure_stubs!(
    BuriedTreasureStructure,
    DesertPyramidStructure,
    EndCityStructure,
    NetherFortressStructure,
    IglooStructure,
    JigsawStructure,
    JungleTempleStructure,
    MineshaftStructure,
    NetherFossilStructure,
    OceanMonumentStructure,
    OceanRuinStructure,
    RuinedPortalStructure,
    ShipwreckStructure,
    StrongholdStructure,
    SwampHutStructure,
    WoodlandMansionStructure
);

macro_rules! structure_types {
    ($($const_name:ident : $struct_ty:ident = $id:literal),* $(,)?) => {
        $(
            pub static $const_name: LazyLock<StructureType<$struct_ty>> =
                LazyLock::new(|| register($id, $struct_ty::CODEC));
        )*
    };
}

structure_types! {
    BURIED_TREASURE: BuriedTreasureStructure = "buried_treasure",
    DESERT_PYRAMID: DesertPyramidStructure = "desert_pyramid",
    END_CITY: EndCityStructure = "end_city",
    FORTRESS: NetherFortressStructure = "fortress",
    IGLOO: IglooStructure = "igloo",
    JIGSAW: JigsawStructure = "jigsaw",
    JUNGLE_TEMPLE: JungleTempleStructure = "jungle_temple",
    MINESHAFT: MineshaftStructure = "mineshaft",
    NETHER_FOSSIL: NetherFossilStructure = "nether_fossil",
    OCEAN_MONUMENT: OceanMonumentStructure = "ocean_monument",
    OCEAN_RUIN: OceanRuinStructure = "ocean_ruin",
    RUINED_PORTAL: RuinedPortalStructure = "ruined_portal",
    SHIPWRECK: ShipwreckStructure = "shipwreck",
    STRONGHOLD: StrongholdStructure = "stronghold",
    SWAMP_HUT: SwampHutStructure = "swamp_hut",
    WOODLAND_MANSION: WoodlandMansionStructure = "woodland_mansion",
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_structure_types_have_expected_names() {
        let cases = [
            (BURIED_TREASURE.name(), "buried_treasure"),
            (DESERT_PYRAMID.name(), "desert_pyramid"),
            (END_CITY.name(), "end_city"),
            (FORTRESS.name(), "fortress"),
            (IGLOO.name(), "igloo"),
            (JIGSAW.name(), "jigsaw"),
            (JUNGLE_TEMPLE.name(), "jungle_temple"),
            (MINESHAFT.name(), "mineshaft"),
            (NETHER_FOSSIL.name(), "nether_fossil"),
            (OCEAN_MONUMENT.name(), "ocean_monument"),
            (OCEAN_RUIN.name(), "ocean_ruin"),
            (RUINED_PORTAL.name(), "ruined_portal"),
            (SHIPWRECK.name(), "shipwreck"),
            (STRONGHOLD.name(), "stronghold"),
            (SWAMP_HUT.name(), "swamp_hut"),
            (WOODLAND_MANSION.name(), "woodland_mansion"),
        ];
        for (actual, expected) in cases {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn codec_returns_reference_to_stored_codec() {
        let c1: &MapCodec<BuriedTreasureStructure> = BURIED_TREASURE.codec();
        let c2: &MapCodec<BuriedTreasureStructure> = BURIED_TREASURE.codec();
        assert!(std::ptr::eq(c1, c2));
    }

    #[test]
    fn register_adds_name_and_handles_edge_cases() {
        #[derive(Debug, Clone, Copy, Default)]
        struct EmptyNameStructure;
        impl Structure for EmptyNameStructure {}

        #[derive(Debug, Clone, Copy, Default)]
        struct NegativeSentinelStructure;
        impl Structure for NegativeSentinelStructure {}

        let empty = register("", MapCodec::<EmptyNameStructure>::new());
        assert_eq!(empty.name(), "");

        let sentinel = register("negative_sentinel", MapCodec::<NegativeSentinelStructure>::new());
        assert_eq!(sentinel.name(), "negative_sentinel");

        let names = registered_structure_type_names();
        assert!(names.contains(&""));
        assert!(names.contains(&"negative_sentinel"));
    }

    #[test]
    fn registry_contains_all_vanilla_names() {
        // Force initialization of all vanilla static structure types.
        let _ = (
            &*BURIED_TREASURE,
            &*DESERT_PYRAMID,
            &*END_CITY,
            &*FORTRESS,
            &*IGLOO,
            &*JIGSAW,
            &*JUNGLE_TEMPLE,
            &*MINESHAFT,
            &*NETHER_FOSSIL,
            &*OCEAN_MONUMENT,
            &*OCEAN_RUIN,
            &*RUINED_PORTAL,
            &*SHIPWRECK,
            &*STRONGHOLD,
            &*SWAMP_HUT,
            &*WOODLAND_MANSION,
        );

        let expected = [
            "buried_treasure",
            "desert_pyramid",
            "end_city",
            "fortress",
            "igloo",
            "jigsaw",
            "jungle_temple",
            "mineshaft",
            "nether_fossil",
            "ocean_monument",
            "ocean_ruin",
            "ruined_portal",
            "shipwreck",
            "stronghold",
            "swamp_hut",
            "woodland_mansion",
        ];
        let names = registered_structure_type_names();
        for name in expected {
            assert!(names.contains(&name), "missing registered name: {name}");
        }
    }

    #[test]
    fn java_numeric_semantics_edge_cases() {
        // Integer overflow/wrapping semantics
        assert_eq!(i32::MAX.wrapping_add(1), i32::MIN);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
        assert_eq!((-1i32).wrapping_mul(i32::MIN), i32::MIN);

        // Negative inputs and masked shifts
        let shift: i32 = -1;
        assert_eq!(1i32.wrapping_shl((shift as u32) & 31), 1i32 << 31);
        assert_eq!((-128i64).wrapping_shr(4u32 & 63), -8i64);

        // Float bit patterns and operation-order-sensitive representation
        assert_eq!((-0.0f32).to_bits(), 0x8000_0000u32);
        assert_eq!(0.0f32.to_bits(), 0x0000_0000u32);
        let nan_bits = f32::NAN.to_bits();
        assert_ne!(nan_bits & 0x7FC0_0000, 0);
        let ordered = (1.0f64 / 3.0f64).to_bits();
        assert_eq!(ordered, 0x3fd5_5555_5555_5555u64);
    }
}
