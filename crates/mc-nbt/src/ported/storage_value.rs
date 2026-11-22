use std::fmt;

use crate::NbtTag;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootNumberProviderType(pub u32);

#[derive(Debug)]
pub struct CommandSyntaxException;

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CommandSyntaxException")
    }
}

impl std::error::Error for CommandSyntaxException {}

pub trait NbtPath {
    fn get(&self, root: &NbtTag) -> Result<Vec<NbtTag>, CommandSyntaxException>;
}

pub trait CommandStorage {
    fn get(&self, storage: &Identifier) -> NbtTag;
}

pub struct LootContext<'a> {
    command_storage: &'a dyn CommandStorage,
}

impl<'a> LootContext<'a> {
    pub fn new(command_storage: &'a dyn CommandStorage) -> Self {
        Self { command_storage }
    }

    pub fn get_command_storage(&self) -> &dyn CommandStorage {
        self.command_storage
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NbtNumber {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl NbtNumber {
    pub fn from_tag(tag: &NbtTag) -> Option<Self> {
        match tag {
            NbtTag::Byte(v) => Some(Self::Byte(*v)),
            NbtTag::Short(v) => Some(Self::Short(*v)),
            NbtTag::Int(v) => Some(Self::Int(*v)),
            NbtTag::Long(v) => Some(Self::Long(*v)),
            NbtTag::Float(v) => Some(Self::Float(*v)),
            NbtTag::Double(v) => Some(Self::Double(*v)),
            _ => None,
        }
    }

    pub fn float_value(self) -> f32 {
        match self {
            Self::Byte(v) => v as f32,
            Self::Short(v) => v as f32,
            Self::Int(v) => v as f32,
            Self::Long(v) => v as f32,
            Self::Float(v) => v,
            Self::Double(v) => v as f32,
        }
    }

    pub fn int_value(self) -> i32 {
        match self {
            Self::Byte(v) => v as i32,
            Self::Short(v) => v as i32,
            Self::Int(v) => v,
            Self::Long(v) => v as i32,
            Self::Float(v) => v as i32,
            Self::Double(v) => v as i32,
        }
    }
}

pub struct StorageValue {
    pub storage: Identifier,
    pub path: Box<dyn NbtPath>,
}

impl StorageValue {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.StorageValue#<init>
    pub fn new(storage: Identifier, path: Box<dyn NbtPath>) -> Self {
        Self { storage, path }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.StorageValue#storage
    pub fn storage(&self) -> &Identifier {
        &self.storage
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.StorageValue#path
    pub fn path(&self) -> &dyn NbtPath {
        &*self.path
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.StorageValue#getType
    pub fn get_type(&self) -> LootNumberProviderType {
        LootNumberProviderType(4)
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.StorageValue#getNumericTag
    fn get_numeric_tag(&self, ctx: &LootContext<'_>, default: NbtNumber) -> NbtNumber {
        let compound = ctx.get_command_storage().get(&self.storage);
        match self.path.get(&compound) {
            Ok(results) => {
                if results.len() == 1 {
                    if let Some(n) = NbtNumber::from_tag(&results[0]) {
                        return n;
                    }
                }
            }
            Err(_) => {}
        }
        default
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.StorageValue#getFloat
    pub fn get_float(&self, ctx: &LootContext<'_>) -> f32 {
        self.get_numeric_tag(ctx, NbtNumber::Float(0.0_f32))
            .float_value()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.StorageValue#getInt
    pub fn get_int(&self, ctx: &LootContext<'_>) -> i32 {
        self.get_numeric_tag(ctx, NbtNumber::Int(0)).int_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct MapStorage {
        map: BTreeMap<String, NbtTag>,
    }

    impl CommandStorage for MapStorage {
        fn get(&self, storage: &Identifier) -> NbtTag {
            self.map
                .get(&storage.0)
                .cloned()
                .unwrap_or(NbtTag::Compound(vec![]))
        }
    }

    struct FixedPath {
        results: Result<Vec<NbtTag>, ()>,
    }

    impl NbtPath for FixedPath {
        fn get(&self, _root: &NbtTag) -> Result<Vec<NbtTag>, CommandSyntaxException> {
            match &self.results {
                Ok(v) => Ok(v.clone()),
                Err(_) => Err(CommandSyntaxException),
            }
        }
    }

    fn ctx_with(storage: &str, tag: NbtTag) -> (MapStorage, Identifier) {
        let mut map = BTreeMap::new();
        map.insert(storage.to_string(), tag);
        (MapStorage { map }, Identifier::new(storage))
    }

    #[test]
    fn returns_numeric_tag_float_and_int() {
        let (storage, id) = ctx_with("ns:store", NbtTag::Compound(vec![]));
        let ctx = LootContext::new(&storage);
        let sv = StorageValue::new(
            id,
            Box::new(FixedPath { results: Ok(vec![NbtTag::Int(42)]) }),
        );
        assert_eq!(sv.get_int(&ctx), 42);
        assert_eq!(sv.get_float(&ctx).to_bits(), 42.0_f32.to_bits());
    }

    #[test]
    fn missing_or_non_numeric_returns_default() {
        let (storage, id) = ctx_with("ns:store", NbtTag::Compound(vec![]));
        let ctx = LootContext::new(&storage);
        let sv = StorageValue::new(
            id.clone(),
            Box::new(FixedPath { results: Ok(vec![NbtTag::String("x".to_string())]) }),
        );
        assert_eq!(sv.get_int(&ctx), 0);
        assert_eq!(sv.get_float(&ctx).to_bits(), 0.0_f32.to_bits());

        let sv2 = StorageValue::new(
            id,
            Box::new(FixedPath { results: Ok(vec![]) }),
        );
        assert_eq!(sv2.get_int(&ctx), 0);
    }

    #[test]
    fn command_syntax_exception_returns_default() {
        let (storage, id) = ctx_with("ns:store", NbtTag::Compound(vec![]));
        let ctx = LootContext::new(&storage);
        let sv = StorageValue::new(id, Box::new(FixedPath { results: Err(()) }));
        assert_eq!(sv.get_int(&ctx), 0);
        assert_eq!(sv.get_float(&ctx).to_bits(), 0.0_f32.to_bits());
    }

    #[test]
    fn multiple_results_returns_default() {
        let (storage, id) = ctx_with("ns:store", NbtTag::Compound(vec![]));
        let ctx = LootContext::new(&storage);
        let sv = StorageValue::new(
            id,
            Box::new(FixedPath {
                results: Ok(vec![NbtTag::Int(1), NbtTag::Int(2)]),
            }),
        );
        assert_eq!(sv.get_int(&ctx), 0);
    }

    #[test]
    fn numeric_conversions_preserve_java_like_widening_narrowing() {
        let (storage, id) = ctx_with("ns:store", NbtTag::Compound(vec![]));
        let ctx = LootContext::new(&storage);
        let sv_long = StorageValue::new(
            id.clone(),
            Box::new(FixedPath { results: Ok(vec![NbtTag::Long(i64::MAX)]) }),
        );
        assert_eq!(sv_long.get_int(&ctx), i64::MAX as i32);

        let sv_double = StorageValue::new(
            id,
            Box::new(FixedPath { results: Ok(vec![NbtTag::Double(f64::INFINITY)]) }),
        );
        assert_eq!(sv_double.get_float(&ctx).to_bits(), f32::INFINITY.to_bits());
    }
}
