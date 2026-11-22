use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;

/// Minimal stand-in for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn parse(value: &str) -> Self {
        if let Some((ns, p)) = value.split_once(':') {
            Self::new(ns, p)
        } else {
            Self::new("minecraft", value)
        }
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Minimal stand-in for `net.minecraft.util.context.ContextKey<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey<T: ?Sized> {
    pub name: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> ContextKey<T> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

/// Minimal stand-in for `net.minecraft.util.context.ContextKeySet`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextKeySet;

/// Minimal stand-in for `net.minecraft.util.context.ContextMap`.
#[derive(Default)]
pub struct ContextMap {
    values: HashMap<&'static str, Box<dyn Any>>,
}

impl std::fmt::Debug for ContextMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMap")
            .field("keys", &self.values.keys().copied().collect::<Vec<_>>())
            .finish()
    }
}

impl ContextMap {
    pub fn builder() -> ContextMapBuilder {
        ContextMapBuilder::default()
    }

    pub fn get<T: 'static>(&self, key: &ContextKey<T>) -> Option<&T> {
        self.values.get(key.name)?.downcast_ref::<T>()
    }

    pub fn contains<T: 'static>(&self, key: &ContextKey<T>) -> bool {
        self.values.contains_key(key.name)
    }
}

/// Minimal stand-in for `net.minecraft.util.context.ContextMap.Builder`.
#[derive(Default)]
pub struct ContextMapBuilder {
    values: HashMap<&'static str, Box<dyn Any>>,
}

impl ContextMapBuilder {
    pub fn with_parameter<T: 'static>(&mut self, key: &ContextKey<T>, value: T) -> &mut Self {
        self.values.insert(key.name, Box::new(value));
        self
    }

    pub fn with_optional_parameter<T: 'static>(
        &mut self,
        key: &ContextKey<T>,
        value: Option<T>,
    ) -> &mut Self {
        if let Some(v) = value {
            self.values.insert(key.name, Box::new(v));
        }
        self
    }

    pub fn get_parameter<T: Clone + 'static>(&self, key: &ContextKey<T>) -> T {
        self.values
            .get(key.name)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
            .unwrap_or_else(|| panic!("Missing required context parameter: {}", key.name))
    }

    pub fn get_optional_parameter<T: Clone + 'static>(&self, key: &ContextKey<T>) -> Option<T> {
        self.values
            .get(key.name)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }

    pub fn create(self, _set: &ContextKeySet) -> ContextMap {
        ContextMap {
            values: self.values,
        }
    }
}

/// Minimal stand-in for `net.minecraft.server.level.ServerLevel`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerLevel {
    pub name: String,
}

impl ServerLevel {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Minimal stand-in for `net.minecraft.world.item.ItemStack`.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
}

impl ItemStack {
    pub fn new(id: impl Into<String>, count: i32) -> Self {
        Self {
            id: id.into(),
            count,
        }
    }
}

/// Port of `net.minecraft.world.level.storage.loot.LootParams.DynamicDrop`.
pub trait DynamicDrop {
    // JAVA: net.minecraft.world.level.storage.loot.LootParams$DynamicDrop#add
    fn add(&self, output: &mut dyn FnMut(ItemStack));
}

/// Port of `net.minecraft.world.level.storage.loot.LootParams`.
pub struct LootParams {
    level: ServerLevel,
    params: ContextMap,
    dynamic_drops: HashMap<Identifier, Box<dyn DynamicDrop>>,
    luck: f32,
}

impl std::fmt::Debug for LootParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LootParams")
            .field("level", &self.level)
            .field("params", &self.params)
            .field("dynamic_drops_len", &self.dynamic_drops.len())
            .field("luck_bits", &self.luck.to_bits())
            .finish()
    }
}

impl LootParams {
    // JAVA: net.minecraft.world.level.storage.loot.LootParams#<init>
    pub fn new(
        level: ServerLevel,
        params: ContextMap,
        dynamic_drops: HashMap<Identifier, Box<dyn DynamicDrop>>,
        luck: f32,
    ) -> Self {
        Self {
            level,
            params,
            dynamic_drops,
            luck,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams#getLevel
    pub fn get_level(&self) -> &ServerLevel {
        &self.level
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams#contextMap
    pub fn context_map(&self) -> &ContextMap {
        &self.params
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams#addDynamicDrops
    pub fn add_dynamic_drops(&self, id: &Identifier, output: &mut dyn FnMut(ItemStack)) {
        if let Some(drop) = self.dynamic_drops.get(id) {
            drop.add(output);
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams#getLuck
    pub fn get_luck(&self) -> f32 {
        self.luck
    }
}

/// Port of `net.minecraft.world.level.storage.loot.LootParams$Builder`.
pub struct LootParamsBuilder {
    level: ServerLevel,
    params: ContextMapBuilder,
    dynamic_drops: HashMap<Identifier, Box<dyn DynamicDrop>>,
    luck: f32,
}

impl std::fmt::Debug for LootParamsBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LootParamsBuilder")
            .field("level", &self.level)
            .field("dynamic_drops_len", &self.dynamic_drops.len())
            .field("luck_bits", &self.luck.to_bits())
            .finish()
    }
}

impl LootParamsBuilder {
    // JAVA: net.minecraft.world.level.storage.loot.LootParams$Builder#<init>
    pub fn new(level: ServerLevel) -> Self {
        Self {
            level,
            params: ContextMapBuilder::default(),
            dynamic_drops: HashMap::new(),
            luck: 0.0,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams$Builder#getLevel
    pub fn get_level(&self) -> &ServerLevel {
        &self.level
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams$Builder#withParameter
    pub fn with_parameter<T: 'static>(&mut self, key: &ContextKey<T>, value: T) -> &mut Self {
        self.params.with_parameter(key, value);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams$Builder#withOptionalParameter
    pub fn with_optional_parameter<T: 'static>(
        &mut self,
        key: &ContextKey<T>,
        value: Option<T>,
    ) -> &mut Self {
        self.params.with_optional_parameter(key, value);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams$Builder#getParameter
    pub fn get_parameter<T: Clone + 'static>(&self, key: &ContextKey<T>) -> T {
        self.params.get_parameter(key)
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams$Builder#getOptionalParameter
    pub fn get_optional_parameter<T: Clone + 'static>(&self, key: &ContextKey<T>) -> Option<T> {
        self.params.get_optional_parameter(key)
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams$Builder#withDynamicDrop
    pub fn with_dynamic_drop(
        &mut self,
        id: Identifier,
        drop: Box<dyn DynamicDrop>,
    ) -> &mut Self {
        if self.dynamic_drops.contains_key(&id) {
            panic!("Duplicated dynamic drop '{}:{}'", id.namespace, id.path);
        }
        self.dynamic_drops.insert(id, drop);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams$Builder#withLuck
    pub fn with_luck(&mut self, luck: f32) -> &mut Self {
        self.luck = luck;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootParams$Builder#create
    pub fn create(self, set: &ContextKeySet) -> LootParams {
        let params = self.params.create(set);
        LootParams::new(self.level, params, self.dynamic_drops, self.luck)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedDrop(ItemStack);
    impl DynamicDrop for FixedDrop {
        fn add(&self, output: &mut dyn FnMut(ItemStack)) {
            output(self.0.clone());
        }
    }

    #[test]
    fn builder_creates_params_and_preserves_luck_bits() {
        let level = ServerLevel::new("overworld");
        let mut builder = LootParamsBuilder::new(level.clone());
        let luck = f32::from_bits(0xBF80_0000); // -1.0
        builder.with_luck(luck);
        let params = builder.create(&ContextKeySet);
        assert_eq!(params.get_level(), &level);
        assert_eq!(params.get_luck().to_bits(), luck.to_bits());
    }

    #[test]
    fn dynamic_drops_are_invoked_when_present() {
        let mut builder = LootParamsBuilder::new(ServerLevel::new("test"));
        let id = Identifier::parse("minecraft:chest");
        builder.with_dynamic_drop(id.clone(), Box::new(FixedDrop(ItemStack::new("stone", 1))));
        let params = builder.create(&ContextKeySet);

        let mut out = Vec::new();
        params.add_dynamic_drops(&id, &mut |s| out.push(s));
        assert_eq!(out, vec![ItemStack::new("stone", 1)]);

        let mut missing_out = Vec::new();
        params.add_dynamic_drops(&Identifier::parse("minecraft:missing"), &mut |s| {
            missing_out.push(s)
        });
        assert!(missing_out.is_empty());
    }

    #[test]
    #[should_panic(expected = "Duplicated dynamic drop")]
    fn duplicate_dynamic_drop_panics() {
        let mut builder = LootParamsBuilder::new(ServerLevel::new("test"));
        let id = Identifier::parse("minecraft:dup");
        builder.with_dynamic_drop(id.clone(), Box::new(FixedDrop(ItemStack::new("a", 1))));
        builder.with_dynamic_drop(id, Box::new(FixedDrop(ItemStack::new("b", 1))));
    }

    #[test]
    fn context_parameters_round_trip() {
        let key: ContextKey<i32> = ContextKey::new("test:int");
        let mut builder = LootParamsBuilder::new(ServerLevel::new("test"));
        builder.with_parameter(&key, 42i32);
        assert_eq!(builder.get_parameter(&key), 42i32);
        assert_eq!(builder.get_optional_parameter(&key), Some(42i32));

        let missing: ContextKey<i32> = ContextKey::new("test:missing");
        assert_eq!(builder.get_optional_parameter(&missing), None);

        let params = builder.create(&ContextKeySet);
        assert_eq!(params.context_map().get(&key), Some(&42i32));
        assert!(!params.context_map().contains(&missing));
    }

    #[test]
    fn optional_parameter_none_does_not_insert() {
        let key: ContextKey<i32> = ContextKey::new("test:opt");
        let mut builder = LootParamsBuilder::new(ServerLevel::new("test"));
        builder.with_optional_parameter(&key, None);
        assert_eq!(builder.get_optional_parameter(&key), None);
        builder.with_optional_parameter(&key, Some(-7));
        assert_eq!(builder.get_parameter(&key), -7);
    }
}
