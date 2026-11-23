#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

pub const GAME_PACKET_TYPES_CLIENTBOUND_TAG_QUERY: PacketType = PacketType {
    name: "minecraft:tag_query",
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_index: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            read_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_index)
    }

    fn ensure_remaining(&self, len: usize) {
        if self.read_index + len > self.data.len() {
            panic!("Not enough bytes: need {len}, have {}", self.remaining());
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte()
    pub fn read_byte(&mut self) -> u8 {
        self.ensure_remaining(1);
        let b = self.data[self.read_index];
        self.read_index += 1;
        b
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte(int)
    pub fn write_byte(&mut self, value: u8) {
        self.data.push(value);
    }

    fn read_i16(&mut self) -> i16 {
        self.ensure_remaining(2);
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.data[self.read_index..self.read_index + 2]);
        self.read_index += 2;
        i16::from_be_bytes(bytes)
    }

    fn read_u16(&mut self) -> u16 {
        self.read_i16() as u16
    }

    fn read_i32(&mut self) -> i32 {
        self.ensure_remaining(4);
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.data[self.read_index..self.read_index + 4]);
        self.read_index += 4;
        i32::from_be_bytes(bytes)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_byte();
            value |= ((current_byte & 0x7F) as i32) << position;
            if (current_byte & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                panic!("VarInt too big");
            }
        }
        value
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F) == 0 {
                self.write_byte(value as u8);
                return;
            }
            self.write_byte(((value & 0x7F) as u8) | 0x80);
            value = ((value as u32) >> 7) as i32;
        }
    }

    fn skip_nbt_string(&mut self) {
        let len = self.read_u16() as usize;
        self.ensure_remaining(len);
        self.read_index += len;
    }

    fn skip_nbt_tag_payload(&mut self, tag_type: u8) {
        match tag_type {
            1 => {
                self.ensure_remaining(1);
                self.read_index += 1;
            } // TAG_Byte
            2 => {
                self.ensure_remaining(2);
                self.read_index += 2;
            } // TAG_Short
            3 => {
                self.ensure_remaining(4);
                self.read_index += 4;
            } // TAG_Int
            4 => {
                self.ensure_remaining(8);
                self.read_index += 8;
            } // TAG_Long
            5 => {
                self.ensure_remaining(4);
                self.read_index += 4;
            } // TAG_Float
            6 => {
                self.ensure_remaining(8);
                self.read_index += 8;
            } // TAG_Double
            7 => {
                let len = self.read_i32();
                if len < 0 {
                    panic!("Negative TAG_Byte_Array length");
                }
                let len = len as usize;
                self.ensure_remaining(len);
                self.read_index += len;
            } // TAG_Byte_Array
            8 => {
                self.skip_nbt_string();
            } // TAG_String
            9 => {
                let list_type = self.read_byte();
                let len = self.read_i32();
                if len < 0 {
                    panic!("Negative TAG_List length");
                }
                for _ in 0..len {
                    self.skip_nbt_tag_payload(list_type);
                }
            } // TAG_List
            10 => loop {
                let child_type = self.read_byte();
                if child_type == 0 {
                    break;
                }
                self.skip_nbt_string();
                self.skip_nbt_tag_payload(child_type);
            }, // TAG_Compound
            11 => {
                let len = self.read_i32();
                if len < 0 {
                    panic!("Negative TAG_Int_Array length");
                }
                let byte_len = (len as usize).saturating_mul(4);
                self.ensure_remaining(byte_len);
                self.read_index += byte_len;
            } // TAG_Int_Array
            12 => {
                let len = self.read_i32();
                if len < 0 {
                    panic!("Negative TAG_Long_Array length");
                }
                let byte_len = (len as usize).saturating_mul(8);
                self.ensure_remaining(byte_len);
                self.read_index += byte_len;
            } // TAG_Long_Array
            _ => panic!("Unknown NBT tag type: {tag_type}"),
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readNbt()
    pub fn read_nbt(&mut self) -> Option<CompoundTag> {
        let start = self.read_index;
        let tag_type = self.read_byte();
        if tag_type == 0 {
            return None;
        }
        if tag_type != 10 {
            panic!("Expected TAG_Compound root, got {tag_type}");
        }
        self.skip_nbt_tag_payload(tag_type);
        let raw = self.data[start..self.read_index].to_vec();
        Some(CompoundTag { raw })
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeNbt(CompoundTag)
    pub fn write_nbt(&mut self, tag: Option<&CompoundTag>) {
        match tag {
            Some(tag) => self.data.extend_from_slice(&tag.raw),
            None => self.write_byte(0),
        }
    }
}

/// Minimal self-contained NBT compound representation.
/// Stores the raw network NBT bytes (including root type byte, anonymous root payload).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundTag {
    pub raw: Vec<u8>,
}

impl CompoundTag {
    pub fn new(raw: Vec<u8>) -> Self {
        Self { raw }
    }
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleTagQueryPacket(ClientboundTagQueryPacket)
    fn handle_tag_query_packet(&mut self, packet: &ClientboundTagQueryPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTagQueryPacket {
    transaction_id: i32,
    tag: Option<CompoundTag>,
}

impl ClientboundTagQueryPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundTagQueryPacket#<init>(int, CompoundTag)
    pub fn new(transaction_id: i32, tag: Option<CompoundTag>) -> Self {
        Self {
            transaction_id,
            tag,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTagQueryPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let transaction_id = buf.read_var_int();
        let tag = buf.read_nbt();
        Self {
            transaction_id,
            tag,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTagQueryPacket#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.transaction_id);
        buf.write_nbt(self.tag.as_ref());
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTagQueryPacket#type()
    pub fn type_(&self) -> PacketType {
        GAME_PACKET_TYPES_CLIENTBOUND_TAG_QUERY
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTagQueryPacket#handle(ClientGamePacketListener)
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_tag_query_packet(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTagQueryPacket#getTransactionId()
    pub fn get_transaction_id(&self) -> i32 {
        self.transaction_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTagQueryPacket#getTag()
    pub fn get_tag(&self) -> Option<&CompoundTag> {
        self.tag.as_ref()
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTagQueryPacket#isSkippable()
    pub fn is_skippable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_compound_nbt() -> CompoundTag {
        // Anonymous network NBT compound: TAG_Compound (10) + TAG_End (0)
        CompoundTag::new(vec![10, 0])
    }

    fn simple_compound_nbt() -> CompoundTag {
        // TAG_Compound, then TAG_Int named "x" = 42, then TAG_End
        // 10
        //   3 (int), name length 1, "x", value 42
        //   0 (end)
        let mut raw = Vec::new();
        raw.push(10); // root compound
        raw.push(3); // TAG_Int
        raw.extend_from_slice(&1_i16.to_be_bytes()); // name length
        raw.push(b'x');
        raw.extend_from_slice(&42_i32.to_be_bytes());
        raw.push(0); // end
        CompoundTag::new(raw)
    }

    #[test]
    fn round_trip_with_null_tag() {
        let packet = ClientboundTagQueryPacket::new(123, None);
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundTagQueryPacket::read(&mut read_buf);
        assert_eq!(decoded.get_transaction_id(), 123);
        assert_eq!(decoded.get_tag(), None);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_with_compound_tag() {
        let tag = simple_compound_nbt();
        let packet = ClientboundTagQueryPacket::new(-7, Some(tag.clone()));
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundTagQueryPacket::read(&mut read_buf);
        assert_eq!(decoded.get_transaction_id(), -7);
        assert_eq!(decoded.get_tag(), Some(&tag));
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_empty_compound_and_extreme_ids() {
        let cases = [
            (0_i32, Some(empty_compound_nbt())),
            (i32::MAX, None),
            (i32::MIN, Some(empty_compound_nbt())),
            (-1_i32, Some(simple_compound_nbt())),
        ];
        for (id, tag) in cases {
            let packet = ClientboundTagQueryPacket::new(id, tag.clone());
            let mut buf = FriendlyByteBuf::default();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ClientboundTagQueryPacket::read(&mut read_buf);
            assert_eq!(decoded.get_transaction_id(), id);
            assert_eq!(decoded.get_tag(), tag.as_ref());
            assert_eq!(read_buf.remaining(), 0);
        }
    }

    #[test]
    fn var_int_round_trip_edge_cases() {
        let mut buf = FriendlyByteBuf::default();
        let values = [0_i32, 1, -1, 127, 128, 255, i32::MAX, i32::MIN];
        for &v in &values {
            buf.write_var_int(v);
        }
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        for &v in &values {
            assert_eq!(read_buf.read_var_int(), v);
        }
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn handle_type_and_skippable() {
        struct TestListener {
            called: bool,
            transaction_id: i32,
            had_tag: bool,
        }
        impl ClientGamePacketListener for TestListener {
            fn handle_tag_query_packet(&mut self, packet: &ClientboundTagQueryPacket) {
                self.called = true;
                self.transaction_id = packet.get_transaction_id();
                self.had_tag = packet.get_tag().is_some();
            }
        }

        let packet = ClientboundTagQueryPacket::new(42, Some(empty_compound_nbt()));
        let mut listener = TestListener {
            called: false,
            transaction_id: 0,
            had_tag: false,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.transaction_id, 42);
        assert!(listener.had_tag);
        assert_eq!(packet.type_(), GAME_PACKET_TYPES_CLIENTBOUND_TAG_QUERY);
        assert!(packet.is_skippable());
    }
}
