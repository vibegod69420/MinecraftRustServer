use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    buf: BytesMut,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            buf: BytesMut::from(bytes),
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#asByteBuf
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, value: i32) {
        let mut v = value as u32;
        loop {
            if (v & !0x7F_u32) == 0 {
                self.buf.put_u8(v as u8);
                return;
            }
            self.buf.put_u8(((v & 0x7F_u32) as u8) | 0x80);
            v >>= 7;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.buf.get_u8();
            result |= ((b & 0x7F) as i32).wrapping_shl(shift);
            if (b & 0x80) == 0 {
                return result;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                panic!("VarInt too big");
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByteArray
    pub fn write_byte_array(&mut self, bytes: &[u8]) {
        self.write_var_int(bytes.len() as i32);
        self.buf.put_slice(bytes);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByteArray
    pub fn read_byte_array(&mut self) -> Vec<u8> {
        let len = self.read_var_int() as usize;
        let bytes = self.buf.split_to(len);
        bytes.to_vec()
    }
}

/// Minimal self-contained stub for `RemoteChatSession.Data`.
/// The real type serializes session UUID and profile public key data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteChatSessionData {
    pub payload: Vec<u8>,
}

impl RemoteChatSessionData {
    // JAVA: net.minecraft.network.chat.RemoteChatSession$Data#read
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let payload = buf.read_byte_array();
        Self { payload }
    }

    // JAVA: net.minecraft.network.chat.RemoteChatSession$Data#write
    pub fn write(buf: &mut FriendlyByteBuf, data: &Self) {
        buf.write_byte_array(&data.payload);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const SERVERBOUND_CHAT_SESSION_UPDATE: PacketType =
    PacketType("minecraft:serverbound_chat_session_update");

pub trait ServerGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ServerGamePacketListener#handleChatSessionUpdate
    fn handle_chat_session_update(&mut self, packet: &ServerboundChatSessionUpdatePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatSessionUpdatePacket {
    pub chat_session: RemoteChatSessionData,
}

impl ServerboundChatSessionUpdatePacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundChatSessionUpdatePacket#<init>
    pub fn new(chat_session: RemoteChatSessionData) -> Self {
        Self { chat_session }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatSessionUpdatePacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        Self::new(RemoteChatSessionData::read(buf))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatSessionUpdatePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        RemoteChatSessionData::write(buf, &self.chat_session);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatSessionUpdatePacket#type
    pub fn packet_type(&self) -> PacketType {
        SERVERBOUND_CHAT_SESSION_UPDATE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatSessionUpdatePacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_chat_session_update(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatSessionUpdatePacket#chatSession
    pub fn chat_session(&self) -> &RemoteChatSessionData {
        &self.chat_session
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_chat_session() {
        let data = RemoteChatSessionData {
            payload: vec![1, 2, 3, 4],
        };
        let pkt = ServerboundChatSessionUpdatePacket::new(data.clone());
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ServerboundChatSessionUpdatePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.chat_session(), &data);
        assert_eq!(decoded.packet_type(), SERVERBOUND_CHAT_SESSION_UPDATE);
    }

    #[test]
    fn round_trip_empty_payload() {
        let data = RemoteChatSessionData { payload: vec![] };
        let pkt = ServerboundChatSessionUpdatePacket::new(data.clone());
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ServerboundChatSessionUpdatePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.chat_session().payload, Vec::<u8>::new());
    }

    #[test]
    fn round_trip_large_payload() {
        let payload: Vec<u8> = (0..512).map(|i| (i & 0xFF) as u8).collect();
        let data = RemoteChatSessionData {
            payload: payload.clone(),
        };
        let pkt = ServerboundChatSessionUpdatePacket::new(data);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ServerboundChatSessionUpdatePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.chat_session().payload, payload);
    }

    #[test]
    fn handle_calls_listener() {
        struct L {
            seen: bool,
        }
        impl ServerGamePacketListener for L {
            fn handle_chat_session_update(
                &mut self,
                _packet: &ServerboundChatSessionUpdatePacket,
            ) {
                self.seen = true;
            }
        }
        let pkt = ServerboundChatSessionUpdatePacket::new(RemoteChatSessionData {
            payload: vec![9],
        });
        let mut l = L { seen: false };
        pkt.handle(&mut l);
        assert!(l.seen);
    }
}
