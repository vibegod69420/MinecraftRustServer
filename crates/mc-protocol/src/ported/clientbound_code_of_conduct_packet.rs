use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    UnexpectedEof,
    InvalidVarInt,
    InvalidUtf8,
    InvalidLength(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_CODE_OF_CONDUCT: PacketType =
    PacketType("configuration/clientbound_code_of_conduct");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCodeOfConductPacket {
    code_of_conduct: String,
}

pub trait ClientConfigurationPacketListener {
    fn handle_code_of_conduct(&mut self, packet: &ClientboundCodeOfConductPacket);
}

impl ClientboundCodeOfConductPacket {
    // JAVA: net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket#<init>
    pub fn new(code_of_conduct: String) -> Self {
        Self { code_of_conduct }
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket#codeOfConduct
    pub fn code_of_conduct(&self) -> &str {
        &self.code_of_conduct
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket#type
    pub fn type_(&self) -> PacketType {
        CLIENTBOUND_CODE_OF_CONDUCT
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket#handle
    pub fn handle<L: ClientConfigurationPacketListener>(&self, listener: &mut L) {
        listener.handle_code_of_conduct(self);
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket#STREAM_CODEC
    pub fn encode(&self, buf: &mut BytesMut) {
        write_utf(buf, &self.code_of_conduct);
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket#STREAM_CODEC
    pub fn decode(buf: &mut &[u8]) -> Result<Self, CodecError> {
        let code_of_conduct = read_utf(buf)?;
        Ok(Self { code_of_conduct })
    }
}

fn write_varint(buf: &mut BytesMut, mut value: i32) {
    loop {
        let mut b = ((value as u32) & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            b |= 0x80;
        }
        buf.put_u8(b);
        if value == 0 {
            break;
        }
    }
}

fn read_varint(buf: &mut &[u8]) -> Result<i32, CodecError> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(CodecError::UnexpectedEof);
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32).wrapping_shl(shift);
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return Err(CodecError::InvalidVarInt);
        }
    }
}

fn write_utf(buf: &mut BytesMut, s: &str) {
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

fn read_utf(buf: &mut &[u8]) -> Result<String, CodecError> {
    let len = read_varint(buf)?;
    if len < 0 {
        return Err(CodecError::InvalidLength(len));
    }
    let len = len as usize;
    if buf.remaining() < len {
        return Err(CodecError::UnexpectedEof);
    }
    let bytes = &buf[..len];
    buf.advance(len);
    String::from_utf8(bytes.to_vec()).map_err(|_| CodecError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        received: Option<String>,
    }

    impl ClientConfigurationPacketListener for MockListener {
        fn handle_code_of_conduct(&mut self, packet: &ClientboundCodeOfConductPacket) {
            self.received = Some(packet.code_of_conduct().to_owned());
        }
    }

    #[test]
    fn type_is_code_of_conduct() {
        let pkt = ClientboundCodeOfConductPacket::new("rules".to_string());
        assert_eq!(pkt.type_(), CLIENTBOUND_CODE_OF_CONDUCT);
        assert_eq!(pkt.type_().0, "configuration/clientbound_code_of_conduct");
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let pkt = ClientboundCodeOfConductPacket::new("be nice".to_string());
        let mut listener = MockListener { received: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.received.as_deref(), Some("be nice"));
    }

    #[test]
    fn encode_decode_roundtrip() {
        let pkt = ClientboundCodeOfConductPacket::new("code of conduct".to_string());
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundCodeOfConductPacket::decode(&mut slice).unwrap();
        assert_eq!(decoded, pkt);
        assert!(slice.is_empty());
    }

    #[test]
    fn encode_decode_empty_string() {
        let pkt = ClientboundCodeOfConductPacket::new(String::new());
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundCodeOfConductPacket::decode(&mut slice).unwrap();
        assert_eq!(decoded.code_of_conduct(), "");
    }

    #[test]
    fn encode_decode_unicode() {
        let pkt = ClientboundCodeOfConductPacket::new("règles 🌍\nline2".to_string());
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundCodeOfConductPacket::decode(&mut slice).unwrap();
        assert_eq!(decoded, pkt);
    }

    #[test]
    fn decode_negative_length_errors() {
        let data = [0xff, 0xff, 0xff, 0xff, 0x0f];
        let mut slice: &[u8] = &data;
        let err = ClientboundCodeOfConductPacket::decode(&mut slice).unwrap_err();
        assert_eq!(err, CodecError::InvalidLength(-1));
    }
}
