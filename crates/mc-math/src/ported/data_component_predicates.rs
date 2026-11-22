use std::marker::PhantomData;
use std::sync::LazyLock;

/// Minimal codec placeholder for self-contained predicate type registration.
#[derive(Debug)]
pub struct Codec<T: ?Sized>(PhantomData<fn() -> T>);

impl<T: ?Sized> Codec<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized> Clone for Codec<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Codec<T> {}

impl<T: ?Sized> Default for Codec<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker trait for data component predicates.
pub trait DataComponentPredicate: Send + Sync + 'static {}

/// Type object for a registered data component predicate.
pub trait DataComponentPredicateType: Send + Sync {
    fn name(&self) -> &'static str;
}

/// Concrete predicate type, corresponding to Java's
/// `DataComponentPredicate.ConcreteType<T>`.
#[derive(Debug)]
pub struct ConcreteType<T: DataComponentPredicate> {
    name: &'static str,
    _codec: Codec<T>,
}

impl<T: DataComponentPredicate> ConcreteType<T> {
    pub const fn new(name: &'static str, codec: Codec<T>) -> Self {
        Self {
            name,
            _codec: codec,
        }
    }

    pub fn codec(&self) -> Codec<T> {
        self._codec
    }
}

impl<T: DataComponentPredicate> DataComponentPredicateType for ConcreteType<T> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// Minimal registry placeholder for self-contained registration.
pub struct Registry<T: ?Sized + Send + Sync + 'static> {
    _marker: PhantomData<fn() -> &'static T>,
}

impl<T: ?Sized + Send + Sync + 'static> Registry<T> {
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn register<R>(&self, _name: &str, value: R) -> R {
        value
    }
}

impl<T: ?Sized + Send + Sync + 'static> Default for Registry<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub mod built_in_registries {
    use super::{DataComponentPredicateType, Registry};
    use std::sync::LazyLock;

    #[allow(non_upper_case_globals)]
    pub static DATA_COMPONENT_PREDICATE_TYPE: LazyLock<Registry<dyn DataComponentPredicateType>> =
        LazyLock::new(Registry::new);
}

macro_rules! define_predicate {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        pub struct $name;

        impl DataComponentPredicate for $name {}

        impl $name {
            pub const CODEC: Codec<Self> = Codec::new();
        }
    };
}

define_predicate!(DamagePredicate);
define_predicate!(PotionsPredicate);
define_predicate!(CustomDataPredicate);
define_predicate!(ContainerPredicate);
define_predicate!(BundlePredicate);
define_predicate!(FireworkExplosionPredicate);
define_predicate!(FireworksPredicate);
define_predicate!(WritableBookPredicate);
define_predicate!(WrittenBookPredicate);
define_predicate!(AttributeModifiersPredicate);
define_predicate!(TrimPredicate);
define_predicate!(JukeboxPlayablePredicate);

pub mod enchantments_predicate {
    use super::{Codec, DataComponentPredicate};

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct Enchantments;

    impl DataComponentPredicate for Enchantments {}

    impl Enchantments {
        pub const CODEC: Codec<Self> = Codec::new();
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct StoredEnchantments;

    impl DataComponentPredicate for StoredEnchantments {}

