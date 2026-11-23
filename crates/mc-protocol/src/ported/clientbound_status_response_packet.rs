use std::io;

/// Minimal local representation of Minecraft's `ServerStatus`.
///
/// In Java this is a record with a Mojang codec; for a self-contained 1:1 packet
/// port we preserve the serialized JSON payload as a string, which is the wire
/// representation produced/consumed by the packet's stream codec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerStatus {
    pub json: String,
}

impl ServerStatus {
    pub fn new(json: String) -> Self {
        Self { json }
    }
}

/// Minimal local packet type marker, mirroring
/// `net.minecraft.network.protocol.PacketType<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

/// Local mirror of `net.minecraft.network.protocol.status.StatusPacketTypes`.
pub struct StatusPacketTypes;

impl StatusPacketTypes {
    pub const CLIENTBOUND_STATUS_RESPONSE: PacketType = PacketType {
        id: "status/clientbound_status_response",
    };
}

/// Minimal local listener trait mirroring
/// `net.minecraft.network.protocol.status.ClientStatusPacketListener`.
pub trait ClientStatusPacketListener {
    fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket);
}

/// Minimal local packet trait mirroring
/// `net.minecraft.network.protocol.Packet<T>`.
pub trait Packet<T> {
    fn type_(&self) -> PacketType;
    fn handle(&self, listener: &mut T);
}

/// Port of `net.minecraft.network.protocol.status.ClientboundStatusResponsePacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundStatusResponsePacket {
    status: ServerStatus,
}

impl ClientboundStatusResponsePacket {
    pub const MAX_STATUS_JSON_LENGTH: usize = 32767;

    // JAVA: net.minecraft.network.protocol.status.ClientboundStatusResponsePacket#<init>
    pub fn new(status: ServerStatus) -> Self {
        Self { status }
    }

    // JAVA: net.minecraft.network.protocol.status.ClientboundStatusResponsePacket#status
    pub fn status(&self) -> &ServerStatus {
        &self.status
    }

    /// Encodes the packet using the same wire shape as Java's
    /// `ByteBufCodecs.lenientJson(32767)` followed by codec mapping:
    /// a VarInt-prefixed UTF-8 string with max length 32767.
    pub fn write(&self, buf: &mut impl std::io::Write) -> io::Result<()> {
        write_utf(&mut *buf, &self.status.json, Self::MAX_STATUS_JSON_LENGTH)
    }

    /// Decodes the packet using the same wire shape as Java's
    /// `ByteBufCodecs.lenientJson(32767)`.
    pub fn read(buf: &mut impl std::io::Read) -> io::Result<Self> {
        let json = read_utf(&mut *buf, Self::MAX_STATUS_JSON_LENGTH)?;
        Ok(Self::new(ServerStatus::new(json)))
    }
}

impl Packet<ClientStatusPacketListenerDyn<'_>> for ClientboundStatusResponsePacket {
    // JAVA: net.minecraft.network.protocol.status.ClientboundStatusResponsePacket#type
    fn type_(&self) -> PacketType {
        StatusPacketTypes::CLIENTBOUND_STATUS_RESPONSE
    }

    // JAVA: net.minecraft.network.protocol.status.ClientboundStatusResponsePacket#handle
    fn handle(&self, listener: &mut ClientStatusPacketListenerDyn<'_>) {
        listener.handle_status_response(self);
    }
}

/// Object-safe adapter for listener dispatch in tests/callers that prefer
/// dynamic dispatch. The Java method takes `ClientStatusPacketListener`.
pub struct ClientStatusPacketListenerDyn<'a> {
    inner: &'a mut dyn ClientStatusPacketListener,
}

impl<'a> ClientStatusPacketListenerDyn<'a> {
    pub fn new(inner: &'a mut dyn ClientStatusPacketListener) -> Self {
        Self { inner }
    }

    pub fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket) {
        self.inner.handle_status_response(packet);
    }
}

/// Writes a Minecraft VarInt (Java-compatible, zigzag not used here).
fn write_varint(mut writer: impl std::io::Write, mut value: i32) -> io::Result<()> {
    loop {
        let mut temp = (value as u32 & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            temp |= 0x80;
        }
        writer.write_all(&[temp])?;
        if value == 0 {
            return Ok(());
        }
    }
}

/// Reads a Minecraft VarInt (Java-compatible).
fn read_varint(mut reader: impl std::io::Read) -> io::Result<i32> {
    let mut value: i32 = 0;
    let mut position: u32 = 0;
    loop {
        let mut byte_buf = [0u8; 1];
        reader.read_exact(&mut byte_buf)?;
        let byte = byte_buf[0];
        value |= ((byte & 0x7F) as i32) << position;

        if (byte & 0x80) == 0 {
            return Ok(value);
        }

        position = position.wrapping_add(7);
        if position >= 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt is too big",
            ));
        }
    }
}

