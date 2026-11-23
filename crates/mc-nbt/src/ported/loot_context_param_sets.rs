use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::{LazyLock, Mutex, OnceLock};

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

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LootContextParam {
    ThisEntity,
    LastDamagePlayer,
    DamageSource,
    AttackingEntity,
    DirectAttackingEntity,
    Origin,
    BlockState,
    BlockEntity,
    Tool,
    ExplosionRadius,
    EnchantmentLevel,
    EnchantmentActive,
    TargetEntity,
    InteractingEntity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKeySet {
    required: BTreeSet<LootContextParam>,
    allowed: BTreeSet<LootContextParam>,
}

impl ContextKeySet {
    // JAVA: net.minecraft.util.context.ContextKeySet#isRequired
    pub fn is_required(&self, param: LootContextParam) -> bool {
        self.required.contains(&param)
    }

    // JAVA: net.minecraft.util.context.ContextKeySet#isAllowed
    pub fn is_allowed(&self, param: LootContextParam) -> bool {
        self.allowed.contains(&param)
    }

    // JAVA: net.minecraft.util.context.ContextKeySet#required
    pub fn required(&self) -> &BTreeSet<LootContextParam> {
        &self.required
    }

    // JAVA: net.minecraft.util.context.ContextKeySet#allowed
    pub fn allowed(&self) -> &BTreeSet<LootContextParam> {
        &self.allowed
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContextKeySetBuilder {
    required: BTreeSet<LootContextParam>,
    allowed: BTreeSet<LootContextParam>,
}

impl ContextKeySetBuilder {
    // JAVA: net.minecraft.util.context.ContextKeySet$Builder#<init>
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: net.minecraft.util.context.ContextKeySet$Builder#required
    pub fn required(&mut self, param: LootContextParam) -> &mut Self {
        self.required.insert(param);
        self.allowed.insert(param);
        self
    }

    // JAVA: net.minecraft.util.context.ContextKeySet$Builder#optional
    pub fn optional(&mut self, param: LootContextParam) -> &mut Self {
        self.allowed.insert(param);
        self
    }

    // JAVA: net.minecraft.util.context.ContextKeySet$Builder#build
    pub fn build(&self) -> ContextKeySet {
        ContextKeySet {
            required: self.required.clone(),
            allowed: self.allowed.clone(),
        }
    }
}

static REGISTRY: LazyLock<Mutex<BTreeMap<Identifier, ContextKeySet>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

fn ensure_registered() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        LazyLock::force(&EMPTY);
        LazyLock::force(&CHEST);
        LazyLock::force(&COMMAND);
        LazyLock::force(&SELECTOR);
        LazyLock::force(&FISHING);
        LazyLock::force(&ENTITY);
        LazyLock::force(&EQUIPMENT);
        LazyLock::force(&ARCHAEOLOGY);
        LazyLock::force(&GIFT);
        LazyLock::force(&PIGLIN_BARTER);
        LazyLock::force(&VAULT);
        LazyLock::force(&ADVANCEMENT_REWARD);
        LazyLock::force(&ADVANCEMENT_ENTITY);
        LazyLock::force(&ADVANCEMENT_LOCATION);
        LazyLock::force(&BLOCK_USE);
        LazyLock::force(&ALL_PARAMS);
        LazyLock::force(&BLOCK);
        LazyLock::force(&SHEARING);
        LazyLock::force(&ENTITY_INTERACT);
        LazyLock::force(&BLOCK_INTERACT);
        LazyLock::force(&ENCHANTED_DAMAGE);
        LazyLock::force(&ENCHANTED_ITEM);
        LazyLock::force(&ENCHANTED_LOCATION);
        LazyLock::force(&ENCHANTED_ENTITY);
        LazyLock::force(&HIT_BLOCK);
    });
}

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParamSets#register
fn register(name: &str, configure: impl FnOnce(&mut ContextKeySetBuilder)) -> ContextKeySet {
    let mut builder = ContextKeySetBuilder::new();
    configure(&mut builder);
    let set = builder.build();
    let id = Identifier::with_default_namespace(name);
    let mut reg = REGISTRY
        .lock()
        .expect("loot context param sets registry poisoned");
    if reg.insert(id.clone(), set.clone()).is_some() {
        panic!("Loot table parameter set {} is already registered", id);
    }
    set
}

