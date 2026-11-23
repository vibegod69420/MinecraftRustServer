//! Faithful port of `net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider`.
//!
//! This module is self-contained for Phase A porting: external loot-context/NBT
//! infrastructure is represented by minimal local traits and types.

use crate::NbtTag;
use std::collections::HashSet;

/// Minimal stub for `net.minecraft.util.context.ContextKey`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

/// Minimal stub for `net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LootNbtProviderType(pub &'static str);

/// Minimal stub for `net.minecraft.world.level.storage.loot.providers.nbt.NbtProviders`.
pub struct NbtProviders;

impl NbtProviders {
    pub const CONTEXT: LootNbtProviderType = LootNbtProviderType("context");
}

/// Minimal stub for `net.minecraft.core.RegistryAccess`.
pub trait RegistryAccess {}

/// Minimal stub for `net.minecraft.world.level.Level`.
pub trait Level {
    fn registry_access(&self) -> &dyn RegistryAccess;
}

/// Minimal stub for `net.minecraft.world.level.block.entity.BlockEntity`.
pub trait BlockEntity {
    fn get_level(&self) -> &dyn Level;
    fn save_with_full_metadata(&self, registry_access: &dyn RegistryAccess) -> NbtTag;
}

/// Minimal stub for `net.minecraft.world.entity.Entity`.
///
/// `entity_tag_to_compare` corresponds to
/// `NbtPredicate.getEntityTagToCompare(entity)`.
pub trait Entity {
    fn entity_tag_to_compare(&self) -> NbtTag;
}

/// Minimal stub for `net.minecraft.world.level.storage.loot.LootContext`.
pub trait LootContext {
    fn get_block_entity(&self, key: ContextKey) -> Option<&dyn BlockEntity>;
    fn get_entity(&self, key: ContextKey) -> Option<&dyn Entity>;
}

/// Minimal stub for `net.minecraft.world.level.storage.loot.LootContext.EntityTarget`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityTarget(pub ContextKey);

impl EntityTarget {
    pub fn context_param(self) -> ContextKey {
        self.0
    }
}

/// Minimal stub for `net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider`.
pub trait NbtProvider {
    fn get_type(&self) -> LootNbtProviderType;
    fn get(&self, ctx: &dyn LootContext) -> Option<NbtTag>;
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
}

/// Port of `ContextNbtProvider.BlockEntitySource`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntitySource {
    context_param: ContextKey,
}

impl BlockEntitySource {
    pub fn new(context_param: ContextKey) -> Self {
        Self { context_param }
    }

    pub fn context_param(&self) -> ContextKey {
        self.context_param
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider$BlockEntitySource#get
    pub fn get(&self, block_entity: &dyn BlockEntity) -> NbtTag {
        block_entity.save_with_full_metadata(block_entity.get_level().registry_access())
    }
}

/// Port of `ContextNbtProvider.EntitySource`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySource {
    context_param: ContextKey,
}

impl EntitySource {
    pub fn new(context_param: ContextKey) -> Self {
        Self { context_param }
    }

    pub fn context_param(&self) -> ContextKey {
        self.context_param
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider$EntitySource#get
    pub fn get(&self, entity: &dyn Entity) -> NbtTag {
        entity.entity_tag_to_compare()
    }
}

/// Minimal self-contained representation of `LootContextArg<Tag>` for this provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootContextArg {
    BlockEntity(BlockEntitySource),
    Entity(EntitySource),
}

impl LootContextArg {
    pub fn get(&self, ctx: &dyn LootContext) -> Option<NbtTag> {
        match self {
            Self::BlockEntity(source) => ctx
                .get_block_entity(source.context_param())
                .map(|be| source.get(be)),
            Self::Entity(source) => ctx
                .get_entity(source.context_param())
                .map(|e| source.get(e)),
        }
    }

    pub fn context_param(&self) -> ContextKey {
        match self {
            Self::BlockEntity(source) => source.context_param(),
            Self::Entity(source) => source.context_param(),
        }
    }
}

