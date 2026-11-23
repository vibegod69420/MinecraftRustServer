use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(c as i32);
    }
    h
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldOptions {
    seed: i64,
    generate_structures: bool,
    generate_bonus_chest: bool,
    legacy_custom_options: Option<String>,
}

pub static DEMO_OPTIONS: LazyLock<WorldOptions> = LazyLock::new(|| {
    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#DEMO_OPTIONS
    WorldOptions::new(java_string_hash("North Carolina") as i64, true, true)
});

impl WorldOptions {
    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#<init>
    pub fn new(seed: i64, generate_structures: bool, generate_bonus_chest: bool) -> Self {
        Self::new_full(seed, generate_structures, generate_bonus_chest, None)
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#<init>
    fn new_full(
        seed: i64,
        generate_structures: bool,
        generate_bonus_chest: bool,
        legacy_custom_options: Option<String>,
    ) -> Self {
        Self {
            seed,
            generate_structures,
            generate_bonus_chest,
            legacy_custom_options,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#defaultWithRandomSeed
    pub fn default_with_random_seed() -> Self {
        Self::new(Self::random_seed(), true, false)
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#testWorldWithRandomSeed
    pub fn test_world_with_random_seed() -> Self {
        Self::new(Self::random_seed(), false, false)
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#seed
    pub fn seed(&self) -> i64 {
        self.seed
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#generateStructures
    pub fn generate_structures(&self) -> bool {
        self.generate_structures
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#generateBonusChest
    pub fn generate_bonus_chest(&self) -> bool {
        self.generate_bonus_chest
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#isOldCustomizedWorld
    pub fn is_old_customized_world(&self) -> bool {
        self.legacy_custom_options.is_some()
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#withBonusChest
    pub fn with_bonus_chest(&self, generate_bonus_chest: bool) -> Self {
        Self::new_full(
            self.seed,
            self.generate_structures,
            generate_bonus_chest,
            self.legacy_custom_options.clone(),
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#withStructures
    pub fn with_structures(&self, generate_structures: bool) -> Self {
        Self::new_full(
            self.seed,
            generate_structures,
            self.generate_bonus_chest,
            self.legacy_custom_options.clone(),
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#withSeed
    pub fn with_seed(&self, seed: Option<i64>) -> Self {
        Self::new_full(
            seed.unwrap_or_else(Self::random_seed),
            self.generate_structures,
            self.generate_bonus_chest,
            self.legacy_custom_options.clone(),
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#parseSeed
    pub fn parse_seed(s: &str) -> Option<i64> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        match trimmed.parse::<i64>() {
            Ok(v) => Some(v),
            Err(_) => Some(java_string_hash(trimmed) as i64),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldOptions#randomSeed
    pub fn random_seed() -> i64 {
        // Self-contained random seed source; Java uses RandomSource.create().nextLong().
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let mut z = nanos.wrapping_add(0x9E3779B97F4A7C15_u64);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9_u64);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB_u64);
        (z ^ (z >> 31)) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_seed_empty_and_whitespace() {
        assert_eq!(WorldOptions::parse_seed(""), None);
        assert_eq!(WorldOptions::parse_seed("   \t\n"), None);
    }

    #[test]
    fn parse_seed_numeric_and_negative() {
        assert_eq!(WorldOptions::parse_seed("123"), Some(123));
        assert_eq!(WorldOptions::parse_seed("  -42  "), Some(-42));
        assert_eq!(
            WorldOptions::parse_seed(&i64::MIN.to_string()),
            Some(i64::MIN)
        );
    }

    #[test]
    fn parse_seed_overflow_falls_back_to_java_hash() {
        let s = "9223372036854775808"; // i64::MAX + 1
        assert_eq!(
            WorldOptions::parse_seed(s),
            Some(java_string_hash(s) as i64)
        );
        let non_numeric = "North Carolina";
        assert_eq!(
            WorldOptions::parse_seed(non_numeric),
            Some(java_string_hash(non_numeric) as i64)
        );
    }

    #[test]
    fn demo_options_and_defaults() {
        let demo = DEMO_OPTIONS.clone();
        assert_eq!(demo.seed(), java_string_hash("North Carolina") as i64);
        assert!(demo.generate_structures());
        assert!(demo.generate_bonus_chest());
        assert!(!demo.is_old_customized_world());

        let def = WorldOptions::default_with_random_seed();
        assert!(def.generate_structures());
        assert!(!def.generate_bonus_chest());

        let test = WorldOptions::test_world_with_random_seed();
        assert!(!test.generate_structures());
        assert!(!test.generate_bonus_chest());
    }

    #[test]
    fn withers_preserve_and_override_fields() {
        let base = WorldOptions::new(123, true, false);
        let bonus = base.with_bonus_chest(true);
        assert_eq!(bonus.seed(), 123);
        assert!(bonus.generate_structures());
        assert!(bonus.generate_bonus_chest());

        let no_struct = bonus.with_structures(false);
        assert_eq!(no_struct.seed(), 123);
        assert!(!no_struct.generate_structures());
        assert!(no_struct.generate_bonus_chest());

        let seeded = no_struct.with_seed(Some(999));
        assert_eq!(seeded.seed(), 999);
        assert!(!seeded.generate_structures());
        assert!(seeded.generate_bonus_chest());
    }

    #[test]
    fn java_string_hash_uses_utf16_wrapping_semantics() {
        assert_eq!(java_string_hash(""), 0);
        assert_eq!(java_string_hash("a"), 97);
        // U+10000 is encoded as a UTF-16 surrogate pair in Java hashCode iteration.
        let s = "\u{10000}";
        let units: Vec<u16> = s.encode_utf16().collect();
        let mut expected: i32 = 0;
        for u in units {
            expected = expected.wrapping_mul(31).wrapping_add(u as i32);
        }
        assert_eq!(java_string_hash(s), expected);
    }
}
