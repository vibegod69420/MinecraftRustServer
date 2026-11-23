use std::fmt;

/// Minimal UUID representation matching Java's `java.util.UUID` wire layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

/// Minimal 3D vector matching Java's `net.minecraft.world.phys.Vec3`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
}

/// Minimal block position matching Java's `net.minecraft.core.BlockPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Minimal entity type handle matching Java's `net.minecraft.world.entity.EntityType<?>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityType {
    pub id: i32,
}

/// Minimal packet type marker matching Java's `net.minecraft.network.protocol.PacketType<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType {
    pub id: &'static str,
}

pub const CLIENTBOUND_ADD_ENTITY_PACKET_TYPE: PacketType = PacketType {
    id: "minecraft:add_entity",
};

/// Minimal entity view required by `ClientboundAddEntityPacket` constructors.
pub trait EntityLike {
    fn get_id(&self) -> i32;
    fn get_uuid(&self) -> Uuid;
    fn get_type(&self) -> EntityType;
    fn get_x_rot(&self) -> f32;
    fn get_y_rot(&self) -> f32;
    fn get_delta_movement(&self) -> Vec3;
    fn get_y_head_rot(&self) -> f32;
}

/// Minimal server entity view required by `ClientboundAddEntityPacket` constructors.
pub trait ServerEntityLike {
    fn get_position_base(&self) -> Vec3;
    fn get_last_sent_x_rot(&self) -> f32;
    fn get_last_sent_y_rot(&self) -> f32;
    fn get_last_sent_movement(&self) -> Vec3;
    fn get_last_sent_y_head_rot(&self) -> f32;
}

/// Minimal buffer abstraction for the wire operations used by this packet.
pub trait RegistryFriendlyByteBuf {
    fn read_var_int(&mut self) -> i32;
    fn write_var_int(&mut self, value: i32);
    fn read_uuid(&mut self) -> Uuid;
    fn write_uuid(&mut self, value: Uuid);
    fn read_double(&mut self) -> f64;
    fn write_double(&mut self, value: f64);
    fn read_byte(&mut self) -> i8;
    fn write_byte(&mut self, value: i8);
    fn read_lp_vec3(&mut self) -> Vec3;
    fn write_lp_vec3(&mut self, value: Vec3);
    fn read_entity_type(&mut self) -> EntityType;
    fn write_entity_type(&mut self, value: EntityType);
}

/// Minimal packet listener for `ClientboundAddEntityPacket`.
pub trait ClientGamePacketListener {
    fn handle_add_entity(&mut self, packet: &ClientboundAddEntityPacket);
}

// JAVA: net.minecraft.util.Mth#floor(float)
pub fn mth_floor_f32(value: f32) -> i32 {
    let i = value as i32;
    if value < i as f32 {
        i.wrapping_sub(1)
    } else {
        i
    }
}

// JAVA: net.minecraft.util.Mth#packDegrees
pub fn mth_pack_degrees(degrees: f32) -> i8 {
    mth_floor_f32(degrees * 256.0_f32 / 360.0_f32) as i8
}

// JAVA: net.minecraft.util.Mth#unpackDegrees
pub fn mth_unpack_degrees(packed: i8) -> f32 {
    (packed as f32 * 360.0_f32) / 256.0_f32
}

/// Port of `net.minecraft.network.protocol.game.ClientboundAddEntityPacket`.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundAddEntityPacket {
    id: i32,
    uuid: Uuid,
    entity_type: EntityType,
    x: f64,
    y: f64,
    z: f64,
    movement: Vec3,
    x_rot: i8,
    y_rot: i8,
    y_head_rot: i8,
    data: i32,
}