pub static EMPTY: LazyLock<ContextKeySet> = LazyLock::new(|| register("empty", |_b| {}));

pub static CHEST: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("chest", |b| {
        b.required(LootContextParam::Origin)
            .optional(LootContextParam::ThisEntity);
    })
});

pub static COMMAND: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("command", |b| {
        b.required(LootContextParam::Origin)
            .optional(LootContextParam::ThisEntity);
    })
});

pub static SELECTOR: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("selector", |b| {
        b.required(LootContextParam::Origin)
            .required(LootContextParam::ThisEntity);
    })
});

pub static FISHING: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("fishing", |b| {
        b.required(LootContextParam::Origin)
            .required(LootContextParam::Tool)
            .optional(LootContextParam::ThisEntity);
    })
});

pub static ENTITY: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("entity", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::Origin)
            .required(LootContextParam::DamageSource)
            .optional(LootContextParam::AttackingEntity)
            .optional(LootContextParam::DirectAttackingEntity)
            .optional(LootContextParam::LastDamagePlayer);
    })
});

pub static EQUIPMENT: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("equipment", |b| {
        b.required(LootContextParam::Origin)
            .required(LootContextParam::ThisEntity);
    })
});

pub static ARCHAEOLOGY: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("archaeology", |b| {
        b.required(LootContextParam::Origin)
            .required(LootContextParam::ThisEntity)
            .required(LootContextParam::Tool);
    })
});

pub static GIFT: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("gift", |b| {
        b.required(LootContextParam::Origin)
            .required(LootContextParam::ThisEntity);
    })
});

pub static PIGLIN_BARTER: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("barter", |b| {
        b.required(LootContextParam::ThisEntity);
    })
});

pub static VAULT: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("vault", |b| {
        b.required(LootContextParam::Origin)
            .optional(LootContextParam::ThisEntity)
            .optional(LootContextParam::Tool);
    })
});

pub static ADVANCEMENT_REWARD: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("advancement_reward", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::Origin);
    })
});

pub static ADVANCEMENT_ENTITY: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("advancement_entity", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::Origin);
    })
});

pub static ADVANCEMENT_LOCATION: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("advancement_location", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::Origin)
            .required(LootContextParam::Tool)
            .required(LootContextParam::BlockState);
    })
});

pub static BLOCK_USE: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("block_use", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::Origin)
            .required(LootContextParam::BlockState);
    })
});

pub static ALL_PARAMS: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("generic", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::LastDamagePlayer)
            .required(LootContextParam::DamageSource)
            .required(LootContextParam::AttackingEntity)
            .required(LootContextParam::DirectAttackingEntity)
            .required(LootContextParam::Origin)
            .required(LootContextParam::BlockState)
            .required(LootContextParam::BlockEntity)
            .required(LootContextParam::Tool)
            .required(LootContextParam::ExplosionRadius);
    })
});

pub static BLOCK: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("block", |b| {
        b.required(LootContextParam::BlockState)
            .required(LootContextParam::Origin)
            .required(LootContextParam::Tool)
            .optional(LootContextParam::ThisEntity)
            .optional(LootContextParam::BlockEntity)
            .optional(LootContextParam::ExplosionRadius);
    })
});

pub static SHEARING: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("shearing", |b| {
        b.required(LootContextParam::Origin)
            .required(LootContextParam::ThisEntity)
            .required(LootContextParam::Tool);
    })
});

pub static ENTITY_INTERACT: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("entity_interact", |b| {
        b.required(LootContextParam::TargetEntity)
            .optional(LootContextParam::InteractingEntity)
            .required(LootContextParam::Tool);
    })
});

