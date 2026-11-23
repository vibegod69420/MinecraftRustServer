//! Port of `net.minecraft.world.level.levelgen.OreVeinifier` (Java 1.21.11).

const VEININESS_THRESHOLD: f32 = 0.4;
const EDGE_ROUNDOFF_BEGIN: i32 = 20;
const MAX_EDGE_ROUNDOFF: f64 = 0.2;
const VEIN_SOLIDNESS: f32 = 0.7;
const MIN_RICHNESS: f32 = 0.1;
const MAX_RICHNESS: f32 = 0.3;
const MAX_RICHNESS_THRESHOLD: f32 = 0.6;
const CHANCE_OF_RAW_ORE_BLOCK: f32 = 0.02;
const SKIP_ORE_IF_GAP_NOISE_IS_BELOW: f32 = -0.3;

const DEBUG_ORE_VEINS: bool = false;

/// Minimal block state representation for ore vein generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState(pub &'static str);

impl BlockState {
    pub const AIR: Self = Self("minecraft:air");
    pub const COPPER_ORE: Self = Self("minecraft:copper_ore");
    pub const RAW_COPPER_BLOCK: Self = Self("minecraft:raw_copper_block");
    pub const GRANITE: Self = Self("minecraft:granite");
    pub const DEEPSLATE_IRON_ORE: Self = Self("minecraft:deepslate_iron_ore");
    pub const RAW_IRON_BLOCK: Self = Self("minecraft:raw_iron_block");
    pub const TUFF: Self = Self("minecraft:tuff");
    pub const OAK_BUTTON: Self = Self("minecraft:oak_button");
}

/// Minimal context providing block coordinates for density/noise sampling.
pub trait FunctionContext {
    fn block_x(&self) -> i32;
    fn block_y(&self) -> i32;
    fn block_z(&self) -> i32;
}

/// Minimal density function interface.
pub trait DensityFunction {
    fn compute(&self, ctx: &dyn FunctionContext) -> f64;
}

/// Minimal random source interface.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

/// Minimal positional random factory interface.
pub trait PositionalRandomFactory {
    type Random: RandomSource;
    fn at(&self, x: i32, y: i32, z: i32) -> Self::Random;
}

/// Port of `OreVeinifier.VeinType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VeinType {
    pub ore: BlockState,
    pub raw_ore_block: BlockState,
    pub filler: BlockState,
    pub min_y: i32,
    pub max_y: i32,
}

impl VeinType {
    pub const COPPER: Self = Self {
        ore: BlockState::COPPER_ORE,
        raw_ore_block: BlockState::RAW_COPPER_BLOCK,
        filler: BlockState::GRANITE,
        min_y: 0,
        max_y: 50,
    };

    pub const IRON: Self = Self {
        ore: BlockState::DEEPSLATE_IRON_ORE,
        raw_ore_block: BlockState::RAW_IRON_BLOCK,
        filler: BlockState::TUFF,
        min_y: -60,
        max_y: -8,
    };
}

fn clamped_map(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let t = ((value - from_min) / (from_max - from_min)).clamp(0.0, 1.0);
    to_min + t * (to_max - to_min)
}

