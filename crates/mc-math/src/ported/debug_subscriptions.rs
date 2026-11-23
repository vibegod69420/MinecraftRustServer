use std::any::Any;
use std::marker::PhantomData;
use std::sync::{Arc, LazyLock, Mutex};

/// Minimal self-contained placeholder for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_owned(),
            path: path.to_owned(),
        }
    }
}

/// Minimal self-contained placeholder for `net.minecraft.network.RegistryFriendlyByteBuf`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegistryFriendlyByteBuf;

/// Minimal self-contained placeholder for `net.minecraft.network.codec.StreamCodec`.
#[derive(Debug)]
pub struct StreamCodec<B, V>(PhantomData<fn(&B) -> V>);

impl<B, V> Clone for StreamCodec<B, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<B, V> Copy for StreamCodec<B, V> {}

/// Marker operation for `ByteBufCodecs.list()`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ListCodecOp;

/// Minimal self-contained placeholder for `net.minecraft.network.codec.ByteBufCodecs`.
pub struct ByteBufCodecs;

impl ByteBufCodecs {
    pub fn list() -> ListCodecOp {
        ListCodecOp
    }
}

impl<B, V> StreamCodec<B, V> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }

    pub fn apply(self, _op: ListCodecOp) -> StreamCodec<B, Vec<V>> {
        StreamCodec::new()
    }
}

/// Minimal self-contained placeholder for `net.minecraft.util.Unit`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Unit;
impl Unit {
    pub const STREAM_CODEC: StreamCodec<RegistryFriendlyByteBuf, Self> = StreamCodec::new();
}

