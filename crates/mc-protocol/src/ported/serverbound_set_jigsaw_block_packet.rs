#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    const PACKED_X_LENGTH: u32 = 26;
    const PACKED_Z_LENGTH: u32 = 26;
    const PACKED_Y_LENGTH: u32 = 12;
    const PACKED_X_MASK: i64 = (1i64 << Self::PACKED_X_LENGTH) - 1;
    const PACKED_Y_MASK: i64 = (1i64 << Self::PACKED_Y_LENGTH) - 1;
    const PACKED_Z_MASK: i64 = (1i64 << Self::PACKED_Z_LENGTH) - 1;
    const Z_OFFSET: u32 = Self::PACKED_Y_LENGTH;
    const X_OFFSET: u32 = Self::PACKED_Y_LENGTH + Self::PACKED_Z_LENGTH;

    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(&self) -> i64 {
        ((self.x as i64 & Self::PACKED_X_MASK) << Self::X_OFFSET)
            | ((self.z as i64 & Self::PACKED_Z_MASK) << Self::Z_OFFSET)
            | (self.y as i64 & Self::PACKED_Y_MASK)
    }

    // JAVA: net.minecraft.core.BlockPos#of(long)
    pub fn of_long(packed: i64) -> Self {
        let x = (packed << (64 - Self::X_OFFSET - Self::PACKED_X_LENGTH)
            >> (64 - Self::PACKED_X_LENGTH)) as i32;
        let y = (packed << (64 - Self::PACKED_Y_LENGTH) >> (64 - Self::PACKED_Y_LENGTH)) as i32;
        let z = (packed << (64 - Self::Z_OFFSET - Self::PACKED_Z_LENGTH)
            >> (64 - Self::PACKED_Z_LENGTH)) as i32;
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub value: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#<init>(String)
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JointType {
    Rollable,
    Aligned,
}

impl JointType {
    // JAVA: net.minecraft.world.level.block.entity.JigsawBlockEntity$JointType#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            JointType::Rollable => "rollable",
            JointType::Aligned => "aligned",
        }
    }

    // JAVA: net.minecraft.util.StringRepresentable.EnumCodec#byName
    pub fn by_name(name: &str) -> Self {
        match name {
            "rollable" => JointType::Rollable,
            _ => JointType::Aligned,
        }
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

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>(ByteBuf)
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#array
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte
    fn read_byte(&mut self) -> u8 {
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        b
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte
    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut value: u32 = 0;
        let mut position: u32 = 0;
        loop {
            let b = self.read_byte();
            value |= ((b & 0x7F) as u32).wrapping_shl(position);
            if (b & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                panic!("VarInt too big");
            }
        }
        value as i32
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            let uval = value as u32;
            if (uval & !0x7Fu32) == 0 {
                self.write_byte(uval as u8);
                return;
            }
            self.write_byte(((uval & 0x7F) | 0x80) as u8);
            value = (uval >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong
    pub fn read_long(&mut self) -> i64 {
        let mut bytes = [0u8; 8];
        for b in &mut bytes {
            *b = self.read_byte();
        }
        i64::from_be_bytes(bytes)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
    pub fn write_long(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBlockPos
    pub fn read_block_pos(&mut self) -> BlockPos {
        BlockPos::of_long(self.read_long())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBlockPos
    pub fn write_block_pos(&mut self, pos: BlockPos) {
        self.write_long(pos.as_long());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf(int)
    pub fn read_utf(&mut self, max_length: usize) -> String {
        let max_bytes = max_length.saturating_mul(3);
        let byte_len = self.read_var_int();
        if byte_len < 0 {
            panic!("The received encoded string buffer length is less than zero! Weird string!");
        }
        let byte_len = byte_len as usize;
        if byte_len > max_bytes {
            panic!(
                "The received encoded string buffer length is longer than maximum allowed ({} > {})",
                byte_len, max_bytes
            );
        }
        let end = self.read_pos + byte_len;
        let s = std::str::from_utf8(&self.data[self.read_pos..end])
            .expect("Invalid UTF-8")
            .to_string();
        self.read_pos = end;
        let char_len = s.encode_utf16().count();
        if char_len > max_length {
            panic!(
                "The received string length is longer than maximum allowed ({} > {})",
                char_len, max_length
            );
        }
        s
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf(String,int)
    pub fn write_utf(&mut self, value: &str, max_length: usize) {
        let char_len = value.encode_utf16().count();
        if char_len > max_length {
            panic!(
                "String too big (was {} characters, max {})",
                char_len, max_length
            );
        }
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readIdentifier
    pub fn read_identifier(&mut self) -> Identifier {
        Identifier::new(self.read_utf(32767))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeIdentifier
    pub fn write_identifier(&mut self, id: &Identifier) {
        self.write_utf(&id.value, 32767);
    }
}

// JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetJigsawBlockPacket {
    pos: BlockPos,
    name: Identifier,
    target: Identifier,
    pool: Identifier,
    final_state: String,
    joint: JointType,
    selection_priority: i32,
    placement_priority: i32,
}

impl ServerboundSetJigsawBlockPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#<init>(BlockPos,Identifier,Identifier,Identifier,String,JointType,int,int)
    pub fn new(
        pos: BlockPos,
        name: Identifier,
        target: Identifier,
        pool: Identifier,
        final_state: String,
        joint: JointType,
        selection_priority: i32,
        placement_priority: i32,
    ) -> Self {
        Self {
            pos,
            name,
            target,
            pool,
            final_state,
            joint,
            selection_priority,
            placement_priority,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let pos = buf.read_block_pos();
        let name = buf.read_identifier();
        let target = buf.read_identifier();
        let pool = buf.read_identifier();
        let final_state = buf.read_utf(32767);
        let joint = JointType::by_name(&buf.read_utf(32767));
        let selection_priority = buf.read_var_int();
        let placement_priority = buf.read_var_int();
        Self {
            pos,
            name,
            target,
            pool,
            final_state,
            joint,
            selection_priority,
            placement_priority,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(self.pos);
        buf.write_identifier(&self.name);
        buf.write_identifier(&self.target);
        buf.write_identifier(&self.pool);
        buf.write_utf(&self.final_state, 32767);
        buf.write_utf(self.joint.get_serialized_name(), 32767);
        buf.write_var_int(self.selection_priority);
        buf.write_var_int(self.placement_priority);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#getName
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#getTarget
    pub fn get_target(&self) -> &Identifier {
        &self.target
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#getPool
    pub fn get_pool(&self) -> &Identifier {
        &self.pool
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#getFinalState
    pub fn get_final_state(&self) -> &str {
        &self.final_state
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#getJoint
    pub fn get_joint(&self) -> JointType {
        self.joint
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#getSelectionPriority
    pub fn get_selection_priority(&self) -> i32 {
        self.selection_priority
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket#getPlacementPriority
    pub fn get_placement_priority(&self) -> i32 {
        self.placement_priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let pkt = ServerboundSetJigsawBlockPacket::new(
            BlockPos { x: 1, y: 64, z: -3 },
            Identifier::new("minecraft:name"),
            Identifier::new("minecraft:target"),
            Identifier::new("minecraft:pool"),
            "minecraft:air".to_string(),
            JointType::Aligned,
            5,
            -2,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ServerboundSetJigsawBlockPacket::read(&mut read_buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn block_pos_negative_and_extreme() {
        let positions = [
            BlockPos {
                x: -33_554_432,
                y: -2048,
                z: -33_554_432,
            },
            BlockPos {
                x: 33_554_431,
                y: 2047,
                z: 33_554_431,
            },
            BlockPos { x: -1, y: -1, z: -1 },
        ];
        for pos in positions {
            let packed = pos.as_long();
            let unpacked = BlockPos::of_long(packed);
            assert_eq!(pos, unpacked);
        }
    }

    #[test]
    fn negative_priorities_and_rollable() {
        let pkt = ServerboundSetJigsawBlockPacket::new(
            BlockPos { x: 0, y: 0, z: 0 },
            Identifier::new("a:b"),
            Identifier::new("c:d"),
            Identifier::new("e:f"),
            "state".to_string(),
            JointType::Rollable,
            i32::MIN,
            i32::MAX,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ServerboundSetJigsawBlockPacket::read(&mut read_buf);
        assert_eq!(decoded.get_selection_priority(), i32::MIN);
        assert_eq!(decoded.get_placement_priority(), i32::MAX);
        assert_eq!(decoded.get_joint(), JointType::Rollable);
    }

    #[test]
    fn unknown_joint_defaults_to_aligned() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_block_pos(BlockPos { x: 0, y: 0, z: 0 });
        buf.write_identifier(&Identifier::new("n:n"));
        buf.write_identifier(&Identifier::new("t:t"));
        buf.write_identifier(&Identifier::new("p:p"));
        buf.write_utf("final", 32767);
        buf.write_utf("unknown_joint", 32767);
        buf.write_var_int(0);
        buf.write_var_int(0);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ServerboundSetJigsawBlockPacket::read(&mut read_buf);
        assert_eq!(decoded.get_joint(), JointType::Aligned);
    }

    #[test]
    fn varint_round_trip_edge_cases() {
        let values = [0, 1, -1, 127, 128, 255, i32::MAX, i32::MIN, -2_147_483_648i32];
        for v in values {
            let mut buf = FriendlyByteBuf::new();
            buf.write_var_int(v);
            let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
            assert_eq!(read_buf.read_var_int(), v);
        }
    }
}