// JAVA: net.minecraft.world.level.levelgen.OreVeinifier#create
pub fn create<D0, D1, D2, F>(
    vein_toggle: D0,
    vein_ridged: D1,
    vein_gap: D2,
    random_factory: F,
) -> impl Fn(&dyn FunctionContext) -> Option<BlockState>
where
    D0: DensityFunction,
    D1: DensityFunction,
    D2: DensityFunction,
    F: PositionalRandomFactory,
{
    let debug_state: Option<BlockState> = if DEBUG_ORE_VEINS {
        Some(BlockState::AIR)
    } else {
        None
    };

    move |ctx: &dyn FunctionContext| {
        let vein_toggle_val: f64 = vein_toggle.compute(ctx);
        let block_y: i32 = ctx.block_y();
        let vein_type: VeinType = if vein_toggle_val > 0.0 {
            VeinType::COPPER
        } else {
            VeinType::IRON
        };
        let abs_vein: f64 = vein_toggle_val.abs();
        let dist_to_max: i32 = vein_type.max_y.wrapping_sub(block_y);
        let dist_to_min: i32 = block_y.wrapping_sub(vein_type.min_y);
        if dist_to_min >= 0 && dist_to_max >= 0 {
            let edge_dist: i32 = dist_to_max.min(dist_to_min);
            let edge_roundoff: f64 = clamped_map(
                edge_dist as f64,
                0.0,
                EDGE_ROUNDOFF_BEGIN as f64,
                -MAX_EDGE_ROUNDOFF,
                0.0,
            );
            if abs_vein + edge_roundoff < (VEININESS_THRESHOLD as f64) {
                return debug_state;
            }

            let mut random = random_factory.at(ctx.block_x(), block_y, ctx.block_z());
            if random.next_float() > VEIN_SOLIDNESS {
                return debug_state;
            } else if vein_ridged.compute(ctx) >= 0.0 {
                return debug_state;
            } else {
                let richness: f64 = clamped_map(
                    abs_vein,
                    VEININESS_THRESHOLD as f64,
                    MAX_RICHNESS_THRESHOLD as f64,
                    MIN_RICHNESS as f64,
                    MAX_RICHNESS as f64,
                );
                if (random.next_float() as f64) < richness
                    && vein_gap.compute(ctx) > (SKIP_ORE_IF_GAP_NOISE_IS_BELOW as f64)
                {
                    return if random.next_float() < CHANCE_OF_RAW_ORE_BLOCK {
                        Some(vein_type.raw_ore_block)
                    } else {
                        Some(vein_type.ore)
                    };
                } else {
                    return if DEBUG_ORE_VEINS {
                        Some(BlockState::OAK_BUTTON)
                    } else {
                        Some(vein_type.filler)
                    };
                }
            }
        } else {
            return debug_state;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct TestCtx {
        x: i32,
        y: i32,
        z: i32,
    }

    impl FunctionContext for TestCtx {
        fn block_x(&self) -> i32 {
            self.x
        }
        fn block_y(&self) -> i32 {
            self.y
        }
        fn block_z(&self) -> i32 {
            self.z
        }
    }

    struct ConstDensity(f64);
    impl DensityFunction for ConstDensity {
        fn compute(&self, _: &dyn FunctionContext) -> f64 {
            self.0
        }
    }

    struct SeqRandom {
        values: Vec<f32>,
        idx: usize,
    }

    impl RandomSource for SeqRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v
        }
    }

    struct FixedFactory {
        values: Vec<f32>,
    }

    impl PositionalRandomFactory for FixedFactory {
        type Random = SeqRandom;
        fn at(&self, _x: i32, _y: i32, _z: i32) -> SeqRandom {
            SeqRandom {
                values: self.values.clone(),
                idx: 0,
            }
        }
    }

    #[test]
    fn y_out_of_range_and_in_range() {
        let filler = create(
            ConstDensity(1.0),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            FixedFactory {
                values: vec![0.0, 0.0, 0.0],
            },
        );
        // Copper range: [0, 50]
        assert_eq!(filler(&TestCtx { x: 0, y: -1, z: 0 }), None);
        assert_eq!(filler(&TestCtx { x: 0, y: 51, z: 0 }), None);
        assert_eq!(
            filler(&TestCtx { x: 0, y: 25, z: 0 }),
            Some(BlockState::RAW_COPPER_BLOCK)
        );
    }

    #[test]
    fn vein_type_selection_and_zero_toggle() {
        let factory_vals = || FixedFactory {
            values: vec![0.0, 0.0, 0.0],
        };
        // Positive -> COPPER
        let copper = create(
            ConstDensity(0.5),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            factory_vals(),
        );
        assert_eq!(
            copper(&TestCtx { x: 0, y: 25, z: 0 }),
            Some(BlockState::RAW_COPPER_BLOCK)
        );

        // Zero -> IRON (not > 0.0), but abs(0.0) is below veininess threshold
        let iron_zero = create(
            ConstDensity(0.0),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            factory_vals(),
        );
        assert_eq!(iron_zero(&TestCtx { x: 0, y: -34, z: 0 }), None);

        // Negative -> IRON with sufficient veininess
        let iron = create(
            ConstDensity(-0.5),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            factory_vals(),
        );
        assert_eq!(
            iron(&TestCtx { x: 0, y: -34, z: 0 }),
            Some(BlockState::RAW_IRON_BLOCK)
        );
    }

    #[test]
    fn edge_roundoff_threshold_uses_f32_literal_semantics() {
        let filler = create(
            ConstDensity(0.5),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            FixedFactory {
                values: vec![0.0, 0.0, 0.0],
            },
        );
        // y=10, edge_dist=10: roundoff=-0.1, sum=0.4. 0.4f64 < 0.4f32_as_f64 is true.
        assert_eq!(filler(&TestCtx { x: 0, y: 10, z: 0 }), None);
        // y=11, edge_dist=11: roundoff=-0.09, sum=0.41 >= 0.4f32_as_f64.
        assert_eq!(
            filler(&TestCtx { x: 0, y: 11, z: 0 }),
            Some(BlockState::RAW_COPPER_BLOCK)
        );
    }

    #[test]
    fn solidness_ridged_and_gap_skip() {
        // Solidness skip: next_float() > 0.7
        let solid_skip = create(
            ConstDensity(0.5),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            FixedFactory { values: vec![0.8] },
        );
        assert_eq!(solid_skip(&TestCtx { x: 0, y: 25, z: 0 }), None);

        // Ridged >= 0.0 skip
        let ridged_skip = create(
            ConstDensity(0.5),
            ConstDensity(0.0),
            ConstDensity(1.0),
            FixedFactory { values: vec![0.0] },
        );
        assert_eq!(ridged_skip(&TestCtx { x: 0, y: 25, z: 0 }), None);

        // Gap below threshold: return filler, not ore
        let gap_skip = create(
            ConstDensity(0.5),
            ConstDensity(-1.0),
            ConstDensity(-0.4),
            FixedFactory {
                values: vec![0.0, 0.0],
            },
        );
        assert_eq!(
            gap_skip(&TestCtx { x: 0, y: 25, z: 0 }),
            Some(BlockState::GRANITE)
        );
    }

    #[test]
    fn richness_and_raw_ore_selection() {
        // richness at abs=0.5 is 0.2; 0.19f32 < 0.2f64, raw 0.01f32 < 0.02f32
        let raw = create(
            ConstDensity(0.5),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            FixedFactory {
                values: vec![0.0, 0.19, 0.01],
            },
        );
        assert_eq!(
            raw(&TestCtx { x: 0, y: 25, z: 0 }),
            Some(BlockState::RAW_COPPER_BLOCK)
        );

        // raw check fails -> normal ore
        let ore = create(
            ConstDensity(0.5),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            FixedFactory {
                values: vec![0.0, 0.19, 0.02],
            },
        );
        assert_eq!(
            ore(&TestCtx { x: 0, y: 25, z: 0 }),
            Some(BlockState::COPPER_ORE)
        );

        // richness check fails (0.2f32 promoted to f64 is not < 0.2f64) -> filler
        let filler = create(
            ConstDensity(0.5),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            FixedFactory {
                values: vec![0.0, 0.2],
            },
        );
        assert_eq!(
            filler(&TestCtx { x: 0, y: 25, z: 0 }),
            Some(BlockState::GRANITE)
        );
    }

    #[test]
    fn wrapping_y_and_nan_vein_toggle() {
        let filler = create(
            ConstDensity(0.5),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            FixedFactory {
                values: vec![0.0, 0.0, 0.0],
            },
        );
        // Extreme Y values must not panic and use wrapping subtraction semantics.
        assert_eq!(filler(&TestCtx { x: 0, y: i32::MIN, z: 0 }), None);
        assert_eq!(filler(&TestCtx { x: 0, y: i32::MAX, z: 0 }), None);

        // NaN vein toggle: NaN > 0.0 is false -> IRON; NaN comparisons fail -> filler.
        let nan_filler = create(
            ConstDensity(f64::NAN),
            ConstDensity(-1.0),
            ConstDensity(1.0),
            FixedFactory {
                values: vec![0.0, 0.0, 0.0],
            },
        );
        assert_eq!(nan_filler(&TestCtx { x: 0, y: 25, z: 0 }), None);
        assert_eq!(
            nan_filler(&TestCtx { x: 0, y: -34, z: 0 }),
            Some(BlockState::TUFF)
        );
    }
}
