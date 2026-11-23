/// Minimal self-contained byte buffer mirroring Minecraft's FriendlyByteBuf.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    rpos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            rpos: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.to_vec(),
            rpos: 0,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.rpos)
    }

    fn read_byte(&mut self) -> u8 {
        assert!(self.rpos < self.data.len(), "FriendlyByteBuf: out of bounds");
        let b = self.data[self.rpos];
        self.rpos = self.rpos.wrapping_add(1);
        b
    }

    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_byte();
            i |= ((b0 & 127) as i32).wrapping_shl((j.wrapping_mul(7)) as u32);
            j = j.wrapping_add(1);
            if j > 5 {
                panic!("VarInt too big");
            }
            if (b0 & 128) == 0 {
                break;
            }
        }
        i
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value & -128i32) == 0 {
                self.write_byte(value as u8);
                return self;
            }
            self.write_byte(((value & 127) as u8) | 128);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
    pub fn read_utf(&mut self) -> String {
        self.read_utf_with_max_length(32767)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
    pub fn read_utf_with_max_length(&mut self, max_length: i32) -> String {
        let max_bytes = max_length.wrapping_mul(3);
        let byte_len = self.read_var_int();
        if byte_len < 0 {
            panic!("The received encoded string buffer length is less than zero! Weird string!");
        }
        if byte_len > max_bytes {
            panic!(
                "The received encoded string buffer length is longer than maximum allowed ({} > {})",
                byte_len, max_bytes
            );
        }
        let byte_len_usize = byte_len as usize;
        if self.remaining() < byte_len_usize {
            panic!("Not enough bytes in buffer, expected {}, got {}", byte_len_usize, self.remaining());
        }
        let start = self.rpos;
        let end = self.rpos.wrapping_add(byte_len_usize);
        let bytes = self.data[start..end].to_vec();
        self.rpos = end;
        let s = String::from_utf8(bytes).expect("Invalid UTF-8 string");
        let utf16_len = s.encode_utf16().count() as i32;
        if utf16_len > max_length {
            panic!(
                "The received string length is longer than maximum allowed ({} > {})",
                utf16_len, max_length
            );
        }
        s
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
    pub fn write_utf(&mut self, value: &str) -> &mut Self {
        self.write_utf_with_max_length(value, 32767)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
    pub fn write_utf_with_max_length(&mut self, value: &str, max_length: i32) -> &mut Self {
        let bytes = value.as_bytes();
        let byte_len = bytes.len() as i32;
        let max_bytes = max_length.wrapping_mul(3);
        if byte_len > max_bytes {
            panic!(
                "String too big (was {} bytes encoded, max {})",
                byte_len, max_bytes
            );
        }
        self.write_var_int(byte_len);
        self.data.extend_from_slice(bytes);
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readEnum
    pub fn read_enum_ordinal(&mut self) -> i32 {
        self.read_var_int()
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeEnum
    pub fn write_enum_ordinal(&mut self, ordinal: i32) -> &mut Self {
        self.write_var_int(ordinal)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readList
    pub fn read_list<T, F>(&mut self, mut reader: F) -> Vec<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        let len = self.read_var_int();
        let mut out = Vec::with_capacity(if len < 0 { 0 } else { len as usize });
        let mut j: i32 = 0;
        while j < len {
            out.push(reader(self));
            j = j.wrapping_add(1);
        }
        out
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeCollection
    pub fn write_collection<T, F>(&mut self, collection: &[T], mut writer: F) -> &mut Self
    where
        F: FnMut(&mut Self, &T),
    {
        self.write_var_int(collection.len() as i32);
        let mut i: usize = 0;
        while i < collection.len() {
            writer(self, &collection[i]);
            i = i.wrapping_add(1);
        }
        self
    }
}

/// Minimal packet type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Add,
    Remove,
    Set,
}

impl Action {
    pub fn from_ordinal(ordinal: i32) -> Self {
        match ordinal {
            0 => Self::Add,
            1 => Self::Remove,
            2 => Self::Set,
            _ => panic!("Unknown Action ordinal: {}", ordinal),
        }
    }

    pub fn ordinal(self) -> i32 {
        match self {
            Self::Add => 0,
            Self::Remove => 1,
            Self::Set => 2,
        }
    }
}

/// Minimal listener stub for client game packet handling.
pub trait ClientGamePacketListener {
    fn handle_custom_chat_completions(&mut self, packet: &ClientboundCustomChatCompletionsPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomChatCompletionsPacket {
    action: Action,
    entries: Vec<String>,
}

impl ClientboundCustomChatCompletionsPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCustomChatCompletionsPacket#<init>
    pub fn new(action: Action, entries: Vec<String>) -> Self {
        Self { action, entries }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCustomChatCompletionsPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let action = Action::from_ordinal(buf.read_enum_ordinal());
        let entries = buf.read_list(|b| b.read_utf());
        Self { action, entries }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCustomChatCompletionsPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_enum_ordinal(self.action.ordinal());
        buf.write_collection(&self.entries, |b, s| {
            b.write_utf(s);
        });
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCustomChatCompletionsPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:custom_chat_completions")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCustomChatCompletionsPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_custom_chat_completions(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCustomChatCompletionsPacket#action
    pub fn action(&self) -> Action {
        self.action
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCustomChatCompletionsPacket#entries
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen: Option<(Action, usize)>,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_custom_chat_completions(
            &mut self,
            packet: &ClientboundCustomChatCompletionsPacket,
        ) {
            self.seen = Some((packet.action(), packet.entries().len()));
        }
    }

    #[test]
    fn round_trip_all_actions() {
        for action in [Action::Add, Action::Remove, Action::Set] {
            let pkt = ClientboundCustomChatCompletionsPacket::new(
                action,
                vec!["foo".to_string(), "bar".to_string()],
            );
            let mut buf = FriendlyByteBuf::new();
            pkt.write(&mut buf);
            let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
            let decoded = ClientboundCustomChatCompletionsPacket::read(&mut r);
            assert_eq!(pkt, decoded);
        }
    }

    #[test]
    fn round_trip_empty_entries() {
        let pkt = ClientboundCustomChatCompletionsPacket::new(Action::Set, vec![]);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
        let decoded = ClientboundCustomChatCompletionsPacket::read(&mut r);
        assert_eq!(decoded.entries().len(), 0);
        assert_eq!(decoded.action(), Action::Set);
    }

    #[test]
    fn round_trip_unicode_and_empty_string() {
        let pkt = ClientboundCustomChatCompletionsPacket::new(
            Action::Add,
            vec!["".to_string(), "héllo".to_string(), "日本語".to_string()],
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
        let decoded = ClientboundCustomChatCompletionsPacket::read(&mut r);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ClientboundCustomChatCompletionsPacket::new(Action::Remove, vec!["x".into()]);
        let mut listener = TestListener { seen: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen, Some((Action::Remove, 1)));
        assert_eq!(
            pkt.packet_type(),
            PacketType("minecraft:custom_chat_completions")
        );
    }

    #[test]
    fn varint_negative_and_extreme_round_trip() {
        let mut buf = FriendlyByteBuf::new();
        for v in [i32::MIN, -1, 0, i32::MAX] {
            buf.write_var_int(v);
        }
        let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
        for expected in [i32::MIN, -1, 0, i32::MAX] {
            assert_eq!(r.read_var_int(), expected);
        }
        assert_eq!(r.remaining(), 0);
    }
}
