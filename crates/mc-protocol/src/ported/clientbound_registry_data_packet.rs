use std::io;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#<init>
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    location: Identifier,
}

impl ResourceKey {
    // JAVA: net.minecraft.resources.ResourceKey#createRegistryKey
    pub fn create_registry_key(location: Identifier) -> Self {
        Self { location }
    }

    // JAVA: net.minecraft.resources.ResourceKey#identifier
    pub fn identifier(&self) -> &Identifier {
        &self.location
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CONFIGURATION_CLIENTBOUND_REGISTRY_DATA: PacketType =
    PacketType("configuration:clientbound_registry_data");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_pos)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte
    fn read_byte(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.read_pos];
        self.read_pos = self.read_pos.wrapping_add(1);
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte
    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_byte()? != 0)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_bool(&mut self, value: bool) {
        self.write_byte(if value { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b = self.read_byte()?;
            let value_part = ((b & 0x7F) as i32).wrapping_shl(((j.wrapping_mul(7)) & 31) as u32);
            i |= value_part;
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if (b & 0x80) == 0 {
                break;
            }
        }
        Ok(i)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F_i32) == 0 {
                self.write_byte(value as u8);
                return;
            }
            self.write_byte(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
    pub fn read_utf(&mut self, max_length: usize) -> io::Result<String> {
        let len_i32 = self.read_var_int()?;
        if len_i32 < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Negative string length",
            ));
        }
        let len = len_i32 as usize;
        let max_bytes = max_length.saturating_mul(3).saturating_add(3);
        if len > max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "String too long",
            ));
        }
        if self.remaining() < len {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let start = self.read_pos;
        let end = start + len;
        let bytes = self.data[start..end].to_vec();
        self.read_pos = end;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
    pub fn write_utf(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readIdentifier
    pub fn read_identifier(&mut self) -> io::Result<Identifier> {
        let s = self.read_utf(32767)?;
        Ok(Identifier::new(s))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeIdentifier
    pub fn write_identifier(&mut self, id: &Identifier) {
        self.write_utf(id.as_str());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readCollection
    pub fn read_collection<T, C, F>(
        &mut self,
        mut make_collection: impl FnMut(usize) -> C,
        mut reader: F,
    ) -> io::Result<C>
    where
        C: Extend<T>,
        F: FnMut(&mut Self) -> io::Result<T>,
    {
        let size_i32 = self.read_var_int()?;
        if size_i32 < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Negative collection size",
            ));
        }
        let size = size_i32 as usize;
        let mut collection = make_collection(size);
        let mut items = Vec::with_capacity(size);
        for _ in 0..size {
            items.push(reader(self)?);
        }
        collection.extend(items);
        Ok(collection)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeCollection
    pub fn write_collection<I, F>(&mut self, collection: I, mut writer: F)
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        F: FnMut(&mut Self, I::Item),
    {
        let iter = collection.into_iter();
        self.write_var_int(iter.len() as i32);
        for item in iter {
            writer(self, item);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedRegistryEntry {
    pub id: Identifier,
    pub data: Option<Vec<u8>>,
}

impl PackedRegistryEntry {
    // JAVA: net.minecraft.core.RegistrySynchronization$PackedRegistryEntry#<init>
    pub fn new(id: Identifier, data: Option<Vec<u8>>) -> Self {
        Self { id, data }
    }

    // JAVA: net.minecraft.core.RegistrySynchronization$PackedRegistryEntry#id
    pub fn id(&self) -> &Identifier {
        &self.id
    }

    // JAVA: net.minecraft.core.RegistrySynchronization$PackedRegistryEntry#data
    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let id = buf.read_identifier()?;
        let has_data = buf.read_bool()?;
        let data = if has_data {
            let len_i32 = buf.read_var_int()?;
            if len_i32 < 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Negative NBT length",
                ));
            }
            let len = len_i32 as usize;
            let mut bytes = Vec::with_capacity(len);
            for _ in 0..len {
                bytes.push(buf.read_byte()?);
            }
            Some(bytes)
        } else {
            None
        };
        Ok(Self { id, data })
    }

    fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_identifier(&self.id);
        match &self.data {
            Some(bytes) => {
                buf.write_bool(true);
                buf.write_var_int(bytes.len() as i32);
                for &b in bytes {
                    buf.write_byte(b);
                }
            }
            None => {
                buf.write_bool(false);
            }
        }
    }
}

