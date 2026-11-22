// === FILE: holder_owner.rs ===

use std::ptr;

/// Faithful port of `net.minecraft.core.HolderOwner<T>`.
///
/// Controls whether holders owned by this owner can be serialized in another
/// owner context. The default Java behavior is reference identity (`this == other`).
pub trait HolderOwner<T> {
    // JAVA: net.minecraft.core.HolderOwner#canSerializeIn
    fn can_serialize_in(&self, other: &dyn HolderOwner<T>) -> bool {
        let self_data = self as *const Self as *const ();
        let other_data = other as *const dyn HolderOwner<T> as *const ();
        ptr::eq(self_data, other_data)
    }
}

#[cfg(test)]
mod tests {
    use super::HolderOwner;

    #[derive(Debug)]
    struct IntOwner {
        id: i32,
    }

    impl HolderOwner<i32> for IntOwner {}

    #[derive(Debug)]
    struct FloatOwner {
        f32_bits: u32,
        f64_bits: u64,
    }

    impl HolderOwner<f32> for FloatOwner {}

    struct PermissiveOwner;

    impl HolderOwner<i32> for PermissiveOwner {
        // JAVA: net.minecraft.core.HolderOwner#canSerializeIn
        fn can_serialize_in(&self, _other: &dyn HolderOwner<i32>) -> bool {
            true
        }
    }

    #[test]
    fn same_owner_can_serialize() {
        let owner = IntOwner { id: 42 };
        let same: &dyn HolderOwner<i32> = &owner;
        assert!(owner.can_serialize_in(same));
    }

    #[test]
    fn different_owners_cannot_serialize() {
        let a = IntOwner { id: 7 };
        let b = IntOwner { id: 7 };
        assert!(!a.can_serialize_in(&b as &dyn HolderOwner<i32>));
    }

    #[test]
    fn identity_with_negative_and_overflow_edge_values() {
        let min_owner = IntOwner { id: i32::MIN };
        let max_owner = IntOwner { id: i32::MAX };
        let neg_owner = IntOwner { id: -1 };
        let wrapped_owner = IntOwner {
            id: i32::MAX.wrapping_add(1),
        };

        let min_ref: &dyn HolderOwner<i32> = &min_owner;
        assert!(min_owner.can_serialize_in(min_ref));
        assert_eq!(wrapped_owner.id, i32::MIN);
        assert!(!min_owner.can_serialize_in(&wrapped_owner as &dyn HolderOwner<i32>));
        assert!(!max_owner.can_serialize_in(&neg_owner as &dyn HolderOwner<i32>));
    }

    #[test]
    fn identity_with_float_bit_patterns() {
        let nan_owner = FloatOwner {
            f32_bits: f32::NAN.to_bits(),
            f64_bits: f64::NAN.to_bits(),
        };
        let neg_zero_owner = FloatOwner {
            f32_bits: (-0.0f32).to_bits(),
            f64_bits: (-0.0f64).to_bits(),
        };
        let inf_owner = FloatOwner {
            f32_bits: f32::INFINITY.to_bits(),
            f64_bits: f64::NEG_INFINITY.to_bits(),
        };

        let nan_ref: &dyn HolderOwner<f32> = &nan_owner;
        assert!(nan_owner.can_serialize_in(nan_ref));
        assert!(!nan_owner.can_serialize_in(&neg_zero_owner as &dyn HolderOwner<f32>));
        assert!(!neg_zero_owner.can_serialize_in(&inf_owner as &dyn HolderOwner<f32>));
        assert_ne!(nan_owner.f32_bits, neg_zero_owner.f32_bits);
        assert_ne!(nan_owner.f64_bits, inf_owner.f64_bits);
    }

    #[test]
    fn default_implementation_can_be_overridden() {
        let permissive = PermissiveOwner;
        let other = IntOwner { id: -123_456 };
        assert!(permissive.can_serialize_in(&other as &dyn HolderOwner<i32>));

        let permissive_ref: &dyn HolderOwner<i32> = &permissive;
        assert!(permissive.can_serialize_in(permissive_ref));
    }
}
