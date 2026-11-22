use std::io::{self, Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ClientGamePacketListener {
    fn handle_set_time(&mut self, packet: &ClientboundSetTimePacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetTimePacket {
    game_time: i64,
    day_time: i64,
    tick_day_time: bool,
}

impl ClientboundSetTimePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTimePacket#<init>
    pub fn new(game_time: i64, day_time: i64, tick_day_time: bool) -> Self {
        Self {
            game_time,
            day_time,
            tick_day_time,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTimePacket#gameTime
    pub fn game_time(&self) -> i64 {
        self.game_time
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTimePacket#dayTime
    pub fn day_time(&self) -> i64 {
        self.day_time
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTimePacket#tickDayTime
    pub fn tick_day_time(&self) -> bool {
        self.tick_day_time
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTimePacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:set_time")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTimePacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_set_time(self);
    }

    /// Encodes gameTime (i64 BE), dayTime (i64 BE), tickDayTime (bool byte).
    pub fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.game_time.to_be_bytes())?;
        writer.write_all(&self.day_time.to_be_bytes())?;
        writer.write_all(&[if self.tick_day_time { 1 } else { 0 }])?;
        Ok(())
    }

    /// Decodes gameTime (i64 BE), dayTime (i64 BE), tickDayTime (nonzero byte = true).
    pub fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf8 = [0u8; 8];
        reader.read_exact(&mut buf8)?;
        let game_time = i64::from_be_bytes(buf8);
        reader.read_exact(&mut buf8)?;
        let day_time = i64::from_be_bytes(buf8);
        let mut buf1 = [0u8; 1];
        reader.read_exact(&mut buf1)?;
        let tick_day_time = buf1[0] != 0;
        Ok(Self {
            game_time,
            day_time,
            tick_day_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[derive(Default)]
    struct MockListener {
        game_time: i64,
        day_time: i64,
        tick_day_time: bool,
        called: bool,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_set_time(&mut self, packet: &ClientboundSetTimePacket) {
            self.game_time = packet.game_time();
            self.day_time = packet.day_time();
            self.tick_day_time = packet.tick_day_time();
            self.called = true;
        }
    }

    #[test]
    fn accessors_and_type() {
        let packet = ClientboundSetTimePacket::new(100, -200, true);
        assert_eq!(packet.game_time(), 100);
        assert_eq!(packet.day_time(), -200);
        assert!(packet.tick_day_time());
        assert_eq!(packet.get_type(), PacketType("minecraft:set_time"));
    }

    #[test]
    fn handle_dispatches() {
        let packet = ClientboundSetTimePacket::new(1, 2, false);
        let mut listener = MockListener::default();
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.game_time, 1);
        assert_eq!(listener.day_time, 2);
        assert!(!listener.tick_day_time);
    }

    #[test]
    fn encode_decode_roundtrip_extremes() {
        let packet = ClientboundSetTimePacket::new(i64::MIN, i64::MAX, true);
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        let decoded = ClientboundSetTimePacket::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn encode_decode_negative_and_false() {
        let packet = ClientboundSetTimePacket::new(-1, i64::MIN, false);
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        let decoded = ClientboundSetTimePacket::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded.game_time(), -1);
        assert_eq!(decoded.day_time(), i64::MIN);
        assert!(!decoded.tick_day_time());
    }

    #[test]
    fn encode_bit_pattern_and_bool_tolerance() {
        let packet = ClientboundSetTimePacket::new(1, -2, true);
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        assert_eq!(&buf[0..8], &1i64.to_be_bytes());
        assert_eq!(&buf[8..16], &(-2i64).to_be_bytes());
        assert_eq!(buf[16], 1);

        let mut tolerant = Vec::new();
        tolerant.extend_from_slice(&0i64.to_be_bytes());
        tolerant.extend_from_slice(&0i64.to_be_bytes());
        tolerant.push(2);
        let decoded = ClientboundSetTimePacket::decode(&mut Cursor::new(&tolerant)).unwrap();
        assert!(decoded.tick_day_time());
    }
}
