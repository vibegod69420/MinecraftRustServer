use bytes::{Buf, BufMut};

fn read_var_int(buf: &mut impl Buf) -> i32 {
    let mut val: i32 = 0;
    for i in 0..5 {
        let b = buf.get_u8() as i32;
        val |= (b & 0x7F) << (i * 7);
        if (b & 0x80) == 0 {
            return val;
        }
    }
    val
}

fn write_var_int(buf: &mut impl BufMut, value: i32) {
    let mut v = value as u32;
    loop {
        if (v & !0x7F) == 0 {
            buf.put_u8(v as u8);
            return;
        }
        buf.put_u8(((v & 0x7F) | 0x80) as u8);
        v >>= 7;
    }
}

fn read_utf(buf: &mut impl Buf, max_len: usize) -> String {
    let byte_len = read_var_int(buf) as usize;
    let max_bytes = max_len.saturating_mul(3);
    assert!(byte_len <= max_bytes, "String too long (bytes)");
    let mut bytes = vec![0u8; byte_len];
    buf.copy_to_slice(&mut bytes);
    let s = String::from_utf8(bytes).expect("invalid utf8");
    assert!(s.chars().count() <= max_len, "String too long (chars)");
    s
}

fn write_utf(buf: &mut impl BufMut, s: &str, max_len: usize) {
    let char_count = s.chars().count();
    assert!(char_count <= max_len, "String too long (chars)");
    let bytes = s.as_bytes();
    assert!(bytes.len() <= max_len.saturating_mul(3), "String too long (bytes)");
    write_var_int(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignature {
    pub bytes: [u8; 256],
}

impl MessageSignature {
    // JAVA: net.minecraft.network.chat.MessageSignature#read
    pub fn read(buf: &mut impl Buf) -> Self {
        let mut bytes = [0u8; 256];
        buf.copy_to_slice(&mut bytes);
        Self { bytes }
    }

    // JAVA: net.minecraft.network.chat.MessageSignature#write
    pub fn write(buf: &mut impl BufMut, sig: &Self) {
        buf.put_slice(&sig.bytes);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenMessagesUpdate {
    pub offset: i32,
    pub acknowledged: [u8; 3],
}

impl LastSeenMessagesUpdate {
    // JAVA: net.minecraft.network.chat.LastSeenMessages$Update#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let offset = read_var_int(buf);
        let mut acknowledged = [0u8; 3];
        buf.copy_to_slice(&mut acknowledged);
        Self { offset, acknowledged }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Update#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.offset);
        buf.put_slice(&self.acknowledged);
    }
}

pub trait ServerGamePacketListener {
    fn handle_chat(&mut self, packet: &ServerboundChatPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatPacket {
    pub message: String,
    pub time_stamp: i64,
    pub salt: i64,
    pub signature: Option<MessageSignature>,
    pub last_seen_messages: LastSeenMessagesUpdate,
}

impl ServerboundChatPacket {
    pub const PACKET_TYPE: &'static str = "minecraft:chat";

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatPacket#<init>(String,Instant,long,MessageSignature,LastSeenMessages.Update)
    pub fn new(
        message: String,
        time_stamp: i64,
        salt: i64,
        signature: Option<MessageSignature>,
        last_seen_messages: LastSeenMessagesUpdate,
    ) -> Self {
        Self {
            message,
            time_stamp,
            salt,
            signature,
            last_seen_messages,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let message = read_utf(buf, 256);
        let time_stamp = buf.get_i64();
        let salt = buf.get_i64();
        let signature = if buf.get_u8() != 0 {
            Some(MessageSignature::read(buf))
        } else {
            None
        };
        let last_seen_messages = LastSeenMessagesUpdate::read(buf);
        Self {
            message,
            time_stamp,
            salt,
            signature,
            last_seen_messages,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_utf(buf, &self.message, 256);
        buf.put_i64(self.time_stamp);
        buf.put_i64(self.salt);
        match &self.signature {
            Some(sig) => {
                buf.put_u8(1);
                MessageSignature::write(buf, sig);
            }
            None => {
                buf.put_u8(0);
            }
        }
        self.last_seen_messages.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatPacket#type
    pub fn packet_type(&self) -> &'static str {
        Self::PACKET_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatPacket#handle
    pub fn handle(&self, listener: &mut impl ServerGamePacketListener) {
        listener.handle_chat(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    fn sample_last_seen() -> LastSeenMessagesUpdate {
        LastSeenMessagesUpdate {
            offset: 5,
            acknowledged: [0xAA, 0xBB, 0x0F],
        }
    }

    #[test]
    fn roundtrip_without_signature() {
        let pkt = ServerboundChatPacket::new(
            "hello".to_string(),
            1_234_567_890i64,
            -42i64,
            None,
            sample_last_seen(),
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundChatPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_with_signature() {
        let pkt = ServerboundChatPacket::new(
            "signed".to_string(),
            -1i64,
            i64::MIN,
            Some(MessageSignature { bytes: [7u8; 256] }),
            LastSeenMessagesUpdate {
                offset: -3,
                acknowledged: [0, 0, 0],
            },
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundChatPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.signature.as_ref().unwrap().bytes[0], 7u8);
    }

    #[test]
    fn var_int_roundtrip() {
        let values = [0i32, 1, 127, 128, 255, -1, i32::MIN, i32::MAX];
        for v in values {
            let mut buf = BytesMut::new();
            write_var_int(&mut buf, v);
            let mut slice = &buf[..];
            let decoded = read_var_int(&mut slice);
            assert_eq!(v, decoded);
        }
    }

    #[test]
    fn utf_roundtrip_and_max() {
        let mut buf = BytesMut::new();
        let s = "a".repeat(256);
        write_utf(&mut buf, &s, 256);
        let mut slice = &buf[..];
        let decoded = read_utf(&mut slice, 256);
        assert_eq!(s, decoded);
    }

    #[test]
    fn packet_type_and_handle() {
        struct Listener {
            seen: Option<String>,
        }
        impl ServerGamePacketListener for Listener {
            fn handle_chat(&mut self, packet: &ServerboundChatPacket) {
                self.seen = Some(packet.message.clone());
            }
        }
        let pkt = ServerboundChatPacket::new(
            "test".to_string(),
            0,
            0,
            None,
            sample_last_seen(),
        );
        assert_eq!(pkt.packet_type(), "minecraft:chat");
        let mut l = Listener { seen: None };
        pkt.handle(&mut l);
        assert_eq!(l.seen.as_deref(), Some("test"));
    }
}
