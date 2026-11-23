use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketType {
    ClientboundDebugBlockValue,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            read_pos: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_varint(&mut self) -> io::Result<i32> {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_u8()? as i32;
            let shift = j.wrapping_mul(7) & 31;
            i |= (b0 & 127) << shift;
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if (b0 & 128) == 0 {
                break;
            }
        }
        Ok(i)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_varint(&mut self, mut value: i32) -> io::Result<()> {
        loop {
            if (value & !0x7F) == 0 {
                self.data.push(value as u8);
                return Ok(());
            }
            self.data.push(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong
    pub fn read_long(&mut self) -> io::Result<i64> {
        if self.data.len().saturating_sub(self.read_pos) < 8 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 8]);
        self.read_pos += 8;
        Ok(i64::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
    pub fn write_long(&mut self, value: i64) -> io::Result<()> {
        self.data.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByteArray
    pub fn read_byte_array(&mut self) -> io::Result<Vec<u8>> {
        let len = self.read_varint()?;
        if len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative byte array length",
            ));
        }
        let len = len as usize;
        let remaining = self.data.len().saturating_sub(self.read_pos);
        if remaining < len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "byte array past end",
            ));
        }
        let out = self.data[self.read_pos..self.read_pos + len].to_vec();
        self.read_pos += len;
        Ok(out)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByteArray
    pub fn write_byte_array(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.write_varint(bytes.len() as i32)?;
        self.data.extend_from_slice(bytes);
        Ok(())
    }
}

pub type RegistryFriendlyByteBuf = FriendlyByteBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(&self) -> i64 {
        let x = (self.x as i64) & 0x3FF_FFFF;
        let z = (self.z as i64) & 0x3FF_FFFF;
        let y = (self.y as i64) & 0xFFF;
        (x << 38) | (z << 12) | y
    }

    // JAVA: net.minecraft.core.BlockPos#of
    pub fn of(packed: i64) -> Self {
        let x = (packed >> 38) as i32;
        let y = ((packed << 52) >> 52) as i32;
        let z = ((packed << 26) >> 38) as i32;
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#STREAM_CODEC
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_long(self.as_long())
    }

    // JAVA: net.minecraft.core.BlockPos#STREAM_CODEC
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        Ok(Self::of(buf.read_long()?))
    }
}

/// Minimal self-contained placeholder for DebugSubscription.Update<?>.
/// The real codec depends on debug subscription registry state; this stub preserves
/// packet structure with an opaque payload for faithful round-trips in isolation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugSubscriptionUpdate {
    pub data: Vec<u8>,
}

impl DebugSubscriptionUpdate {
    // JAVA: net.minecraft.util.debug.DebugSubscription$Update#STREAM_CODEC
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        Ok(Self {
            data: buf.read_byte_array()?,
        })
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription$Update#STREAM_CODEC
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_byte_array(&self.data)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundDebugBlockValuePacket {
    pub block_pos: BlockPos,
    pub update: DebugSubscriptionUpdate,
}

pub trait ClientGamePacketListener {
    fn handle_debug_block_value(&mut self, packet: &ClientboundDebugBlockValuePacket);
}

impl ClientboundDebugBlockValuePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket#<init>
    pub fn new(block_pos: BlockPos, update: DebugSubscriptionUpdate) -> Self {
        Self { block_pos, update }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket#blockPos
    pub fn block_pos(&self) -> BlockPos {
        self.block_pos
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket#update
    pub fn update(&self) -> &DebugSubscriptionUpdate {
        &self.update
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket#STREAM_CODEC
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let block_pos = BlockPos::read(buf)?;
        let update = DebugSubscriptionUpdate::read(buf)?;
        Ok(Self::new(block_pos, update))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket#STREAM_CODEC
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) -> io::Result<()> {
        self.block_pos.write(buf)?;
        self.update.write(buf)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ClientboundDebugBlockValue
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_debug_block_value(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_pos_pack_unpack_edges() {
        let cases = [
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: -1, y: -1, z: -1 },
            BlockPos { x: 12345, y: -2048, z: -30000000 },
            BlockPos { x: -30000000, y: 2047, z: 30000000 },
        ];
        for p in cases {
            let packed = p.as_long();
            assert_eq!(BlockPos::of(packed), p);
        }
    }

    #[test]
    fn round_trip() {
        let pkt = ClientboundDebugBlockValuePacket::new(
            BlockPos { x: -10, y: 64, z: 999 },
            DebugSubscriptionUpdate { data: vec![7, 8, 9] },
        );
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundDebugBlockValuePacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn type_and_handle() {
        let pkt = ClientboundDebugBlockValuePacket::new(
            BlockPos { x: 1, y: 2, z: 3 },
            DebugSubscriptionUpdate { data: vec![] },
        );
        assert_eq!(pkt.packet_type(), PacketType::ClientboundDebugBlockValue);

        struct Listener(Option<ClientboundDebugBlockValuePacket>);
        impl ClientGamePacketListener for Listener {
            fn handle_debug_block_value(&mut self, packet: &ClientboundDebugBlockValuePacket) {
                self.0 = Some(packet.clone());
            }
        }
        let mut listener = Listener(None);
        pkt.handle(&mut listener);
        assert_eq!(listener.0, Some(pkt));
    }

    #[test]
    fn negative_varint_round_trip() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_varint(-123456).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        assert_eq!(read_buf.read_varint().unwrap(), -123456);
    }
}
