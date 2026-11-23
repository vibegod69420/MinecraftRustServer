//! Port of `net.minecraft.network.protocol.game.DebugEntityNameGenerator`.

/// Trait abstraction for the minimal entity surface needed by this generator.
///
/// This keeps the module self-contained while matching the Java call shape:
/// - `get_plain_text_name()` for player-name fast path
/// - `get_custom_name()` for named non-player entities
/// - `get_uuid()` for deterministic random-name generation
pub trait DebugEntity {
    fn is_player(&self) -> bool;
    fn get_plain_text_name(&self) -> String;
    fn get_custom_name(&self) -> Option<String>;
    fn get_uuid(&self) -> u128;
}

const NAMES_FIRST_PART: [&str; 33] = [
    "Slim", "Far", "River", "Silly", "Fat", "Thin", "Fish", "Bat", "Dark", "Oak", "Sly", "Bush",
    "Zen", "Bark", "Cry", "Slack", "Soup", "Grim", "Hook", "Dirt", "Mud", "Sad", "Hard", "Crook",
    "Sneak", "Stink", "Weird", "Fire", "Soot", "Soft", "Rough", "Cling", "Scar",
];

const NAMES_SECOND_PART: [&str; 30] = [
    "Fox", "Tail", "Jaw", "Whisper", "Twig", "Root", "Finder", "Nose", "Brow", "Blade", "Fry",
    "Seek", "Wart", "Tooth", "Foot", "Leaf", "Stone", "Fall", "Face", "Tongue", "Voice", "Lip",
    "Mouth", "Snail", "Toe", "Ear", "Hair", "Beard", "Shirt", "Fist",
];

/// Minimal deterministic random source matching the Java behavior used here:
/// `RandomSource.create(seed)` + `nextInt(bound)`.
#[derive(Debug, Clone)]
struct RandomSource {
    seed: i64,
}

impl RandomSource {
    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#<init>(long) / RandomSource#create(long)
    fn create(seed: i64) -> Self {
        // Java RandomSource.create(long) uses a linear congruential generator seed mix.
        // LegacyRandomSource / Random constructors mix with (seed ^ multiplier) & mask,
        // but RandomSource.create(long) in modern MC returns a LinearCongruentialGenerator-backed source.
        // For the exact behavior used by DebugEntityNameGenerator (nextInt on small bounds),
        // we port the Java Random LCG used by RandomSource.create(long) implementation path:
        // new LegacyRandomSource(seed) -> seed = (seed ^ 25214903917L) & ((1L<<48)-1)
        let mixed = (seed ^ 0x0005_DEEC_E66Di64) & ((1i64 << 48) - 1);
        Self { seed: mixed }
    }

    // JAVA: java.util.Random#next(int)
    fn next_bits(&mut self, bits: i32) -> i32 {
        self.seed = (self
            .seed
            .wrapping_mul(0x0005_DEEC_E66Di64)
            .wrapping_add(0xBi64))
            & ((1i64 << 48) - 1);
        ((self.seed as u64) >> (48 - bits)) as i32
    }

    // JAVA: java.util.Random#nextInt(int)
    fn next_int_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        let m = bound.wrapping_sub(1);
        if (bound & m) == 0 {
            // power of two
            let bits = ((bound as i64).wrapping_mul((self.next_bits(31) as i64)) >> 31) as i32;
            return bits;
        }

        loop {
            let bits = self.next_bits(31);
            let val = bits % bound;
            if bits.wrapping_sub(val).wrapping_add(m) >= 0 {
                return val;
            }
        }
    }
}

// JAVA: net.minecraft.network.protocol.game.DebugEntityNameGenerator#getEntityName(net.minecraft.world.entity.Entity)
pub fn get_entity_name(entity: &impl DebugEntity) -> String {
    if entity.is_player() {
        return entity.get_plain_text_name();
    }

    if let Some(custom_name) = entity.get_custom_name() {
        return custom_name;
    }

    get_entity_name_from_uuid(entity.get_uuid())
}

// JAVA: net.minecraft.network.protocol.game.DebugEntityNameGenerator#getEntityName(java.util.UUID)
pub fn get_entity_name_from_uuid(uuid: u128) -> String {
    let mut random = get_random(uuid);
    let first = get_random_string(&mut random, &NAMES_FIRST_PART);
    let second = get_random_string(&mut random, &NAMES_SECOND_PART);
    let mut out = String::with_capacity(first.len() + second.len());
    out.push_str(first);
    out.push_str(second);
    out
}

