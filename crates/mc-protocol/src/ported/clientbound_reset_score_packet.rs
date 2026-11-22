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

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
    pub fn write_utf(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.buf.put_slice(bytes);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
    pub fn read_utf(&mut self) -> String {
        let len = self.read_var_int() as usize;
        let bytes = self.buf.split_to(len);
        String::from_utf8(bytes.to_vec()).expect("invalid utf8")
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_bool(&mut self, value: bool) {
        self.buf.put_u8(if value { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
    pub fn read_bool(&mut self) -> bool {
        self.buf.get_u8() != 0
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeNullable
    pub fn write_nullable<T, F>(&mut self, value: Option<&T>, mut writer: F)
    where
        F: FnMut(&mut Self, &T),
    {
        match value {
            Some(v) => {
                self.write_bool(true);
                writer(self, v);
            }
            None => {
                self.write_bool(false);
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readNullable
    pub fn read_nullable<T, F>(&mut self, mut reader: F) -> Option<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        if self.read_bool() {
            Some(reader(self))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_RESET_SCORE: PacketType = PacketType("minecraft:clientbound_reset_score");

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleResetScore
    fn handle_reset_score(&mut self, packet: &ClientboundResetScorePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundResetScorePacket {
    pub owner: String,
    pub objective_name: Option<String>,
}

impl ClientboundResetScorePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundResetScorePacket#<init>
    pub fn new(owner: String, objective_name: Option<String>) -> Self {
        Self {
            owner,
            objective_name,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundResetScorePacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        let owner = buf.read_utf();
        let objective_name = buf.read_nullable(|b| b.read_utf());
        Self::new(owner, objective_name)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundResetScorePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.owner);
        buf.write_nullable(self.objective_name.as_ref(), |b, v| b.write_utf(v));
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundResetScorePacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_RESET_SCORE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundResetScorePacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_reset_score(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundResetScorePacket#owner
    pub fn owner(&self) -> &str {
        &self.owner
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundResetScorePacket#objectiveName
    pub fn objective_name(&self) -> Option<&str> {
        self.objective_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_with_objective() {
        let pkt = ClientboundResetScorePacket::new(
            "player".to_string(),
            Some("obj".to_string()),
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ClientboundResetScorePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.owner(), "player");
        assert_eq!(decoded.objective_name(), Some("obj"));
        assert_eq!(decoded.packet_type(), CLIENTBOUND_RESET_SCORE);
    }

    #[test]
    fn round_trip_null_objective() {
        let pkt = ClientboundResetScorePacket::new("owner".to_string(), None);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ClientboundResetScorePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.owner(), "owner");
        assert_eq!(decoded.objective_name(), None);
    }

    #[test]
    fn round_trip_empty_strings() {
        let pkt = ClientboundResetScorePacket::new(String::new(), Some(String::new()));
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ClientboundResetScorePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.owner(), "");
        assert_eq!(decoded.objective_name(), Some(""));
    }

    #[test]
    fn varint_negative_one_round_trip() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        assert_eq!(read_buf.read_var_int(), -1);
    }

    #[test]
    fn handle_calls_listener() {
        struct L {
            owner: String,
            obj: Option<String>,
        }
        impl ClientGamePacketListener for L {
            fn handle_reset_score(&mut self, packet: &ClientboundResetScorePacket) {
                self.owner = packet.owner().to_string();
                self.obj = packet.objective_name().map(|s| s.to_string());
            }
        }
        let pkt = ClientboundResetScorePacket::new("p".to_string(), None);
        let mut l = L {
            owner: String::new(),
            obj: Some("x".to_string()),
        };
        pkt.handle(&mut l);
        assert_eq!(l.owner, "p");
        assert_eq!(l.obj, None);
    }
}