/// Port of `net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextNbtProvider {
    source: LootContextArg,
}

impl ContextNbtProvider {
    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider#<init>
    pub fn new(source: LootContextArg) -> Self {
        Self { source }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider#getType
    pub fn get_type(&self) -> LootNbtProviderType {
        NbtProviders::CONTEXT
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider#get
    pub fn get(&self, ctx: &dyn LootContext) -> Option<NbtTag> {
        self.source.get(ctx)
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        HashSet::from([self.source.context_param()])
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider#forContextEntity
    pub fn for_context_entity(target: EntityTarget) -> Self {
        Self::new(LootContextArg::Entity(EntitySource::new(
            target.context_param(),
        )))
    }
}

impl NbtProvider for ContextNbtProvider {
    fn get_type(&self) -> LootNbtProviderType {
        Self::get_type(self)
    }

    fn get(&self, ctx: &dyn LootContext) -> Option<NbtTag> {
        Self::get(self, ctx)
    }

    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        Self::get_referenced_context_params(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NbtTag;

    struct MockRegistryAccess;
    impl RegistryAccess for MockRegistryAccess {}

    struct MockLevel {
        ra: MockRegistryAccess,
    }

    impl Level for MockLevel {
        fn registry_access(&self) -> &dyn RegistryAccess {
            &self.ra
        }
    }

    struct MockBlockEntity {
        level: MockLevel,
        tag: NbtTag,
    }

    impl BlockEntity for MockBlockEntity {
        fn get_level(&self) -> &dyn Level {
            &self.level
        }

        fn save_with_full_metadata(&self, _registry_access: &dyn RegistryAccess) -> NbtTag {
            self.tag.clone()
        }
    }

    struct MockEntity {
        tag: NbtTag,
    }

    impl Entity for MockEntity {
        fn entity_tag_to_compare(&self) -> NbtTag {
            self.tag.clone()
        }
    }

    struct MockLootContext {
        block_entities: Vec<(ContextKey, MockBlockEntity)>,
        entities: Vec<(ContextKey, MockEntity)>,
    }

    impl LootContext for MockLootContext {
        fn get_block_entity(&self, key: ContextKey) -> Option<&dyn BlockEntity> {
            self.block_entities
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v as &dyn BlockEntity)
        }

        fn get_entity(&self, key: ContextKey) -> Option<&dyn Entity> {
            self.entities
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v as &dyn Entity)
        }
    }

    #[test]
    fn entity_source_provider_returns_tag_and_metadata() {
        let key = ContextKey("this");
        let tag = NbtTag::Compound(vec![(
            "id".to_string(),
            NbtTag::String("minecraft:pig".to_string()),
        )]);
        let ctx = MockLootContext {
            block_entities: vec![],
            entities: vec![(key, MockEntity { tag: tag.clone() })],
        };
        let provider = ContextNbtProvider::for_context_entity(EntityTarget(key));
        assert_eq!(provider.get_type(), NbtProviders::CONTEXT);
        assert_eq!(provider.get(&ctx), Some(tag));
        assert_eq!(
            provider.get_referenced_context_params(),
            HashSet::from([key])
        );
    }

    #[test]
    fn block_entity_source_returns_saved_tag_with_negative_int() {
        let key = ContextKey("block_entity");
        let tag = NbtTag::Compound(vec![("power".to_string(), NbtTag::Int(i32::MIN))]);
        let ctx = MockLootContext {
            block_entities: vec![(
                key,
                MockBlockEntity {
                    level: MockLevel {
                        ra: MockRegistryAccess,
                    },
                    tag: tag.clone(),
                },
            )],
            entities: vec![],
        };
        let provider = ContextNbtProvider::new(LootContextArg::BlockEntity(
            BlockEntitySource::new(key),
        ));
        assert_eq!(provider.get(&ctx), Some(tag));
        assert_eq!(
            provider.get_referenced_context_params(),
            HashSet::from([key])
        );
    }

    #[test]
    fn missing_context_param_returns_none() {
        let key = ContextKey("missing");
        let ctx = MockLootContext {
            block_entities: vec![],
            entities: vec![],
        };
        let provider = ContextNbtProvider::for_context_entity(EntityTarget(key));
        assert_eq!(provider.get(&ctx), None);
        assert_eq!(
            provider.get_referenced_context_params(),
            HashSet::from([key])
        );
    }

    #[test]
    fn source_get_methods_preserve_negative_and_float_bits() {
        let be_key = ContextKey("be");
        let be_tag = NbtTag::Compound(vec![
            ("byte".to_string(), NbtTag::Byte(i8::MIN)),
            ("short".to_string(), NbtTag::Short(i16::MIN)),
            ("int".to_string(), NbtTag::Int(i32::wrapping_add(i32::MAX, 1))),
            (
                "long".to_string(),
                NbtTag::Long(i64::wrapping_sub(i64::MIN, 1)),
            ),
        ]);
        let be = MockBlockEntity {
            level: MockLevel {
                ra: MockRegistryAccess,
            },
            tag: be_tag.clone(),
        };
        let be_source = BlockEntitySource::new(be_key);
        assert_eq!(be_source.context_param(), be_key);
        assert_eq!(be_source.get(&be), be_tag);

        let e_key = ContextKey("e");
        let nan_bits = f32::NAN.to_bits();
        let neg_zero_f32 = (-0.0f32).to_bits();
        let neg_zero_f64 = (-0.0f64).to_bits();
        let e_tag = NbtTag::Compound(vec![
            ("nan".to_string(), NbtTag::Float(f32::from_bits(nan_bits))),
            (
                "neg_zero_f32".to_string(),
                NbtTag::Float(f32::from_bits(neg_zero_f32)),
            ),
            (
                "neg_zero_f64".to_string(),
                NbtTag::Double(f64::from_bits(neg_zero_f64)),
            ),
        ]);
        let entity = MockEntity { tag: e_tag.clone() };
        let e_source = EntitySource::new(e_key);
        assert_eq!(e_source.context_param(), e_key);
        let got = e_source.get(&entity);
        assert!(got.structural_eq(&e_tag));
        let NbtTag::Compound(entries) = got else {
            panic!("expected compound");
        };
        let find = |name: &str| entries.iter().find(|(k, _)| k == name).unwrap().1.clone();
        match find("nan") {
            NbtTag::Float(v) => assert_eq!(v.to_bits(), nan_bits),
            _ => panic!("expected float"),
        }
        match find("neg_zero_f32") {
            NbtTag::Float(v) => assert_eq!(v.to_bits(), neg_zero_f32),
            _ => panic!("expected float"),
        }
        match find("neg_zero_f64") {
            NbtTag::Double(v) => assert_eq!(v.to_bits(), neg_zero_f64),
            _ => panic!("expected double"),
        }
    }

    #[test]
    fn provider_preserves_float_bits_through_context_lookup() {
        let key = ContextKey("target");
        let nan_bits = f32::from_bits(0x7FC0_0001u32).to_bits();
        let payload = NbtTag::Compound(vec![
            ("nan".to_string(), NbtTag::Float(f32::from_bits(nan_bits))),
            ("neg".to_string(), NbtTag::Int(-123_456)),
            (
                "wrap".to_string(),
                NbtTag::Long(i64::wrapping_mul(i64::MIN, 1)),
            ),
        ]);
        let ctx = MockLootContext {
            block_entities: vec![],
            entities: vec![(key, MockEntity { tag: payload.clone() })],
        };
        let provider = ContextNbtProvider::for_context_entity(EntityTarget(key));
        let got = provider.get(&ctx).expect("tag");
        assert!(got.structural_eq(&payload));
        let NbtTag::Compound(entries) = got else {
            panic!("expected compound");
        };
        let nan = entries
            .iter()
            .find(|(k, _)| k == "nan")
            .map(|(_, v)| v.clone())
            .unwrap();
        match nan {
            NbtTag::Float(v) => assert_eq!(v.to_bits(), nan_bits),
            _ => panic!("expected float"),
        }
    }
}
