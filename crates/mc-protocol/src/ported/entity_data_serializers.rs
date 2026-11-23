use std::sync::{LazyLock, RwLock};

/// Minimal self-contained stand-in for `net.minecraft.network.syncher.EntityDataSerializer<T>`.
/// The registry uses pointer identity, matching Java's identity-hash map semantics.
#[derive(Debug)]
pub struct EntityDataSerializer {
    name: &'static str,
}

impl EntityDataSerializer {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }
}

static SERIALIZERS: LazyLock<RwLock<Vec<&'static EntityDataSerializer>>> =
    LazyLock::new(|| RwLock::new(Vec::with_capacity(16)));

static INIT: LazyLock<()> = LazyLock::new(|| {
    // JAVA: static initializer registration order
    register_serializer(&BYTE);
    register_serializer(&INT);
    register_serializer(&LONG);
    register_serializer(&FLOAT);
    register_serializer(&STRING);
    register_serializer(&COMPONENT);
    register_serializer(&OPTIONAL_COMPONENT);
    register_serializer(&ITEM_STACK);
    register_serializer(&BOOLEAN);
    register_serializer(&ROTATIONS);
    register_serializer(&BLOCK_POS);
    register_serializer(&OPTIONAL_BLOCK_POS);
    register_serializer(&DIRECTION);
    register_serializer(&OPTIONAL_LIVING_ENTITY_REFERENCE);
    register_serializer(&BLOCK_STATE);
    register_serializer(&OPTIONAL_BLOCK_STATE);
    register_serializer(&PARTICLE);
    register_serializer(&PARTICLES);
    register_serializer(&VILLAGER_DATA);
    register_serializer(&OPTIONAL_UNSIGNED_INT);
    register_serializer(&POSE);
    register_serializer(&CAT_VARIANT);
    register_serializer(&COW_VARIANT);
    register_serializer(&WOLF_VARIANT);
    register_serializer(&WOLF_SOUND_VARIANT);
    register_serializer(&FROG_VARIANT);
    register_serializer(&PIG_VARIANT);
    register_serializer(&CHICKEN_VARIANT);
    register_serializer(&ZOMBIE_NAUTILUS_VARIANT);
    register_serializer(&OPTIONAL_GLOBAL_POS);
    register_serializer(&PAINTING_VARIANT);
    register_serializer(&SNIFFER_STATE);
    register_serializer(&ARMADILLO_STATE);
    register_serializer(&COPPER_GOLEM_STATE);
    register_serializer(&WEATHERING_COPPER_STATE);
    register_serializer(&VECTOR3);
    register_serializer(&QUATERNION);
    register_serializer(&RESOLVABLE_PROFILE);
    register_serializer(&HUMANOID_ARM);
});

fn ensure_init() {
    LazyLock::force(&INIT);
}

pub static BYTE: EntityDataSerializer = EntityDataSerializer::new("BYTE");
pub static INT: EntityDataSerializer = EntityDataSerializer::new("INT");
pub static LONG: EntityDataSerializer = EntityDataSerializer::new("LONG");
pub static FLOAT: EntityDataSerializer = EntityDataSerializer::new("FLOAT");
pub static STRING: EntityDataSerializer = EntityDataSerializer::new("STRING");
pub static COMPONENT: EntityDataSerializer = EntityDataSerializer::new("COMPONENT");
pub static OPTIONAL_COMPONENT: EntityDataSerializer =
    EntityDataSerializer::new("OPTIONAL_COMPONENT");
pub static ITEM_STACK: EntityDataSerializer = EntityDataSerializer::new("ITEM_STACK");
pub static BLOCK_STATE: EntityDataSerializer = EntityDataSerializer::new("BLOCK_STATE");
pub static OPTIONAL_BLOCK_STATE: EntityDataSerializer =
    EntityDataSerializer::new("OPTIONAL_BLOCK_STATE");
pub static BOOLEAN: EntityDataSerializer = EntityDataSerializer::new("BOOLEAN");
pub static PARTICLE: EntityDataSerializer = EntityDataSerializer::new("PARTICLE");
pub static PARTICLES: EntityDataSerializer = EntityDataSerializer::new("PARTICLES");
pub static ROTATIONS: EntityDataSerializer = EntityDataSerializer::new("ROTATIONS");
pub static BLOCK_POS: EntityDataSerializer = EntityDataSerializer::new("BLOCK_POS");
pub static OPTIONAL_BLOCK_POS: EntityDataSerializer =
    EntityDataSerializer::new("OPTIONAL_BLOCK_POS");
