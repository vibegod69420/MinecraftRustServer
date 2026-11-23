#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

pub const GAME_PACKET_TYPES_SERVERBOUND_CHAT_COMMAND_SIGNED: PacketType = PacketType {
    name: "minecraft:chat_command_signed",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instant {
    pub epoch_millis: i64,
}

impl Instant {
    // JAVA: java.time.Instant#ofEpochMilli(long)
    pub fn of_epoch_milli(epoch_millis: i64) -> Self {
        Self { epoch_millis }
    }

    // JAVA: java.time.Instant#toEpochMilli()
    pub fn to_epoch_milli(&self) -> i64 {
        self.epoch_millis
    }
}

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

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_index)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte()
    pub fn read_byte(&mut self) -> u8 {
        let b = self.data[self.read_index];
        self.read_index += 1;
        b
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte(int)
    pub fn write_byte(&mut self, value: u8) {
        self.data.push(value);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong()
    pub fn read_long(&mut self) -> i64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.read_index..self.read_index + 8]);
        self.read_index += 8;
        i64::from_be_bytes(bytes)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong(long)
    pub fn write_long(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_be_bytes());
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

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf()
    pub fn read_utf(&mut self) -> String {
        self.read_utf_max(32767)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf(int)
    pub fn read_utf_max(&mut self, max_chars: usize) -> String {
        let max_bytes = max_chars.saturating_mul(3);
        let len = self.read_var_int();
        if len < 0 {
            panic!("Negative string length");
        }
        let len = len as usize;
        if len > max_bytes {
            panic!("String too long (bytes)");
        }
        if self.read_index + len > self.data.len() {
            panic!("Not enough bytes for string");
        }
        let bytes = &self.data[self.read_index..self.read_index + len];
        let s = std::str::from_utf8(bytes)
            .expect("Invalid UTF-8")
            .to_string();
        self.read_index += len;
        if s.chars().count() > max_chars {
            panic!("String too long (chars)");
        }
        s
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf(String)
    pub fn write_utf(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readInstant()
    pub fn read_instant(&mut self) -> Instant {
        Instant::of_epoch_milli(self.read_long())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeInstant(Instant)
    pub fn write_instant(&mut self, instant: Instant) {
        self.write_long(instant.to_epoch_milli());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readFixedBitSet(int)
    pub fn read_fixed_bitset(&mut self, nbits: usize) -> Vec<u8> {
        let nbytes = nbits.div_ceil(8);
        if self.read_index + nbytes > self.data.len() {
            panic!("Not enough bytes for fixed bitset");
        }
        let out = self.data[self.read_index..self.read_index + nbytes].to_vec();
        self.read_index += nbytes;
        out
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeFixedBitSet(BitSet, int)
    pub fn write_fixed_bitset(&mut self, bits: &[u8], nbits: usize) {
        let nbytes = nbits.div_ceil(8);
        if bits.len() != nbytes {
            panic!("Fixed bitset length mismatch: expected {}, got {}", nbytes, bits.len());
        }
        self.data.extend_from_slice(bits);
    }

    fn read_bytes_exact(&mut self, len: usize) -> Vec<u8> {
        if self.read_index + len > self.data.len() {
            panic!("Not enough bytes");
        }
        let out = self.data[self.read_index..self.read_index + len].to_vec();
        self.read_index += len;
        out
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignature {
    pub bytes: [u8; 256],
}

impl MessageSignature {
    // JAVA: net.minecraft.network.chat.MessageSignature#read(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let bytes_vec = buf.read_bytes_exact(256);
        let mut bytes = [0u8; 256];
        bytes.copy_from_slice(&bytes_vec);
        Self { bytes }
    }

    // JAVA: net.minecraft.network.chat.MessageSignature#write(FriendlyByteBuf, MessageSignature)
    pub fn write(buf: &mut FriendlyByteBuf, signature: &MessageSignature) {
        buf.write_bytes(&signature.bytes);
    }
}

pub const MAX_ARGUMENT_COUNT: usize = 8;
pub const MAX_ARGUMENT_NAME_LENGTH: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentSignaturesEntry {
    pub name: String,
    pub signature: MessageSignature,
}

impl ArgumentSignaturesEntry {
    // JAVA: net.minecraft.commands.arguments.ArgumentSignatures$Entry#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let name = buf.read_utf_max(MAX_ARGUMENT_NAME_LENGTH);
        let signature = MessageSignature::read(buf);
        Self { name, signature }
    }

    // JAVA: net.minecraft.commands.arguments.ArgumentSignatures$Entry#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.name);
        MessageSignature::write(buf, &self.signature);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArgumentSignatures {
    pub entries: Vec<ArgumentSignaturesEntry>,
}

impl ArgumentSignatures {
    // JAVA: net.minecraft.commands.arguments.ArgumentSignatures#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let count = buf.read_var_int();
        if count < 0 || count as usize > MAX_ARGUMENT_COUNT {
            panic!("Argument signatures count out of range: {count}");
        }
        let count = count as usize;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(ArgumentSignaturesEntry::read(buf));
        }
        Self { entries }
    }

    // JAVA: net.minecraft.commands.arguments.ArgumentSignatures#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entries.len() as i32);
        for entry in &self.entries {
            entry.write(buf);
        }
    }
}

pub const LAST_SEEN_MESSAGES_MAX_LENGTH: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenMessagesUpdate {
    pub offset: i32,
    pub acknowledged: Vec<u8>,
}

impl LastSeenMessagesUpdate {
    // JAVA: net.minecraft.network.chat.LastSeenMessages$Update#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let offset = buf.read_var_int();
        let acknowledged = buf.read_fixed_bitset(LAST_SEEN_MESSAGES_MAX_LENGTH);
        Self {
            offset,
            acknowledged,
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Update#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.offset);
        buf.write_fixed_bitset(&self.acknowledged, LAST_SEEN_MESSAGES_MAX_LENGTH);
    }
}

pub trait ServerGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ServerGamePacketListener#handleSignedChatCommand(ServerboundChatCommandSignedPacket)
    fn handle_signed_chat_command(&mut self, packet: &ServerboundChatCommandSignedPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatCommandSignedPacket {
    pub command: String,
    pub time_stamp: Instant,
    pub salt: i64,
    pub argument_signatures: ArgumentSignatures,
    pub last_seen_messages: LastSeenMessagesUpdate,
}

impl ServerboundChatCommandSignedPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandSignedPacket#<init>(String, Instant, long, ArgumentSignatures, LastSeenMessages.Update)
    pub fn new(
        command: String,
        time_stamp: Instant,
        salt: i64,
        argument_signatures: ArgumentSignatures,
        last_seen_messages: LastSeenMessagesUpdate,
    ) -> Self {
        Self {
            command,
            time_stamp,
            salt,
            argument_signatures,
            last_seen_messages,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandSignedPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let command = buf.read_utf();
        let time_stamp = buf.read_instant();
        let salt = buf.read_long();
        let argument_signatures = ArgumentSignatures::read(buf);
        let last_seen_messages = LastSeenMessagesUpdate::read(buf);
        Self {
            command,
            time_stamp,
            salt,
            argument_signatures,
            last_seen_messages,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandSignedPacket#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.command);
        buf.write_instant(self.time_stamp);
        buf.write_long(self.salt);
        self.argument_signatures.write(buf);
        self.last_seen_messages.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandSignedPacket#type()
    pub fn type_(&self) -> PacketType {
        GAME_PACKET_TYPES_SERVERBOUND_CHAT_COMMAND_SIGNED
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandSignedPacket#handle(ServerGamePacketListener)
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_signed_chat_command(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_packet() -> ServerboundChatCommandSignedPacket {
        let mut sig_bytes = [0u8; 256];
        sig_bytes[0] = 0xAA;
        sig_bytes[255] = 0x55;
        ServerboundChatCommandSignedPacket::new(
            "say hello".to_string(),
            Instant::of_epoch_milli(1_700_000_000_000_i64),
            -123_456_789_i64,
            ArgumentSignatures {
                entries: vec![ArgumentSignaturesEntry {
                    name: "msg".to_string(),
                    signature: MessageSignature { bytes: sig_bytes },
                }],
            },
            LastSeenMessagesUpdate {
                offset: 42,
                acknowledged: vec![0b1010_0001, 0b0000_1111, 0b0000_0000],
            },
        )
    }

    #[test]
    fn round_trip_packet() {
        let packet = sample_packet();
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundChatCommandSignedPacket::read(&mut read_buf);
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_empty_and_negative_values() {
        let packet = ServerboundChatCommandSignedPacket::new(
            String::new(),
            Instant::of_epoch_milli(i64::MIN),
            i64::MIN,
            ArgumentSignatures { entries: vec![] },
            LastSeenMessagesUpdate {
                offset: -1,
                acknowledged: vec![0, 0, 0],
            },
        );
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundChatCommandSignedPacket::read(&mut read_buf);
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn var_int_round_trip_edge_cases() {
        let mut buf = FriendlyByteBuf::default();
        let values = [
            0i32,
            1,
            -1,
            i32::MAX,
            i32::MIN,
            127,
            128,
            255,
            256,
            -2_147_483_648_i32,
        ];
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
    fn argument_signatures_max_entries_round_trip() {
        let mut entries = Vec::new();
        for i in 0..MAX_ARGUMENT_COUNT {
            let mut bytes = [0u8; 256];
            bytes[0] = i as u8;
            bytes[255] = 255_u8.wrapping_sub(i as u8);
            entries.push(ArgumentSignaturesEntry {
                name: format!("a{i}"),
                signature: MessageSignature { bytes },
            });
        }
        let sigs = ArgumentSignatures { entries };
        let mut buf = FriendlyByteBuf::default();
        sigs.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ArgumentSignatures::read(&mut read_buf);
        assert_eq!(decoded, sigs);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn handle_calls_listener_and_type_matches() {
        struct TestListener {
            called: bool,
            command: String,
            salt: i64,
        }
        impl ServerGamePacketListener for TestListener {
            fn handle_signed_chat_command(&mut self, packet: &ServerboundChatCommandSignedPacket) {
                self.called = true;
                self.command = packet.command.clone();
                self.salt = packet.salt;
            }
        }

        let packet = sample_packet();
        let mut listener = TestListener {
            called: false,
            command: String::new(),
            salt: 0,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.command, "say hello");
        assert_eq!(listener.salt, -123_456_789_i64);
        assert_eq!(
            packet.type_(),
            GAME_PACKET_TYPES_SERVERBOUND_CHAT_COMMAND_SIGNED
        );
    }
}
