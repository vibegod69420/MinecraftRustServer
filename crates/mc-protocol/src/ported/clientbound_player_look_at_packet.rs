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
pub enum Anchor {
    Feet,
    Eyes,
}

impl Anchor {
    fn from_ordinal(ordinal: i32) -> Self {
        match ordinal {
            0 => Anchor::Feet,
            1 => Anchor::Eyes,
            _ => panic!("invalid Anchor ordinal: {}", ordinal),
        }
    }

    fn ordinal(self) -> i32 {
        match self {
            Anchor::Feet => 0,
            Anchor::Eyes => 1,
        }
    }

    // JAVA: net.minecraft.commands.arguments.EntityAnchorArgument$Anchor#apply
    pub fn apply(self, entity: &impl EntityView) -> Vec3 {
        match self {
            Anchor::Feet => entity.position(),
            Anchor::Eyes => entity.eye_position(),
        }
    }
}

pub trait EntityView {
    fn get_id(&self) -> i32;
    fn position(&self) -> Vec3;
    fn eye_position(&self) -> Vec3;
}

pub trait LevelView {
    type Entity: EntityView;
    fn get_entity(&self, id: i32) -> Option<&Self::Entity>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

impl PacketType {
    pub const CLIENTBOUND_PLAYER_LOOK_AT: PacketType =
        PacketType("minecraft:clientbound_player_look_at");
}

pub trait ClientGamePacketListener {
    fn handle_look_at(&mut self, packet: &ClientboundPlayerLookAtPacket);
}

pub trait WireEnum: Copy {
    fn from_wire_ordinal(ordinal: i32) -> Self;
    fn wire_ordinal(self) -> i32;
}

impl WireEnum for Anchor {
    fn from_wire_ordinal(ordinal: i32) -> Self {
        Anchor::from_ordinal(ordinal)
    }