pub static DIRECTION: EntityDataSerializer = EntityDataSerializer::new("DIRECTION");
pub static OPTIONAL_LIVING_ENTITY_REFERENCE: EntityDataSerializer =
    EntityDataSerializer::new("OPTIONAL_LIVING_ENTITY_REFERENCE");
pub static OPTIONAL_GLOBAL_POS: EntityDataSerializer =
    EntityDataSerializer::new("OPTIONAL_GLOBAL_POS");
pub static VILLAGER_DATA: EntityDataSerializer = EntityDataSerializer::new("VILLAGER_DATA");
pub static OPTIONAL_UNSIGNED_INT: EntityDataSerializer =
    EntityDataSerializer::new("OPTIONAL_UNSIGNED_INT");
pub static POSE: EntityDataSerializer = EntityDataSerializer::new("POSE");
pub static CAT_VARIANT: EntityDataSerializer = EntityDataSerializer::new("CAT_VARIANT");
pub static CHICKEN_VARIANT: EntityDataSerializer = EntityDataSerializer::new("CHICKEN_VARIANT");
pub static COW_VARIANT: EntityDataSerializer = EntityDataSerializer::new("COW_VARIANT");
pub static WOLF_VARIANT: EntityDataSerializer = EntityDataSerializer::new("WOLF_VARIANT");
pub static WOLF_SOUND_VARIANT: EntityDataSerializer =
    EntityDataSerializer::new("WOLF_SOUND_VARIANT");
pub static FROG_VARIANT: EntityDataSerializer = EntityDataSerializer::new("FROG_VARIANT");
pub static PIG_VARIANT: EntityDataSerializer = EntityDataSerializer::new("PIG_VARIANT");
pub static ZOMBIE_NAUTILUS_VARIANT: EntityDataSerializer =
    EntityDataSerializer::new("ZOMBIE_NAUTILUS_VARIANT");
pub static PAINTING_VARIANT: EntityDataSerializer = EntityDataSerializer::new("PAINTING_VARIANT");
pub static ARMADILLO_STATE: EntityDataSerializer = EntityDataSerializer::new("ARMADILLO_STATE");
pub static SNIFFER_STATE: EntityDataSerializer = EntityDataSerializer::new("SNIFFER_STATE");
pub static WEATHERING_COPPER_STATE: EntityDataSerializer =
    EntityDataSerializer::new("WEATHERING_COPPER_STATE");
pub static COPPER_GOLEM_STATE: EntityDataSerializer =
    EntityDataSerializer::new("COPPER_GOLEM_STATE");
pub static VECTOR3: EntityDataSerializer = EntityDataSerializer::new("VECTOR3");
pub static QUATERNION: EntityDataSerializer = EntityDataSerializer::new("QUATERNION");
pub static RESOLVABLE_PROFILE: EntityDataSerializer =
    EntityDataSerializer::new("RESOLVABLE_PROFILE");
pub static HUMANOID_ARM: EntityDataSerializer = EntityDataSerializer::new("HUMANOID_ARM");

// JAVA: net.minecraft.network.syncher.EntityDataSerializers#registerSerializer
pub fn register_serializer(serializer: &'static EntityDataSerializer) {
    let mut registry = SERIALIZERS
        .write()
        .expect("serializer registry poisoned");
    registry.push(serializer);
}

// JAVA: net.minecraft.network.syncher.EntityDataSerializers#getSerializer
pub fn get_serializer(id: i32) -> Option<&'static EntityDataSerializer> {
    ensure_init();
    let registry = SERIALIZERS
        .read()
        .expect("serializer registry poisoned");
    if id < 0 {
        return None;
    }
    registry.get(id as usize).copied()
}

// JAVA: net.minecraft.network.syncher.EntityDataSerializers#getSerializedId
pub fn get_serialized_id(serializer: &EntityDataSerializer) -> i32 {
    ensure_init();
    let registry = SERIALIZERS
        .read()
        .expect("serializer registry poisoned");
    for (i, &s) in registry.iter().enumerate() {
        if std::ptr::eq(s, serializer) {
            return i as i32;
        }
    }
    -1
}

// JAVA: net.minecraft.network.syncher.EntityDataSerializers#OPTIONAL_BLOCK_STATE_CODEC.decode
/// Decodes the optional block state wire value.
/// Java: reads VarInt; 0 => empty, otherwise `Block.stateById(id)`.
/// Self-contained: returns the raw block state id as `Some(id)`.
pub fn optional_block_state_decode(varint_value: i32) -> Option<i32> {
    if varint_value == 0 {
        None
    } else {
        Some(varint_value)
    }
}