// JAVA: net.minecraft.network.protocol.game.DebugEntityNameGenerator#getRandomString(net.minecraft.util.RandomSource, java.lang.String[])
fn get_random_string<'a>(random: &mut RandomSource, parts: &[&'a str]) -> &'a str {
    get_random_element(parts, random)
}

// JAVA: net.minecraft.Util#getRandom(T[], net.minecraft.util.RandomSource)
fn get_random_element<'a>(parts: &[&'a str], random: &mut RandomSource) -> &'a str {
    let idx = random.next_int_bound(parts.len() as i32) as usize;
    parts[idx]
}

// JAVA: net.minecraft.network.protocol.game.DebugEntityNameGenerator#getRandom(java.util.UUID)
fn get_random(uuid: u128) -> RandomSource {
    // Java UUID.hashCode():
    // long hilo = mostSigBits ^ leastSigBits;
    // return ((int)(hilo >> 32)) ^ (int)hilo;
    let msb = (uuid >> 64) as u64 as i64;
    let lsb = (uuid as u64) as i64;
    let hilo = msb ^ lsb;
    let hash = (((hilo as u64) >> 32) as i32) ^ (hilo as i32);
    let seed = (hash >> 2) as i64;
    RandomSource::create(seed)
}

/// Helper for UUID conversion when you have standard Java UUID components.
pub fn uuid_from_most_least(most_significant: i64, least_significant: i64) -> u128 {
    ((most_significant as u64 as u128) << 64) | (least_significant as u64 as u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntity {
        player: bool,
        plain_name: String,
        custom_name: Option<String>,
        uuid: u128,
    }

    impl DebugEntity for TestEntity {
        fn is_player(&self) -> bool {
            self.player
        }

        fn get_plain_text_name(&self) -> String {
            self.plain_name.clone()
        }

        fn get_custom_name(&self) -> Option<String> {
            self.custom_name.clone()
        }

        fn get_uuid(&self) -> u128 {
            self.uuid
        }
    }

    #[test]
    fn player_uses_plain_text_name() {
        let e = TestEntity {
            player: true,
            plain_name: "Steve".to_string(),
            custom_name: Some("Ignored".to_string()),
            uuid: 0,
        };
        assert_eq!(get_entity_name(&e), "Steve");
    }

    #[test]
    fn custom_name_takes_precedence_for_non_player() {
        let e = TestEntity {
            player: false,
            plain_name: "Entity".to_string(),
            custom_name: Some("Bob".to_string()),
            uuid: uuid_from_most_least(1, 2),
        };
        assert_eq!(get_entity_name(&e), "Bob");
    }

    #[test]
    fn uuid_name_is_deterministic() {
        let uuid = uuid_from_most_least(0x0123_4567_89ab_cdef_u64 as i64, 0xfedc_ba98_7654_3210_u64 as i64);
        let a = get_entity_name_from_uuid(uuid);
        let b = get_entity_name_from_uuid(uuid);
        assert_eq!(a, b);
        assert!(!a.is_empty());
        assert!(NAMES_FIRST_PART.iter().any(|p| a.starts_with(p)));
        assert!(NAMES_SECOND_PART.iter().any(|p| a.ends_with(p)));
    }

    #[test]
    fn uuid_hash_shift_and_negative_seed_behavior() {
        // Same UUID should always produce same name; includes negative hash/seed paths.
        let uuid = uuid_from_most_least(-1i64, i64::MIN);
        let name = get_entity_name_from_uuid(uuid);
        let again = get_entity_name_from_uuid(uuid);
        assert_eq!(name, again);

        let random = get_random(uuid);
        // Seed is derived from (hashCode() >> 2), verify constructor does not panic for negative seeds.
        assert!(random.seed >= 0);
    }

    #[test]
    fn random_next_int_bound_power_of_two_and_non_power_of_two() {
        let mut r1 = RandomSource::create(12345);
        for _ in 0..100 {
            let v = r1.next_int_bound(16);
            assert!((0..16).contains(&v));
        }

        let mut r2 = RandomSource::create(-987654321);
        for _ in 0..100 {
            let v = r2.next_int_bound(30);
            assert!((0..30).contains(&v));
        }
    }

    #[test]
    fn get_random_element_uses_java_lcg_sequence() {
        // Verify deterministic sequence for a known seed through our LCG implementation.
        let mut r = RandomSource::create(0);
        let first = get_random_string(&mut r, &NAMES_FIRST_PART);
        let second = get_random_string(&mut r, &NAMES_SECOND_PART);
        let name = format!("{first}{second}");
        assert_eq!(name, get_entity_name_from_uuid(uuid_from_most_least(0, 0)));
    }
}
