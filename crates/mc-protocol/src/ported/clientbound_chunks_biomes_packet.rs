use std::fmt;

pub const TWO_MEGABYTES: usize = 2_097_152;
pub const CLIENTBOUND_CHUNKS_BIOMES_TYPE: PacketType = PacketType("clientbound_chunks_biomes");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#toLong
    pub fn to_long(self) -> i64 {
        ((self.x as i64) & 0xFFFF_FFFF) | (((self.z as i64) & 0xFFFF_FFFF) << 32)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#<init>
    pub fn from_long(l: i64) -> Self {
        Self {
            x: l as i32,
            z: (l >> 32) as i32,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_index: usize,
    write_index: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn with_capacity(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
            read_index: 0,
            write_index: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn from_vec(data: Vec<u8>) -> Self {
        let write_index = data.len();
        Self {
            data,
            read_index: 0,
            write_index,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    // JAVA: io.netty.buffer.ByteBuf#writerIndex
    pub fn writer_index(&self) -> usize {
        self.write_index
    }

    // JAVA: io.netty.buffer.ByteBuf#capacity
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte
    pub fn read_byte(&mut self) -> i8 {
        let b = self.data[self.read_index];
        self.read_index += 1;
        b as i8
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte
    pub fn write_byte(&mut self, b: i8) -> &mut Self {
        if self.write_index < self.data.len() {
            self.data[self.write_index] = b as u8;
        } else {
            self.data.push(b as u8);
        }
        self.write_index += 1;
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBytes
    pub fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        let end = self.read_index + len;
        let out = self.data[self.read_index..end].to_vec();
        self.read_index = end;
        out
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBytes
    pub fn write_bytes(&mut self, src: &[u8]) -> &mut Self {
        for &b in src {
            self.write_byte(b as i8);
        }
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong
    pub fn read_long(&mut self) -> i64 {
        let mut bytes = [0u8; 8];
        for b in &mut bytes {
            *b = self.read_byte() as u8;
        }
        i64::from_be_bytes(bytes)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
    pub fn write_long(&mut self, v: i64) -> &mut Self {
        for &b in &v.to_be_bytes() {
            self.write_byte(b as i8);
        }
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let b = self.read_byte() as i32;
            value |= (b & 0x7F) << position;
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

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            let mut b = (value & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if value != 0 {
                b |= 0x80;
            }
            self.write_byte(b as i8);
            if value == 0 {
                break;
            }
        }
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByteArray
    pub fn read_byte_array(&mut self, max_size: usize) -> Vec<u8> {
        let len = self.read_var_int() as usize;
        if len > max_size {
            panic!(
                "ByteArray with size {} is bigger than allowed {}",
                len, max_size
            );
        }
        self.read_bytes(len)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByteArray
    pub fn write_byte_array(&mut self, array: &[u8]) -> &mut Self {
        self.write_var_int(array.len() as i32);
        self.write_bytes(array);
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readChunkPos
    pub fn read_chunk_pos(&mut self) -> ChunkPos {
        ChunkPos::from_long(self.read_long())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeChunkPos
    pub fn write_chunk_pos(&mut self, pos: ChunkPos) -> &mut Self {
        self.write_long(pos.to_long());
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readList
    pub fn read_list<T>(&mut self, reader: impl Fn(&mut FriendlyByteBuf) -> T) -> Vec<T> {
        let len = self.read_var_int() as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(reader(self));
        }
        out
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeCollection
    pub fn write_collection<T>(
        &mut self,
        collection: &[T],
        writer: impl Fn(&mut FriendlyByteBuf, &T),
    ) -> &mut Self {
        self.write_var_int(collection.len() as i32);
        for item in collection {
            writer(self, item);
        }
        self
    }
}

pub trait BiomeContainer {
    fn get_serialized_size(&self) -> usize;
    fn write(&self, buf: &mut FriendlyByteBuf);
}

pub trait LevelChunkSectionLike {
    type Biomes: BiomeContainer;
    fn get_biomes(&self) -> &Self::Biomes;
}

pub trait LevelChunkLike {
    type Section: LevelChunkSectionLike;
    fn get_pos(&self) -> ChunkPos;
    fn get_sections(&self) -> &[Self::Section];
}

pub trait ClientGamePacketListener {
    fn handle_chunks_biomes(&mut self, packet: &ClientboundChunksBiomesPacket);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkBiomeData {
    pos: ChunkPos,
    buffer: Vec<u8>,
}

impl ChunkBiomeData {
    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket$ChunkBiomeData#<init>
    pub fn from_chunk<C: LevelChunkLike>(chunk: &C) -> Self {
        let pos = chunk.get_pos();
        let size = Self::calculate_chunk_size(chunk);
        let mut buf = FriendlyByteBuf::with_capacity(size);
        Self::extract_chunk_data(&mut buf, chunk);
        Self {
            pos,
            buffer: buf.into_inner(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket$ChunkBiomeData#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let pos = buf.read_chunk_pos();
        let buffer = buf.read_byte_array(TWO_MEGABYTES);
        Self { pos, buffer }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket$ChunkBiomeData#calculateChunkSize
    pub fn calculate_chunk_size<C: LevelChunkLike>(chunk: &C) -> usize {
        let mut size: usize = 0;
        for section in chunk.get_sections() {
            size += section.get_biomes().get_serialized_size();
        }
        size
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket$ChunkBiomeData#getReadBuffer
    pub fn get_read_buffer(&self) -> FriendlyByteBuf {
        FriendlyByteBuf::from_vec(self.buffer.clone())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket$ChunkBiomeData#getWriteBuffer
    fn get_write_buffer(&self) -> FriendlyByteBuf {
        FriendlyByteBuf::with_capacity(self.buffer.len())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket$ChunkBiomeData#extractChunkData
    pub fn extract_chunk_data<C: LevelChunkLike>(buf: &mut FriendlyByteBuf, chunk: &C) {
        for section in chunk.get_sections() {
            section.get_biomes().write(buf);
        }
        if buf.writer_index() != buf.capacity() {
            panic!(
                "Didn't fill biome buffer: expected {} bytes, got {}",
                buf.capacity(),
                buf.writer_index()
            );
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket$ChunkBiomeData#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_chunk_pos(self.pos);
        buf.write_byte_array(&self.buffer);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket$ChunkBiomeData#pos
    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket$ChunkBiomeData#buffer
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundChunksBiomesPacket {
    chunk_biome_data: Vec<ChunkBiomeData>,
}

impl ClientboundChunksBiomesPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let chunk_biome_data = buf.read_list(ChunkBiomeData::read);
        Self { chunk_biome_data }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket#forChunks
    pub fn for_chunks<C: LevelChunkLike>(chunks: &[C]) -> Self {
        let chunk_biome_data = chunks.iter().map(ChunkBiomeData::from_chunk).collect();
        Self { chunk_biome_data }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.chunk_biome_data, |b, item| item.write(b));
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_CHUNKS_BIOMES_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_chunks_biomes(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket#chunkBiomeData
    pub fn chunk_biome_data(&self) -> &[ChunkBiomeData] {
        &self.chunk_biome_data
    }
}

impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBiomes(Vec<u8>);
    impl BiomeContainer for MockBiomes {
        fn get_serialized_size(&self) -> usize {
            self.0.len()
        }
        fn write(&self, buf: &mut FriendlyByteBuf) {
            buf.write_bytes(&self.0);
        }
    }

    struct MockSection(MockBiomes);
    impl LevelChunkSectionLike for MockSection {
        type Biomes = MockBiomes;
        fn get_biomes(&self) -> &Self::Biomes {
            &self.0
        }
    }

    struct MockChunk {
        pos: ChunkPos,
        sections: Vec<MockSection>,
    }
    impl LevelChunkLike for MockChunk {
        type Section = MockSection;
        fn get_pos(&self) -> ChunkPos {
            self.pos
        }
        fn get_sections(&self) -> &[Self::Section] {
            &self.sections
        }
    }

    #[test]
    fn roundtrip_chunks_biomes_packet() {
        let chunk = MockChunk {
            pos: ChunkPos { x: 3, z: -5 },
            sections: vec![
                MockSection(MockBiomes(vec![1, 2, 3])),
                MockSection(MockBiomes(vec![4, 5])),
            ],
        };
        let packet = ClientboundChunksBiomesPacket::for_chunks(&[chunk]);
        assert_eq!(packet.chunk_biome_data().len(), 1);
        assert_eq!(packet.chunk_biome_data()[0].pos(), ChunkPos { x: 3, z: -5 });
        assert_eq!(packet.chunk_biome_data()[0].buffer(), &[1, 2, 3, 4, 5]);

        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundChunksBiomesPacket::read(&mut read_buf);
        assert_eq!(packet, decoded);
    }

    #[test]
    fn negative_coords_and_empty_chunk() {
        let chunk = MockChunk {
            pos: ChunkPos { x: -1, z: i32::MIN },
            sections: vec![],
        };
        let packet = ClientboundChunksBiomesPacket::for_chunks(&[chunk]);
        assert!(packet.chunk_biome_data()[0].buffer().is_empty());

        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let decoded =
            ClientboundChunksBiomesPacket::read(&mut FriendlyByteBuf::from_vec(buf.into_inner()));
        assert_eq!(
            decoded.chunk_biome_data()[0].pos(),
            ChunkPos { x: -1, z: i32::MIN }
        );
    }

    #[test]
    #[should_panic(expected = "Didn't fill biome buffer")]
    fn extract_chunk_data_panics_on_size_mismatch() {
        let chunk = MockChunk {
            pos: ChunkPos { x: 0, z: 0 },
            sections: vec![MockSection(MockBiomes(vec![1, 2, 3]))],
        };
        let mut buf = FriendlyByteBuf::with_capacity(5);
        ChunkBiomeData::extract_chunk_data(&mut buf, &chunk);
    }

    #[test]
    fn varint_roundtrip_edge_cases() {
        let values = [0i32, 1, 127, 128, 255, -1, i32::MIN, i32::MAX];
        let mut buf = FriendlyByteBuf::default();
        for &v in &values {
            buf.write_var_int(v);
        }
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        for &v in &values {
            assert_eq!(read_buf.read_var_int(), v);
        }
    }

    #[test]
    fn chunk_pos_long_roundtrip_negative_and_extremes() {
        let positions = [
            ChunkPos { x: 0, z: 0 },
            ChunkPos { x: -1, z: -1 },
            ChunkPos { x: i32::MIN, z: i32::MAX },
            ChunkPos { x: 12345, z: -67890 },
        ];
        for pos in positions {
            let l = pos.to_long();
            assert_eq!(ChunkPos::from_long(l), pos);
        }
    }

    #[test]
    #[should_panic(expected = "bigger than allowed")]
    fn read_byte_array_rejects_oversized() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_byte_array(&[1, 2, 3, 4]);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        read_buf.read_byte_array(3);
    }
}