    impl StoredEnchantments {
        pub const CODEC: Codec<Self> = Codec::new();
    }
}

pub static DAMAGE: LazyLock<ConcreteType<DamagePredicate>> =
    LazyLock::new(|| register("damage", DamagePredicate::CODEC));

pub static ENCHANTMENTS: LazyLock<ConcreteType<enchantments_predicate::Enchantments>> =
    LazyLock::new(|| register("enchantments", enchantments_predicate::Enchantments::CODEC));

pub static STORED_ENCHANTMENTS: LazyLock<ConcreteType<enchantments_predicate::StoredEnchantments>> =
    LazyLock::new(|| {
        register(
            "stored_enchantments",
            enchantments_predicate::StoredEnchantments::CODEC,
        )
    });

pub static POTIONS: LazyLock<ConcreteType<PotionsPredicate>> =
    LazyLock::new(|| register("potion_contents", PotionsPredicate::CODEC));

pub static CUSTOM_DATA: LazyLock<ConcreteType<CustomDataPredicate>> =
    LazyLock::new(|| register("custom_data", CustomDataPredicate::CODEC));

pub static CONTAINER: LazyLock<ConcreteType<ContainerPredicate>> =
    LazyLock::new(|| register("container", ContainerPredicate::CODEC));

pub static BUNDLE_CONTENTS: LazyLock<ConcreteType<BundlePredicate>> =
    LazyLock::new(|| register("bundle_contents", BundlePredicate::CODEC));

pub static FIREWORK_EXPLOSION: LazyLock<ConcreteType<FireworkExplosionPredicate>> =
    LazyLock::new(|| register("firework_explosion", FireworkExplosionPredicate::CODEC));

pub static FIREWORKS: LazyLock<ConcreteType<FireworksPredicate>> =
    LazyLock::new(|| register("fireworks", FireworksPredicate::CODEC));

pub static WRITABLE_BOOK: LazyLock<ConcreteType<WritableBookPredicate>> =
    LazyLock::new(|| register("writable_book_content", WritableBookPredicate::CODEC));

pub static WRITTEN_BOOK: LazyLock<ConcreteType<WrittenBookPredicate>> =
    LazyLock::new(|| register("written_book_content", WrittenBookPredicate::CODEC));

pub static ATTRIBUTE_MODIFIERS: LazyLock<ConcreteType<AttributeModifiersPredicate>> =
    LazyLock::new(|| register("attribute_modifiers", AttributeModifiersPredicate::CODEC));

pub static ARMOR_TRIM: LazyLock<ConcreteType<TrimPredicate>> =
    LazyLock::new(|| register("trim", TrimPredicate::CODEC));

pub static JUKEBOX_PLAYABLE: LazyLock<ConcreteType<JukeboxPlayablePredicate>> =
    LazyLock::new(|| register("jukebox_playable", JukeboxPlayablePredicate::CODEC));

// JAVA: net.minecraft.core.component.predicates.DataComponentPredicates#register
fn register<T: DataComponentPredicate>(
    name: &'static str,
    codec: Codec<T>,
) -> ConcreteType<T> {
    built_in_registries::DATA_COMPONENT_PREDICATE_TYPE.register(name, ConcreteType::new(name, codec))
}

// JAVA: net.minecraft.core.component.predicates.DataComponentPredicates#bootstrap
pub fn bootstrap(
    _registry: &Registry<dyn DataComponentPredicateType>,
) -> &'static dyn DataComponentPredicateType {
    &*DAMAGE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_returns_damage() {
        let reg = Registry::<dyn DataComponentPredicateType>::new();
        let ty = bootstrap(&reg);
        assert_eq!(ty.name(), "damage");
        assert_eq!(ty.name(), DAMAGE.name());
    }

    #[test]
    fn register_preserves_name_and_codec_type() {
        let ct = register("test_predicate", DamagePredicate::CODEC);
        assert_eq!(ct.name(), "test_predicate");
        let _codec: Codec<DamagePredicate> = ct.codec();
    }

    #[test]
    fn all_constants_have_expected_names() {
        assert_eq!(DAMAGE.name(), "damage");
        assert_eq!(ENCHANTMENTS.name(), "enchantments");
        assert_eq!(STORED_ENCHANTMENTS.name(), "stored_enchantments");
        assert_eq!(POTIONS.name(), "potion_contents");
        assert_eq!(CUSTOM_DATA.name(), "custom_data");
        assert_eq!(CONTAINER.name(), "container");
        assert_eq!(BUNDLE_CONTENTS.name(), "bundle_contents");
        assert_eq!(FIREWORK_EXPLOSION.name(), "firework_explosion");
        assert_eq!(FIREWORKS.name(), "fireworks");
        assert_eq!(WRITABLE_BOOK.name(), "writable_book_content");
        assert_eq!(WRITTEN_BOOK.name(), "written_book_content");
        assert_eq!(ATTRIBUTE_MODIFIERS.name(), "attribute_modifiers");
        assert_eq!(ARMOR_TRIM.name(), "trim");
        assert_eq!(JUKEBOX_PLAYABLE.name(), "jukebox_playable");
    }

    #[test]
    fn registry_register_returns_value() {
        let reg = Registry::<dyn DataComponentPredicateType>::new();
        let value = reg.register("example", 42_i32);
        assert_eq!(value, 42_i32);
        let neg = reg.register("neg", -7_i32);
        assert_eq!(neg.wrapping_mul(3), -21_i32);
    }

    #[test]
    fn java_numeric_semantics_edge_cases() {
        // Overflow/wrapping
        assert_eq!(i32::MAX.wrapping_add(1), i32::MIN);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
        // Negative inputs
        assert_eq!((-1_i32).wrapping_mul(i32::MIN), i32::MIN);
        assert_eq!((-123_i64).wrapping_add(-1_i64), -124_i64);
        // Float bits
        assert_eq!((-0.0_f32).to_bits(), 0x8000_0000_u32);
        assert_eq!(0.0_f32.to_bits(), 0x0000_0000_u32);
        let nan_bits = f32::NAN.to_bits();
        assert_ne!(nan_bits & 0x7fc0_0000, 0);
    }

    #[test]
    fn concrete_type_and_codec_are_usable() {
        let codec: Codec<PotionsPredicate> = Codec::new();
        let copy = codec;
        let _ = (codec, copy);
        let ct = ConcreteType::new("edge", PotionsPredicate::CODEC);
        assert_eq!(ct.name(), "edge");
        let as_trait: &dyn DataComponentPredicateType = &ct;
        assert_eq!(as_trait.name(), "edge");
    }
}
