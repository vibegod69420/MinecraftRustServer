use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    pub message: String,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecodeError {}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_varint(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(DecodeError {
                message: "VarInt too short".into(),
            });
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32) << (shift & 31);
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
        if shift >= 35 {
            return Err(DecodeError {
                message: "VarInt too big".into(),
            });
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_varint(buf: &mut BytesMut, value: i32) {
    let mut val = value as u32;
    loop {
        if (val & !0x7F) == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8(((val & 0x7F) as u8) | 0x80);
        val >>= 7;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
fn read_boolean(buf: &mut &[u8]) -> Result<bool, DecodeError> {
    if !buf.has_remaining() {
        return Err(DecodeError {
            message: "Boolean too short".into(),
        });
    }
    Ok(buf.get_u8() != 0)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
fn write_boolean(buf: &mut BytesMut, value: bool) {
    buf.put_u8(if value { 1 } else { 0 });
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readLong
fn read_long(buf: &mut &[u8]) -> Result<i64, DecodeError> {
    if buf.remaining() < 8 {
        return Err(DecodeError {
            message: "Long too short".into(),
        });
    }
    Ok(buf.get_i64())
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
fn write_long(buf: &mut BytesMut, value: i64) {
    buf.put_i64(value);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
fn read_string(buf: &mut &[u8]) -> Result<String, DecodeError> {
    let len = read_varint(buf)?;
    if len < 0 {
        return Err(DecodeError {
            message: "String length negative".into(),
        });
    }
    let len = len as usize;
    if buf.remaining() < len {
        return Err(DecodeError {
            message: "String too short".into(),
        });
    }
    let s = std::str::from_utf8(&buf[..len])
        .map_err(|_| DecodeError {
            message: "Invalid UTF-8".into(),
        })?
        .to_owned();
    buf.advance(len);
    Ok(s)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
fn write_string(buf: &mut BytesMut, value: &str) {
    write_varint(buf, value.len() as i32);
    buf.put_slice(value.as_bytes());
}

// JAVA: net.minecraft.network.codec.ByteBufCodecs#optional
fn read_optional_string(buf: &mut &[u8]) -> Result<Option<String>, DecodeError> {
    if read_boolean(buf)? {
        Ok(Some(read_string(buf)?))
    } else {
        Ok(None)
    }
}

// JAVA: net.minecraft.network.codec.ByteBufCodecs#optional
fn write_optional_string(buf: &mut BytesMut, value: Option<&str>) {
    match value {
        Some(v) => {
            write_boolean(buf, true);
            write_string(buf, v);
        }
        None => write_boolean(buf, false),
    }
}

const PACKED_X_LENGTH: i32 = 26;
const PACKED_Z_LENGTH: i32 = 26;
const PACKED_Y_LENGTH: i32 = 12;
const PACKED_X_MASK: i64 = (1_i64 << PACKED_X_LENGTH) - 1;
const PACKED_Y_MASK: i64 = (1_i64 << PACKED_Y_LENGTH) - 1;
const PACKED_Z_MASK: i64 = (1_i64 << PACKED_Z_LENGTH) - 1;
const Z_OFFSET: i32 = PACKED_Y_LENGTH;
const X_OFFSET: i32 = PACKED_Y_LENGTH + PACKED_Z_LENGTH;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(&self) -> i64 {
        let x_bits = (self.x as i64) & PACKED_X_MASK;
        let y_bits = (self.y as i64) & PACKED_Y_MASK;
        let z_bits = (self.z as i64) & PACKED_Z_MASK;
        (x_bits << (X_OFFSET & 63))
            | (z_bits << (Z_OFFSET & 63))
            | y_bits
    }

    // JAVA: net.minecraft.core.BlockPos#of
    pub fn from_long(packed: i64) -> Self {
        let x = (packed << ((64_i32.wrapping_sub(X_OFFSET).wrapping_sub(PACKED_X_LENGTH)) & 63))
            >> ((64_i32.wrapping_sub(PACKED_X_LENGTH)) & 63);
        let y = (packed << ((64_i32.wrapping_sub(PACKED_Y_LENGTH)) & 63))
            >> ((64_i32.wrapping_sub(PACKED_Y_LENGTH)) & 63);
        let z = (packed << ((64_i32.wrapping_sub(Z_OFFSET).wrapping_sub(PACKED_Z_LENGTH)) & 63))
            >> ((64_i32.wrapping_sub(PACKED_Z_LENGTH)) & 63);
        Self {
            x: x as i32,
            y: y as i32,
            z: z as i32,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#<streamDecode>
    pub fn decode(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_long(read_long(buf)?))
    }

    // JAVA: net.minecraft.core.BlockPos#<streamEncode>
    pub fn encode(&self, buf: &mut BytesMut) {
        write_long(buf, self.as_long());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    // JAVA: net.minecraft.core.Vec3i#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.Vec3i#<streamDecode>
    pub fn decode(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let x = read_varint(buf)?;
        let y = read_varint(buf)?;
        let z = read_varint(buf)?;
        Ok(Self { x, y, z })
    }

    // JAVA: net.minecraft.core.Vec3i#<streamEncode>
    pub fn encode(&self, buf: &mut BytesMut) {
        write_varint(buf, self.x);
        write_varint(buf, self.y);
        write_varint(buf, self.z);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Rotation {
    #[default]
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl Rotation {
    // JAVA: net.minecraft.world.level.block.Rotation#<id>
    pub fn id(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Clockwise90 => 1,
            Self::Clockwise180 => 2,
            Self::Counterclockwise90 => 3,
        }
    }

    // JAVA: net.minecraft.world.level.block.Rotation#<byId>
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Clockwise90,
            2 => Self::Clockwise180,
            3 => Self::Counterclockwise90,
            _ => Self::None,
        }
    }

    // JAVA: net.minecraft.world.level.block.Rotation#<streamDecode>
    pub fn decode(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_id(read_varint(buf)?))
    }

    // JAVA: net.minecraft.world.level.block.Rotation#<streamEncode>
    pub fn encode(&self, buf: &mut BytesMut) {
        write_varint(buf, self.id());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Status {
    #[default]
    Cleared,
    Running,
    Finished,
    Failed,
}

impl Status {
    // JAVA: net.minecraft.world.level.block.entity.TestInstanceBlockEntity$Status#<id>
    pub fn id(self) -> i32 {
        match self {
            Self::Cleared => 0,
            Self::Running => 1,
            Self::Finished => 2,
            Self::Failed => 3,
        }
    }

    // JAVA: net.minecraft.world.level.block.entity.TestInstanceBlockEntity$Status#<byId>
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Running,
            2 => Self::Finished,
            3 => Self::Failed,
            _ => Self::Cleared,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Data {
    pub test: Option<String>,
    pub size: Vec3i,
    pub rotation: Rotation,
    pub ignore_entities: bool,
    pub status: Status,
    pub error_message: Option<String>,
}

impl Data {
    // JAVA: net.minecraft.world.level.block.entity.TestInstanceBlockEntity$Data#<init>
    pub fn new(
        test: Option<String>,
        size: Vec3i,
        rotation: Rotation,
        ignore_entities: bool,
        status: Status,
        error_message: Option<String>,
    ) -> Self {
        Self {
            test,
            size,
            rotation,
            ignore_entities,
            status,
            error_message,
        }
    }

    // JAVA: net.minecraft.world.level.block.entity.TestInstanceBlockEntity$Data#<streamDecode>
    pub fn decode(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let test = read_optional_string(buf)?;
        let size = Vec3i::decode(buf)?;
        let rotation = Rotation::decode(buf)?;
        let ignore_entities = read_boolean(buf)?;
        let status = Status::from_id(read_varint(buf)?);
        let error_message = read_optional_string(buf)?;
        Ok(Self {
            test,
            size,
            rotation,
            ignore_entities,
            status,
            error_message,
        })
    }

    // JAVA: net.minecraft.world.level.block.entity.TestInstanceBlockEntity$Data#<streamEncode>
    pub fn encode(&self, buf: &mut BytesMut) {
        write_optional_string(buf, self.test.as_deref());
        self.size.encode(buf);
        self.rotation.encode(buf);
        write_boolean(buf, self.ignore_entities);
        write_varint(buf, self.status.id());
        write_optional_string(buf, self.error_message.as_deref());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Init,
    Query,
    Set,
    Reset,
    Save,
    Export,
    Run,
}

impl Action {
    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket$Action#<id>
    pub fn id(self) -> i32 {
        match self {
            Self::Init => 0,
            Self::Query => 1,
            Self::Set => 2,
            Self::Reset => 3,
            Self::Save => 4,
            Self::Export => 5,
            Self::Run => 6,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket$Action#<byId>
    pub fn by_id(id: i32) -> Self {
        match id {
            1 => Self::Query,
            2 => Self::Set,
            3 => Self::Reset,
            4 => Self::Save,
            5 => Self::Export,
            6 => Self::Run,
            _ => Self::Init,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket$Action#<streamDecode>
    pub fn decode(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::by_id(read_varint(buf)?))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket$Action#<streamEncode>
    pub fn encode(&self, buf: &mut BytesMut) {
        write_varint(buf, self.id());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_TYPE: PacketType =
    PacketType("minecraft:serverbound_test_instance_block_action");

pub trait ServerGamePacketListener {
    fn handle_test_instance_block_action(
        &mut self,
        packet: &ServerboundTestInstanceBlockActionPacket,
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundTestInstanceBlockActionPacket {
    pos: BlockPos,
    action: Action,
    data: Data,
}

impl ServerboundTestInstanceBlockActionPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket#<init>
    pub fn new(pos: BlockPos, action: Action, data: Data) -> Self {
        Self { pos, action, data }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket#<init>
    pub fn new_with_params(
        pos: BlockPos,
        action: Action,
        test: Option<String>,
        size: Vec3i,
        rotation: Rotation,
        ignore_entities: bool,
    ) -> Self {
        Self {
            pos,
            action,
            data: Data::new(
                test,
                size,
                rotation,
                ignore_entities,
                Status::Cleared,
                None,
            ),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let pos = BlockPos::decode(buf)?;
        let action = Action::decode(buf)?;
        let data = Data::decode(buf)?;
        Ok(Self { pos, action, data })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket#write
    pub fn write(&self, buf: &mut BytesMut) {
        self.pos.encode(buf);
        self.action.encode(buf);
        self.data.encode(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket#type
    pub fn packet_type(&self) -> PacketType {
        SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_test_instance_block_action(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket#pos
    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket#action
    pub fn action(&self) -> Action {
        self.action
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket#data
    pub fn data(&self) -> &Data {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_by_id_out_of_bounds_returns_init() {
        assert_eq!(Action::by_id(-1), Action::Init);
        assert_eq!(Action::by_id(7), Action::Init);
        assert_eq!(Action::by_id(999), Action::Init);
        assert_eq!(Action::by_id(0), Action::Init);
        assert_eq!(Action::by_id(6), Action::Run);
    }

    #[test]
    fn block_pos_packed_long_round_trip() {
        let pos = BlockPos::new(-12345, -64, 9876);
        let packed = pos.as_long();
        let decoded = BlockPos::from_long(packed);
        assert_eq!(decoded, pos);

        let extremes = BlockPos::new(33_554_431, 2047, -33_554_432);
        assert_eq!(BlockPos::from_long(extremes.as_long()), extremes);
        let extremes2 = BlockPos::new(-33_554_432, -2048, 33_554_431);
        assert_eq!(BlockPos::from_long(extremes2.as_long()), extremes2);
    }

    #[test]
    fn convenience_constructor_defaults() {
        let packet = ServerboundTestInstanceBlockActionPacket::new_with_params(
            BlockPos::new(1, 2, 3),
            Action::Save,
            Some("minecraft:test".to_string()),
            Vec3i::new(4, 5, 6),
            Rotation::Clockwise90,
            true,
        );
        assert_eq!(packet.pos(), BlockPos::new(1, 2, 3));
        assert_eq!(packet.action(), Action::Save);
        assert_eq!(packet.data().test.as_deref(), Some("minecraft:test"));
        assert_eq!(packet.data().size, Vec3i::new(4, 5, 6));
        assert_eq!(packet.data().rotation, Rotation::Clockwise90);
        assert!(packet.data().ignore_entities);
        assert_eq!(packet.data().status, Status::Cleared);
        assert_eq!(packet.data().error_message, None);
    }

    #[test]
    fn packet_round_trip() {
        let data = Data::new(
            Some("ns:test".to_string()),
            Vec3i::new(1, -2, 3),
            Rotation::Counterclockwise90,
            false,
            Status::Running,
            Some("err".to_string()),
        );
        let packet = ServerboundTestInstanceBlockActionPacket::new(
            BlockPos::new(10, 20, 30),
            Action::Run,
            data,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf[..];
        let decoded =
            ServerboundTestInstanceBlockActionPacket::read(&mut slice).unwrap();
        assert!(slice.is_empty());
        assert_eq!(decoded, packet);
    }

    #[test]
    fn handle_and_packet_type() {
        let packet = ServerboundTestInstanceBlockActionPacket::new(
            BlockPos::default(),
            Action::Query,
            Data::default(),
        );
        assert_eq!(
            packet.packet_type(),
            SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_TYPE
        );

        struct MockListener(Option<Action>);
        impl ServerGamePacketListener for MockListener {
            fn handle_test_instance_block_action(
                &mut self,
                packet: &ServerboundTestInstanceBlockActionPacket,
            ) {
                self.0 = Some(packet.action());
            }
        }

        let mut listener = MockListener(None);
        packet.handle(&mut listener);
        assert_eq!(listener.0, Some(Action::Query));
    }
}
