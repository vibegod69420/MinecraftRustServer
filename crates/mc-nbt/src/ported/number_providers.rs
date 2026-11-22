use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootNumberProviderType {
    pub id: Identifier,
}

impl LootNumberProviderType {
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }
}

fn registry() -> &'static Mutex<BTreeMap<Identifier, LootNumberProviderType>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<Identifier, LootNumberProviderType>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

// JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProviders#register
fn register(name: &str) -> LootNumberProviderType {
    let id = Identifier::with_default_namespace(name);
    let ty = LootNumberProviderType::new(id.clone());
    registry()
        .lock()
        .expect("number provider registry lock")
        .insert(id, ty.clone());
    ty
}

pub struct NumberProviders;

impl NumberProviders {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProviders#CONSTANT
    pub fn constant() -> LootNumberProviderType {
        static VALUE: OnceLock<LootNumberProviderType> = OnceLock::new();
        VALUE.get_or_init(|| register("constant")).clone()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProviders#UNIFORM
    pub fn uniform() -> LootNumberProviderType {
        static VALUE: OnceLock<LootNumberProviderType> = OnceLock::new();
        VALUE.get_or_init(|| register("uniform")).clone()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProviders#BINOMIAL
    pub fn binomial() -> LootNumberProviderType {
        static VALUE: OnceLock<LootNumberProviderType> = OnceLock::new();
        VALUE.get_or_init(|| register("binomial")).clone()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProviders#SCORE
    pub fn score() -> LootNumberProviderType {
        static VALUE: OnceLock<LootNumberProviderType> = OnceLock::new();
        VALUE.get_or_init(|| register("score")).clone()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProviders#STORAGE
    pub fn storage() -> LootNumberProviderType {
        static VALUE: OnceLock<LootNumberProviderType> = OnceLock::new();
        VALUE.get_or_init(|| register("storage")).clone()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProviders#ENCHANTMENT_LEVEL
    pub fn enchantment_level() -> LootNumberProviderType {
        static VALUE: OnceLock<LootNumberProviderType> = OnceLock::new();
        VALUE.get_or_init(|| register("enchantment_level")).clone()
    }

    pub fn registered_types() -> BTreeMap<Identifier, LootNumberProviderType> {
        registry()
            .lock()
            .expect("number provider registry lock")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_builtin_types_with_default_namespace() {
        let constant = NumberProviders::constant();
        let uniform = NumberProviders::uniform();
        let binomial = NumberProviders::binomial();
        let score = NumberProviders::score();
        let storage = NumberProviders::storage();
        let enchantment = NumberProviders::enchantment_level();

        assert_eq!(constant.id, Identifier::with_default_namespace("constant"));
        assert_eq!(uniform.id, Identifier::with_default_namespace("uniform"));
        assert_eq!(binomial.id, Identifier::with_default_namespace("binomial"));
        assert_eq!(score.id, Identifier::with_default_namespace("score"));
        assert_eq!(storage.id, Identifier::with_default_namespace("storage"));
        assert_eq!(enchantment.id, Identifier::with_default_namespace("enchantment_level"));
    }

    #[test]
    fn registry_contains_registered_entries() {
        let _ = NumberProviders::constant();
        let _ = NumberProviders::uniform();
        let types = NumberProviders::registered_types();
        assert!(types.contains_key(&Identifier::with_default_namespace("constant")));
        assert!(types.contains_key(&Identifier::with_default_namespace("uniform")));
    }

    #[test]
    fn register_is_idempotent_for_same_name() {
        let a = NumberProviders::storage();
        let b = NumberProviders::storage();
        assert_eq!(a, b);
        let types = NumberProviders::registered_types();
        let count = types
            .keys()
            .filter(|k| *k == &Identifier::with_default_namespace("storage"))
            .count();
        assert_eq!(count, 1);
    }
}
