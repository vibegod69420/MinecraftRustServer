use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#<init>(int,int)
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#toLong
    pub fn to_long(&self) -> i64 {
        ((self.x as i64) & 0xFFFF_FFFF_i64) | (((self.z as i64) & 0xFFFF_FFFF_i64) << 32)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#<init>(long)
    pub fn from_long(packed: i64) -> Self {
        Self {
            x: packed as i32,
            z: ((packed as u64) >> 32) as i32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSubscriptionUpdate {
    pub payload: Vec<u8>,
}

impl DebugSubscriptionUpdate {
    // JAVA: net.minecraft.util.debug.DebugSubscription$Update#<init>
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }
}

fn write_var_int(buf: &mut Vec<u8>, value: i32) {
    let mut v = value as u32;
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        buf.push(b);
        if v == 0 {
            break;
        }
    }
}

fn read_var_int(buf: &[u8], pos: &mut usize) -> io::Result<i32> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        if *pos >= buf.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "VarInt too short",
            ));
        }
        let b = buf[*pos];
        *pos += 1;
        result |= ((b & 0x7f) as i32) << shift;
        if (b & 0x80) == 0 {
            break;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt too big",
            ));
        }
    }
    Ok(result)
}

fn write_long(buf: &mut Vec<u8>, value: i64) {
    buf.extend_from_slice(&value.to_be_bytes());
}

fn read_long(buf: &[u8], pos: &mut usize) -> io::Result<i64> {
    if pos.saturating_add(8) > buf.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes for long",
        ));
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buf[*pos..*pos + 8]);
    *pos += 8;
    Ok(i64::from_be_bytes(bytes))
}

fn write_update(buf: &mut Vec<u8>, update: &DebugSubscriptionUpdate) -> io::Result<()> {
    write_var_int(buf, update.payload.len() as i32);
    buf.extend_from_slice(&update.payload);
    Ok(())
}

fn read_update(buf: &[u8], pos: &mut usize) -> io::Result<DebugSubscriptionUpdate> {
    let len = read_var_int(buf, pos)?;
    if len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Negative payload length",
        ));
    }
    let len = len as usize;
    if pos.saturating_add(len) > buf.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes for update payload",
        ));
    }
    let payload = buf[*pos..*pos + len].to_vec();
    *pos += len;
    Ok(DebugSubscriptionUpdate::new(payload))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugChunkValuePacket {
    chunk_pos: ChunkPos,
    update: DebugSubscriptionUpdate,
}

pub trait ClientGamePacketListener {
    fn handle_debug_chunk_value(&mut self, packet: &ClientboundDebugChunkValuePacket);
}

impl ClientboundDebugChunkValuePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugChunkValuePacket#<init>(ChunkPos,DebugSubscription.Update)
    pub fn new(chunk_pos: ChunkPos, update: DebugSubscriptionUpdate) -> Self {
        Self { chunk_pos, update }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugChunkValuePacket#chunkPos
    pub fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugChunkValuePacket#update
    pub fn update(&self) -> &DebugSubscriptionUpdate {
        &self.update
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugChunkValuePacket#STREAM_CODEC
    pub fn read(buf: &[u8], pos: &mut usize) -> io::Result<Self> {
        let packed = read_long(buf, pos)?;
        let chunk_pos = ChunkPos::from_long(packed);
        let update = read_update(buf, pos)?;
        Ok(Self { chunk_pos, update })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugChunkValuePacket#STREAM_CODEC
    pub fn write(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        write_long(buf, self.chunk_pos.to_long());
        write_update(buf, &self.update)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugChunkValuePacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:debug/chunk_value")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugChunkValuePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_debug_chunk_value(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen: Option<(ChunkPos, Vec<u8>)>,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_debug_chunk_value(&mut self, packet: &ClientboundDebugChunkValuePacket) {
            self.seen = Some((packet.chunk_pos(), packet.update().payload.clone()));
        }
    }

    #[test]
    fn round_trip_normal() {
        let pkt = ClientboundDebugChunkValuePacket::new(
            ChunkPos::new(12, -34),
            DebugSubscriptionUpdate::new(vec![1, 2, 3, 4]),
        );
        let mut buf = Vec::new();
        pkt.write(&mut buf).unwrap();
        let mut pos = 0;
        let decoded = ClientboundDebugChunkValuePacket::read(&buf, &mut pos).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(pos, buf.len());
    }

    #[test]
    fn chunk_pos_long_packing_extremes_and_negatives() {
        for &(x, z) in &[
            (0, 0),
            (-1, -1),
            (i32::MIN, i32::MAX),
            (i32::MAX, i32::MIN),
            (-123456, 789012),
        ] {
            let cp = ChunkPos::new(x, z);
            let packed = cp.to_long();
            let back = ChunkPos::from_long(packed);
            assert_eq!(cp, back, "round-trip failed for ({x}, {z})");
        }
    }

    #[test]
    fn handle_and_type() {
        let pkt = ClientboundDebugChunkValuePacket::new(
            ChunkPos::new(5, 6),
            DebugSubscriptionUpdate::new(vec![9]),
        );
        let mut listener = TestListener { seen: None };
        pkt.handle(&mut listener);
        assert_eq!(
            listener.seen,
            Some((ChunkPos::new(5, 6), vec![9]))
        );
        assert_eq!(pkt.get_type(), PacketType("minecraft:debug/chunk_value"));
    }

    #[test]
    fn malformed_negative_payload_length() {
        let mut buf = Vec::new();
        write_long(&mut buf, ChunkPos::new(0, 0).to_long());
        // VarInt -1
        buf.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
        let mut pos = 0;
        assert!(ClientboundDebugChunkValuePacket::read(&buf, &mut pos).is_err());
    }

    #[test]
    fn truncated_buffer() {
        let mut pos = 0;
        assert!(ClientboundDebugChunkValuePacket::read(&[], &mut pos).is_err());
        let buf = vec![0u8; 3];
        let mut pos2 = 0;
        assert!(ClientboundDebugChunkValuePacket::read(&buf, &mut pos2).is_err());
    }
}
