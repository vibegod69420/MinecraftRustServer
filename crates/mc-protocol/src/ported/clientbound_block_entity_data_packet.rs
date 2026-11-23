use std::io::{self, Cursor, Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn read(buf: &mut Cursor<&[u8]>) -> io::Result<Self> {
        let v = read_i64(buf)?;
        Ok(Self::from_packed(v))
    }

    pub fn write(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        write_i64(buf, self.as_packed())
    }

    pub fn from_packed(packed: i64) -> Self {
        // Minecraft 1.21 BlockPos packing: x:26, z:26, y:12
        let x = (packed >> 38) as i32;
        let z = (packed << 26 >> 38) as i32;
        let y = (packed << 52 >> 52) as i32;
        Self { x, y, z }
    }

    pub fn as_packed(&self) -> i64 {
        (((self.x as i64) & 0x3FF_FFFF) << 38)
            | (((self.z as i64) & 0x3FF_FFFF) << 12)
            | ((self.y as i64) & 0xFFF)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockEntityType {
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundTag {
    pub bytes: Vec<u8>,
}

impl CompoundTag {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn empty() -> Self {
        // Minimal NBT end tag representation for a self-contained placeholder
        Self { bytes: vec![0u8] }
    }

    pub fn read(buf: &mut Cursor<&[u8]>) -> io::Result<Self> {
        // Self-contained trusted compound tag read: read remaining-safe NBT-like payload.
        // For 1:1 wire compatibility in this isolated module, we read a length-prefixed blob
        // if present in test helpers, but real NBT is external. We store raw bytes.
        // Here we use a simple i32 length-prefixed format as a local codec placeholder.
        let len = read_varint(buf)? as usize;
        let mut bytes = vec![0u8; len];
        buf.read_exact(&mut bytes)?;
        Ok(Self { bytes })
    }

    pub fn write(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        write_varint(buf, self.bytes.len() as i32)?;
        buf.extend_from_slice(&self.bytes);
        Ok(())
    }
}

pub trait BlockEntity {
    fn get_block_pos(&self) -> BlockPos;
    fn get_type(&self) -> BlockEntityType;
    fn get_update_tag(&self, registry_access: &RegistryAccess) -> CompoundTag;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RegistryAccess;

pub trait ClientGamePacketListener {
    fn handle_block_entity_data(&mut self, packet: &ClientboundBlockEntityDataPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub &'static str);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_BLOCK_ENTITY_DATA: PacketType =
        PacketType("minecraft:clientbound_block_entity_data");
}

fn read_i64(buf: &mut Cursor<&[u8]>) -> io::Result<i64> {
    let mut bytes = [0u8; 8];
    buf.read_exact(&mut bytes)?;
    Ok(i64::from_be_bytes(bytes))
}

fn write_i64(buf: &mut Vec<u8>, value: i64) -> io::Result<()> {
    buf.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

fn read_varint(buf: &mut Cursor<&[u8]>) -> io::Result<i32> {
    let mut result: i32 = 0;
    for shift in (0..32).step_by(7) {
        let mut byte = [0u8; 1];
        buf.read_exact(&mut byte)?;
        let b = byte[0] as i32;
        result |= (b & 0x7F) << shift;
        if (b & 0x80) == 0 {
            return Ok(result);
        }
    }
    Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"))
}

fn write_varint(buf: &mut Vec<u8>, mut value: i32) -> io::Result<()> {
    let mut val = value as u32;
    loop {
        if (val & !0x7F) == 0 {
            buf.push(val as u8);
            return Ok(());
        }
        buf.push(((val & 0x7F) as u8) | 0x80);
        val >>= 7;
    }
}

fn read_block_entity_type(buf: &mut Cursor<&[u8]>) -> io::Result<BlockEntityType> {
    Ok(BlockEntityType {
        id: read_varint(buf)?,
    })
}

fn write_block_entity_type(buf: &mut Vec<u8>, ty: BlockEntityType) -> io::Result<()> {
    write_varint(buf, ty.id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundBlockEntityDataPacket {
    pos: BlockPos,
    type_: BlockEntityType,
    tag: CompoundTag,
}

impl ClientboundBlockEntityDataPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket#create
    pub fn create_with<F>(block_entity: &impl BlockEntity, mut tag_fn: F) -> Self
    where
        F: FnMut(&dyn BlockEntity, &RegistryAccess) -> CompoundTag,
    {
        let registry_access = RegistryAccess;
        let tag = tag_fn(block_entity, &registry_access);
        Self {
            pos: block_entity.get_block_pos(),
            type_: block_entity.get_type(),
            tag,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket#create
    pub fn create(block_entity: &impl BlockEntity) -> Self {
        Self::create_with(block_entity, |be, ra| be.get_update_tag(ra))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket#<init>
    fn new(pos: BlockPos, type_: BlockEntityType, tag: CompoundTag) -> Self {
        Self { pos, type_, tag }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket#type
    pub fn packet_type(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_BLOCK_ENTITY_DATA
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_block_entity_data(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket#getType
    pub fn get_type(&self) -> BlockEntityType {
        self.type_
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket#getTag
    pub fn get_tag(&self) -> &CompoundTag {
        &self.tag
    }

    pub fn read(buf: &mut Cursor<&[u8]>) -> io::Result<Self> {
        let pos = BlockPos::read(buf)?;
        let type_ = read_block_entity_type(buf)?;
        let tag = CompoundTag::read(buf)?;
        Ok(Self::new(pos, type_, tag))
    }

    pub fn write(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        self.pos.write(buf)?;
        write_block_entity_type(buf, self.type_)?;
        self.tag.write(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBlockEntity {
        pos: BlockPos,
        ty: BlockEntityType,
        tag: CompoundTag,
    }

    impl BlockEntity for MockBlockEntity {
        fn get_block_pos(&self) -> BlockPos {
            self.pos
        }

        fn get_type(&self) -> BlockEntityType {
            self.ty
        }

        fn get_update_tag(&self, _registry_access: &RegistryAccess) -> CompoundTag {
            self.tag.clone()
        }
    }

    struct MockListener {
        seen: Option<ClientboundBlockEntityDataPacket>,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_block_entity_data(&mut self, packet: &ClientboundBlockEntityDataPacket) {
            self.seen = Some(packet.clone());
        }
    }

    #[test]
    fn create_and_accessors() {
        let be = MockBlockEntity {
            pos: BlockPos::new(1, 2, 3),
            ty: BlockEntityType { id: 7 },
            tag: CompoundTag::new(vec![1, 2, 3]),
        };
        let packet = ClientboundBlockEntityDataPacket::create(&be);
        assert_eq!(packet.get_pos(), BlockPos::new(1, 2, 3));
        assert_eq!(packet.get_type(), BlockEntityType { id: 7 });
        assert_eq!(packet.get_tag(), &CompoundTag::new(vec![1, 2, 3]));
        assert_eq!(
            packet.packet_type(),
            GamePacketTypes::CLIENTBOUND_BLOCK_ENTITY_DATA
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ClientboundBlockEntityDataPacket::new(
            BlockPos::new(-5, 64, 9),
            BlockEntityType { id: 2 },
            CompoundTag::new(vec![9]),
        );
        let mut listener = MockListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen, Some(packet));
    }

    #[test]
    fn write_read_roundtrip() {
        let packet = ClientboundBlockEntityDataPacket::new(
            BlockPos::new(-30000000, -2048, 30000000),
            BlockEntityType { id: 255 },
            CompoundTag::new(vec![10, 0, 1, 2, 255]),
        );
        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf.as_slice());
        let decoded = ClientboundBlockEntityDataPacket::read(&mut cursor).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(cursor.position() as usize, buf.len());
    }

    #[test]
    fn block_pos_packing_negative_and_extremes() {
        let positions = [
            BlockPos::new(0, 0, 0),
            BlockPos::new(-1, -1, -1),
            BlockPos::new(33554431, 2047, 33554431),
            BlockPos::new(-33554432, -2048, -33554432),
        ];
        for pos in positions {
            let packed = pos.as_packed();
            let unpacked = BlockPos::from_packed(packed);
            assert_eq!(unpacked, pos);
        }
    }

    #[test]
    fn varint_and_compound_tag_edge_cases() {
        let mut buf = Vec::new();
        write_varint(&mut buf, -1).unwrap();
        write_varint(&mut buf, 0).unwrap();
        let mut cursor = Cursor::new(buf.as_slice());
        assert_eq!(read_varint(&mut cursor).unwrap(), -1);
        assert_eq!(read_varint(&mut cursor).unwrap(), 0);

        let tag = CompoundTag::empty();
        let mut tag_buf = Vec::new();
        tag.write(&mut tag_buf).unwrap();
        let mut tag_cursor = Cursor::new(tag_buf.as_slice());
        let decoded = CompoundTag::read(&mut tag_cursor).unwrap();
        assert_eq!(decoded, tag);
    }
}
