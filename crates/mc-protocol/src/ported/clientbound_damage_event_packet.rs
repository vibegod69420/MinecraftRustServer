#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    // JAVA: net.minecraft.world.phys.Vec3#<init>(double,double,double)
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HolderDamageType {
    pub id: i32,
}

impl HolderDamageType {
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let raw = buf.read_var_int();
        let id = raw.wrapping_sub(1);
        if id < 0 {
            panic!("direct damage type holders are not supported in this self-contained port");
        }
        Self { id }
    }

    fn encode(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id.wrapping_add(1));
    }
}

pub trait EntityView {
    fn get_id(&self) -> i32;
}

pub trait LevelView {
    type Entity: EntityView;
    fn get_entity(&self, id: i32) -> Option<&Self::Entity>;
}

pub trait DamageSourceView {
    fn type_holder(&self) -> HolderDamageType;
    fn causing_entity_id(&self) -> Option<i32>;
    fn direct_entity_id(&self) -> Option<i32>;
    fn source_position_raw(&self) -> Option<Vec3>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamageSource {
    pub type_holder: HolderDamageType,
    pub direct_entity_id: Option<i32>,
    pub causing_entity_id: Option<i32>,
    pub position: Option<Vec3>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

impl PacketType {
    pub const CLIENTBOUND_DAMAGE_EVENT: PacketType =
        PacketType("minecraft:clientbound_damage_event");
}

pub trait ClientGamePacketListener {
    fn handle_damage_event(&mut self, packet: &ClientboundDamageEventPacket);
}

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

pub type RegistryFriendlyByteBuf = FriendlyByteBuf;

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    fn read_byte(&mut self) -> u8 {
        if self.read_pos >= self.data.len() {
            panic!("FriendlyByteBuf: read past end");
        }
        let b = self.data[self.read_pos];
        self.read_pos = self.read_pos.wrapping_add(1);
        b
    }

    fn write_byte(&mut self, value: u8) {
        self.data.push(value);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let byte = self.read_byte();
            value |= ((byte & 0x7F) as i32) << position;
            if (byte & 0x80) == 0 {
                break;
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                panic!("VarInt too big");
            }
        }
        value
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if ((value as u32) & !0x7F_u32) == 0 {
                self.write_byte(value as u8);
                return self;
            }
            self.write_byte(((value as u8) & 0x7F) | 0x80);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readDouble
    pub fn read_double(&mut self) -> f64 {
        let mut bytes = [0u8; 8];
        for b in &mut bytes {
            *b = self.read_byte();
        }
        f64::from_be_bytes(bytes)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeDouble
    pub fn write_double(&mut self, value: f64) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
    pub fn read_boolean(&mut self) -> bool {
        self.read_byte() != 0
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_boolean(&mut self, value: bool) -> &mut Self {
        self.write_byte(if value { 1 } else { 0 });
        self
    }
}

// JAVA: net.minecraft.network.protocol.game.ClientboundDamageEventPacket#writeOptionalEntityId
fn write_optional_entity_id(buf: &mut FriendlyByteBuf, id: i32) {
    buf.write_var_int(id.wrapping_add(1));
}

// JAVA: net.minecraft.network.protocol.game.ClientboundDamageEventPacket#readOptionalEntityId
fn read_optional_entity_id(buf: &mut FriendlyByteBuf) -> i32 {
    buf.read_var_int().wrapping_sub(1)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundDamageEventPacket {
    pub entity_id: i32,
    pub source_type: HolderDamageType,
    pub source_cause_id: i32,
    pub source_direct_id: i32,
    pub source_position: Option<Vec3>,
}

impl ClientboundDamageEventPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundDamageEventPacket#<init>(int,net.minecraft.core.Holder,int,int,java.util.Optional)
    pub fn new(
        entity_id: i32,
        source_type: HolderDamageType,
        source_cause_id: i32,
        source_direct_id: i32,
        source_position: Option<Vec3>,
    ) -> Self {
        Self {
            entity_id,
            source_type,
            source_cause_id,
            source_direct_id,
            source_position,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDamageEventPacket#<init>(net.minecraft.world.entity.Entity,net.minecraft.world.damagesource.DamageSource)
    pub fn from_entity_and_source(entity: &impl EntityView, source: &impl DamageSourceView) -> Self {
        Self {
            entity_id: entity.get_id(),
            source_type: source.type_holder(),
            source_cause_id: source.causing_entity_id().unwrap_or(-1),
            source_direct_id: source.direct_entity_id().unwrap_or(-1),
            source_position: source.source_position_raw(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDamageEventPacket#<init>(net.minecraft.network.RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let entity_id = buf.read_var_int();
        let source_type = HolderDamageType::decode(buf);
        let source_cause_id = read_optional_entity_id(buf);
        let source_direct_id = read_optional_entity_id(buf);
        let source_position = if buf.read_boolean() {
            Some(Vec3::new(
                buf.read_double(),
                buf.read_double(),
                buf.read_double(),
            ))
        } else {
            None
        };
        Self {
            entity_id,
            source_type,
            source_cause_id,
            source_direct_id,
            source_position,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDamageEventPacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        self.source_type.encode(buf);
        write_optional_entity_id(buf, self.source_cause_id);
        write_optional_entity_id(buf, self.source_direct_id);
        match self.source_position {
            Some(pos) => {
                buf.write_boolean(true);
                buf.write_double(pos.x);
                buf.write_double(pos.y);
                buf.write_double(pos.z);
            }
            None => {
                buf.write_boolean(false);
            }
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDamageEventPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::CLIENTBOUND_DAMAGE_EVENT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDamageEventPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_damage_event(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDamageEventPacket#getSource
    pub fn get_source<L: LevelView>(&self, level: &L) -> DamageSource {
        if let Some(pos) = self.source_position {
            return DamageSource {
                type_holder: self.source_type,
                direct_entity_id: None,
                causing_entity_id: None,
                position: Some(pos),
            };
        }
        let causing_entity_id = level.get_entity(self.source_cause_id).map(|e| e.get_id());
        let direct_entity_id = level.get_entity(self.source_direct_id).map(|e| e.get_id());
        DamageSource {
            type_holder: self.source_type,
            direct_entity_id,
            causing_entity_id,
            position: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestEntity {
        id: i32,
    }

    impl EntityView for TestEntity {
        fn get_id(&self) -> i32 {
            self.id
        }
    }

    #[derive(Debug, Default)]
    struct TestLevel {
        entities: Vec<TestEntity>,
    }

    impl LevelView for TestLevel {
        type Entity = TestEntity;

        fn get_entity(&self, id: i32) -> Option<&Self::Entity> {
            self.entities.iter().find(|e| e.get_id() == id)
        }
    }

    #[derive(Debug, Clone)]
    struct TestDamageSource {
        type_holder: HolderDamageType,
        causing_entity_id: Option<i32>,
        direct_entity_id: Option<i32>,
        source_position_raw: Option<Vec3>,
    }

    impl DamageSourceView for TestDamageSource {
        fn type_holder(&self) -> HolderDamageType {
            self.type_holder
        }

        fn causing_entity_id(&self) -> Option<i32> {
            self.causing_entity_id
        }

        fn direct_entity_id(&self) -> Option<i32> {
            self.direct_entity_id
        }

        fn source_position_raw(&self) -> Option<Vec3> {
            self.source_position_raw
        }
    }

    #[test]
    fn round_trip_with_position_preserves_float_bits() {
        let pkt = ClientboundDamageEventPacket::new(
            42,
            HolderDamageType::new(7),
            -1,
            -1,
            Some(Vec3::new(
                f64::from_bits(0x7FF8_0000_0000_0001),
                f64::NEG_INFINITY,
                -0.0,
            )),
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundDamageEventPacket::read(&mut r);
        assert_eq!(decoded.entity_id, 42);
        assert_eq!(decoded.source_type, HolderDamageType::new(7));
        assert_eq!(decoded.source_cause_id, -1);
        assert_eq!(decoded.source_direct_id, -1);
        let pos = decoded.source_position.expect("position should be present");
        assert_eq!(pos.x.to_bits(), 0x7FF8_0000_0000_0001);
        assert_eq!(pos.y.to_bits(), f64::NEG_INFINITY.to_bits());
        assert_eq!(pos.z.to_bits(), (-0.0f64).to_bits());
        assert!(pos.z.is_sign_negative());
    }

    #[test]
    fn round_trip_without_position_and_negative_entity_id() {
        let pkt = ClientboundDamageEventPacket::new(-5, HolderDamageType::new(0), 100, -1, None);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundDamageEventPacket::read(&mut r);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn optional_entity_id_wrapping_semantics() {
        let mut buf = FriendlyByteBuf::new();
        write_optional_entity_id(&mut buf, -1);
        write_optional_entity_id(&mut buf, i32::MAX);
        write_optional_entity_id(&mut buf, i32::MIN);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        assert_eq!(read_optional_entity_id(&mut r), -1);
        assert_eq!(read_optional_entity_id(&mut r), i32::MAX);
        assert_eq!(read_optional_entity_id(&mut r), i32::MIN);
    }

    #[test]
    fn get_source_with_position_ignores_entity_lookups() {
        let pkt = ClientboundDamageEventPacket::new(
            1,
            HolderDamageType::new(3),
            10,
            20,
            Some(Vec3::new(1.0, 2.0, 3.0)),
        );
        let level = TestLevel {
            entities: vec![TestEntity { id: 10 }, TestEntity { id: 20 }],
        };
        let src = pkt.get_source(&level);
        assert_eq!(src.type_holder, HolderDamageType::new(3));
        assert_eq!(src.position, Some(Vec3::new(1.0, 2.0, 3.0)));
        assert_eq!(src.causing_entity_id, None);
        assert_eq!(src.direct_entity_id, None);
    }

    #[test]
    fn get_source_without_position_looks_up_entities() {
        let pkt = ClientboundDamageEventPacket::new(1, HolderDamageType::new(5), 10, 20, None);
        let level = TestLevel {
            entities: vec![TestEntity { id: 10 }],
        };
        let src = pkt.get_source(&level);
        assert_eq!(src.type_holder, HolderDamageType::new(5));
        assert_eq!(src.position, None);
        assert_eq!(src.causing_entity_id, Some(10));
        assert_eq!(src.direct_entity_id, None);
    }

    #[test]
    fn from_entity_and_source_constructor_maps_missing_entities_to_minus_one() {
        let entity = TestEntity { id: 99 };
        let source = TestDamageSource {
            type_holder: HolderDamageType::new(4),
            causing_entity_id: Some(10),
            direct_entity_id: None,
            source_position_raw: None,
        };
        let pkt = ClientboundDamageEventPacket::from_entity_and_source(&entity, &source);
        assert_eq!(pkt.entity_id, 99);
        assert_eq!(pkt.source_type, HolderDamageType::new(4));
        assert_eq!(pkt.source_cause_id, 10);
        assert_eq!(pkt.source_direct_id, -1);
        assert_eq!(pkt.source_position, None);
    }
}