pub static BLOCK_INTERACT: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("block_interact", |b| {
        b.required(LootContextParam::BlockState)
            .optional(LootContextParam::BlockEntity)
            .optional(LootContextParam::InteractingEntity)
            .optional(LootContextParam::Tool);
    })
});

pub static ENCHANTED_DAMAGE: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("enchanted_damage", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::EnchantmentLevel)
            .required(LootContextParam::Origin)
            .required(LootContextParam::DamageSource)
            .optional(LootContextParam::DirectAttackingEntity)
            .optional(LootContextParam::AttackingEntity);
    })
});

pub static ENCHANTED_ITEM: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("enchanted_item", |b| {
        b.required(LootContextParam::Tool)
            .required(LootContextParam::EnchantmentLevel);
    })
});

pub static ENCHANTED_LOCATION: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("enchanted_location", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::EnchantmentLevel)
            .required(LootContextParam::Origin)
            .required(LootContextParam::EnchantmentActive);
    })
});

pub static ENCHANTED_ENTITY: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("enchanted_entity", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::EnchantmentLevel)
            .required(LootContextParam::Origin);
    })
});

pub static HIT_BLOCK: LazyLock<ContextKeySet> = LazyLock::new(|| {
    register("hit_block", |b| {
        b.required(LootContextParam::ThisEntity)
            .required(LootContextParam::EnchantmentLevel)
            .required(LootContextParam::Origin)
            .required(LootContextParam::BlockState);
    })
});

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParamSets#CODEC
pub fn codec_decode(id: &Identifier) -> Result<ContextKeySet, String> {
    ensure_registered();
    let reg = REGISTRY
        .lock()
        .expect("loot context param sets registry poisoned");
    reg.get(id)
        .cloned()
        .ok_or_else(|| format!("No parameter set exists with id: '{}'", id))
}

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParamSets#CODEC
pub fn codec_encode(set: &ContextKeySet) -> Option<Identifier> {
    ensure_registered();
    let reg = REGISTRY
        .lock()
        .expect("loot context param sets registry poisoned");
    reg.iter().find(|(_, v)| *v == set).map(|(k, _)| k.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_set_has_no_params() {
        ensure_registered();
        assert!(EMPTY.required().is_empty());
        assert!(EMPTY.allowed().is_empty());
    }

    #[test]
    fn chest_required_and_optional_semantics() {
        ensure_registered();
        assert!(CHEST.is_required(LootContextParam::Origin));
        assert!(CHEST.is_allowed(LootContextParam::Origin));
        assert!(!CHEST.is_required(LootContextParam::ThisEntity));
        assert!(CHEST.is_allowed(LootContextParam::ThisEntity));
        assert!(!CHEST.is_allowed(LootContextParam::Tool));
    }

    #[test]
    fn builder_required_overrides_optional() {
        let mut b = ContextKeySetBuilder::new();
        b.optional(LootContextParam::Origin);
        b.required(LootContextParam::Origin);
        let set = b.build();
        assert!(set.is_required(LootContextParam::Origin));
        assert!(set.is_allowed(LootContextParam::Origin));
    }

    #[test]
    fn codec_decode_unknown_id_error() {
        let id = Identifier::with_default_namespace("nope");
        let err = codec_decode(&id).unwrap_err();
        assert_eq!(err, "No parameter set exists with id: 'minecraft:nope'");
    }

    #[test]
    fn codec_encode_inverse_lookup() {
        ensure_registered();
        let id = codec_encode(&EMPTY).expect("empty should have id");
        assert_eq!(id, Identifier::with_default_namespace("empty"));
    }

    #[test]
    #[should_panic(
        expected = "Loot table parameter set minecraft:test_dup_param_set is already registered"
    )]
    fn duplicate_registration_panics() {
        let _ = register("test_dup_param_set", |_b| {});
        let _ = register("test_dup_param_set", |_b| {});
    }
}