impl ClientboundAddEntityPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#<init>(Entity,ServerEntity)
    pub fn new_from_entity(entity: &impl EntityLike, server_entity: &impl ServerEntityLike) -> Self {
        Self::new_from_entity_and_data(entity, server_entity, 0)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#<init>(Entity,ServerEntity,int)
    pub fn new_from_entity_and_data(
        entity: &impl EntityLike,
        server_entity: &impl ServerEntityLike,
        data: i32,
    ) -> Self {
        let pos = server_entity.get_position_base();
        Self::new_full(
            entity.get_id(),
            entity.get_uuid(),
            pos.x,
            pos.y,
            pos.z,
            server_entity.get_last_sent_x_rot(),
            server_entity.get_last_sent_y_rot(),
            entity.get_type(),
            data,
            server_entity.get_last_sent_movement(),
            server_entity.get_last_sent_y_head_rot() as f64,
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#<init>(Entity,int,BlockPos)
    pub fn new_from_entity_block_pos(entity: &impl EntityLike, data: i32, block_pos: BlockPos) -> Self {
        Self::new_full(
            entity.get_id(),
            entity.get_uuid(),
            block_pos.x as f64,
            block_pos.y as f64,
            block_pos.z as f64,
            entity.get_x_rot(),
            entity.get_y_rot(),
            entity.get_type(),
            data,
            entity.get_delta_movement(),
            entity.get_y_head_rot() as f64,
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#<init>(int,UUID,double,double,double,float,float,EntityType,int,Vec3,double)
    pub fn new_full(
        id: i32,
        uuid: Uuid,
        x: f64,
        y: f64,
        z: f64,
        x_rot: f32,
        y_rot: f32,
        entity_type: EntityType,
        data: i32,
        movement: Vec3,
        y_head_rot: f64,
    ) -> Self {
        Self {
            id,
            uuid,
            x,
            y,
            z,
            movement,
            x_rot: mth_pack_degrees(x_rot),
            y_rot: mth_pack_degrees(y_rot),
            y_head_rot: mth_pack_degrees(y_head_rot as f32),
            entity_type,
            data,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut impl RegistryFriendlyByteBuf) -> Self {
        let id = buf.read_var_int();
        let uuid = buf.read_uuid();
        let entity_type = buf.read_entity_type();
        let x = buf.read_double();
        let y = buf.read_double();
        let z = buf.read_double();
        let movement = buf.read_lp_vec3();
        let x_rot = buf.read_byte();
        let y_rot = buf.read_byte();
        let y_head_rot = buf.read_byte();
        let data = buf.read_var_int();
        Self {
            id,
            uuid,
            entity_type,
            x,
            y,
            z,
            movement,
            x_rot,
            y_rot,
            y_head_rot,
            data,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#write
    pub fn write(&self, buf: &mut impl RegistryFriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_uuid(self.uuid);
        buf.write_entity_type(self.entity_type);
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_lp_vec3(self.movement);
        buf.write_byte(self.x_rot);
        buf.write_byte(self.y_rot);
        buf.write_byte(self.y_head_rot);
        buf.write_var_int(self.data);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_ADD_ENTITY_PACKET_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_add_entity(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getUUID
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getType
    pub fn get_type(&self) -> EntityType {
        self.entity_type
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getX
    pub fn get_x(&self) -> f64 {
        self.x
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getY
    pub fn get_y(&self) -> f64 {
        self.y
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getZ
    pub fn get_z(&self) -> f64 {
        self.z
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getMovement
    pub fn get_movement(&self) -> Vec3 {
        self.movement
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getXRot
    pub fn get_x_rot(&self) -> f32 {
        mth_unpack_degrees(self.x_rot)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getYRot
    pub fn get_y_rot(&self) -> f32 {
        mth_unpack_degrees(self.y_rot)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getYHeadRot
    pub fn get_y_head_rot(&self) -> f32 {
        mth_unpack_degrees(self.y_head_rot)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAddEntityPacket#getData
    pub fn get_data(&self) -> i32 {
        self.data
    }
}

impl fmt::Display for ClientboundAddEntityPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundAddEntityPacket[id={}, type={:?}, pos=({}, {}, {}), data={}]",
            self.id, self.entity_type, self.x, self.y, self.z, self.data
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestBuf {
        data: Vec<u8>,
        read_pos: usize,
    }

    impl TestBuf {
        fn into_inner(self) -> Vec<u8> {
            self.data
        }

        fn from_bytes(data: Vec<u8>) -> Self {
            Self { data, read_pos: 0 }
        }

        fn read_i64(&mut self) -> i64 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 8]);
            self.read_pos += 8;
            i64::from_be_bytes(bytes)
        }

        fn write_i64(&mut self, value: i64) {
            self.data.extend_from_slice(&value.to_be_bytes());
        }
    }

    impl RegistryFriendlyByteBuf for TestBuf {
        fn read_var_int(&mut self) -> i32 {
            let mut val: i32 = 0;
            let mut pos: u32 = 0;
            loop {
                let b = self.data[self.read_pos] as i8;
                self.read_pos += 1;
                val |= ((b & 127) as i32).wrapping_shl(pos.wrapping_mul(7));
                pos = pos.wrapping_add(1);
                if (b & -128i8) == 0 {
                    break;
                }
                if pos > 5 {
                    panic!("VarInt too big");
                }
            }
            val
        }

        fn write_var_int(&mut self, mut value: i32) {
            loop {
                if (value & -128i32) == 0 {
                    self.data.push(value as u8);
                    return;
                }
                self.data.push(((value & 0x7F) | 0x80) as u8);
                value = ((value as u32) >> 7) as i32;
            }
        }

        fn read_uuid(&mut self) -> Uuid {
            Uuid {
                most_significant_bits: self.read_i64(),
                least_significant_bits: self.read_i64(),
            }
        }

        fn write_uuid(&mut self, value: Uuid) {
            self.write_i64(value.most_significant_bits);
            self.write_i64(value.least_significant_bits);
        }

        fn read_double(&mut self) -> f64 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 8]);
            self.read_pos += 8;
            f64::from_be_bytes(bytes)
        }

        fn write_double(&mut self, value: f64) {
            self.data.extend_from_slice(&value.to_be_bytes());
        }

        fn read_byte(&mut self) -> i8 {
            let b = self.data[self.read_pos] as i8;
            self.read_pos += 1;
            b
        }

        fn write_byte(&mut self, value: i8) {
            self.data.push(value as u8);
        }

        fn read_lp_vec3(&mut self) -> Vec3 {
            Vec3 {
                x: self.read_double(),
                y: self.read_double(),
                z: self.read_double(),
            }
        }

        fn write_lp_vec3(&mut self, value: Vec3) {
            self.write_double(value.x);
            self.write_double(value.y);
            self.write_double(value.z);
        }

        fn read_entity_type(&mut self) -> EntityType {
            EntityType {
                id: self.read_var_int(),
            }
        }

        fn write_entity_type(&mut self, value: EntityType) {
            self.write_var_int(value.id);
        }
    }

    struct MockEntity {
        id: i32,
        uuid: Uuid,
        entity_type: EntityType,
        x_rot: f32,
        y_rot: f32,
        delta: Vec3,
        y_head_rot: f32,
    }

    impl EntityLike for MockEntity {
        fn get_id(&self) -> i32 {
            self.id
        }
        fn get_uuid(&self) -> Uuid {
            self.uuid
        }
        fn get_type(&self) -> EntityType {
            self.entity_type
        }
        fn get_x_rot(&self) -> f32 {
            self.x_rot
        }
        fn get_y_rot(&self) -> f32 {
            self.y_rot
        }
        fn get_delta_movement(&self) -> Vec3 {
            self.delta
        }
        fn get_y_head_rot(&self) -> f32 {
            self.y_head_rot
        }
    }

    struct MockServerEntity {
        pos: Vec3,
        x_rot: f32,
        y_rot: f32,
        movement: Vec3,
        y_head_rot: f32,
    }

    impl ServerEntityLike for MockServerEntity {
        fn get_position_base(&self) -> Vec3 {
            self.pos
        }
        fn get_last_sent_x_rot(&self) -> f32 {
            self.x_rot
        }
        fn get_last_sent_y_rot(&self) -> f32 {
            self.y_rot
        }
        fn get_last_sent_movement(&self) -> Vec3 {
            self.movement
        }
        fn get_last_sent_y_head_rot(&self) -> f32 {
            self.y_head_rot
        }
    }

    #[test]
    fn round_trip_write_read_preserves_fields_and_float_bits() {
        let packet = ClientboundAddEntityPacket::new_full(
            42,
            Uuid {
                most_significant_bits: 0x0123_4567_89AB_CDEFu64 as i64,
                least_significant_bits: -1i64,
            },
            1.5,
            -64.0,
            9999.25,
            90.0,
            -180.0,
            EntityType { id: 7 },
            3,
            Vec3 {
                x: 0.1,
                y: -0.2,
                z: 0.3,
            },
            45.0,
        );

        let mut buf = TestBuf::default();
        packet.write(&mut buf);
        let mut reader = TestBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundAddEntityPacket::read(&mut reader);

        assert_eq!(decoded.get_id(), 42);
        assert_eq!(decoded.get_uuid(), packet.get_uuid());
        assert_eq!(decoded.get_type(), EntityType { id: 7 });
        assert_eq!(decoded.get_x().to_bits(), 1.5f64.to_bits());
        assert_eq!(decoded.get_y().to_bits(), (-64.0f64).to_bits());
        assert_eq!(decoded.get_z().to_bits(), 9999.25f64.to_bits());
        assert_eq!(decoded.get_movement().x.to_bits(), 0.1f64.to_bits());
        assert_eq!(decoded.get_movement().y.to_bits(), (-0.2f64).to_bits());
        assert_eq!(decoded.get_movement().z.to_bits(), 0.3f64.to_bits());
        assert_eq!(decoded.get_data(), 3);
        assert_eq!(decoded.x_rot, packet.x_rot);
        assert_eq!(decoded.y_rot, packet.y_rot);
        assert_eq!(decoded.y_head_rot, packet.y_head_rot);
    }

    #[test]
    fn pack_unpack_degrees_edge_cases_and_overflow() {
        assert_eq!(mth_pack_degrees(0.0), 0i8);
        assert_eq!(mth_unpack_degrees(0i8), 0.0f32);

        assert_eq!(mth_pack_degrees(180.0), -128i8);
        assert_eq!(mth_unpack_degrees(-128i8), -180.0f32);

        let packed_neg = mth_pack_degrees(-90.0);
        assert_eq!(packed_neg, -64i8);
        assert_eq!(mth_unpack_degrees(packed_neg), -90.0f32);
        assert_eq!(mth_pack_degrees(270.0), packed_neg);

        assert_eq!(mth_pack_degrees(360.0), 0i8);
        assert_eq!(mth_pack_degrees(-360.0), 0i8);
        assert_eq!(mth_pack_degrees(720.0), 0i8);

        // Large magnitude exercises i8 truncation/wrapping behavior of Java byte cast.
        let packed_large = mth_pack_degrees(100000.0);
        let unpacked_large = mth_unpack_degrees(packed_large);
        assert!(unpacked_large.is_finite());
    }

    #[test]
    fn negative_extreme_and_float_bit_patterns_round_trip() {
        let nan_bits = f64::NAN.to_bits();
        let packet = ClientboundAddEntityPacket::new_full(
            -1,
            Uuid {
                most_significant_bits: i64::MIN,
                least_significant_bits: -1,
            },
            -1.0e308,
            f64::MIN_POSITIVE,
            f64::from_bits(nan_bits),
            -720.0,
            720.0,
            EntityType { id: -1 },
            i32::MIN,
            Vec3 {
                x: -0.0,
                y: f64::INFINITY,
                z: f64::NEG_INFINITY,
            },
            -360.0,
        );

        let mut buf = TestBuf::default();
        packet.write(&mut buf);
        let mut reader = TestBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundAddEntityPacket::read(&mut reader);

        assert_eq!(decoded.get_id(), -1);
        assert_eq!(decoded.get_data(), i32::MIN);
        assert_eq!(decoded.get_type(), EntityType { id: -1 });
        assert_eq!(decoded.get_uuid().most_significant_bits, i64::MIN);
        assert_eq!(decoded.get_uuid().least_significant_bits, -1);
        assert_eq!(decoded.get_x().to_bits(), (-1.0e308f64).to_bits());
        assert_eq!(decoded.get_y().to_bits(), f64::MIN_POSITIVE.to_bits());
        assert_eq!(decoded.get_z().to_bits(), nan_bits);
        assert_eq!(decoded.get_movement().x.to_bits(), (-0.0f64).to_bits());
        assert_eq!(decoded.get_movement().y.to_bits(), f64::INFINITY.to_bits());
        assert_eq!(
            decoded.get_movement().z.to_bits(),
            f64::NEG_INFINITY.to_bits()
        );
    }

    #[test]
    fn constructors_map_entity_and_block_pos_fields() {
        let entity = MockEntity {
            id: 7,
            uuid: Uuid {
                most_significant_bits: 1,
                least_significant_bits: 2,
            },
            entity_type: EntityType { id: 5 },
            x_rot: 45.0,
            y_rot: -45.0,
            delta: Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            y_head_rot: 90.0,
        };
        let server_entity = MockServerEntity {
            pos: Vec3 {
                x: 10.5,
                y: -20.25,
                z: 30.0,
            },
            x_rot: 15.0,
            y_rot: -30.0,
            movement: Vec3 {
                x: -0.1,
                y: 0.2,
                z: -0.3,
            },
            y_head_rot: 180.0,
        };

        let from_entity = ClientboundAddEntityPacket::new_from_entity(&entity, &server_entity);
        assert_eq!(from_entity.get_id(), 7);
        assert_eq!(from_entity.get_data(), 0);
        assert_eq!(from_entity.get_x().to_bits(), 10.5f64.to_bits());
        assert_eq!(from_entity.get_y().to_bits(), (-20.25f64).to_bits());
        assert_eq!(from_entity.get_z().to_bits(), 30.0f64.to_bits());
        assert_eq!(from_entity.get_movement(), server_entity.movement);
        assert_eq!(
            from_entity.get_x_rot(),
            mth_unpack_degrees(mth_pack_degrees(15.0))
        );
        assert_eq!(
            from_entity.get_y_head_rot(),
            mth_unpack_degrees(mth_pack_degrees(180.0))
        );

        let from_block = ClientboundAddEntityPacket::new_from_entity_block_pos(
            &entity,
            99,
            BlockPos {
                x: -10,
                y: 64,
                z: 20,
            },
        );
        assert_eq!(from_block.get_x(), -10.0);
        assert_eq!(from_block.get_y(), 64.0);
        assert_eq!(from_block.get_z(), 20.0);
        assert_eq!(from_block.get_data(), 99);
        assert_eq!(from_block.get_movement(), entity.delta);
        assert_eq!(
            from_block.get_y_rot(),
            mth_unpack_degrees(mth_pack_degrees(-45.0))
        );
    }

    #[test]
    fn handle_and_packet_type_dispatch() {
        struct Listener {
            handled_id: Option<i32>,
        }
        impl ClientGamePacketListener for Listener {
            fn handle_add_entity(&mut self, packet: &ClientboundAddEntityPacket) {
                self.handled_id = Some(packet.get_id());
            }
        }

        let packet = ClientboundAddEntityPacket::new_full(
            123,
            Uuid {
                most_significant_bits: 0,
                least_significant_bits: 0,
            },
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            EntityType { id: 1 },
            0,
            Vec3::ZERO,
            0.0,
        );

        assert_eq!(packet.packet_type(), CLIENTBOUND_ADD_ENTITY_PACKET_TYPE);
        let mut listener = Listener { handled_id: None };
        packet.handle(&mut listener);
        assert_eq!(listener.handled_id, Some(123));
    }
}
