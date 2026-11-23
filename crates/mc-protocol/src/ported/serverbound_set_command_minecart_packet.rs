#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, rpos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.data[self.rpos];
        self.rpos += 1;
        b
    }

    fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        let bytes = self.data[self.rpos..self.rpos + len].to_vec();
        self.rpos += len;
        bytes
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut val: i32 = 0;
        let mut pos: u32 = 0;
        loop {
            let b = self.read_byte() as i32;
            val |= (b & 0x7F) << pos;
            if (b & 0x80) == 0 {
                return val;
            }
            pos += 7;
            if pos >= 32 {
                panic!("VarInt too big");
            }
        }
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7Fu32) == 0 {
                self.data.push(val as u8);
                return;
            }
            self.data.push(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
    }

    pub fn read_utf(&mut self) -> String {
        self.read_utf_with_max(32767)
    }

    pub fn read_utf_with_max(&mut self, max_len: usize) -> String {
        let byte_len = self.read_var_int();
        if byte_len < 0 {
            panic!("The received encoded string buffer length is less than zero! Weird string!");
        }
        let byte_len = byte_len as usize;
        let max_bytes = max_len.saturating_mul(3);
        if byte_len > max_bytes {
            panic!("The received encoded string buffer length is longer than maximum allowed");
        }
        let bytes = self.read_bytes(byte_len);
        let s = String::from_utf8(bytes).expect("Invalid UTF-8 string");
        if s.chars().count() > max_len {
            panic!("The received string length is longer than maximum allowed");
        }
        s
    }

    pub fn write_utf(&mut self, s: &str) {
        let bytes = s.as_bytes();
        if bytes.len() > 32767 * 3 {
            panic!("String too big");
        }
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    pub fn read_boolean(&mut self) -> bool {
        self.read_byte() != 0
    }

    pub fn write_boolean(&mut self, value: bool) {
        self.data.push(if value { 1 } else { 0 });
    }
}

impl Default for FriendlyByteBuf {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCommandMinecartPacket {
    entity: i32,
    command: String,
    track_output: bool,
}

impl ServerboundSetCommandMinecartPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandMinecartPacket#<init>
    pub fn new(entity: i32, command: String, track_output: bool) -> Self {
        Self {
            entity,
            command,
            track_output,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandMinecartPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let entity = buf.read_var_int();
        let command = buf.read_utf();
        let track_output = buf.read_boolean();
        Self {
            entity,
            command,
            track_output,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandMinecartPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity);
        buf.write_utf(&self.command);
        buf.write_boolean(self.track_output);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandMinecartPacket#getCommand
    pub fn get_command(&self) -> &str {
        &self.command
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandMinecartPacket#isTrackOutput
    pub fn is_track_output(&self) -> bool {
        self.track_output
    }

    pub fn entity(&self) -> i32 {
        self.entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_basic() {
        let pkt = ServerboundSetCommandMinecartPacket::new(42, "say hello".to_string(), true);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ServerboundSetCommandMinecartPacket::read(&mut r);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_command(), "say hello");
        assert!(decoded.is_track_output());
        assert_eq!(decoded.entity(), 42);
    }

    #[test]
    fn round_trip_negative_entity_and_empty_command() {
        let pkt = ServerboundSetCommandMinecartPacket::new(-12345, String::new(), false);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ServerboundSetCommandMinecartPacket::read(&mut r);
        assert_eq!(decoded.entity(), -12345);
        assert_eq!(decoded.get_command(), "");
        assert!(!decoded.is_track_output());
    }

    #[test]
    fn varint_encoding_negative_one() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        assert_eq!(buf.as_bytes(), &[0xff, 0xff, 0xff, 0xff, 0x0f]);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        assert_eq!(r.read_var_int(), -1);
    }

    #[test]
    fn varint_encoding_edge_values() {
        let values = [0i32, 1, 127, 128, 255, i32::MAX, i32::MIN, -2];
        for &v in &values {
            let mut buf = FriendlyByteBuf::new();
            buf.write_var_int(v);
            let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
            assert_eq!(r.read_var_int(), v, "varint round-trip failed for {v}");
        }
    }

    #[test]
    fn utf_round_trip_unicode() {
        let s = "héllo 🌍 /say test";
        let mut buf = FriendlyByteBuf::new();
        buf.write_utf(s);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        assert_eq!(r.read_utf(), s);
    }
}