    fn wire_ordinal(self) -> i32 {
        self.ordinal()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

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

    // JAVA: net.minecraft.network.FriendlyByteBuf#readEnum
    pub fn read_enum<T: WireEnum>(&mut self) -> T {
        T::from_wire_ordinal(self.read_var_int())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeEnum
    pub fn write_enum<T: WireEnum>(&mut self, value: T) -> &mut Self {
        self.write_var_int(value.wire_ordinal());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundPlayerLookAtPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub entity: i32,
    pub from_anchor: Anchor,
    pub to_anchor: Option<Anchor>,
    pub at_entity: bool,
}

impl ClientboundPlayerLookAtPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket#<init>(net.minecraft.commands.arguments.EntityAnchorArgument$Anchor,double,double,double)
    pub fn new_from_position(from_anchor: Anchor, x: f64, y: f64, z: f64) -> Self {
        Self {
            from_anchor,
            x,
            y,
            z,
            entity: 0,
            at_entity: false,
            to_anchor: None,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket#<init>(net.minecraft.commands.arguments.EntityAnchorArgument$Anchor,net.minecraft.world.entity.Entity,net.minecraft.commands.arguments.EntityAnchorArgument$Anchor)
    pub fn new_from_entity(from_anchor: Anchor, entity: &impl EntityView, to_anchor: Anchor) -> Self {
        let pos = to_anchor.apply(entity);
        Self {
            from_anchor,
            entity: entity.get_id(),
            to_anchor: Some(to_anchor),
            x: pos.x,
            y: pos.y,
            z: pos.z,
            at_entity: true,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket#<init>(net.minecraft.network.FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let from_anchor = buf.read_enum::<Anchor>();
        let x = buf.read_double();
        let y = buf.read_double();
        let z = buf.read_double();
        let at_entity = buf.read_boolean();
        let (entity, to_anchor) = if at_entity {
            (buf.read_var_int(), Some(buf.read_enum::<Anchor>()))
        } else {
            (0, None)
        };
        Self {
            x,
            y,
            z,
            entity,
            from_anchor,
            to_anchor,
            at_entity,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_enum(self.from_anchor);
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_boolean(self.at_entity);
        if self.at_entity {
            buf.write_var_int(self.entity);
            let to_anchor = self
                .to_anchor
                .expect("ClientboundPlayerLookAtPacket: at_entity is true but to_anchor is missing");
            buf.write_enum(to_anchor);
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::CLIENTBOUND_PLAYER_LOOK_AT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_look_at(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket#getFromAnchor
    pub fn get_from_anchor(&self) -> Anchor {
        self.from_anchor
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket#getPosition
    pub fn get_position<L: LevelView>(&self, level: &L) -> Vec3 {
        if self.at_entity {
            if let Some(entity) = level.get_entity(self.entity) {
                let to_anchor = self
                    .to_anchor
                    .expect("ClientboundPlayerLookAtPacket: at_entity is true but to_anchor is missing");
                return to_anchor.apply(entity);
            }
            return Vec3::new(self.x, self.y, self.z);
        }
        Vec3::new(self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestEntity {
        id: i32,
        pos: Vec3,
        eye: Vec3,
    }

    impl EntityView for TestEntity {
        fn get_id(&self) -> i32 {
            self.id
        }

        fn position(&self) -> Vec3 {
            self.pos
        }

        fn eye_position(&self) -> Vec3 {
            self.eye
        }
    }

    #[derive(Debug, Default)]
    struct TestLevel {
        entity: Option<TestEntity>,
    }

    impl LevelView for TestLevel {
        type Entity = TestEntity;

        fn get_entity(&self, id: i32) -> Option<&Self::Entity> {
            match &self.entity {
                Some(e) if e.get_id() == id => Some(e),
                _ => None,
            }
        }
    }

    #[test]
    fn round_trip_position_variant() {
        let pkt = ClientboundPlayerLookAtPacket::new_from_position(Anchor::Eyes, 1.5, -64.25, 3.0);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundPlayerLookAtPacket::read(&mut read_buf);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_from_anchor(), Anchor::Eyes);
        assert!(!decoded.at_entity);
    }

    #[test]
    fn round_trip_entity_variant() {
        let entity = TestEntity {
            id: 42,
            pos: Vec3::new(10.0, 20.0, 30.0),
            eye: Vec3::new(10.0, 21.62, 30.0),
        };
        let pkt = ClientboundPlayerLookAtPacket::new_from_entity(Anchor::Feet, &entity, Anchor::Eyes);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundPlayerLookAtPacket::read(&mut read_buf);
        assert_eq!(pkt, decoded);
        assert!(decoded.at_entity);
        assert_eq!(decoded.entity, 42);
        assert_eq!(decoded.to_anchor, Some(Anchor::Eyes));
    }

    #[test]
    fn get_position_static_returns_stored_coordinates() {
        let pkt = ClientboundPlayerLookAtPacket::new_from_position(Anchor::Feet, -1.0, 2.0, -3.5);
        let level = TestLevel { entity: None };
        assert_eq!(pkt.get_position(&level), Vec3::new(-1.0, 2.0, -3.5));
    }

    #[test]
    fn get_position_at_entity_uses_current_entity_anchor() {
        let source_entity = TestEntity {
            id: 7,
            pos: Vec3::new(1.0, 2.0, 3.0),
            eye: Vec3::new(1.0, 3.62, 3.0),
        };
        let pkt = ClientboundPlayerLookAtPacket::new_from_entity(Anchor::Feet, &source_entity, Anchor::Eyes);
        let level = TestLevel {
            entity: Some(TestEntity {
                id: 7,
                pos: Vec3::new(100.0, 200.0, 300.0),
                eye: Vec3::new(100.0, 201.8, 300.0),
            }),
        };
        assert_eq!(pkt.get_position(&level), Vec3::new(100.0, 201.8, 300.0));
    }

    #[test]
    fn get_position_at_entity_missing_falls_back_to_stored() {
        let source_entity = TestEntity {
            id: 7,
            pos: Vec3::new(1.0, 2.0, 3.0),
            eye: Vec3::new(1.0, 3.62, 3.0),
        };
        let pkt = ClientboundPlayerLookAtPacket::new_from_entity(Anchor::Feet, &source_entity, Anchor::Eyes);
        let level = TestLevel { entity: None };
        assert_eq!(pkt.get_position(&level), Vec3::new(1.0, 3.62, 3.0));
    }

    #[test]
    fn float_bits_preserved_through_wire_round_trip() {
        let x = f64::from_bits(0x7FF8_0000_0000_0001); // NaN payload
        let y = f64::NEG_INFINITY;
        let z = -0.0f64;
        let pkt = ClientboundPlayerLookAtPacket::new_from_position(Anchor::Feet, x, y, z);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundPlayerLookAtPacket::read(&mut read_buf);
        assert_eq!(decoded.x.to_bits(), x.to_bits());
        assert_eq!(decoded.y.to_bits(), y.to_bits());
        assert_eq!(decoded.z.to_bits(), z.to_bits());
        assert!(decoded.z.is_sign_negative());
    }
}
