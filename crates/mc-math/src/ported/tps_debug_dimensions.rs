//! Faithful port of `net.minecraft.util.debugchart.TpsDebugDimensions`.
//!
//! Source: `net.minecraft.util.debugchart.TpsDebugDimensions` (Minecraft Java 1.21.11).

/// Dimensions tracked by the TPS debug chart.
///
/// JAVA: `net.minecraft.util.debugchart.TpsDebugDimensions`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum TpsDebugDimensions {
    /// JAVA: `net.minecraft.util.debugchart.TpsDebugDimensions#FULL_TICK`
    FullTick = 0,
    /// JAVA: `net.minecraft.util.debugchart.TpsDebugDimensions#TICK_SERVER_METHOD`
    TickServerMethod = 1,
    /// JAVA: `net.minecraft.util.debugchart.TpsDebugDimensions#SCHEDULED_TASKS`
    ScheduledTasks = 2,
    /// JAVA: `net.minecraft.util.debugchart.TpsDebugDimensions#IDLE`
    Idle = 3,
}

impl TpsDebugDimensions {
    /// Returns all enum constants in declaration order.
    ///
    /// In Java, `values()` returns a newly allocated array. Returning a fixed
    /// array by value preserves the copy-per-call behavior in Rust.
    // JAVA: net.minecraft.util.debugchart.TpsDebugDimensions#values
    pub const fn values() -> [TpsDebugDimensions; 4] {
        [
            TpsDebugDimensions::FullTick,
            TpsDebugDimensions::TickServerMethod,
            TpsDebugDimensions::ScheduledTasks,
            TpsDebugDimensions::Idle,
        ]
    }

    /// Returns the declaration ordinal of this constant, matching Java
    /// `Enum#ordinal()`.
    // JAVA: net.minecraft.util.debugchart.TpsDebugDimensions#ordinal
    pub const fn ordinal(self) -> i32 {
        self as i32
    }

