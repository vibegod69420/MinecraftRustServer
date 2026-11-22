//! Faithful port of `net.minecraft.util.profiling.jfr.stats.FpsStat`.
//!
//! The JDK JFR `RecordedEvent` dependency is represented by a minimal local
//! trait so this module remains self-contained.

/// Minimal interface for the JDK JFR `RecordedEvent` API used by this stat.
///
/// Mirrors the `getInt(String)` capability of
/// `jdk.jfr.consumer.RecordedEvent`.
pub trait JfrRecordedEvent {
    /// Returns the integer field value for the given field name.
    ///
    /// Corresponds to `jdk.jfr.consumer.RecordedEvent#getInt(String)`.
    fn get_int(&self, name: &str) -> i32;
}

/// Faithful port of `net.minecraft.util.profiling.jfr.stats.FpsStat`.
///
/// Java record: `public record FpsStat(int fps)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FpsStat {
    /// The recorded frames-per-second value.
    pub fps: i32,
}

impl FpsStat {
    // JAVA: net.minecraft.util.profiling.jfr.stats.FpsStat#from
    pub fn from<E: JfrRecordedEvent>(event: &E, name: &str) -> Self {
        Self {
            fps: event.get_int(name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Debug)]
    struct ConstEvent(i32);

    impl JfrRecordedEvent for ConstEvent {
        fn get_int(&self, _name: &str) -> i32 {
            self.0
        }
    }

    #[derive(Debug)]
    struct NamedEvent {
        expected_name: String,
        value: i32,
    }

    impl JfrRecordedEvent for NamedEvent {
        fn get_int(&self, name: &str) -> i32 {
            assert_eq!(name, self.expected_name, "unexpected field name");
            self.value
        }
    }

    #[derive(Debug)]
    struct MapEvent {
        ints: BTreeMap<String, i32>,
    }

    impl JfrRecordedEvent for MapEvent {
        fn get_int(&self, name: &str) -> i32 {
            *self
                .ints
                .get(name)
                .unwrap_or_else(|| panic!("missing int field: {name}"))
        }
    }

    #[test]
    fn from_reads_fps_field() {
        let event = NamedEvent {
            expected_name: "fps".to_string(),
            value: 60,
        };
        let stat = FpsStat::from(&event, "fps");
        assert_eq!(stat, FpsStat { fps: 60 });
    }

    #[test]
    fn from_reads_arbitrary_field_name() {
        let event = NamedEvent {
            expected_name: "customFps".to_string(),
            value: 144,
        };
        let stat = FpsStat::from(&event, "customFps");
        assert_eq!(stat.fps, 144);
    }

    #[test]
    fn from_handles_empty_field_name() {
        let event = NamedEvent {
            expected_name: String::new(),
            value: 30,
        };
        let stat = FpsStat::from(&event, "");
        assert_eq!(stat.fps, 30);
    }

    #[test]
    fn from_with_map_backed_event_reads_correct_key() {
        let mut ints = BTreeMap::new();
        ints.insert("fps".to_string(), 120);
        ints.insert("other".to_string(), 999);
        let event = MapEvent { ints };

        let stat = FpsStat::from(&event, "fps");
        assert_eq!(stat.fps, 120);
    }

    #[test]
    fn from_matches_direct_construction() {
        let value = -12_345_i32;
        let from_event = FpsStat::from(&ConstEvent(value), "fps");
        let direct = FpsStat { fps: value };
        assert_eq!(from_event, direct);
    }

    #[test]
    fn from_handles_zero() {
        let stat = FpsStat::from(&ConstEvent(0), "fps");
        assert_eq!(stat.fps, 0);
    }

    #[test]
    fn from_handles_negative_inputs() {
        let cases = [
            -1_i32,
            -60,
            -144,
            -1_000,
            i32::MIN.wrapping_add(1),
            i32::MIN,
        ];
        for value in cases {
            let stat = FpsStat::from(&ConstEvent(value), "fps");
            assert_eq!(stat.fps, value, "negative value not preserved: {value}");
        }
    }

    #[test]
    fn from_handles_overflow_boundaries() {
        let max_stat = FpsStat::from(&ConstEvent(i32::MAX), "fps");
        assert_eq!(max_stat.fps, i32::MAX);

        let min_stat = FpsStat::from(&ConstEvent(i32::MIN), "fps");
        assert_eq!(min_stat.fps, i32::MIN);

        // Explicit wrapping boundary behavior: values are preserved as i32 bits.
        let wrapped = i32::MAX.wrapping_add(1);
        let wrapped_stat = FpsStat::from(&ConstEvent(wrapped), "fps");
        assert_eq!(wrapped_stat.fps, i32::MIN);
    }

    #[test]
    fn from_preserves_float_bit_patterns_f32() {
        let patterns: [u32; 10] = [
            0x0000_0000, // +0.0
            0x8000_0000, // -0.0
            0x0000_0001, // smallest positive subnormal
            0x007F_FFFF, // largest subnormal
            0x3F80_0000, // 1.0
            0xBF80_0000, // -1.0
            0x7F80_0000, // +Infinity
            0xFF80_0000, // -Infinity
            0x7FC0_0000, // quiet NaN
            0xFFC0_0001, // NaN payload with sign bit
        ];

        for bits in patterns {
            let value = bits as i32;
            let stat = FpsStat::from(&ConstEvent(value), "fps");
            assert_eq!(
                stat.fps as u32, bits,
                "f32 bit pattern not preserved for 0x{bits:08X}"
            );

            let recovered = f32::from_bits(stat.fps as u32);
            assert_eq!(
                recovered.to_bits(),
                bits,
                "f32 bit round-trip failed for 0x{bits:08X}"
            );
        }
    }

    #[test]
    fn from_preserves_float_bit_patterns_f64_split() {
        // FpsStat stores an i32, but this verifies bit-exact preservation for
        // values derived from notable f64 bit patterns.
        let patterns: [u64; 8] = [
            0x0000_0000_0000_0000, // +0.0
            0x8000_0000_0000_0000, // -0.0
            0x0000_0000_0000_0001, // smallest positive subnormal
            0x3FF0_0000_0000_0000, // 1.0
            0xBFF0_0000_0000_0000, // -1.0
            0x7FF0_0000_0000_0000, // +Infinity
            0xFFF0_0000_0000_0000, // -Infinity
            0x7FF8_0000_0000_0001, // NaN payload
        ];

        for bits in patterns {
            let low = bits as u32 as i32;
            let high = ((bits >> 32) as u32) as i32;

            let low_stat = FpsStat::from(&ConstEvent(low), "fps_low");
            let high_stat = FpsStat::from(&ConstEvent(high), "fps_high");

            let reconstructed =
                ((high_stat.fps as u32 as u64) << 32) | (low_stat.fps as u32 as u64);
            assert_eq!(
                reconstructed, bits,
                "f64 bit pattern not preserved for 0x{bits:016X}"
            );

            let recovered = f64::from_bits(reconstructed);
            assert_eq!(
                recovered.to_bits(),
                bits,
                "f64 bit round-trip failed for 0x{bits:016X}"
            );
        }
    }

    #[test]
    fn struct_is_copy_clone_eq_hash_debug() {
        use std::collections::HashSet;

        let a = FpsStat { fps: 42 };
        let b = a; // Copy
        let c = a.clone();

        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(a.fps, 42);

        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&FpsStat { fps: 42 }));

        let dbg = format!("{:?}", a);
        assert!(dbg.contains("FpsStat"));
        assert!(dbg.contains("42"));
    }
}
