//! Port of `net.minecraft.network.protocol.game.ClientboundLoginPacket`.

use std::collections::HashSet;
use std::hash::Hash;
use std::io::{self, Read, Write};

/// Resource key for a Minecraft registry entry.
/// Java: `net.minecraft.resources.ResourceKey<T>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub registry: String,
    pub location: String,
}

impl ResourceKey {
    pub fn new(registry: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            registry: registry.into(),
            location: location.into(),
        }
    }

    /// Convenience for dimension keys (`minecraft:dimension` registry).
    pub fn dimension(location: impl Into<String>) -> Self {
        Self::new("minecraft:dimension", location)
    }
}

/// Minimal self-contained packet buffer matching the Java `FriendlyByteBuf`
/// methods used by `ClientboundLoginPacket`.
#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn reader_index(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn read_exact_bytes(&mut self, len: usize) -> io::Result<Vec<u8>> {
        if self.remaining() < len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "not enough bytes",
            ));
        }
        let out = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(out)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readInt()
    pub fn read_int(&mut self) -> io::Result<i32> {
        let bytes = self.read_exact_bytes(4)?;
        Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeInt(int)
    pub fn write_int(&mut self, value: i32) -> io::Result<()> {
        self.data.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean()
    pub fn read_boolean(&mut self) -> io::Result<bool> {
        let b = self.read_exact_bytes(1)?[0];
        Ok(b != 0)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean(boolean)
    pub fn write_boolean(&mut self, value: bool) -> io::Result<()> {
        self.data.push(if value { 1 } else { 0 });
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.read_exact_bytes(1)?[0];
            result |= ((byte & 0x7F) as i32) << shift;
            if (byte & 0x80) == 0 {
                return Ok(result);
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
    pub fn write_var_int(&mut self, mut value: i32) -> io::Result<()> {
        let mut v = value as u32;
        loop {
            if (v & !0x7F) == 0 {
                self.data.push(v as u8);
                return Ok(());
            }
            self.data.push(((v & 0x7F) as u8) | 0x80);
            v >>= 7;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf(int) / readUtf()
    pub fn read_utf(&mut self, max_length: usize) -> io::Result<String> {
        let len = self.read_var_int()? as usize;
        let max_bytes = max_length.saturating_mul(3);
        if len > max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "string too long",
            ));
        }
        let bytes = self.read_exact_bytes(len)?;
        let s = String::from_utf8(bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf8"))?;
        if s.chars().count() > max_length {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "string too long",
            ));
        }
        Ok(s)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf(String, int) / writeUtf(String)
    pub fn write_utf(&mut self, value: &str, max_length: usize) -> io::Result<()> {
        if value.chars().count() > max_length {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "string too long",
            ));
        }
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32)?;
        self.data.extend_from_slice(bytes);
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readResourceLocation()
    pub fn read_resource_location(&mut self) -> io::Result<String> {
        self.read_utf(32767)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeResourceLocation(net.minecraft.resources.ResourceLocation)
    pub fn write_resource_location(&mut self, value: &str) -> io::Result<()> {
        self.write_utf(value, 32767)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readResourceKey(net.minecraft.resources.ResourceKey)
    pub fn read_resource_key(&mut self, registry: &str) -> io::Result<ResourceKey> {
        let location = self.read_resource_location()?;
        Ok(ResourceKey::new(registry, location))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeResourceKey(net.minecraft.resources.ResourceKey)
    pub fn write_resource_key(&mut self, key: &ResourceKey) -> io::Result<()> {
        self.write_resource_location(&key.location)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readCollection(IntFunction, Reader)
    pub fn read_collection<T, C, F>(&mut self, mut make_collection: impl FnMut(usize) -> C, mut reader: F) -> io::Result<C>
    where
        C: Extend<T>,
        F: FnMut(&mut Self) -> io::Result<T>,
    {
        let len = self.read_var_int()? as usize;
        let mut collection = make_collection(len);
        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            items.push(reader(self)?);
        }
        collection.extend(items);
        Ok(collection)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeCollection(Collection, Writer)
    pub fn write_collection<T, I, F>(&mut self, collection: I, mut writer: F) -> io::Result<()>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
        F: FnMut(&mut Self, &T) -> io::Result<()>,
    {
        let iter = collection.into_iter();
        self.write_var_int(iter.len() as i32)?;
        for item in iter {
            writer(self, &item)?;
        }
        Ok(())
    }
}

/// Minimal self-contained `RegistryFriendlyByteBuf` placeholder.
/// In the real port this wraps registry access; `ClientboundLoginPacket` only
/// uses inherited `FriendlyByteBuf` methods in the provided Java.
#[derive(Debug, Clone, Default)]
pub struct RegistryFriendlyByteBuf {
    pub buf: FriendlyByteBuf,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self {
            buf: FriendlyByteBuf::from_vec(data),
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf.into_inner()
    }

    pub fn read_int(&mut self) -> io::Result<i32> {
        self.buf.read_int()
    }
    pub fn write_int(&mut self, value: i32) -> io::Result<()> {
        self.buf.write_int(value)
    }
    pub fn read_boolean(&mut self) -> io::Result<bool> {
        self.buf.read_boolean()
    }
    pub fn write_boolean(&mut self, value: bool) -> io::Result<()> {
        self.buf.write_boolean(value)
    }
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        self.buf.read_var_int()
    }
    pub fn write_var_int(&mut self, value: i32) -> io::Result<()> {
        self.buf.write_var_int(value)
    }
    pub fn read_resource_key(&mut self, registry: &str) -> io::Result<ResourceKey> {
        self.buf.read_resource_key(registry)
    }
    pub fn write_resource_key(&mut self, key: &ResourceKey) -> io::Result<()> {
        self.buf.write_resource_key(key)
    }
    pub fn read_collection<T, C, F>(&mut self, make_collection: impl FnMut(usize) -> C, reader: F) -> io::Result<C>
    where
        C: Extend<T>,
        F: FnMut(&mut FriendlyByteBuf) -> io::Result<T>,
    {
        self.buf.read_collection(make_collection, reader)
    }
    pub fn write_collection<T, I, F>(&mut self, collection: I, writer: F) -> io::Result<()>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
        F: FnMut(&mut FriendlyByteBuf, &T) -> io::Result<()>,
    {
        self.buf.write_collection(collection, writer)
    }
}

/// Port of `net.minecraft.network.protocol.game.CommonPlayerSpawnInfo`.
///
/// The Java class is not provided here; this is a minimal self-contained codec
/// stub that preserves the wire shape as an opaque byte payload for roundtrip
/// safety within this module. Replace with the real field-level port when the
/// dependency is ported.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommonPlayerSpawnInfo {
    pub raw_data: Vec<u8>,
}

impl CommonPlayerSpawnInfo {
    // JAVA: net.minecraft.network.protocol.game.CommonPlayerSpawnInfo#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        // Unknown external layout in this isolated module. Read a length-prefixed blob
        // so tests and local roundtrips are possible. The real port should delegate
        // to the actual CommonPlayerSpawnInfo decoder.
        let len = buf.read_var_int()? as usize;
        let bytes = buf.buf.read_exact_bytes(len)?;
        Ok(Self { raw_data: bytes })
    }

    // JAVA: net.minecraft.network.protocol.game.CommonPlayerSpawnInfo#write(RegistryFriendlyByteBuf)
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) -> io::Result<()> {
        buf.write_var_int(self.raw_data.len() as i32)?;
        buf.buf.data.extend_from_slice(&self.raw_data);
        Ok(())
    }
}

/// Port of `net.minecraft.network.protocol.game.ClientboundLoginPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginPacket {
    pub player_id: i32,
    pub hardcore: bool,
    pub levels: HashSet<ResourceKey>,
    pub max_players: i32,
    pub chunk_radius: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub do_limited_crafting: bool,
    pub common_player_spawn_info: CommonPlayerSpawnInfo,
    pub enforces_secure_chat: bool,
}