    /// Returns the Java enum constant name.
    // JAVA: net.minecraft.util.debugchart.TpsDebugDimensions#name
    pub const fn name(self) -> &'static str {
        match self {
            TpsDebugDimensions::FullTick => "FULL_TICK",
            TpsDebugDimensions::TickServerMethod => "TICK_SERVER_METHOD",
            TpsDebugDimensions::ScheduledTasks => "SCHEDULED_TASKS",
            TpsDebugDimensions::Idle => "IDLE",
        }
    }

    /// Returns the enum constant with the exact Java name.
    ///
    /// Java throws `IllegalArgumentException` for unknown names and
    /// `NullPointerException` for null. Rust uses `&str` (non-null) and returns
    /// `Result` with a Java-style error message for unknown names.
    // JAVA: net.minecraft.util.debugchart.TpsDebugDimensions#valueOf
    pub fn value_of(name: &str) -> Result<Self, String> {
        match name {
            "FULL_TICK" => Ok(TpsDebugDimensions::FullTick),
            "TICK_SERVER_METHOD" => Ok(TpsDebugDimensions::TickServerMethod),
            "SCHEDULED_TASKS" => Ok(TpsDebugDimensions::ScheduledTasks),
            "IDLE" => Ok(TpsDebugDimensions::Idle),
            _ => Err(format!(
                "No enum constant net.minecraft.util.debugchart.TpsDebugDimensions.{}",
                name
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TpsDebugDimensions;

    #[test]
    fn ordinal_matches_java_declaration_order() {
        assert_eq!(TpsDebugDimensions::FullTick.ordinal(), 0);
        assert_eq!(TpsDebugDimensions::TickServerMethod.ordinal(), 1);
        assert_eq!(TpsDebugDimensions::ScheduledTasks.ordinal(), 2);
        assert_eq!(TpsDebugDimensions::Idle.ordinal(), 3);
    }

    #[test]
    fn values_matches_java_declaration_order() {
        let values = TpsDebugDimensions::values();
        assert_eq!(values.len(), 4);
        assert_eq!(
            values,
            [
                TpsDebugDimensions::FullTick,
                TpsDebugDimensions::TickServerMethod,
                TpsDebugDimensions::ScheduledTasks,
                TpsDebugDimensions::Idle,
            ]
        );
    }

    #[test]
    fn values_returns_a_copy_each_call() {
        let mut first = TpsDebugDimensions::values();
        let second = TpsDebugDimensions::values();
        first[0] = TpsDebugDimensions::Idle;
        assert_eq!(second[0], TpsDebugDimensions::FullTick);
        assert_eq!(TpsDebugDimensions::values()[0], TpsDebugDimensions::FullTick);
    }

    #[test]
    fn name_matches_java_identifiers() {
        assert_eq!(TpsDebugDimensions::FullTick.name(), "FULL_TICK");
        assert_eq!(
            TpsDebugDimensions::TickServerMethod.name(),
            "TICK_SERVER_METHOD"
        );
        assert_eq!(TpsDebugDimensions::ScheduledTasks.name(), "SCHEDULED_TASKS");
        assert_eq!(TpsDebugDimensions::Idle.name(), "IDLE");
    }

    #[test]
    fn value_of_round_trip_all_variants() {
        for &dim in &TpsDebugDimensions::values() {
            let parsed =
                TpsDebugDimensions::value_of(dim.name()).expect("value_of should parse name");
            assert_eq!(parsed, dim);
            assert_eq!(parsed.ordinal(), dim.ordinal());
            assert_eq!(parsed.name(), dim.name());
        }
    }

    #[test]
    fn value_of_rejects_invalid_names() {
        let invalid = [
            "",
            " ",
            "FULL_TICK ",
            " FULL_TICK",
            "full_tick",
            "FullTick",
            "TICK_SERVER_METHOD\n",
            "SCHEDULED_TASKS\0",
            "IDLE_EXTRA",
            "UNKNOWN",
            "0",
            "-1",
        ];
        for name in invalid {
            assert!(
                TpsDebugDimensions::value_of(name).is_err(),
                "expected value_of to reject {:?}",
                name
            );
        }
    }

    #[test]
    fn value_of_error_message_matches_java_style() {
        let err = TpsDebugDimensions::value_of("NOPE").unwrap_err();
        assert_eq!(
            err,
            "No enum constant net.minecraft.util.debugchart.TpsDebugDimensions.NOPE"
        );

        let empty_err = TpsDebugDimensions::value_of("").unwrap_err();
        assert_eq!(
            empty_err,
            "No enum constant net.minecraft.util.debugchart.TpsDebugDimensions."
        );
    }

    #[test]
    fn ordering_matches_declaration_ordinal() {
        assert!(TpsDebugDimensions::FullTick < TpsDebugDimensions::TickServerMethod);
        assert!(TpsDebugDimensions::TickServerMethod < TpsDebugDimensions::ScheduledTasks);
        assert!(TpsDebugDimensions::ScheduledTasks < TpsDebugDimensions::Idle);

        let mut vals = [
            TpsDebugDimensions::Idle,
            TpsDebugDimensions::FullTick,
            TpsDebugDimensions::ScheduledTasks,
            TpsDebugDimensions::TickServerMethod,
        ];
        vals.sort();
        assert_eq!(vals, TpsDebugDimensions::values());
    }

    #[test]
    fn ordinal_wrapping_arithmetic_edge_cases() {
        // Java int semantics for ordinal arithmetic.
        for &dim in &TpsDebugDimensions::values() {
            let ord = dim.ordinal();
            assert_eq!(ord.wrapping_add(1).wrapping_sub(1), ord);
            assert_eq!(ord.wrapping_sub(1).wrapping_add(1), ord);
            assert_eq!(ord.wrapping_mul(2), ord.wrapping_shl(1));
            assert_eq!(ord.wrapping_mul(0), 0);
            assert_eq!(ord.wrapping_neg().wrapping_neg(), ord);
        }

        let min = i32::MIN;
        let max = i32::MAX;
        assert_eq!(min.wrapping_sub(1), max);
        assert_eq!(max.wrapping_add(1), min);
        assert_eq!(min.wrapping_mul(-1), min);
        assert_eq!((-1i32).wrapping_mul(min), min);
        assert_eq!(max.wrapping_mul(2), -2);
    }

    #[test]
    fn negative_and_boundary_ordinal_lookups() {
        // No variant has a negative or out-of-range ordinal.
        let invalid_ordinals = [i32::MIN, -128, -1, 4, 5, 255, i32::MAX];
        for ord in invalid_ordinals {
            let found = TpsDebugDimensions::values()
                .iter()
                .copied()
                .find(|d| d.ordinal() == ord);
            assert!(found.is_none(), "ordinal {ord} should not map to a variant");
        }

        // Boundary valid ordinals map to the expected variants.
        assert_eq!(
            TpsDebugDimensions::values()
                .iter()
                .copied()
                .find(|d| d.ordinal() == 0),
            Some(TpsDebugDimensions::FullTick)
        );
        assert_eq!(
            TpsDebugDimensions::values()
                .iter()
                .copied()
                .find(|d| d.ordinal() == 3),
            Some(TpsDebugDimensions::Idle)
        );
    }

    #[test]
    fn ordinal_float_bit_representations() {
        // Small integer ordinals have exact f32/f64 representations; verify bit patterns.
        for &dim in &TpsDebugDimensions::values() {
            let ord = dim.ordinal();
            let f32_val = ord as f32;
            let f64_val = ord as f64;

            assert_eq!(f32_val as i32, ord);
            assert_eq!(f64_val as i32, ord);

            let f32_bits = f32_val.to_bits();
            let f64_bits = f64_val.to_bits();
            assert_eq!(f32::from_bits(f32_bits), f32_val);
            assert_eq!(f64::from_bits(f64_bits), f64_val);

            let (expected_f32_bits, expected_f64_bits) = match ord {
                0 => (0.0f32.to_bits(), 0.0f64.to_bits()),
                1 => (1.0f32.to_bits(), 1.0f64.to_bits()),
                2 => (2.0f32.to_bits(), 2.0f64.to_bits()),
                3 => (3.0f32.to_bits(), 3.0f64.to_bits()),
                _ => panic!("unexpected ordinal {ord}"),
            };
            assert_eq!(f32_bits, expected_f32_bits);
            assert_eq!(f64_bits, expected_f64_bits);
        }

        // Explicit negative float bit edge case: -0.0 differs from +0.0 in bits.
        let neg_zero_f32 = (-0.0f32).to_bits();
        let pos_zero_f32 = 0.0f32.to_bits();
        assert_ne!(neg_zero_f32, pos_zero_f32);
        assert_eq!(f32::from_bits(neg_zero_f32), 0.0f32);

        let neg_zero_f64 = (-0.0f64).to_bits();
        let pos_zero_f64 = 0.0f64.to_bits();
        assert_ne!(neg_zero_f64, pos_zero_f64);
        assert_eq!(f64::from_bits(neg_zero_f64), 0.0f64);
    }

    #[test]
    fn enum_copy_eq_hash_debug_are_usable() {
        use std::collections::HashSet;

        let a = TpsDebugDimensions::Idle;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(TpsDebugDimensions::FullTick, TpsDebugDimensions::Idle);

        let mut set = HashSet::new();
        for &dim in &TpsDebugDimensions::values() {
            assert!(set.insert(dim));
        }
        assert_eq!(set.len(), 4);
        assert!(set.contains(&TpsDebugDimensions::ScheduledTasks));

        let dbg = format!("{:?}", TpsDebugDimensions::TickServerMethod);
        assert_eq!(dbg, "TickServerMethod");
    }

    #[test]
    fn repr_i32_discriminants_are_stable() {
        assert_eq!(TpsDebugDimensions::FullTick as i32, 0i32);
        assert_eq!(TpsDebugDimensions::TickServerMethod as i32, 1i32);
        assert_eq!(TpsDebugDimensions::ScheduledTasks as i32, 2i32);
        assert_eq!(TpsDebugDimensions::Idle as i32, 3i32);

        // Masked shifts are not part of enum semantics, but Java int shift masking
        // must be respected when ordinals participate in bit operations.
        for &dim in &TpsDebugDimensions::values() {
            let ord = dim.ordinal();
            assert_eq!(ord.wrapping_shl(32), ord.wrapping_shl(32 & 31));
            assert_eq!(ord.wrapping_shr(32), ord.wrapping_shr(32 & 31));
            let ord64 = ord as i64;
            assert_eq!(ord64.wrapping_shl(64), ord64.wrapping_shl(64 & 63));
        }
    }
}
