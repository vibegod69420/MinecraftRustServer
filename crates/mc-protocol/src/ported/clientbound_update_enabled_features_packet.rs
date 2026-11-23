use std::collections::HashSet;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CONFIGURATION_CLIENTBOUND_UPDATE_ENABLED_FEATURES: PacketType =
    PacketType("configuration:clientbound_update_enabled_features");

pub trait ClientConfigurationPacketListener {
    fn handle_enabled_features(&mut self, packet: &ClientboundUpdateEnabledFeaturesPacket);
}

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
pub struct ClientboundUpdateEnabledFeaturesPacket {
    pub features: HashSet<Identifier>,
}

impl ClientboundUpdateEnabledFeaturesPacket {
    // JAVA: net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let features = buf.read_collection(|_| HashSet::new(), |b| b.read_identifier())?;
        Ok(Self { features })
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.features, |b, id| b.write_identifier(id));
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket#features
    pub fn features(&self) -> &HashSet<Identifier> {
        &self.features
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket#type
    pub fn get_type(&self) -> PacketType {
        CONFIGURATION_CLIENTBOUND_UPDATE_ENABLED_FEATURES
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientConfigurationPacketListener) {
        listener.handle_enabled_features(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn id(s: &str) -> Identifier {
        Identifier::new(s.to_string())
    }

    #[test]
    fn round_trip_empty_features() {
        let pkt = ClientboundUpdateEnabledFeaturesPacket {
            features: HashSet::new(),
        };
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundUpdateEnabledFeaturesPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_multiple_features() {
        let mut features = HashSet::new();
        features.insert(id("minecraft:vanilla"));
        features.insert(id("minecraft:bundle"));
        features.insert(id("custom:test_feature"));
        let pkt = ClientboundUpdateEnabledFeaturesPacket { features };
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundUpdateEnabledFeaturesPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.features().len(), 3);
    }

    #[test]
    fn varint_round_trip_negative_and_extreme_values() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(0);
        buf.write_var_int(-1);
        buf.write_var_int(i32::MIN);
        buf.write_var_int(i32::MAX);
        buf.write_var_int(128);
        buf.write_var_int(-128);
        let mut r = FriendlyByteBuf::from_bytes(buf.into_inner());
        assert_eq!(r.read_var_int().unwrap(), 0);
        assert_eq!(r.read_var_int().unwrap(), -1);
        assert_eq!(r.read_var_int().unwrap(), i32::MIN);
        assert_eq!(r.read_var_int().unwrap(), i32::MAX);
        assert_eq!(r.read_var_int().unwrap(), 128);
        assert_eq!(r.read_var_int().unwrap(), -128);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn varint_too_big_returns_error() {
        let mut r = FriendlyByteBuf::from_bytes(vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80]);
        let err = r.read_var_int().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn handle_dispatches_and_type_matches() {
        struct Listener {
            called: bool,
            count: usize,
        }
        impl ClientConfigurationPacketListener for Listener {
            fn handle_enabled_features(&mut self, packet: &ClientboundUpdateEnabledFeaturesPacket) {
                self.called = true;
                self.count = packet.features().len();
            }
        }

        let mut features = HashSet::new();
        features.insert(id("minecraft:test"));
        let pkt = ClientboundUpdateEnabledFeaturesPacket { features };
        assert_eq!(
            pkt.get_type(),
            CONFIGURATION_CLIENTBOUND_UPDATE_ENABLED_FEATURES
        );

        let mut listener = Listener {
            called: false,
            count: 0,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.count, 1);
    }
}
