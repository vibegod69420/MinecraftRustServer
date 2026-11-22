#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleType {
    pub id: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockParticleOption {
    type_: ParticleType,
    state: BlockState,
}

impl BlockParticleOption {
    // JAVA: net.minecraft.core.particles.BlockParticleOption#<init>
    pub fn new(type_: ParticleType, state: BlockState) -> Self {
        Self { type_, state }
    }

    // JAVA: net.minecraft.core.particles.BlockParticleOption#getType
    pub fn get_type(&self) -> ParticleType {
        self.type_
    }

    // JAVA: net.minecraft.core.particles.BlockParticleOption#getState
    pub fn get_state(&self) -> &BlockState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_type_and_state() {
        let t = ParticleType {
            id: "minecraft:block",
        };
        let s = BlockState { id: 42 };
        let o = BlockParticleOption::new(t, s.clone());
        assert_eq!(o.get_type(), t);
        assert_eq!(o.get_state(), &s);
    }

    #[test]
    fn different_states_not_equal() {
        let t = ParticleType {
            id: "minecraft:block",
        };
        let a = BlockParticleOption::new(t, BlockState { id: 0 });
        let b = BlockParticleOption::new(t, BlockState { id: 1 });
        assert_ne!(a, b);
    }

    #[test]
    fn state_max_id_bits() {
        let t = ParticleType {
            id: "minecraft:falling_dust",
        };
        let s = BlockState { id: u32::MAX };
        let o = BlockParticleOption::new(t, s);
        assert_eq!(o.get_state().id, u32::MAX);
        assert_eq!(o.get_type().id, "minecraft:falling_dust");
    }

    #[test]
    fn clone_preserves_values() {
        let o = BlockParticleOption::new(
            ParticleType { id: "minecraft:block" },
            BlockState { id: 7 },
        );
        let c = o.clone();
        assert_eq!(o, c);
    }
}