/// Writes a UTF string in Minecraft wire format: VarInt byte length + UTF-8 bytes.
/// `max_chars` matches Java's `Utf8String`/`ByteBufCodecs.string` character limit.
fn write_utf(
    mut writer: impl std::io::Write,
    value: &str,
    max_chars: usize,
) -> io::Result<()> {
    let char_count = value.chars().count();
    if char_count > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "String too long: {} characters, max {}",
                char_count, max_chars
            ),
        ));
    }
    let bytes = value.as_bytes();
    let byte_len = i32::try_from(bytes.len()).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "String byte length exceeds i32")
    })?;
    write_varint(&mut writer, byte_len)?;
    writer.write_all(bytes)?;
    Ok(())
}

/// Reads a UTF string in Minecraft wire format with Java-style validation.
fn read_utf(mut reader: impl std::io::Read, max_chars: usize) -> io::Result<String> {
    let max_bytes = max_chars.saturating_mul(3);
    let byte_len = read_varint(&mut reader)?;
    if byte_len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "The received encoded string buffer length is less than zero! Weird string!",
        ));
    }
    let byte_len_usize = byte_len as usize;
    if byte_len_usize > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "The received encoded string buffer length is longer than maximum allowed ({} > {})",
                byte_len_usize, max_bytes
            ),
        ));
    }

    let mut bytes = vec![0u8; byte_len_usize];
    reader.read_exact(&mut bytes)?;
    let s = String::from_utf8(bytes).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e))
    })?;

    let char_count = s.chars().count();
    if char_count > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "The received string length is longer than maximum allowed ({} > {})",
                char_count, max_chars
            ),
        ));
    }

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn packet_type_and_status_access() {
        let status = ServerStatus::new(r#"{"description":"hi"}"#.to_string());
        let packet = ClientboundStatusResponsePacket::new(status.clone());
        assert_eq!(packet.status(), &status);
        assert_eq!(
            Packet::<ClientStatusPacketListenerDyn<'_>>::type_(&packet),
            StatusPacketTypes::CLIENTBOUND_STATUS_RESPONSE
        );
        assert_eq!(
            StatusPacketTypes::CLIENTBOUND_STATUS_RESPONSE.id,
            "status/clientbound_status_response"
        );
    }

    #[test]
    fn write_read_round_trip() {
        let json = r#"{"version":{"name":"1.21.11","protocol":0},"players":{"max":20,"online":1},"description":{"text":"hello"}}"#.to_string();
        let packet = ClientboundStatusResponsePacket::new(ServerStatus::new(json.clone()));

        let mut buf = Vec::new();
        packet.write(&mut buf).expect("write should succeed");

        let mut cursor = Cursor::new(buf);
        let decoded = ClientboundStatusResponsePacket::read(&mut cursor).expect("read should succeed");
        assert_eq!(decoded.status().json, json);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct TestListener {
            seen: Option<String>,
        }
        impl ClientStatusPacketListener for TestListener {
            fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket) {
                self.seen = Some(packet.status().json.clone());
            }
        }

        let packet = ClientboundStatusResponsePacket::new(ServerStatus::new("{}".to_string()));
        let mut listener = TestListener { seen: None };
        let mut dyn_listener = ClientStatusPacketListenerDyn::new(&mut listener);
        Packet::handle(&packet, &mut dyn_listener);
        assert_eq!(listener.seen.as_deref(), Some("{}"));
    }

    #[test]
    fn rejects_too_long_string_on_write() {
        let long = "a".repeat(ClientboundStatusResponsePacket::MAX_STATUS_JSON_LENGTH + 1);
        let packet = ClientboundStatusResponsePacket::new(ServerStatus::new(long));
        let mut buf = Vec::new();
        let err = packet.write(&mut buf).expect_err("should reject long string");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_negative_varint_length_on_read() {
        // VarInt -1 encodes as 0xff 0xff 0xff 0xff 0x0f in Java VarInt format.
        let data = [0xff, 0xff, 0xff, 0xff, 0x0f];
        let mut cursor = Cursor::new(&data[..]);
        let err = ClientboundStatusResponsePacket::read(&mut cursor)
            .expect_err("negative length should fail");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn varint_round_trip_edge_values() {
        fn round_trip(v: i32) -> i32 {
            let mut buf = Vec::new();
            write_varint(&mut buf, v).unwrap();
            let mut cursor = Cursor::new(buf);
            read_varint(&mut cursor).unwrap()
        }

        assert_eq!(round_trip(0), 0);
        assert_eq!(round_trip(1), 1);
        assert_eq!(round_trip(127), 127);
        assert_eq!(round_trip(128), 128);
        assert_eq!(round_trip(255), 255);
        assert_eq!(round_trip(i32::MAX), i32::MAX);
        assert_eq!(round_trip(-1), -1);
        assert_eq!(round_trip(i32::MIN), i32::MIN);
    }
}
