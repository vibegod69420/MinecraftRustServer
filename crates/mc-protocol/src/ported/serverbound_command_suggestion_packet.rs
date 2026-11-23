use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCommandSuggestionPacket {
    id: i32,
    command: String,
}

pub trait ServerGamePacketListener {
    fn handle_custom_command_suggestions(&mut self, packet: &ServerboundCommandSuggestionPacket);
}

impl ServerboundCommandSuggestionPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundCommandSuggestionPacket#<init>(int,String)
    pub fn new(id: i32, command: String) -> Self {
        Self { id, command }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundCommandSuggestionPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let id = read_varint(buf);
        let command = read_utf(buf, 32500);
        Self { id, command }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundCommandSuggestionPacket#write(FriendlyByteBuf)
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_varint(buf, self.id);
        write_utf(buf, &self.command, 32500);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundCommandSuggestionPacket#type()
    pub fn type_name(&self) -> &'static str {
        "SERVERBOUND_COMMAND_SUGGESTION"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundCommandSuggestionPacket#handle(ServerGamePacketListener)
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_custom_command_suggestions(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundCommandSuggestionPacket#getId()
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundCommandSuggestionPacket#getCommand()
    pub fn get_command(&self) -> &str {
        &self.command
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
fn read_varint<B: Buf>(buf: &mut B) -> i32 {
    let mut i: i32 = 0;
    let mut j: u32 = 0;
    loop {
        let b0 = buf.get_u8();
        i |= ((b0 & 0x7F) as i32).wrapping_shl(j.wrapping_mul(7) & 31);
        j = j.wrapping_add(1);
        if j > 5 {
            panic!("VarInt too big");
        }
        if (b0 & 0x80) == 0 {
            break;
        }
    }
    i
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
fn write_varint<B: BufMut>(buf: &mut B, mut val: i32) {
    loop {
        if (val & !0x7F) == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8(((val & 0x7F) | 0x80) as u8);
        val = ((val as u32) >> 7) as i32;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readUtf(int)
fn read_utf<B: Buf>(buf: &mut B, max_length: usize) -> String {
    let max_bytes = max_length.saturating_mul(3);
    let j = read_varint(buf);
    if j < 0 {
        panic!("The received encoded string buffer length is less than zero! Weird string!");
    }
    let j = j as usize;
    if j > max_bytes {
        panic!("The received encoded string buffer length is longer than maximum allowed");
    }
    if buf.remaining() < j {
        panic!("Not enough bytes in buffer");
    }
    let mut bytes = vec![0u8; j];
    buf.copy_to_slice(&mut bytes);
    let s = String::from_utf8(bytes).expect("Invalid UTF-8");
    if s.encode_utf16().count() > max_length {
        panic!("The received string length is longer than maximum allowed");
    }
    s
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf(String,int)
fn write_utf<B: BufMut>(buf: &mut B, s: &str, max_length: usize) {
    let char_len = s.encode_utf16().count();
    if char_len > max_length {
        panic!(
            "String too big (was {} characters, max {})",
            char_len, max_length
        );
    }
    let bytes = s.as_bytes();
    let max_bytes = max_length.saturating_mul(3);
    if bytes.len() > max_bytes {
        panic!(
            "String too big (was {} bytes encoded, max {})",
            bytes.len(),
            max_bytes
        );
    }
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_basic() {
        let pkt = ServerboundCommandSuggestionPacket::new(7, "/gamemode c".to_string());
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundCommandSuggestionPacket::read(&mut buf.freeze());
        assert_eq!(decoded.get_id(), 7);
        assert_eq!(decoded.get_command(), "/gamemode c");
        assert_eq!(decoded.type_name(), "SERVERBOUND_COMMAND_SUGGESTION");
    }

    #[test]
    fn roundtrip_negative_id_and_empty_command() {
        let pkt = ServerboundCommandSuggestionPacket::new(-1, String::new());
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundCommandSuggestionPacket::read(&mut buf.freeze());
        assert_eq!(decoded.get_id(), -1);
        assert_eq!(decoded.get_command(), "");
    }

    #[test]
    fn varint_roundtrip_negative_and_large() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, -1);
        write_varint(&mut buf, i32::MIN);
        write_varint(&mut buf, i32::MAX);
        let mut r = buf.freeze();
        assert_eq!(read_varint(&mut r), -1);
        assert_eq!(read_varint(&mut r), i32::MIN);
        assert_eq!(read_varint(&mut r), i32::MAX);
    }

    #[test]
    fn handle_dispatches_listener() {
        struct L {
            id: i32,
            cmd: String,
        }
        impl ServerGamePacketListener for L {
            fn handle_custom_command_suggestions(
                &mut self,
                packet: &ServerboundCommandSuggestionPacket,
            ) {
                self.id = packet.get_id();
                self.cmd = packet.get_command().to_string();
            }
        }
        let pkt = ServerboundCommandSuggestionPacket::new(3, "/help".to_string());
        let mut l = L {
            id: 0,
            cmd: String::new(),
        };
        pkt.handle(&mut l);
        assert_eq!(l.id, 3);
        assert_eq!(l.cmd, "/help");
    }

    #[test]
    #[should_panic]
    fn write_utf_rejects_too_long() {
        let mut buf = BytesMut::new();
        let s = "a".repeat(32501);
        write_utf(&mut buf, &s, 32500);
    }
}
