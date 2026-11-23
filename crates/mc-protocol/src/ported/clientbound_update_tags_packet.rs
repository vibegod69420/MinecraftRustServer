use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ResourceLocation {
    pub fn parse(s: &str) -> Self {
        match s.split_once(':') {
            Some((ns, path)) => Self {
                namespace: ns.to_string(),
                path: path.to_string(),
            },
            None => Self {
                namespace: "minecraft".to_string(),
                path: s.to_string(),
            },
        }
    }
}

impl fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub location: ResourceLocation,
}

impl ResourceKey {
    pub fn new(location: ResourceLocation) -> Self {
        Self { location }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_idx: usize,
}

impl FriendlyByteBuf {
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

    pub fn read_resource_location(&mut self) -> ResourceLocation {
        ResourceLocation::parse(&self.read_utf(32767))
    }

    pub fn write_resource_location(&mut self, loc: &ResourceLocation) {
        self.write_utf(&loc.to_string(), 32767);
    }

    pub fn read_registry_key(&mut self) -> ResourceKey {
        ResourceKey::new(self.read_resource_location())
    }

    pub fn write_resource_key(&mut self, key: &ResourceKey) {
        self.write_resource_location(&key.location);
    }

    pub fn read_list<T>(&mut self, reader: fn(&mut FriendlyByteBuf) -> T) -> Vec<T> {
        let size = self.read_var_int() as usize;
        let mut list = Vec::with_capacity(size);
        for _ in 0..size {
            list.push(reader(self));
        }
        list
    }

    pub fn write_list<T>(&mut self, list: &[T], writer: fn(&mut FriendlyByteBuf, &T)) {
        self.write_var_int(list.len() as i32);
        for item in list {
            writer(self, item);
        }
    }

    pub fn read_map<K, V>(
        &mut self,
        key_reader: fn(&mut FriendlyByteBuf) -> K,
        value_reader: fn(&mut FriendlyByteBuf) -> V,
    ) -> HashMap<K, V>
    where
        K: Eq + Hash,
    {
        let size = self.read_var_int() as usize;
        let mut map = HashMap::with_capacity(size);
        for _ in 0..size {
            let k = key_reader(self);
            let v = value_reader(self);
            map.insert(k, v);
        }
        map
    }