// JAVA: net.minecraft.network.syncher.EntityDataSerializers#OPTIONAL_BLOCK_STATE_CODEC.encode
/// Encodes the optional block state wire value.
/// Java: writes `Block.getId(state)` if present, otherwise 0.
/// Self-contained: takes the raw block state id.
pub fn optional_block_state_encode(block_state_id: Option<i32>) -> i32 {
    match block_state_id {
        Some(id) => id,
        None => 0,
    }
}

// JAVA: net.minecraft.network.syncher.EntityDataSerializers#OPTIONAL_UNSIGNED_INT_CODEC.decode
/// Decodes the optional unsigned int wire value.
/// Java: `VarInt.read() == 0 ? empty : of(value - 1)`.
pub fn optional_unsigned_int_decode(varint_value: i32) -> Option<i32> {
    if varint_value == 0 {
        None
    } else {
        Some(varint_value.wrapping_sub(1))
    }
}

// JAVA: net.minecraft.network.syncher.EntityDataSerializers#OPTIONAL_UNSIGNED_INT_CODEC.encode
/// Encodes the optional unsigned int wire value.
/// Java: `VarInt.write(value.orElse(-1) + 1)`.
pub fn optional_unsigned_int_encode(value: Option<i32>) -> i32 {
    value.unwrap_or(-1).wrapping_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_order_and_ids() {
        assert_eq!(get_serialized_id(&BYTE), 0);
        assert_eq!(get_serialized_id(&INT), 1);
        assert_eq!(get_serialized_id(&LONG), 2);
        assert_eq!(get_serialized_id(&FLOAT), 3);
        assert_eq!(get_serialized_id(&HUMANOID_ARM), 38);
        assert!(std::ptr::eq(get_serializer(0).unwrap(), &BYTE));
        assert!(std::ptr::eq(get_serializer(38).unwrap(), &HUMANOID_ARM));
        assert_eq!(get_serializer(0).unwrap().name(), "BYTE");
    }

    #[test]
    fn get_serializer_negative_and_out_of_range() {
        assert!(get_serializer(-1).is_none());
        assert!(get_serializer(-128).is_none());
        assert!(get_serializer(39).is_none());
        assert!(get_serializer(i32::MAX).is_none());
    }

    #[test]
    fn get_serialized_id_unknown_returns_minus_one() {
        static UNKNOWN: EntityDataSerializer = EntityDataSerializer::new("UNKNOWN");
        assert_eq!(get_serialized_id(&UNKNOWN), -1);
    }

    #[test]
    fn optional_unsigned_int_codec_edge_cases() {
        // decode
        assert_eq!(optional_unsigned_int_decode(0), None);
        assert_eq!(optional_unsigned_int_decode(1), Some(0));
        assert_eq!(optional_unsigned_int_decode(-1), Some(-2));
        assert_eq!(optional_unsigned_int_decode(i32::MIN), Some(i32::MAX));
        // encode
        assert_eq!(optional_unsigned_int_encode(None), 0);
        assert_eq!(optional_unsigned_int_encode(Some(0)), 1);
        assert_eq!(optional_unsigned_int_encode(Some(-1)), 0);
        assert_eq!(optional_unsigned_int_encode(Some(i32::MAX)), i32::MIN);
    }

    #[test]
    fn optional_block_state_codec_edge_cases() {
        assert_eq!(optional_block_state_decode(0), None);
        assert_eq!(optional_block_state_decode(42), Some(42));
        assert_eq!(optional_block_state_decode(-7), Some(-7));
        assert_eq!(optional_block_state_encode(None), 0);
        assert_eq!(optional_block_state_encode(Some(42)), 42);
        assert_eq!(optional_block_state_encode(Some(i32::MIN)), i32::MIN);
    }

    #[test]
    fn float_bits_edge_cases() {
        // FLOAT serializer is registered and float bit patterns are preserved
        assert_eq!(get_serialized_id(&FLOAT), 3);
        assert_eq!(1.0f32.to_bits(), 0x3f80_0000);
        assert_eq!((-0.0f32).to_bits(), 0x8000_0000);
        assert_eq!(f32::NEG_INFINITY.to_bits(), 0xFF80_0000);
        let nan_bits = f32::NAN.to_bits();
        assert_eq!(nan_bits & 0x7F80_0000, 0x7F80_0000);
        assert_ne!(nan_bits & 0x007F_FFFF, 0);
    }
}
