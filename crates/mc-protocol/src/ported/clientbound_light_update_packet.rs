#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    reader_index: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            reader_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.reader_index)
    }

    fn read_byte(&mut self) -> u8 {
        assert!(self.reader_index < self.data.len(), "buffer underflow");
        let b = self.data[self.reader_index];
        self.reader_index += 1;
        b
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let b = self.read_byte();
            value |= ((b & 0x7F) as i32) << position;
            if (b & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                panic!("VarInt too big");
            }
        }
        value
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut v = value as u32;
        loop {
            if (v & !0x7F_u32) == 0 {
                self.data.push(v as u8);
                return;
            }
            self.data.push(((v as u8) & 0x7F) | 0x80);
            v >>= 7;
        }
    }

    pub fn read_long(&mut self) -> i64 {
        let mut bytes = [0u8; 8];
        for b in &mut bytes {
            *b = self.read_byte();
        }
        i64::from_be_bytes(bytes)
    }

    pub fn write_long(&mut self, v: i64) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    pub fn read_long_array(&mut self) -> Vec<i64> {
        let len = self.read_var_int();
        assert!(len >= 0, "negative long array length");
        let len = len as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(self.read_long());
        }
        out
    }

    pub fn write_long_array(&mut self, values: &[i64]) {
        self.write_var_int(values.len() as i32);
        for &v in values {
            self.write_long(v);
        }
    }

    pub fn read_byte_array(&mut self, max_len: Option<usize>) -> Vec<u8> {
        let len = self.read_var_int();
        assert!(len >= 0, "negative byte array length");
        let len = len as usize;
        if let Some(max_len) = max_len {
            assert!(len <= max_len, "byte array too long");
        }
        let end = self
            .reader_index
            .checked_add(len)
            .expect("length overflow");
        assert!(end <= self.data.len(), "buffer underflow");
        let out = self.data[self.reader_index..end].to_vec();
        self.reader_index = end;
        out
    }

    pub fn write_byte_array(&mut self, values: &[u8]) {
        self.write_var_int(values.len() as i32);
        self.data.extend_from_slice(values);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JavaBitSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LevelLightEngine;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientboundLightUpdatePacketData {
    pub sky_y_mask: Vec<i64>,
    pub block_y_mask: Vec<i64>,
    pub empty_sky_y_mask: Vec<i64>,
    pub empty_block_y_mask: Vec<i64>,
    pub sky_updates: Vec<Vec<u8>>,
    pub block_updates: Vec<Vec<u8>>,
}

impl ClientboundLightUpdatePacketData {
    pub fn new(
        sky_y_mask: Vec<i64>,
        block_y_mask: Vec<i64>,
        empty_sky_y_mask: Vec<i64>,
        empty_block_y_mask: Vec<i64>,
        sky_updates: Vec<Vec<u8>>,
        block_updates: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            sky_y_mask,
            block_y_mask,
            empty_sky_y_mask,
            empty_block_y_mask,
            sky_updates,
            block_updates,
        }
    }

    pub fn from_buf(buf: &mut FriendlyByteBuf, _x: i32, _z: i32) -> Self {
        let sky_y_mask = buf.read_long_array();
        let block_y_mask = buf.read_long_array();
        let empty_sky_y_mask = buf.read_long_array();
        let empty_block_y_mask = buf.read_long_array();

        let sky_count = buf.read_var_int();
        assert!(sky_count >= 0, "negative sky update count");
        let mut sky_updates = Vec::with_capacity(sky_count as usize);
        for _ in 0..sky_count {
            sky_updates.push(buf.read_byte_array(Some(2048)));
        }

        let block_count = buf.read_var_int();
        assert!(block_count >= 0, "negative block update count");
        let mut block_updates = Vec::with_capacity(block_count as usize);
        for _ in 0..block_count {
            block_updates.push(buf.read_byte_array(Some(2048)));
        }

        Self {
            sky_y_mask,
            block_y_mask,
            empty_sky_y_mask,
            empty_block_y_mask,
            sky_updates,
            block_updates,
        }
    }

    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_long_array(&self.sky_y_mask);
        buf.write_long_array(&self.block_y_mask);
        buf.write_long_array(&self.empty_sky_y_mask);
        buf.write_long_array(&self.empty_block_y_mask);

        buf.write_var_int(self.sky_updates.len() as i32);
        for arr in &self.sky_updates {
            buf.write_byte_array(arr);
        }

        buf.write_var_int(self.block_updates.len() as i32);
        for arr in &self.block_updates {
            buf.write_byte_array(arr);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub trait ClientGamePacketListener {
    fn handle_light_update_packet(&mut self, packet: &ClientboundLightUpdatePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLightUpdatePacket {
    x: i32,
    z: i32,
    light_data: ClientboundLightUpdatePacketData,
}

impl ClientboundLightUpdatePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacket#<init>
    pub fn new(
        chunk_pos: ChunkPos,
        _light_engine: &LevelLightEngine,
        _sky_y_mask: Option<&JavaBitSet>,
        _block_y_mask: Option<&JavaBitSet>,
    ) -> Self {
        Self {
            x: chunk_pos.x,
            z: chunk_pos.z,
            light_data: ClientboundLightUpdatePacketData::default(),
        }
    }

    pub fn new_with_data(x: i32, z: i32, light_data: ClientboundLightUpdatePacketData) -> Self {
        Self { x, z, light_data }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        let x = buf.read_var_int();
        let z = buf.read_var_int();
        let light_data = ClientboundLightUpdatePacketData::from_buf(buf, x, z);
        Self { x, z, light_data }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.x);
        buf.write_var_int(self.z);
        self.light_data.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_light_update_packet(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacket#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacket#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacket#getLightData
    pub fn get_light_data(&self) -> &ClientboundLightUpdatePacketData {
        &self.light_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen: Option<ClientboundLightUpdatePacket>,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_light_update_packet(&mut self, packet: &ClientboundLightUpdatePacket) {
            self.seen = Some(packet.clone());
        }
    }

    #[test]
    fn round_trip_with_light_data() {
        let light_data = ClientboundLightUpdatePacketData::new(
            vec![0x0F_i64, -1_i64],
            vec![0xAA_i64],
            vec![],
            vec![i64::MIN, i64::MAX],
            vec![vec![1u8; 2048]],
            vec![vec![2u8; 2048], vec![3u8; 2048]],
        );
        let packet = ClientboundLightUpdatePacket::new_with_data(123, -456, light_data.clone());
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundLightUpdatePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_x(), 123);
        assert_eq!(decoded.get_z(), -456);
        assert_eq!(decoded.get_light_data(), &light_data);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_empty_light_data() {
        let packet = ClientboundLightUpdatePacket::new_with_data(
            0,
            0,
            ClientboundLightUpdatePacketData::default(),
        );
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundLightUpdatePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_x(), 0);
        assert_eq!(decoded.get_z(), 0);
        assert!(decoded.get_light_data().sky_updates.is_empty());
        assert!(decoded.get_light_data().block_updates.is_empty());
    }

    #[test]
    fn constructor_from_chunk_pos_creates_default_light_data() {
        let packet = ClientboundLightUpdatePacket::new(
            ChunkPos { x: -7, z: 42 },
            &LevelLightEngine,
            None,
            None,
        );
        assert_eq!(packet.get_x(), -7);
        assert_eq!(packet.get_z(), 42);
        assert_eq!(
            packet.get_light_data(),
            &ClientboundLightUpdatePacketData::default()
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ClientboundLightUpdatePacket::new_with_data(
            1,
            2,
            ClientboundLightUpdatePacketData::default(),
        );
        let mut listener = TestListener { seen: None };
        packet.handle(&mut listener);
        let seen = listener.seen.expect("packet not handled");
        assert_eq!(seen.get_x(), 1);
        assert_eq!(seen.get_z(), 2);
    }

    #[test]
    fn negative_coords_round_trip() {
        let packet = ClientboundLightUpdatePacket::new_with_data(
            i32::MIN,
            i32::MAX,
            ClientboundLightUpdatePacketData::default(),
        );
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundLightUpdatePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_x(), i32::MIN);
        assert_eq!(decoded.get_z(), i32::MAX);
    }
}