impl ClientboundLoginPacket {
    pub const PACKET_TYPE: &'static str = "minecraft:login";
    pub const DIMENSION_REGISTRY: &'static str = "minecraft:dimension";

    // JAVA: net.minecraft.network.protocol.game.ClientboundLoginPacket#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let player_id = buf.read_int()?;
        let hardcore = buf.read_boolean()?;
        let levels: HashSet<ResourceKey> = buf.read_collection(
            |cap| HashSet::with_capacity(cap),
            |b| b.read_resource_key(Self::DIMENSION_REGISTRY),
        )?;
        let max_players = buf.read_var_int()?;
        let chunk_radius = buf.read_var_int()?;
        let simulation_distance = buf.read_var_int()?;
        let reduced_debug_info = buf.read_boolean()?;
        let show_death_screen = buf.read_boolean()?;
        let do_limited_crafting = buf.read_boolean()?;
        let common_player_spawn_info = CommonPlayerSpawnInfo::read(buf)?;
        let enforces_secure_chat = buf.read_boolean()?;

        Ok(Self {
            player_id,
            hardcore,
            levels,
            max_players,
            chunk_radius,
            simulation_distance,
            reduced_debug_info,
            show_death_screen,
            do_limited_crafting,
            common_player_spawn_info,
            enforces_secure_chat,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLoginPacket#write(RegistryFriendlyByteBuf)
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) -> io::Result<()> {
        buf.write_int(self.player_id)?;
        buf.write_boolean(self.hardcore)?;
        let levels: Vec<&ResourceKey> = self.levels.iter().collect();
        buf.write_collection(levels, |b, key| b.write_resource_key(key))?;
        buf.write_var_int(self.max_players)?;
        buf.write_var_int(self.chunk_radius)?;
        buf.write_var_int(self.simulation_distance)?;
        buf.write_boolean(self.reduced_debug_info)?;
        buf.write_boolean(self.show_death_screen)?;
        buf.write_boolean(self.do_limited_crafting)?;
        self.common_player_spawn_info.write(buf)?;
        buf.write_boolean(self.enforces_secure_chat)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLoginPacket#type()
    pub fn packet_type(&self) -> &'static str {
        Self::PACKET_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLoginPacket#handle(ClientGamePacketListener)
    pub fn handle<H: ClientGamePacketListener>(&self, handler: &mut H) {
        handler.handle_login(self);
    }
}

/// Minimal listener trait mirroring `ClientGamePacketListener#handleLogin`.
pub trait ClientGamePacketListener {
    fn handle_login(&mut self, packet: &ClientboundLoginPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_packet() -> ClientboundLoginPacket {
        let mut levels = HashSet::new();
        levels.insert(ResourceKey::dimension("minecraft:overworld"));
        levels.insert(ResourceKey::dimension("minecraft:the_nether"));
        ClientboundLoginPacket {
            player_id: 42,
            hardcore: true,
            levels,
            max_players: 20,
            chunk_radius: 10,
            simulation_distance: 8,
            reduced_debug_info: false,
            show_death_screen: true,
            do_limited_crafting: false,
            common_player_spawn_info: CommonPlayerSpawnInfo {
                raw_data: vec![1, 2, 3, 4, 5],
            },
            enforces_secure_chat: true,
        }
    }

    #[test]
    fn write_read_roundtrip() {
        let packet = sample_packet();
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf).expect("write");
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundLoginPacket::read(&mut read_buf).expect("read");
        assert_eq!(decoded.player_id, packet.player_id);
        assert_eq!(decoded.hardcore, packet.hardcore);
        assert_eq!(decoded.levels, packet.levels);
        assert_eq!(decoded.max_players, packet.max_players);
        assert_eq!(decoded.chunk_radius, packet.chunk_radius);
        assert_eq!(decoded.simulation_distance, packet.simulation_distance);
        assert_eq!(decoded.reduced_debug_info, packet.reduced_debug_info);
        assert_eq!(decoded.show_death_screen, packet.show_death_screen);
        assert_eq!(decoded.do_limited_crafting, packet.do_limited_crafting);
        assert_eq!(
            decoded.common_player_spawn_info,
            packet.common_player_spawn_info
        );
        assert_eq!(decoded.enforces_secure_chat, packet.enforces_secure_chat);
        assert_eq!(read_buf.buf.remaining(), 0);
    }

    #[test]
    fn handles_negative_and_large_varints() {
        let mut packet = sample_packet();
        packet.player_id = -1;
        packet.max_players = i32::MAX;
        packet.chunk_radius = -1;
        packet.simulation_distance = i32::MIN;
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf).expect("write");
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundLoginPacket::read(&mut read_buf).expect("read");
        assert_eq!(decoded.player_id, -1);
        assert_eq!(decoded.max_players, i32::MAX);
        assert_eq!(decoded.chunk_radius, -1);
        assert_eq!(decoded.simulation_distance, i32::MIN);
    }

    #[test]
    fn empty_levels_roundtrip() {
        let mut packet = sample_packet();
        packet.levels.clear();
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf).expect("write");
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundLoginPacket::read(&mut read_buf).expect("read");
        assert!(decoded.levels.is_empty());
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct TestHandler {
            seen_player_id: Option<i32>,
        }
        impl ClientGamePacketListener for TestHandler {
            fn handle_login(&mut self, packet: &ClientboundLoginPacket) {
                self.seen_player_id = Some(packet.player_id);
            }
        }

        let packet = sample_packet();
        let mut handler = TestHandler {
            seen_player_id: None,
        };
        packet.handle(&mut handler);
        assert_eq!(handler.seen_player_id, Some(42));
        assert_eq!(packet.packet_type(), "minecraft:login");
    }

    #[test]
    fn varint_encoding_edges() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(0).unwrap();
        buf.write_var_int(127).unwrap();
        buf.write_var_int(128).unwrap();
        buf.write_var_int(-1).unwrap();
        buf.write_var_int(i32::MAX).unwrap();
        buf.write_var_int(i32::MIN).unwrap();
        let mut r = FriendlyByteBuf::from_vec(buf.into_inner());
        assert_eq!(r.read_var_int().unwrap(), 0);
        assert_eq!(r.read_var_int().unwrap(), 127);
        assert_eq!(r.read_var_int().unwrap(), 128);
        assert_eq!(r.read_var_int().unwrap(), -1);
        assert_eq!(r.read_var_int().unwrap(), i32::MAX);
        assert_eq!(r.read_var_int().unwrap(), i32::MIN);
    }
}