/// Minimal self-contained placeholder for `net.minecraft.core.BlockPos`.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockPos;
impl BlockPos {
    pub const STREAM_CODEC: StreamCodec<RegistryFriendlyByteBuf, Self> = StreamCodec::new();
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.redstone.Orientation`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Orientation;
impl Orientation {
    pub const STREAM_CODEC: StreamCodec<RegistryFriendlyByteBuf, Self> = StreamCodec::new();
}

macro_rules! placeholder_debug_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $name;
        impl $name {
            pub const STREAM_CODEC: StreamCodec<RegistryFriendlyByteBuf, Self> =
                StreamCodec::new();
        }
    };
}

placeholder_debug_type!(DebugBeeInfo);
placeholder_debug_type!(DebugBrainDump);
placeholder_debug_type!(DebugBreezeInfo);
placeholder_debug_type!(DebugGoalInfo);
placeholder_debug_type!(DebugPathInfo);
placeholder_debug_type!(DebugEntityBlockIntersection);
placeholder_debug_type!(DebugHiveInfo);
placeholder_debug_type!(DebugPoiInfo);
placeholder_debug_type!(DebugStructureInfo);
placeholder_debug_type!(DebugGameEventListenerInfo);
placeholder_debug_type!(DebugGameEventInfo);

/// Minimal self-contained representation of `net.minecraft.util.debug.DebugSubscription`.
#[derive(Debug)]
pub struct DebugSubscription<T: 'static> {
    pub value_codec: Option<StreamCodec<RegistryFriendlyByteBuf, T>>,
    pub expiry_ticks: Option<i32>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> DebugSubscription<T> {
    pub fn new(value_codec: Option<StreamCodec<RegistryFriendlyByteBuf, T>>) -> Self {
        Self {
            value_codec,
            expiry_ticks: None,
            _marker: PhantomData,
        }
    }

    pub fn with_expiry(
        value_codec: Option<StreamCodec<RegistryFriendlyByteBuf, T>>,
        expiry_ticks: i32,
    ) -> Self {
        Self {
            value_codec,
            expiry_ticks: Some(expiry_ticks),
            _marker: PhantomData,
        }
    }
}

/// Minimal self-contained registry for debug subscriptions.
#[derive(Default)]
pub struct DebugSubscriptionRegistry {
    entries: Vec<(Identifier, Arc<dyn Any + Send + Sync>)>,
}

impl DebugSubscriptionRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register<T: 'static + Send + Sync>(
        &mut self,
        name: Identifier,
        value: DebugSubscription<T>,
    ) -> Arc<DebugSubscription<T>> {
        let arc = Arc::new(value);
        self.entries.push((name, arc.clone()));
        arc
    }

    pub fn entries(&self) -> &[(Identifier, Arc<dyn Any + Send + Sync>)] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Global stand-in for `BuiltInRegistries.DEBUG_SUBSCRIPTION`.
pub static DEBUG_SUBSCRIPTION_REGISTRY: LazyLock<Mutex<DebugSubscriptionRegistry>> =
    LazyLock::new(|| Mutex::new(DebugSubscriptionRegistry::new()));

fn register<T: 'static + Send + Sync>(
    name: Identifier,
    value: DebugSubscription<T>,
) -> Arc<DebugSubscription<T>> {
    DEBUG_SUBSCRIPTION_REGISTRY
        .lock()
        .expect("debug subscription registry poisoned")
        .register(name, value)
}

// JAVA: net.minecraft.util.debug.DebugSubscriptions#bootstrap
pub fn bootstrap(_registry: &mut DebugSubscriptionRegistry) -> Arc<DebugSubscription<()>> {
    Arc::clone(&DEDICATED_SERVER_TICK_TIME)
}

// JAVA: net.minecraft.util.debug.DebugSubscriptions#registerSimple
fn register_simple(name: &str) -> Arc<DebugSubscription<()>> {
    register(
        Identifier::with_default_namespace(name),
        DebugSubscription::new(None),
    )
}

// JAVA: net.minecraft.util.debug.DebugSubscriptions#registerWithValue
fn register_with_value<T: 'static + Send + Sync>(
    name: &str,
    codec: StreamCodec<RegistryFriendlyByteBuf, T>,
) -> Arc<DebugSubscription<T>> {
    register(
        Identifier::with_default_namespace(name),
        DebugSubscription::new(Some(codec)),
    )
}

// JAVA: net.minecraft.util.debug.DebugSubscriptions#registerTemporaryValue
fn register_temporary_value<T: 'static + Send + Sync>(
    name: &str,
    codec: StreamCodec<RegistryFriendlyByteBuf, T>,
    expiry_ticks: i32,
) -> Arc<DebugSubscription<T>> {
    register(
        Identifier::with_default_namespace(name),
        DebugSubscription::with_expiry(Some(codec), expiry_ticks),
    )
}

pub static DEDICATED_SERVER_TICK_TIME: LazyLock<Arc<DebugSubscription<()>>> =
    LazyLock::new(|| register_simple("dedicated_server_tick_time"));

pub static BEES: LazyLock<Arc<DebugSubscription<DebugBeeInfo>>> =
    LazyLock::new(|| register_with_value("bees", DebugBeeInfo::STREAM_CODEC));

pub static BRAINS: LazyLock<Arc<DebugSubscription<DebugBrainDump>>> =
    LazyLock::new(|| register_with_value("brains", DebugBrainDump::STREAM_CODEC));

pub static BREEZES: LazyLock<Arc<DebugSubscription<DebugBreezeInfo>>> =
    LazyLock::new(|| register_with_value("breezes", DebugBreezeInfo::STREAM_CODEC));

pub static GOAL_SELECTORS: LazyLock<Arc<DebugSubscription<DebugGoalInfo>>> =
    LazyLock::new(|| register_with_value("goal_selectors", DebugGoalInfo::STREAM_CODEC));

pub static ENTITY_PATHS: LazyLock<Arc<DebugSubscription<DebugPathInfo>>> =
    LazyLock::new(|| register_with_value("entity_paths", DebugPathInfo::STREAM_CODEC));

pub static ENTITY_BLOCK_INTERSECTIONS: LazyLock<
    Arc<DebugSubscription<DebugEntityBlockIntersection>>,
> = LazyLock::new(|| {
    register_temporary_value(
        "entity_block_intersections",
        DebugEntityBlockIntersection::STREAM_CODEC,
        100,
    )
});

pub static BEE_HIVES: LazyLock<Arc<DebugSubscription<DebugHiveInfo>>> =
    LazyLock::new(|| register_with_value("bee_hives", DebugHiveInfo::STREAM_CODEC));

pub static POIS: LazyLock<Arc<DebugSubscription<DebugPoiInfo>>> =
    LazyLock::new(|| register_with_value("pois", DebugPoiInfo::STREAM_CODEC));

pub static REDSTONE_WIRE_ORIENTATIONS: LazyLock<Arc<DebugSubscription<Orientation>>> =
    LazyLock::new(|| {
        register_temporary_value(
            "redstone_wire_orientations",
            Orientation::STREAM_CODEC,
            200,
        )
    });

pub static VILLAGE_SECTIONS: LazyLock<Arc<DebugSubscription<Unit>>> =
    LazyLock::new(|| register_with_value("village_sections", Unit::STREAM_CODEC));

pub static RAIDS: LazyLock<Arc<DebugSubscription<Vec<BlockPos>>>> = LazyLock::new(|| {
    register_with_value(
        "raids",
        BlockPos::STREAM_CODEC.apply(ByteBufCodecs::list()),
    )
});

pub static STRUCTURES: LazyLock<Arc<DebugSubscription<Vec<DebugStructureInfo>>>> =
    LazyLock::new(|| {
        register_with_value(
            "structures",
            DebugStructureInfo::STREAM_CODEC.apply(ByteBufCodecs::list()),
        )
    });

pub static GAME_EVENT_LISTENERS: LazyLock<Arc<DebugSubscription<DebugGameEventListenerInfo>>> =
    LazyLock::new(|| {
        register_with_value(
            "game_event_listeners",
            DebugGameEventListenerInfo::STREAM_CODEC,
        )
    });

pub static NEIGHBOR_UPDATES: LazyLock<Arc<DebugSubscription<BlockPos>>> = LazyLock::new(|| {
    register_temporary_value("neighbor_updates", BlockPos::STREAM_CODEC, 200)
});

pub static GAME_EVENTS: LazyLock<Arc<DebugSubscription<DebugGameEventInfo>>> =
    LazyLock::new(|| {
        register_temporary_value("game_events", DebugGameEventInfo::STREAM_CODEC, 60)
    });

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_initialized() {
        let _ = &*DEDICATED_SERVER_TICK_TIME;
        let _ = &*BEES;
        let _ = &*BRAINS;
        let _ = &*BREEZES;
        let _ = &*GOAL_SELECTORS;
        let _ = &*ENTITY_PATHS;
        let _ = &*ENTITY_BLOCK_INTERSECTIONS;
        let _ = &*BEE_HIVES;
        let _ = &*POIS;
        let _ = &*REDSTONE_WIRE_ORIENTATIONS;
        let _ = &*VILLAGE_SECTIONS;
        let _ = &*RAIDS;
        let _ = &*STRUCTURES;
        let _ = &*GAME_EVENT_LISTENERS;
        let _ = &*NEIGHBOR_UPDATES;
        let _ = &*GAME_EVENTS;
    }

    #[test]
    fn bootstrap_returns_dedicated_server_tick_time() {
        ensure_initialized();
        let mut reg = DebugSubscriptionRegistry::new();
        let got = bootstrap(&mut reg);
        assert!(Arc::ptr_eq(&got, &DEDICATED_SERVER_TICK_TIME));
    }

    #[test]
    fn simple_subscription_has_no_codec_and_no_expiry() {
        let sub = &*DEDICATED_SERVER_TICK_TIME;
        assert!(sub.value_codec.is_none());
        assert_eq!(sub.expiry_ticks, None);
    }

    #[test]
    fn value_and_temporary_subscriptions_match_java() {
        let bees = &*BEES;
        assert!(bees.value_codec.is_some());
        assert_eq!(bees.expiry_ticks, None);

        assert_eq!(
            ENTITY_BLOCK_INTERSECTIONS.expiry_ticks,
            Some(100_i32)
        );
        assert_eq!(REDSTONE_WIRE_ORIENTATIONS.expiry_ticks, Some(200_i32));
        assert_eq!(NEIGHBOR_UPDATES.expiry_ticks, Some(200_i32));
        assert_eq!(GAME_EVENTS.expiry_ticks, Some(60_i32));
    }

    #[test]
    fn expiry_edge_cases_negative_and_wrapping_overflow() {
        let neg = DebugSubscription::<()>::with_expiry(None, -1_i32);
        assert_eq!(neg.expiry_ticks, Some(-1_i32));

        let min = DebugSubscription::<()>::with_expiry(None, i32::MIN);
        assert_eq!(min.expiry_ticks, Some(i32::MIN));

        let max = DebugSubscription::<()>::with_expiry(None, i32::MAX);
        assert_eq!(max.expiry_ticks, Some(i32::MAX));

        // Java int arithmetic semantics: wrapping overflow.
        let wrapped = i32::MAX.wrapping_add(1_i32);
        assert_eq!(wrapped, i32::MIN);
        let wrapped_sub = DebugSubscription::<()>::with_expiry(None, wrapped);
        assert_eq!(wrapped_sub.expiry_ticks, Some(i32::MIN));
    }

    #[test]
    fn registry_contains_expected_minecraft_identifiers() {
        ensure_initialized();
        let reg = DEBUG_SUBSCRIPTION_REGISTRY
            .lock()
            .expect("debug subscription registry poisoned");
        assert!(reg.len() >= 16);

        let has = |path: &str| {
            reg.entries()
                .iter()
                .any(|(id, _)| id.namespace == "minecraft" && id.path == path)
        };
        assert!(has("dedicated_server_tick_time"));
        assert!(has("bees"));
        assert!(has("entity_block_intersections"));
        assert!(has("raids"));
        assert!(has("game_events"));
    }
}
