#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace(String)
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }

    // JAVA: net.minecraft.resources.Identifier#parse(String)
    pub fn parse(s: &str) -> Self {
        if let Some((ns, path)) = s.split_once(':') {
            Self {
                namespace: ns.to_string(),
                path: path.to_string(),
            }
        } else {
            Self {
                namespace: "minecraft".to_string(),
                path: s.to_string(),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType {
    pub flow: PacketFlow,
    pub id: Identifier,
}

impl PacketType {
    // JAVA: net.minecraft.network.protocol.PacketType#<init>(PacketFlow, Identifier)
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self { flow, id }
    }
}

#[derive(Debug, Clone)]
pub struct FriendlyByteBuf {
    buf: Vec<u8>,
    rpos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>()
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            rpos: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>(byte[])
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { buf: bytes, rpos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readableBytes()
    pub fn readable_bytes(&self) -> usize {
        self.buf.len().saturating_sub(self.rpos)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#skipBytes(int)
    pub fn skip_bytes(&mut self, len: usize) {
        self.rpos = self.rpos.saturating_add(len).min(self.buf.len());
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.buf[self.rpos];
        self.rpos += 1;
        b
    }

    fn write_byte(&mut self, b: u8) {
        self.buf.push(b);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
    pub fn read_var_int(&mut self) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b = self.read_byte() as i32;
            i |= (b & 127) << (j * 7);
            j += 1;
            if j > 5 {
                panic!("VarInt too big");
            }
            if (b & 128) != 128 {
                return i;
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F) == 0 {
                self.write_byte(value as u8);
                return;
            }
            self.write_byte(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf(int)
    pub fn read_utf(&mut self, max_len: usize) -> String {
        let len = self.read_var_int();
        if len < 0 {
            panic!("The received encoded string buffer length is less than zero! Weird string!");
        }
        let len = len as usize;
        let max_bytes = max_len.saturating_mul(3);
        if len > max_bytes {
            panic!("The received encoded string buffer length is longer than maximum allowed");
        }
        let bytes = self.buf[self.rpos..self.rpos + len].to_vec();
        self.rpos += len;
        let s = String::from_utf8(bytes).expect("Invalid UTF-8");
        if s.chars().count() > max_len {
            panic!("The received string length is longer than maximum allowed");
        }
        s
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf(String)
    pub fn write_utf(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.buf.extend_from_slice(bytes);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readIdentifier()
    pub fn read_identifier(&mut self) -> Identifier {
        let s = self.read_utf(32767);
        Identifier::parse(&s)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeIdentifier(Identifier)
    pub fn write_identifier(&mut self, id: &Identifier) {
        let s = format!("{}:{}", id.namespace, id.path);
        self.write_utf(&s);
    }
}

impl Default for FriendlyByteBuf {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscardedQueryPayload {
    pub id: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomQueryPayload {
    Discarded(DiscardedQueryPayload),
}

impl CustomQueryPayload {
    // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryPayload#id()
    pub fn id(&self) -> &Identifier {
        match self {
            CustomQueryPayload::Discarded(p) => &p.id,
        }
    }

    // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryPayload#write(FriendlyByteBuf)
    pub fn write(&self, _buf: &mut FriendlyByteBuf) {
        match self {
            CustomQueryPayload::Discarded(_) => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomQueryPacket {
    pub transaction_id: i32,
    pub payload: CustomQueryPayload,
}

impl ClientboundCustomQueryPacket {
    pub const MAX_PAYLOAD_SIZE: i32 = 1_048_576;

    // JAVA: net.minecraft.network.protocol.login.ClientboundCustomQueryPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let transaction_id = buf.read_var_int();
        let id = buf.read_identifier();
        let payload = Self::read_payload(id, buf);
        Self {
            transaction_id,
            payload,
        }
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundCustomQueryPacket#readPayload(Identifier, FriendlyByteBuf)
    fn read_payload(id: Identifier, buf: &mut FriendlyByteBuf) -> CustomQueryPayload {
        Self::read_unknown_payload(id, buf)
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundCustomQueryPacket#readUnknownPayload(Identifier, FriendlyByteBuf)
    fn read_unknown_payload(id: Identifier, buf: &mut FriendlyByteBuf) -> CustomQueryPayload {
        let len = buf.readable_bytes() as i32;
        if len >= 0 && len <= Self::MAX_PAYLOAD_SIZE {
            buf.skip_bytes(len as usize);
            CustomQueryPayload::Discarded(DiscardedQueryPayload { id })
        } else {
            panic!("Payload may not be larger than 1048576 bytes");
        }
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundCustomQueryPacket#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.transaction_id);
        let id = self.payload.id().clone();
        buf.write_identifier(&id);
        self.payload.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundCustomQueryPacket#type()
    pub fn get_type(&self) -> PacketType {
        PacketType::new(
            PacketFlow::Clientbound,
            Identifier::with_default_namespace("custom_query"),
        )
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundCustomQueryPacket#handle(ClientLoginPacketListener)
    pub fn handle<L: ClientLoginPacketListener>(&self, listener: &mut L) {
        listener.handle_custom_query(self);
    }
}

pub trait ClientLoginPacketListener {
    // JAVA: net.minecraft.network.protocol.login.ClientLoginPacketListener#handleCustomQuery(ClientboundCustomQueryPacket)
    fn handle_custom_query(&mut self, packet: &ClientboundCustomQueryPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_custom_query() {
        let packet = ClientboundCustomQueryPacket {
            transaction_id: 42,
            payload: CustomQueryPayload::Discarded(DiscardedQueryPayload {
                id: Identifier::with_default_namespace("test"),
            }),
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundCustomQueryPacket::read(&mut read_buf);
        assert_eq!(decoded.transaction_id, 42);
        assert_eq!(
            decoded.payload.id(),
            &Identifier::with_default_namespace("test")
        );
        assert_eq!(read_buf.readable_bytes(), 0);
    }

    #[test]
    fn negative_transaction_id_round_trip() {
        let packet = ClientboundCustomQueryPacket {
            transaction_id: -1,
            payload: CustomQueryPayload::Discarded(DiscardedQueryPayload {
                id: Identifier::parse("custom:payload"),
            }),
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundCustomQueryPacket::read(&mut read_buf);
        assert_eq!(decoded.transaction_id, -1);
        assert_eq!(decoded.payload.id().namespace, "custom");
        assert_eq!(decoded.payload.id().path, "payload");
    }

    #[test]
    fn read_unknown_payload_skips_bytes() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(1);
        buf.write_identifier(&Identifier::with_default_namespace("test"));
        buf.buf.extend_from_slice(&[1, 2, 3, 4, 5]);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundCustomQueryPacket::read(&mut read_buf);
        assert_eq!(decoded.transaction_id, 1);
        assert_eq!(read_buf.readable_bytes(), 0);
    }

    #[test]
    fn max_payload_size_boundary() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(7);
        buf.write_identifier(&Identifier::with_default_namespace("big"));
        buf.buf
            .extend(std::iter::repeat_n(0u8, 1_048_576));
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundCustomQueryPacket::read(&mut read_buf);
        assert_eq!(decoded.transaction_id, 7);
        assert_eq!(read_buf.readable_bytes(), 0);
    }

    #[test]
    #[should_panic(expected = "Payload may not be larger than 1048576 bytes")]
    fn oversized_payload_panics() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(1);
        buf.write_identifier(&Identifier::with_default_namespace("big"));
        buf.buf
            .extend(std::iter::repeat_n(0u8, 1_048_577));
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let _ = ClientboundCustomQueryPacket::read(&mut read_buf);
    }

    #[test]
    fn packet_type_is_clientbound_custom_query() {
        let p = ClientboundCustomQueryPacket {
            transaction_id: 0,
            payload: CustomQueryPayload::Discarded(DiscardedQueryPayload {
                id: Identifier::with_default_namespace("x"),
            }),
        };
        let t = p.get_type();
        assert_eq!(t.flow, PacketFlow::Clientbound);
        assert_eq!(
            t.id,
            Identifier::with_default_namespace("custom_query")
        );
    }
}
