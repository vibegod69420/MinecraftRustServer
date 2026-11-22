use std::io::{self, Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.split_once(':') {
            Some((namespace, path)) => Self::new(namespace, path),
            None => Self::new("minecraft", value),
        }
    }

    pub fn to_string_value(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

pub trait ClientGamePacketListener {
    fn handle_item_cooldown(&mut self, packet: &ClientboundCooldownPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCooldownPacket {
    cooldown_group: Identifier,
    duration: i32,
}

fn write_var_int<W: Write>(writer: &mut W, value: i32) -> io::Result<()> {
    let mut val = value as u32;
    loop {
        let mut b = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            b |= 0x80;
        }
        writer.write_all(&[b])?;
        if val == 0 {
            return Ok(());
        }
    }
}

fn read_var_int<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut val: i32 = 0;
    let mut pos: u32 = 0;
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let b = buf[0];
        val |= ((b & 0x7F) as i32) << pos;
        if (b & 0x80) == 0 {
            return Ok(val);
        }
        pos = pos.wrapping_add(7);
        if pos >= 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt too big",
            ));
        }
    }
}

fn write_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    let bytes = value.as_bytes();
    let len = i32::try_from(bytes.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "string too large"))?;
    write_var_int(writer, len)?;
    writer.write_all(bytes)
}

fn read_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let len = read_var_int(reader)?;
    if len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative string length",
        ));
    }
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

impl ClientboundCooldownPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCooldownPacket#<init>
    pub fn new(cooldown_group: Identifier, duration: i32) -> Self {
        Self {
            cooldown_group,
            duration,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCooldownPacket#cooldownGroup
    pub fn cooldown_group(&self) -> &Identifier {
        &self.cooldown_group
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCooldownPacket#duration
    pub fn duration(&self) -> i32 {
        self.duration
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCooldownPacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:cooldown")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCooldownPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_item_cooldown(self);
    }

    /// Encodes Identifier as UTF string and duration as VarInt.
    pub fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.cooldown_group.to_string_value())?;
        write_var_int(writer, self.duration)?;
        Ok(())
    }

    /// Decodes Identifier from UTF string and duration from VarInt.
    pub fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let value = read_string(reader)?;
        let cooldown_group = Identifier::parse(&value);
        let duration = read_var_int(reader)?;
        Ok(Self {
            cooldown_group,
            duration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[derive(Default)]
    struct MockListener {
        group: Option<Identifier>,
        duration: i32,
        called: bool,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_item_cooldown(&mut self, packet: &ClientboundCooldownPacket) {
            self.group = Some(packet.cooldown_group().clone());
            self.duration = packet.duration();
            self.called = true;
        }
    }

    #[test]
    fn accessors_and_type() {
        let id = Identifier::new("minecraft", "ender_pearl");
        let packet = ClientboundCooldownPacket::new(id.clone(), 20);
        assert_eq!(packet.cooldown_group(), &id);
        assert_eq!(packet.duration(), 20);
        assert_eq!(packet.get_type(), PacketType("minecraft:cooldown"));
    }

    #[test]
    fn handle_dispatches() {
        let packet =
            ClientboundCooldownPacket::new(Identifier::new("minecraft", "shield"), 100);
        let mut listener = MockListener::default();
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(
            listener.group.unwrap().to_string_value(),
            "minecraft:shield"
        );
        assert_eq!(listener.duration, 100);
    }

    #[test]
    fn encode_decode_roundtrip_negative_duration() {
        let packet =
            ClientboundCooldownPacket::new(Identifier::new("custom", "ability"), -1);
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        let decoded = ClientboundCooldownPacket::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn encode_decode_extreme_durations() {
        for duration in [i32::MIN, 0, i32::MAX] {
            let packet =
                ClientboundCooldownPacket::new(Identifier::new("minecraft", "test"), duration);
            let mut buf = Vec::new();
            packet.encode(&mut buf).unwrap();
            let decoded = ClientboundCooldownPacket::decode(&mut Cursor::new(&buf)).unwrap();
            assert_eq!(decoded.duration(), duration);
            assert_eq!(
                decoded.cooldown_group().to_string_value(),
                "minecraft:test"
            );
        }
    }

    #[test]
    fn identifier_parse_and_string_encoding_bits() {
        let parsed = Identifier::parse("ender_pearl");
        assert_eq!(parsed.namespace, "minecraft");
        assert_eq!(parsed.path, "ender_pearl");

        let packet = ClientboundCooldownPacket::new(Identifier::new("a", "b"), 1);
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        // "a:b" is 3 bytes, VarInt length 3 = 0x03
        assert_eq!(buf[0], 3);
        assert_eq!(&buf[1..4], b"a:b");
        assert_eq!(buf[4], 1); // VarInt 1
    }
}
