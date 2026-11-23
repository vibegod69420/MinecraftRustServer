/// Self-contained stub for net.minecraft.network.chat.Component.
/// Wire format is simplified for this self-contained module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    fn read_trusted_context_free(buf: &mut ByteBuf) -> Self {
        let text = buf.read_utf(262144);
        Self { text }
    }

    fn write_trusted_context_free(&self, buf: &mut ByteBuf) {
        buf.write_utf(&self.text, 262144);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ByteBuf {
    data: Vec<u8>,
    read_idx: usize,
}

impl ByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, read_idx: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.data[self.read_idx];
        self.read_idx += 1;
        b
    }

    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        let end = self.read_idx + len;
        let bytes = self.data[self.read_idx..end].to_vec();
        self.read_idx = end;
        bytes
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b = self.read_byte();
            i |= ((b & 127) as i32) << (j * 7);
            j += 1;
            if j > 5 {
                panic!("VarInt too big");
            }
            if (b & 128) == 0 {
                break;
            }
        }
        i
    }

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

    pub fn read_utf(&mut self, max_length: usize) -> String {
        let byte_len = self.read_var_int() as usize;
        let max_bytes = max_length.saturating_mul(3).saturating_add(3);
        if byte_len > max_bytes {
            panic!("String too long (bytes)");
        }
        let bytes = self.read_bytes(byte_len);
        let s = String::from_utf8(bytes).expect("invalid utf8");
        if s.chars().count() > max_length {
            panic!("String too long (chars)");
        }
        s
    }

    pub fn write_utf(&mut self, value: &str, max_length: usize) {
        if value.chars().count() > max_length {
            panic!("String too long");
        }
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.write_bytes(bytes);
    }

    pub fn read_long(&mut self) -> i64 {
        let bytes = self.read_bytes(8);
        let arr: [u8; 8] = bytes.try_into().expect("8 bytes");
        i64::from_be_bytes(arr)
    }

    pub fn write_long(&mut self, value: i64) {
        self.write_bytes(&value.to_be_bytes());
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_byte() != 0
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_byte(if value { 1 } else { 0 });
    }

    pub fn read_uuid(&mut self) -> Uuid {
        let most_significant_bits = self.read_long();
        let least_significant_bits = self.read_long();
        Uuid {
            most_significant_bits,
            least_significant_bits,
        }
    }

    pub fn write_uuid(&mut self, uuid: &Uuid) {
        self.write_long(uuid.most_significant_bits);
        self.write_long(uuid.least_significant_bits);
    }

    pub fn read_optional<T>(&mut self, reader: fn(&mut ByteBuf) -> T) -> Option<T> {
        if self.read_bool() {
            Some(reader(self))
        } else {
            None
        }
    }

    pub fn write_optional<T>(&mut self, value: &Option<T>, writer: fn(&mut ByteBuf, &T)) {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub struct CommonPacketTypes;

impl CommonPacketTypes {
    pub const CLIENTBOUND_RESOURCE_PACK_PUSH: PacketType = PacketType {
        id: "clientbound_resource_pack_push",
    };
}

pub trait ClientCommonPacketListener {
    fn handle_resource_pack_push(&mut self, packet: &ClientboundResourcePackPushPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundResourcePackPushPacket {
    id: Uuid,
    url: String,
    hash: String,
    required: bool,
    prompt: Option<Component>,
}

fn read_component(buf: &mut ByteBuf) -> Component {
    Component::read_trusted_context_free(buf)
}

fn write_component(buf: &mut ByteBuf, component: &Component) {
    component.write_trusted_context_free(buf);
}

fn encode_resource_pack_push(packet: &ClientboundResourcePackPushPacket, buf: &mut ByteBuf) {
    buf.write_uuid(&packet.id);
    buf.write_utf(&packet.url, 32767);
    buf.write_utf(&packet.hash, 40);
    buf.write_bool(packet.required);
    buf.write_optional(&packet.prompt, write_component);
}

fn decode_resource_pack_push(buf: &mut ByteBuf) -> ClientboundResourcePackPushPacket {
    let id = buf.read_uuid();
    let url = buf.read_utf(32767);
    let hash = buf.read_utf(40);
    let required = buf.read_bool();
    let prompt = buf.read_optional(read_component);
    ClientboundResourcePackPushPacket::new(id, url, hash, required, prompt)
}

impl ClientboundResourcePackPushPacket {
    pub const MAX_HASH_LENGTH: usize = 40;

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket#<init>
    pub fn new(
        id: Uuid,
        url: String,
        hash: String,
        required: bool,
        prompt: Option<Component>,
    ) -> Self {
        let hash_len = hash.encode_utf16().count();
        if hash_len > 40 {
            panic!("Hash is too long (max 40, was {})", hash_len);
        }
        Self {
            id,
            url,
            hash,
            required,
            prompt,
        }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket#type
    pub fn get_type(&self) -> PacketType {
        CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_PUSH
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientCommonPacketListener) {
        listener.handle_resource_pack_push(self);
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket#id
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket#url
    pub fn url(&self) -> &str {
        &self.url
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket#hash
    pub fn hash(&self) -> &str {
        &self.hash
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket#required
    pub fn required(&self) -> bool {
        self.required
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket#prompt
    pub fn prompt(&self) -> Option<&Component> {
        self.prompt.as_ref()
    }
}

pub struct StreamCodec<T> {
    pub encode: fn(&T, &mut ByteBuf),
    pub decode: fn(&mut ByteBuf) -> T,
}

pub static STREAM_CODEC: StreamCodec<ClientboundResourcePackPushPacket> = StreamCodec {
    encode: encode_resource_pack_push,
    decode: decode_resource_pack_push,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_hash_too_long() {
        let long_hash = "a".repeat(41);
        let result = std::panic::catch_unwind(|| {
            ClientboundResourcePackPushPacket::new(
                Uuid {
                    most_significant_bits: 0,
                    least_significant_bits: 0,
                },
                "https://example.com".to_string(),
                long_hash,
                false,
                None,
            )
        });
        assert!(result.is_err());
    }

    #[test]
    fn accepts_hash_at_max_length() {
        let hash = "a".repeat(40);
        let packet = ClientboundResourcePackPushPacket::new(
            Uuid {
                most_significant_bits: 1,
                least_significant_bits: 2,
            },
            "https://example.com".to_string(),
            hash.clone(),
            true,
            None,
        );
        assert_eq!(packet.hash(), &hash);
        assert_eq!(ClientboundResourcePackPushPacket::MAX_HASH_LENGTH, 40);
    }

    #[test]
    fn roundtrip_with_prompt_and_negative_uuid() {
        let packet = ClientboundResourcePackPushPacket::new(
            Uuid {
                most_significant_bits: i64::MIN,
                least_significant_bits: -1,
            },
            "https://example.com/pack.zip".to_string(),
            "0123456789abcdef0123456789abcdef01234567".to_string(),
            true,
            Some(Component::literal("Accept?")),
        );
        let mut buf = ByteBuf::new();
        (STREAM_CODEC.encode)(&packet, &mut buf);
        let mut r = ByteBuf::from_bytes(buf.into_inner());
        let decoded = (STREAM_CODEC.decode)(&mut r);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn roundtrip_without_prompt() {
        let packet = ClientboundResourcePackPushPacket::new(
            Uuid {
                most_significant_bits: 0,
                least_significant_bits: 0,
            },
            "url".to_string(),
            "hash".to_string(),
            false,
            None,
        );
        let mut buf = ByteBuf::new();
        (STREAM_CODEC.encode)(&packet, &mut buf);
        let mut r = ByteBuf::from_bytes(buf.into_inner());
        let decoded = (STREAM_CODEC.decode)(&mut r);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn handle_dispatches_and_type() {
        struct MockListener {
            seen: bool,
        }
        impl ClientCommonPacketListener for MockListener {
            fn handle_resource_pack_push(
                &mut self,
                _packet: &ClientboundResourcePackPushPacket,
            ) {
                self.seen = true;
            }
        }

        let packet = ClientboundResourcePackPushPacket::new(
            Uuid {
                most_significant_bits: 7,
                least_significant_bits: 8,
            },
            "u".to_string(),
            "h".to_string(),
            false,
            None,
        );
        let mut listener = MockListener { seen: false };
        packet.handle(&mut listener);
        assert!(listener.seen);
        assert_eq!(
            packet.get_type(),
            CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_PUSH
        );
    }
}
