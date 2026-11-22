pub trait FeatureConfiguration {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpikeConfiguration<S> {
    crystal_invulnerable: bool,
    spikes: Vec<S>,
    crystal_beam_target: Option<BlockPos>,
}

impl<S> FeatureConfiguration for SpikeConfiguration<S> {}

impl<S> SpikeConfiguration<S> {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SpikeConfiguration#<init>
    pub fn new(
        crystal_invulnerable: bool,
        spikes: Vec<S>,
        crystal_beam_target: Option<BlockPos>,
    ) -> Self {
        Self::from_optional(crystal_invulnerable, spikes, crystal_beam_target)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SpikeConfiguration#<init>
    fn from_optional(
        crystal_invulnerable: bool,
        spikes: Vec<S>,
        crystal_beam_target: Option<BlockPos>,
    ) -> Self {
        Self {
            crystal_invulnerable,
            spikes,
            crystal_beam_target,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SpikeConfiguration#isCrystalInvulnerable
    pub fn is_crystal_invulnerable(&self) -> bool {
        self.crystal_invulnerable
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SpikeConfiguration#getSpikes
    pub fn get_spikes(&self) -> &[S] {
        &self.spikes
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SpikeConfiguration#getCrystalBeamTarget
    pub fn get_crystal_beam_target(&self) -> Option<BlockPos> {
        self.crystal_beam_target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockSpike {
        radius: i32,
        height: i32,
    }

    #[test]
    fn new_and_accessors() {
        let target = BlockPos::new(1, 2, 3);
        let spikes = vec![MockSpike {
            radius: 2,
            height: 10,
        }];
        let c = SpikeConfiguration::new(true, spikes.clone(), Some(target));
        assert!(c.is_crystal_invulnerable());
        assert_eq!(c.get_spikes(), spikes.as_slice());
        assert_eq!(c.get_crystal_beam_target(), Some(target));
    }

    #[test]
    fn nullable_beam_target() {
        let c = SpikeConfiguration::new(false, Vec::<MockSpike>::new(), None);
        assert!(!c.is_crystal_invulnerable());
        assert!(c.get_spikes().is_empty());
        assert_eq!(c.get_crystal_beam_target(), None);
    }

    #[test]
    fn handles_negative_and_extreme_integer_values() {
        let target = BlockPos::new(i32::MIN, -1, i32::MAX);
        let spikes = vec![
            MockSpike {
                radius: i32::MIN,
                height: i32::MAX,
            },
            MockSpike {
                radius: -1,
                height: 0,
            },
        ];
        let c = SpikeConfiguration::new(true, spikes, Some(target));
        let t = c.get_crystal_beam_target().unwrap();
        assert_eq!(t.x, i32::MIN);
        assert_eq!(t.y, -1);
        assert_eq!(t.z, i32::MAX);
        assert_eq!(c.get_spikes()[0].radius, i32::MIN);
        assert_eq!(c.get_spikes()[0].height, i32::MAX);
        assert_eq!(t.x.wrapping_sub(1), i32::MAX);
        assert_eq!(t.z.wrapping_add(1), i32::MIN);
    }

    #[test]
    fn preserves_float_bits_in_generic_spikes() {
        let neg_zero = -0.0_f32;
        let spikes = vec![f32::NAN, neg_zero, f32::INFINITY, f32::NEG_INFINITY];
        let c = SpikeConfiguration::new(false, spikes, None);
        assert!(c.get_spikes()[0].is_nan());
        assert_eq!(c.get_spikes()[1].to_bits(), neg_zero.to_bits());
        assert_eq!(c.get_spikes()[2].to_bits(), f32::INFINITY.to_bits());
        assert_eq!(c.get_spikes()[3].to_bits(), f32::NEG_INFINITY.to_bits());
    }
}
