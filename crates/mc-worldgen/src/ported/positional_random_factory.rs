#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    value: String,
}

impl Identifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

pub trait PositionalRandomFactory {
    type Random;

    // JAVA: net.minecraft.world.level.levelgen.PositionalRandomFactory#at
    fn at_block_pos(&self, pos: &BlockPos) -> Self::Random {
        self.at(pos.get_x(), pos.get_y(), pos.get_z())
    }

    // JAVA: net.minecraft.world.level.levelgen.PositionalRandomFactory#fromHashOf
    fn from_hash_of_identifier(&self, id: &Identifier) -> Self::Random {
        self.from_hash_of(&id.to_string())
    }

    // JAVA: net.minecraft.world.level.levelgen.PositionalRandomFactory#fromHashOf
    fn from_hash_of(&self, name: &str) -> Self::Random;

    // JAVA: net.minecraft.world.level.levelgen.PositionalRandomFactory#fromSeed
    fn from_seed(&self, seed: i64) -> Self::Random;

    // JAVA: net.minecraft.world.level.levelgen.PositionalRandomFactory#at
    fn at(&self, x: i32, y: i32, z: i32) -> Self::Random;

    // JAVA: net.minecraft.world.level.levelgen.PositionalRandomFactory#parityConfigString
    fn parity_config_string(&self, builder: &mut String);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyRandom {
        x: i32,
        y: i32,
        z: i32,
        seed: i64,
        name: String,
    }

    struct DummyFactory;

    impl PositionalRandomFactory for DummyFactory {
        type Random = DummyRandom;

        fn from_hash_of(&self, name: &str) -> Self::Random {
            DummyRandom {
                x: 0,
                y: 0,
                z: 0,
                seed: 0,
                name: name.to_string(),
            }
        }

        fn from_seed(&self, seed: i64) -> Self::Random {
            DummyRandom {
                x: 0,
                y: 0,
                z: 0,
                seed,
                name: String::new(),
            }
        }

        fn at(&self, x: i32, y: i32, z: i32) -> Self::Random {
            DummyRandom {
                x,
                y,
                z,
                seed: 0,
                name: String::new(),
            }
        }

        fn parity_config_string(&self, builder: &mut String) {
            builder.push_str("dummy-factory");
        }
    }

    #[test]
    fn at_block_pos_delegates_to_at_xyz() {
        let factory = DummyFactory;
        let pos = BlockPos::new(1, -2, 3);
        let r = factory.at_block_pos(&pos);
        assert_eq!(
            r,
            DummyRandom {
                x: 1,
                y: -2,
                z: 3,
                seed: 0,
                name: String::new()
            }
        );
    }

    #[test]
    fn from_hash_of_identifier_delegates_to_from_hash_of_string() {
        let factory = DummyFactory;
        let id = Identifier::new("minecraft:overworld");
        let r = factory.from_hash_of_identifier(&id);
        assert_eq!(r.name, "minecraft:overworld");
    }

    #[test]
    fn at_handles_negative_and_extreme_coords() {
        let factory = DummyFactory;
        let r = factory.at(i32::MIN, -1, i32::MAX);
        assert_eq!(r.x, i32::MIN);
        assert_eq!(r.y, -1);
        assert_eq!(r.z, i32::MAX);
    }

    #[test]
    fn from_seed_preserves_i64_bits() {
        let factory = DummyFactory;
        let seed = -0x1234_5678_9ABC_DEF0_i64;
        let r = factory.from_seed(seed);
        assert_eq!(r.seed, seed);
    }

    #[test]
    fn parity_config_string_appends() {
        let factory = DummyFactory;
        let mut s = String::from("prefix:");
        factory.parity_config_string(&mut s);
        assert_eq!(s, "prefix:dummy-factory");
    }
}
