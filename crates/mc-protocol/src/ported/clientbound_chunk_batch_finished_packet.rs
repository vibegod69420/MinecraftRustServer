use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError(pub &'static str);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for DecodeError {}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
pub fn read_var_int(buf: &mut impl Buf) -> Result<i32, DecodeError> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(DecodeError("VarInt too short"));
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32).wrapping_shl(shift & 31);
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
        if shift >= 35 {
            return Err(DecodeError("VarInt too big"));
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
pub fn write_var_int(buf: &mut impl BufMut, mut value: i32) {
    loop {
        let mut b = (value as u32 & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            b |= 0x80;
            buf.put_u8(b);
        } else {
            buf.put_u8(b);
            return;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundChunkBatchFinishedPacket {
    pub batch_size: i32,
}

pub trait ClientGamePacketListener {
    fn handle_chunk_batch_finished(&mut self, packet: &ClientboundChunkBatchFinishedPacket);
}

impl ClientboundChunkBatchFinishedPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundChunkBatchFinishedPacket#<init>
    pub fn new(batch_size: i32) -> Self {
        Self { batch_size }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunkBatchFinishedPacket#<init>
    pub fn read(buf: &mut impl Buf) -> Result<Self, DecodeError> {
        let batch_size = read_var_int(buf)?;
        Ok(Self::new(batch_size))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunkBatchFinishedPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.batch_size);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunkBatchFinishedPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:chunk_batch_finished")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunkBatchFinishedPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_chunk_batch_finished(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BytesMut};

    #[test]
    fn var_int_roundtrip_edge_values() {
        let values = [0, 1, 127, 128, 255, -1, i32::MIN, i32::MAX];
        for v in values {
            let mut buf = BytesMut::new();
            write_var_int(&mut buf, v);
            let mut slice = &buf[..];
            let decoded = read_var_int(&mut slice).unwrap();
            assert_eq!(decoded, v, "varint roundtrip failed for {v}");
            assert!(!slice.has_remaining());
        }
    }

    #[test]
    fn var_int_too_big() {
        let data = [0x80u8, 0x80, 0x80, 0x80, 0x80, 0x01];
        let mut slice = &data[..];
        assert!(read_var_int(&mut slice).is_err());
    }

    #[test]
    fn packet_roundtrip() {
        let packet = ClientboundChunkBatchFinishedPacket::new(42);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ClientboundChunkBatchFinishedPacket::read(&mut slice).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn packet_negative_batch_size() {
        let packet = ClientboundChunkBatchFinishedPacket::new(-1);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ClientboundChunkBatchFinishedPacket::read(&mut slice).unwrap();
        assert_eq!(decoded.batch_size, -1);
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener(Option<i32>);
        impl ClientGamePacketListener for MockListener {
            fn handle_chunk_batch_finished(
                &mut self,
                packet: &ClientboundChunkBatchFinishedPacket,
            ) {
                self.0 = Some(packet.batch_size);
            }
        }
        let packet = ClientboundChunkBatchFinishedPacket::new(7);
        let mut listener = MockListener(None);
        packet.handle(&mut listener);
        assert_eq!(listener.0, Some(7));
        assert_eq!(
            packet.packet_type(),
            PacketType("minecraft:chunk_batch_finished")
        );
    }
}
