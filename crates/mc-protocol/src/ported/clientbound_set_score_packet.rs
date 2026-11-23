use bytes::{Buf, BufMut};

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int<B: Buf>(buf: &mut B) -> i32 {
    let mut val: u32 = 0;
    let mut pos: u32 = 0;
    loop {
        let b = buf.get_u8();
        val |= ((b & 0x7F) as u32).wrapping_shl(pos.wrapping_mul(7));
        pos = pos.wrapping_add(1);
        if pos > 5 {
            panic!("VarInt too big");
        }
        if (b & 0x80) == 0 {
            break;
        }
    }
    val as i32
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value as u32 & !0x7F_u32) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value as u8) & 0x7F) | 0x80);
        value = ((value as u32) >> 7) as i32;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
fn read_utf<B: Buf>(buf: &mut B) -> String {
    let len = read_var_int(buf);
    if len < 0 {
        panic!("Bad string length");
    }
    let len = len as usize;
    let mut bytes = vec![0u8; len];
    buf.copy_to_slice(&mut bytes);
    String::from_utf8(bytes).expect("Invalid UTF-8")
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
fn write_utf<B: BufMut>(buf: &mut B, value: &str) {
    let bytes = value.as_bytes();
    write_var_int(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

fn read_optional_bytes<B: Buf>(buf: &mut B) -> Option<Vec<u8>> {
    let present = buf.get_u8() != 0;
    if present {
        let len = read_var_int(buf);
        if len < 0 {
            panic!("Bad optional byte length");
        }
        let mut bytes = vec![0u8; len as usize];
        buf.copy_to_slice(&mut bytes);
        Some(bytes)
    } else {
        None
    }
}

fn write_optional_bytes<B: BufMut>(buf: &mut B, value: Option<&[u8]>) {
    match value {
        Some(bytes) => {
            buf.put_u8(1);
            write_var_int(buf, bytes.len() as i32);
            buf.put_slice(bytes);
        }
        None => {
            buf.put_u8(0);
        }
    }
}

pub trait ClientGamePacketListener {
    fn handle_set_score(&mut self, packet: &ClientboundSetScorePacket);
}

/// Self-contained port. `display` and `number_format` are stored as opaque
/// encoded byte payloads because Component/NumberFormat codecs are not available
/// in this standalone module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetScorePacket {
    owner: String,
    objective_name: String,
    score: i32,
    display: Option<Vec<u8>>,
    number_format: Option<Vec<u8>>,
}

impl ClientboundSetScorePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetScorePacket#<init>
    pub fn new(
        owner: String,
        objective_name: String,
        score: i32,
        display: Option<Vec<u8>>,
        number_format: Option<Vec<u8>>,
    ) -> Self {
        Self {
            owner,
            objective_name,
            score,
            display,
            number_format,
        }
    }

    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let owner = read_utf(buf);
        let objective_name = read_utf(buf);
        let score = read_var_int(buf);
        let display = read_optional_bytes(buf);
        let number_format = read_optional_bytes(buf);
        Self {
            owner,
            objective_name,
            score,
            display,
            number_format,
        }
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_utf(buf, &self.owner);
        write_utf(buf, &self.objective_name);
        write_var_int(buf, self.score);
        write_optional_bytes(buf, self.display.as_deref());
        write_optional_bytes(buf, self.number_format.as_deref());
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetScorePacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:set_score"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetScorePacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_set_score(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetScorePacket#owner
    pub fn owner(&self) -> &str {
        &self.owner
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetScorePacket#objectiveName
    pub fn objective_name(&self) -> &str {
        &self.objective_name
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetScorePacket#score
    pub fn score(&self) -> i32 {
        self.score
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetScorePacket#display
    pub fn display(&self) -> Option<&[u8]> {
        self.display.as_deref()
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetScorePacket#numberFormat
    pub fn number_format(&self) -> Option<&[u8]> {
        self.number_format.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_no_optionals() {
        let packet = ClientboundSetScorePacket::new(
            "player".to_string(),
            "objective".to_string(),
            42,
            None,
            None,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSetScorePacket::read(&mut reader);
        assert_eq!(packet, decoded);
    }

    #[test]
    fn roundtrip_with_optionals() {
        let packet = ClientboundSetScorePacket::new(
            "p".to_string(),
            "o".to_string(),
            -1,
            Some(vec![1, 2, 3]),
            Some(vec![4, 5]),
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSetScorePacket::read(&mut reader);
        assert_eq!(packet, decoded);
    }

    #[test]
    fn negative_and_min_score_roundtrip() {
        let packet = ClientboundSetScorePacket::new(
            String::new(),
            String::new(),
            i32::MIN,
            None,
            None,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSetScorePacket::read(&mut reader);
        assert_eq!(decoded.score(), i32::MIN);
        assert_eq!(decoded.owner(), "");
        assert_eq!(decoded.objective_name(), "");
    }

    #[test]
    fn accessors() {
        let packet = ClientboundSetScorePacket::new(
            "owner".to_string(),
            "obj".to_string(),
            10,
            Some(vec![9]),
            None,
        );
        assert_eq!(packet.owner(), "owner");
        assert_eq!(packet.objective_name(), "obj");
        assert_eq!(packet.score(), 10);
        assert_eq!(packet.display(), Some([9u8].as_slice()));
        assert_eq!(packet.number_format(), None);
    }

    #[test]
    fn handle_calls_listener() {
        struct L {
            owner: String,
            score: i32,
        }
        impl ClientGamePacketListener for L {
            fn handle_set_score(&mut self, packet: &ClientboundSetScorePacket) {
                self.owner = packet.owner().to_string();
                self.score = packet.score();
            }
        }
        let packet = ClientboundSetScorePacket::new(
            "alice".to_string(),
            "obj".to_string(),
            7,
            None,
            None,
        );
        let mut l = L {
            owner: String::new(),
            score: 0,
        };
        packet.handle(&mut l);
        assert_eq!(l.owner, "alice");
        assert_eq!(l.score, 7);
    }
}
