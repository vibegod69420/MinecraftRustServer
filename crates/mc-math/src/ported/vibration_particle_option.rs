/// Port of `net.minecraft.core.particles.VibrationParticleOption`.
///
/// Codec/stream codec fields are omitted because this module is self-contained
/// and does not depend on Mojang serialization infrastructure.
use std::fmt::Debug;

/// Minimal self-contained stand-in for `net.minecraft.world.level.gameevent.PositionSource`.
pub trait PositionSource: Debug + Clone + PartialEq {
    /// Returns `true` for entity position sources, which are rejected by
    /// `SAFE_POSITION_SOURCE_CODEC` in Java.
    fn is_entity_position_source(&self) -> bool;
}

/// Minimal self-contained stand-in for particle type identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParticleTypeId {
    Vibration,
}

/// Minimal self-contained stand-in for `net.minecraft.core.particles.ParticleOptions`.
pub trait ParticleOptions {
    fn get_type(&self) -> ParticleTypeId;
}

// JAVA: net.minecraft.core.particles.VibrationParticleOption#SAFE_POSITION_SOURCE_CODEC
pub fn is_valid_position_source<P: PositionSource>(source: &P) -> bool {
    !source.is_entity_position_source()
}

#[derive(Debug, Clone, PartialEq)]
pub struct VibrationParticleOption<P: PositionSource> {
    destination: P,
    arrival_in_ticks: i32,
}

impl<P: PositionSource> VibrationParticleOption<P> {
    // JAVA: net.minecraft.core.particles.VibrationParticleOption#<init>
    pub fn new(destination: P, arrival_in_ticks: i32) -> Self {
        Self {
            destination,
            arrival_in_ticks,
        }
    }

    // JAVA: net.minecraft.core.particles.VibrationParticleOption#getDestination
    pub fn get_destination(&self) -> &P {
        &self.destination
    }

    // JAVA: net.minecraft.core.particles.VibrationParticleOption#getArrivalInTicks
    pub fn get_arrival_in_ticks(&self) -> i32 {
        self.arrival_in_ticks
    }
}

impl<P: PositionSource> ParticleOptions for VibrationParticleOption<P> {
    // JAVA: net.minecraft.core.particles.VibrationParticleOption#getType
    fn get_type(&self) -> ParticleTypeId {
        ParticleTypeId::Vibration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct MockPos {
        x: i32,
        y: i32,
        z: i32,
    }

    impl PositionSource for MockPos {
        fn is_entity_position_source(&self) -> bool {
            false
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct MockEntityPos;

    impl PositionSource for MockEntityPos {
        fn is_entity_position_source(&self) -> bool {
            true
        }
    }

    #[test]
    fn new_and_getters() {
        let opt = VibrationParticleOption::new(MockPos { x: 1, y: -2, z: 3 }, 20);
        assert_eq!(opt.get_destination(), &MockPos { x: 1, y: -2, z: 3 });
        assert_eq!(opt.get_arrival_in_ticks(), 20);
    }

    #[test]
    fn get_type_is_vibration() {
        let opt = VibrationParticleOption::new(MockPos { x: 0, y: 0, z: 0 }, 0);
        assert_eq!(opt.get_type(), ParticleTypeId::Vibration);
    }

    #[test]
    fn negative_and_edge_arrival_ticks() {
        let neg = VibrationParticleOption::new(MockPos { x: 0, y: 0, z: 0 }, -1);
        let min = VibrationParticleOption::new(MockPos { x: 0, y: 0, z: 0 }, i32::MIN);
        let max = VibrationParticleOption::new(MockPos { x: 0, y: 0, z: 0 }, i32::MAX);
        assert_eq!(neg.get_arrival_in_ticks(), -1);
        assert_eq!(min.get_arrival_in_ticks(), i32::MIN);
        assert_eq!(max.get_arrival_in_ticks(), i32::MAX);
    }

    #[test]
    fn position_source_validation() {
        let normal = MockPos { x: 0, y: 0, z: 0 };
        let entity = MockEntityPos;
        assert!(is_valid_position_source(&normal));
        assert!(!is_valid_position_source(&entity));
        assert!(!normal.is_entity_position_source());
        assert!(entity.is_entity_position_source());
    }

    #[test]
    fn clone_and_equality() {
        let a = VibrationParticleOption::new(MockPos { x: 7, y: 8, z: 9 }, 5);
        let b = a.clone();
        assert_eq!(a, b);
        let c = VibrationParticleOption::new(MockPos { x: 7, y: 8, z: 9 }, 6);
        assert_ne!(a, c);
    }
}