    pub fn write_map<K, V>(
        &mut self,
        map: &HashMap<K, V>,
        key_writer: fn(&mut FriendlyByteBuf, &K),
        value_writer: fn(&mut FriendlyByteBuf, &V),
    ) where
        K: Eq + Hash,
    {
        self.write_var_int(map.len() as i32);
        for (k, v) in map {
            key_writer(self, k);
            value_writer(self, v);
        }
    }
}

fn read_var_int_entry(buf: &mut FriendlyByteBuf) -> i32 {
    buf.read_var_int()
}

fn write_var_int_entry(buf: &mut FriendlyByteBuf, value: &i32) {
    buf.write_var_int(*value);
}

fn read_int_list(buf: &mut FriendlyByteBuf) -> Vec<i32> {
    buf.read_list(read_var_int_entry)
}

fn write_int_list(buf: &mut FriendlyByteBuf, list: &Vec<i32>) {
    buf.write_list(list, write_var_int_entry);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPayload {
    pub tags: HashMap<ResourceLocation, Vec<i32>>,
}

impl NetworkPayload {
    // JAVA: net.minecraft.tags.TagNetworkSerialization$NetworkPayload#read
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let tags = buf.read_map(FriendlyByteBuf::read_resource_location, read_int_list);
        Self { tags }
    }

    // JAVA: net.minecraft.tags.TagNetworkSerialization$NetworkPayload#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_map(
            &self.tags,
            FriendlyByteBuf::write_resource_location,
            write_int_list,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub struct CommonPacketTypes;

impl CommonPacketTypes {
    pub const CLIENTBOUND_UPDATE_TAGS: PacketType = PacketType {
        id: "clientbound_update_tags",
    };
}

pub trait ClientCommonPacketListener {
    fn handle_update_tags(&mut self, packet: &ClientboundUpdateTagsPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateTagsPacket {
    tags: HashMap<ResourceKey, NetworkPayload>,
}

fn write_network_payload(buf: &mut FriendlyByteBuf, payload: &NetworkPayload) {
    payload.write(buf);
}

impl ClientboundUpdateTagsPacket {
    // JAVA: net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket#<init>
    pub fn new(tags: HashMap<ResourceKey, NetworkPayload>) -> Self {
        Self { tags }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let tags = buf.read_map(FriendlyByteBuf::read_registry_key, NetworkPayload::read);
        Self { tags }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_map(
            &self.tags,
            FriendlyByteBuf::write_resource_key,
            write_network_payload,
        );
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket#type
    pub fn get_type(&self) -> PacketType {
        CommonPacketTypes::CLIENTBOUND_UPDATE_TAGS
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientCommonPacketListener) {
        listener.handle_update_tags(self);
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket#getTags
    pub fn get_tags(&self) -> &HashMap<ResourceKey, NetworkPayload> {
        &self.tags
    }
}

pub struct StreamCodec<T> {
    pub encode: fn(&T, &mut FriendlyByteBuf),
    pub decode: fn(&mut FriendlyByteBuf) -> T,
}

fn encode_update_tags(packet: &ClientboundUpdateTagsPacket, buf: &mut FriendlyByteBuf) {
    packet.write(buf);
}

fn decode_update_tags(buf: &mut FriendlyByteBuf) -> ClientboundUpdateTagsPacket {
    ClientboundUpdateTagsPacket::read(buf)
}

pub static STREAM_CODEC: StreamCodec<ClientboundUpdateTagsPacket> = StreamCodec {
    encode: encode_update_tags,
    decode: decode_update_tags,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_int_roundtrip_edge_cases() {
        let cases = [0, 1, 127, 128, 255, -1, i32::MIN, i32::MAX];
        for v in cases {
            let mut buf = FriendlyByteBuf::new();
            buf.write_var_int(v);
            let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
            assert_eq!(r.read_var_int(), v);
        }
    }

    #[test]
    fn var_int_negative_one_encoding() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        assert_eq!(buf.as_bytes(), &[0xff, 0xff, 0xff, 0xff, 0x0f]);
    }

    #[test]
    fn empty_tags_packet_roundtrip() {
        let packet = ClientboundUpdateTagsPacket::new(HashMap::new());
        let mut buf = FriendlyByteBuf::new();
        (STREAM_CODEC.encode)(&packet, &mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = (STREAM_CODEC.decode)(&mut r);
        assert_eq!(decoded, packet);
        assert!(decoded.get_tags().is_empty());
    }

    #[test]
    fn tags_packet_roundtrip_with_negative_ids() {
        let mut payload_tags = HashMap::new();
        payload_tags.insert(
            ResourceLocation::parse("minecraft:mineable/pickaxe"),
            vec![1, -1, i32::MIN, i32::MAX, 0],
        );
        let payload = NetworkPayload { tags: payload_tags };

        let mut tags = HashMap::new();
        tags.insert(
            ResourceKey::new(ResourceLocation::parse("minecraft:block")),
            payload,
        );
        let packet = ClientboundUpdateTagsPacket::new(tags);

        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundUpdateTagsPacket::read(&mut r);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct MockListener {
            called: bool,
        }
        impl ClientCommonPacketListener for MockListener {
            fn handle_update_tags(&mut self, _packet: &ClientboundUpdateTagsPacket) {
                self.called = true;
            }
        }

        let packet = ClientboundUpdateTagsPacket::new(HashMap::new());
        let mut listener = MockListener { called: false };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(packet.get_type(), CommonPacketTypes::CLIENTBOUND_UPDATE_TAGS);
    }

    #[test]
    fn resource_location_parse_and_display() {
        let loc = ResourceLocation::parse("minecraft:block");
        assert_eq!(loc.namespace, "minecraft");
        assert_eq!(loc.path, "block");
        assert_eq!(loc.to_string(), "minecraft:block");

        let short = ResourceLocation::parse("stone");
        assert_eq!(short.namespace, "minecraft");
        assert_eq!(short.path, "stone");
    }
}