pub trait ClientConfigurationPacketListener {
    fn handle_registry_data(&mut self, packet: &ClientboundRegistryDataPacket);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundRegistryDataPacket {
    pub registry: ResourceKey,
    pub entries: Vec<PackedRegistryEntry>,
}

impl ClientboundRegistryDataPacket {
    // JAVA: net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket#<init>
    pub fn new(registry: ResourceKey, entries: Vec<PackedRegistryEntry>) -> Self {
        Self { registry, entries }
    }

    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let registry_id = buf.read_identifier()?;
        let registry = ResourceKey::create_registry_key(registry_id);
        let entries =
            buf.read_collection(|cap| Vec::with_capacity(cap), |b| PackedRegistryEntry::read(b))?;
        Ok(Self { registry, entries })
    }

    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_identifier(self.registry.identifier());
        buf.write_collection(&self.entries, |b, entry| entry.write(b));
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket#registry
    pub fn registry(&self) -> &ResourceKey {
        &self.registry
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket#entries
    pub fn entries(&self) -> &[PackedRegistryEntry] {
        &self.entries
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket#type
    pub fn get_type(&self) -> PacketType {
        CONFIGURATION_CLIENTBOUND_REGISTRY_DATA
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientConfigurationPacketListener) {
        listener.handle_registry_data(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> Identifier {
        Identifier::new(s.to_string())
    }

    #[test]
    fn round_trip_empty_entries() {
        let pkt = ClientboundRegistryDataPacket::new(
            ResourceKey::create_registry_key(id("minecraft:dimension_type")),
            vec![],
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundRegistryDataPacket::read(&mut r).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn round_trip_with_entries_and_optional_data() {
        let pkt = ClientboundRegistryDataPacket::new(
            ResourceKey::create_registry_key(id("minecraft:worldgen/biome")),
            vec![
                PackedRegistryEntry::new(id("minecraft:plains"), None),
                PackedRegistryEntry::new(id("minecraft:desert"), Some(vec![10, 0, 0, 1, 2, 255])),
            ],
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundRegistryDataPacket::read(&mut r).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.entries().len(), 2);
        assert_eq!(decoded.entries()[1].data(), Some(&[10u8, 0, 0, 1, 2, 255][..]));
    }

    #[test]
    fn varint_negative_and_extreme_values_round_trip() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        buf.write_var_int(i32::MIN);
        buf.write_var_int(i32::MAX);
        buf.write_var_int(0);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        assert_eq!(r.read_var_int().unwrap(), -1);
        assert_eq!(r.read_var_int().unwrap(), i32::MIN);
        assert_eq!(r.read_var_int().unwrap(), i32::MAX);
        assert_eq!(r.read_var_int().unwrap(), 0);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn varint_too_big_returns_error() {
        let mut r = FriendlyByteBuf::from_bytes(vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);
        let err = r.read_var_int().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn handle_dispatches_and_type_matches() {
        struct Listener {
            called: bool,
            registry: String,
            count: usize,
        }
        impl ClientConfigurationPacketListener for Listener {
            fn handle_registry_data(&mut self, packet: &ClientboundRegistryDataPacket) {
                self.called = true;
                self.registry = packet.registry().identifier().as_str().to_string();
                self.count = packet.entries().len();
            }
        }

        let pkt = ClientboundRegistryDataPacket::new(
            ResourceKey::create_registry_key(id("minecraft:damage_type")),
            vec![PackedRegistryEntry::new(id("minecraft:fall"), None)],
        );
        assert_eq!(pkt.get_type(), CONFIGURATION_CLIENTBOUND_REGISTRY_DATA);

        let mut listener = Listener {
            called: false,
            registry: String::new(),
            count: 0,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.registry, "minecraft:damage_type");
        assert_eq!(listener.count, 1);
    }
}
