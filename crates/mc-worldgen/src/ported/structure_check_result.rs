//! Port of `net.minecraft.world.level.levelgen.structure.StructureCheckResult`.
//!
//! Java source: `net/minecraft/world/level/levelgen/structure/StructureCheckResult.java`

/// Result of checking whether a structure start is present in a chunk.
///
/// This enum mirrors the Java enum declaration order so that discriminant values
/// match Java `Enum.ordinal()` semantics:
/// - `START_PRESENT`      = 0
/// - `START_NOT_PRESENT`  = 1
/// - `CHUNK_LOAD_NEEDED`  = 2
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructureCheckResult {
    /// JAVA: `START_PRESENT`
    StartPresent = 0,
    /// JAVA: `START_NOT_PRESENT`
    StartNotPresent = 1,
    /// JAVA: `CHUNK_LOAD_NEEDED`
    ChunkLoadNeeded = 2,
}

#[cfg(test)]
mod tests {
    use super::StructureCheckResult;
    use std::collections::hash_map::DefaultHasher;
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};

    fn hash_of(value: StructureCheckResult) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn ordinal_values_match_java_declaration_order() {
        // JAVA: START_PRESENT=0, START_NOT_PRESENT=1, CHUNK_LOAD_NEEDED=2
        assert_eq!(StructureCheckResult::StartPresent as i32, 0_i32);
        assert_eq!(StructureCheckResult::StartNotPresent as i32, 1_i32);
        assert_eq!(StructureCheckResult::ChunkLoadNeeded as i32, 2_i32);
    }

    #[test]
    fn variants_are_distinct() {
        let all = [
            StructureCheckResult::StartPresent,
            StructureCheckResult::StartNotPresent,
            StructureCheckResult::ChunkLoadNeeded,
        ];

        for (i, left) in all.iter().enumerate() {
            for (j, right) in all.iter().enumerate() {
                if i == j {
                    assert_eq!(left, right);
                } else {
                    assert_ne!(left, right);
                }
            }
        }
    }

    #[test]
    fn copy_and_clone_semantics() {
        let original = StructureCheckResult::StartPresent;
        let copied = original;
        let cloned = original.clone();

        assert_eq!(original, copied);
        assert_eq!(original, cloned);

        let original = StructureCheckResult::ChunkLoadNeeded;
        let copied = original;
        let cloned = original.clone();
        assert_eq!(original, copied);
        assert_eq!(original, cloned);
    }

    #[test]
    fn debug_format_matches_variant_names() {
        assert_eq!(
            format!("{:?}", StructureCheckResult::StartPresent),
            "StartPresent"
        );
        assert_eq!(
            format!("{:?}", StructureCheckResult::StartNotPresent),
            "StartNotPresent"
        );
        assert_eq!(
            format!("{:?}", StructureCheckResult::ChunkLoadNeeded),
            "ChunkLoadNeeded"
        );
    }

    #[test]
    fn ordering_matches_java_ordinal_order() {
        assert!(StructureCheckResult::StartPresent < StructureCheckResult::StartNotPresent);
        assert!(StructureCheckResult::StartNotPresent < StructureCheckResult::ChunkLoadNeeded);
        assert!(StructureCheckResult::StartPresent < StructureCheckResult::ChunkLoadNeeded);

        let mut variants = [
            StructureCheckResult::ChunkLoadNeeded,
            StructureCheckResult::StartPresent,
            StructureCheckResult::StartNotPresent,
        ];
        variants.sort();
        assert_eq!(
            variants,
            [
                StructureCheckResult::StartPresent,
                StructureCheckResult::StartNotPresent,
                StructureCheckResult::ChunkLoadNeeded,
            ]
        );
    }

    #[test]
    fn hash_is_consistent_and_distinguishes_variants() {
        let present_a = hash_of(StructureCheckResult::StartPresent);
        let present_b = hash_of(StructureCheckResult::StartPresent);
        let not_present = hash_of(StructureCheckResult::StartNotPresent);
        let needed = hash_of(StructureCheckResult::ChunkLoadNeeded);

        assert_eq!(present_a, present_b);
        assert_ne!(present_a, not_present);
        assert_ne!(present_a, needed);
        assert_ne!(not_present, needed);

        let mut set = HashSet::new();
        assert!(set.insert(StructureCheckResult::StartPresent));
        assert!(set.insert(StructureCheckResult::StartNotPresent));
        assert!(set.insert(StructureCheckResult::ChunkLoadNeeded));
        assert!(!set.insert(StructureCheckResult::StartPresent));
        assert_eq!(set.len(), 3);
        assert!(set.contains(&StructureCheckResult::StartPresent));
        assert!(set.contains(&StructureCheckResult::StartNotPresent));
        assert!(set.contains(&StructureCheckResult::ChunkLoadNeeded));
    }

    #[test]
    fn exhaustive_match_covers_all_variants() {
        fn label(value: StructureCheckResult) -> &'static str {
            match value {
                StructureCheckResult::StartPresent => "present",
                StructureCheckResult::StartNotPresent => "not_present",
                StructureCheckResult::ChunkLoadNeeded => "chunk_load_needed",
            }
        }

        assert_eq!(label(StructureCheckResult::StartPresent), "present");
        assert_eq!(
            label(StructureCheckResult::StartNotPresent),
            "not_present"
        );
        assert_eq!(
            label(StructureCheckResult::ChunkLoadNeeded),
            "chunk_load_needed"
        );
    }

    #[test]
    fn ordinal_integer_edge_cases_wrapping_and_negative_inputs() {
        let present = StructureCheckResult::StartPresent as i32;
        let not_present = StructureCheckResult::StartNotPresent as i32;
        let needed = StructureCheckResult::ChunkLoadNeeded as i32;

        // Negative inputs
        assert_eq!((-1_i32).wrapping_add(present), -1_i32);
        assert_eq!((-1_i32).wrapping_add(not_present), 0_i32);
        assert_eq!((-10_i32).wrapping_mul(needed), -20_i32);

        // Overflow/wrapping behavior
        assert_eq!(present.wrapping_sub(1), -1_i32);
        assert_eq!(i32::MAX.wrapping_add(needed), i32::MIN.wrapping_add(1));
        assert_eq!(i32::MIN.wrapping_sub(not_present), i32::MAX);

        // Masked shift semantics on ordinal-derived values (Java shift mask style)
        let shift_amount: i32 = 33; // 33 & 31 == 1
        assert_eq!(not_present.wrapping_shl((shift_amount as u32) & 31), 2_i32);
        let shift_amount_64: i64 = 65; // 65 & 63 == 1
        assert_eq!(
            (needed as i64).wrapping_shl((shift_amount_64 as u32) & 63),
            4_i64
        );
    }

    #[test]
    fn ordinal_float_bit_representations_are_stable() {
        // Float bit checks for ordinal integer values.
        for (ordinal, variant) in [
            (0_i32, StructureCheckResult::StartPresent),
            (1_i32, StructureCheckResult::StartNotPresent),
            (2_i32, StructureCheckResult::ChunkLoadNeeded),
        ] {
            assert_eq!(variant as i32, ordinal);

            let u32_bits = ordinal as u32;
            let f32_val = f32::from_bits(u32_bits);
            assert_eq!(f32_val.to_bits(), u32_bits);

            let u64_bits = ordinal as u64;
            let f64_val = f64::from_bits(u64_bits);
            assert_eq!(f64_val.to_bits(), u64_bits);

            // Ensure round-trip through signed integer bit patterns is exact.
            let i32_bits = ordinal.to_ne_bytes();
            assert_eq!(i32::from_ne_bytes(i32_bits), ordinal);
        }

        // Explicit negative-input float bit pattern check.
        let neg_one: i32 = -1;
        let neg_one_u32 = neg_one as u32;
        let neg_one_f32 = f32::from_bits(neg_one_u32);
        assert_eq!(neg_one_f32.to_bits(), neg_one_u32);
        assert!(neg_one_f32.is_nan());

        let neg_one_u64 = (neg_one as i64) as u64;
        let neg_one_f64 = f64::from_bits(neg_one_u64);
        assert_eq!(neg_one_f64.to_bits(), neg_one_u64);
        assert!(neg_one_f64.is_nan());
    }
}
