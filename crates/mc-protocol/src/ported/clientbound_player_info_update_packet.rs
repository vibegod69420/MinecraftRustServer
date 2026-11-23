use std::collections::BTreeSet;
use std::fmt;
use std::io;

use bytes::{BufMut, BytesMut};

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("unexpected end of buffer")]
    Eof,
    #[error("invalid varint")]
    InvalidVarInt,
    #[error("invalid enum ordinal: {0}")]
    InvalidEnum(i32),
    #[error("invalid utf-8 string")]
    InvalidString,
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uuid {
    pub most_significant: i64,
    pub least_significant: i64,
}

impl Uuid {
    pub const fn new(most_significant: i64, least_significant: i64) -> Self {
        Self {
            most_significant,
            least_significant,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropertyMap(pub Vec<Property>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: PropertyMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePublicKeyData {
    pub expires_at: i64,
    pub key: Vec<u8>,
    pub key_signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteChatSessionData {
    pub session_id: Uuid,
    pub profile_public_key: ProfilePublicKeyData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum GameType {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

impl GameType {
    pub const DEFAULT_MODE: GameType = GameType::Survival;

    pub fn by_id(id: i32) -> GameType {
        match id {
            0 => GameType::Survival,
            1 => GameType::Creative,
            2 => GameType::Adventure,
            3 => GameType::Spectator,
            _ => Self::DEFAULT_MODE,
        }
    }

    pub fn get_id(self) -> i32 {
        self as i32
    }
}

/// Minimal server player stub for self-contained packet construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayer {
    pub uuid: Uuid,
    pub game_profile: GameProfile,
    pub latency: i32,
    pub game_mode: GameType,
    pub tab_list_display_name: Option<Component>,
    pub show_hat: bool,
    pub tab_list_order: i32,
    pub chat_session: Option<RemoteChatSessionData>,
}

pub trait ClientGamePacketListener {
    fn handle_player_info_update(&mut self, packet: &ClientboundPlayerInfoUpdatePacket);
}

#[derive(Debug, Clone, Default)]
pub struct RegistryFriendlyByteBuf {
    buf: Vec<u8>,
    rpos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            rpos: 0,
        }
    }

    pub fn from_vec(buf: Vec<u8>) -> Self {
        Self { buf, rpos: 0 }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.rpos)
    }

    fn read_u8(&mut self) -> Result<u8, ProtocolError> {
        if self.rpos >= self.buf.len() {
            return Err(ProtocolError::Eof);
        }
        let b = self.buf[self.rpos];
        self.rpos = self.rpos.wrapping_add(1);
        Ok(b)
    }

    fn write_u8(&mut self, value: u8) {
        self.buf.push(value);
    }

    pub fn read_boolean(&mut self) -> Result<bool, ProtocolError> {
        Ok(self.read_u8()? != 0)
    }

    pub fn write_boolean(&mut self, value: bool) {
        self.write_u8(if value { 1 } else { 0 });
    }

    pub fn read_var_int(&mut self) -> Result<i32, ProtocolError> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        for _ in 0..5 {
            let b = self.read_u8()? as i32;
            result |= (b & 0x7F) << shift;
            if (b & 0x80) == 0 {
                return Ok(result);
            }
            shift = shift.wrapping_add(7);
        }
        Err(ProtocolError::InvalidVarInt)
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7Fu32) == 0 {
                self.write_u8(val as u8);
                return;
            }
            self.write_u8(((val & 0x7F) as u8) | 0x80);
            val >>= 7;
        }
    }

    pub fn read_long(&mut self) -> Result<i64, ProtocolError> {
        if self.remaining() < 8 {
            return Err(ProtocolError::Eof);
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.buf[self.rpos..self.rpos.wrapping_add(8)]);
        self.rpos = self.rpos.wrapping_add(8);
        Ok(i64::from_be_bytes(bytes))
    }

    pub fn write_long(&mut self, value: i64) {
        self.buf.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_uuid(&mut self) -> Result<Uuid, ProtocolError> {
        let most = self.read_long()?;
        let least = self.read_long()?;
        Ok(Uuid::new(most, least))
    }

    pub fn write_uuid(&mut self, value: Uuid) {
        self.write_long(value.most_significant);
        self.write_long(value.least_significant);
    }

    pub fn read_utf(&mut self, max_chars: usize) -> Result<String, ProtocolError> {
        let len = self.read_var_int()? as usize;
        let max_bytes = max_chars.saturating_mul(4);
        if len > max_bytes {
            return Err(ProtocolError::InvalidString);
        }
        if self.remaining() < len {
            return Err(ProtocolError::Eof);
        }
        let bytes = &self.buf[self.rpos..self.rpos.wrapping_add(len)];
        let s = std::str::from_utf8(bytes)
            .map_err(|_| ProtocolError::InvalidString)?
            .to_owned();
        if s.chars().count() > max_chars {
            return Err(ProtocolError::InvalidString);
        }
        self.rpos = self.rpos.wrapping_add(len);
        Ok(s)
    }

    pub fn write_utf(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.buf.extend_from_slice(bytes);
    }

    pub fn read_byte_array(&mut self) -> Result<Vec<u8>, ProtocolError> {
        let len = self.read_var_int()? as usize;
        if self.remaining() < len {
            return Err(ProtocolError::Eof);
        }
        let bytes = self.buf[self.rpos..self.rpos.wrapping_add(len)].to_vec();
        self.rpos = self.rpos.wrapping_add(len);
        Ok(bytes)
    }

    pub fn write_byte_array(&mut self, value: &[u8]) {
        self.write_var_int(value.len() as i32);
        self.buf.extend_from_slice(value);
    }

    pub fn read_enum_set_action(&mut self) -> Result<BTreeSet<Action>, ProtocolError> {
        let n = Action::VALUES.len();
        let byte_count = (n + 7) / 8;
        let mut bytes = vec![0u8; byte_count];
        for b in &mut bytes {
            *b = self.read_u8()?;
        }
        let mut set = BTreeSet::new();
        for (i, &action) in Action::VALUES.iter().enumerate() {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if (bytes[byte_idx] & (1u8 << bit_idx)) != 0 {
                set.insert(action);
            }
        }
        Ok(set)
    }

    pub fn write_enum_set_action(&mut self, actions: &BTreeSet<Action>) {
        let n = Action::VALUES.len();
        let byte_count = (n + 7) / 8;
        let mut bytes = vec![0u8; byte_count];
        for (i, &action) in Action::VALUES.iter().enumerate() {
            if actions.contains(&action) {
                let byte_idx = i / 8;
                let bit_idx = i % 8;
                bytes[byte_idx] |= 1u8 << bit_idx;
            }
        }
        self.buf.extend_from_slice(&bytes);
    }

    pub fn read_game_profile_properties(&mut self) -> Result<PropertyMap, ProtocolError> {
        let size = self.read_var_int()? as usize;
        let mut props = Vec::with_capacity(size);
        for _ in 0..size {
            let name = self.read_utf(64)?;
            let value = self.read_utf(32767)?;
            let signature = if self.read_boolean()? {
                Some(self.read_utf(1024)?)
            } else {
                None
            };
            props.push(Property {
                name,
                value,
                signature,
            });
        }
        Ok(PropertyMap(props))
    }

    pub fn write_game_profile_properties(&mut self, properties: &PropertyMap) {
        self.write_var_int(properties.0.len() as i32);
        for prop in &properties.0 {
            self.write_utf(&prop.name);
            self.write_utf(&prop.value);
            match &prop.signature {
                Some(sig) => {
                    self.write_boolean(true);
                    self.write_utf(sig);
                }
                None => {
                    self.write_boolean(false);
                }
            }
        }
    }

    pub fn read_game_profile(&mut self, profile_id: Uuid) -> Result<GameProfile, ProtocolError> {
        let name = self.read_utf(16)?;
        let properties = self.read_game_profile_properties()?;
        Ok(GameProfile {
            id: profile_id,
            name,
            properties,
        })
    }

    pub fn write_game_profile(&mut self, profile: &GameProfile) {
        self.write_utf(&profile.name);
        self.write_game_profile_properties(&profile.properties);
    }

    pub fn read_component(&mut self) -> Result<Component, ProtocolError> {
        // Minimal self-contained representation: network component encoded as a UTF string.
        let s = self.read_utf(262144)?;
        Ok(Component(s))
    }

    pub fn write_component(&mut self, component: &Component) {
        self.write_utf(&component.0);
    }

    pub fn read_nullable_component(&mut self) -> Result<Option<Component>, ProtocolError> {
        if self.read_boolean()? {
            Ok(Some(self.read_component()?))
        } else {
            Ok(None)
        }
    }

    pub fn write_nullable_component(&mut self, component: Option<&Component>) {
        match component {
            Some(c) => {
                self.write_boolean(true);
                self.write_component(c);
            }
            None => {
                self.write_boolean(false);
            }
        }
    }

    pub fn read_remote_chat_session_data(&mut self) -> Result<RemoteChatSessionData, ProtocolError> {
        let session_id = self.read_uuid()?;
        let expires_at = self.read_long()?;
        let key = self.read_byte_array()?;
        let key_signature = self.read_byte_array()?;
        Ok(RemoteChatSessionData {
            session_id,
            profile_public_key: ProfilePublicKeyData {
                expires_at,
                key,
                key_signature,
            },
        })
    }

    pub fn write_remote_chat_session_data(&mut self, data: &RemoteChatSessionData) {
        self.write_uuid(data.session_id);
        self.write_long(data.profile_public_key.expires_at);
        self.write_byte_array(&data.profile_public_key.key);
        self.write_byte_array(&data.profile_public_key.key_signature);
    }

    pub fn read_nullable_remote_chat_session_data(
        &mut self,
    ) -> Result<Option<RemoteChatSessionData>, ProtocolError> {
        if self.read_boolean()? {
            Ok(Some(self.read_remote_chat_session_data()?))
        } else {
            Ok(None)
        }
    }

    pub fn write_nullable_remote_chat_session_data(
        &mut self,
        data: Option<&RemoteChatSessionData>,
    ) {
        match data {
            Some(d) => {
                self.write_boolean(true);
                self.write_remote_chat_session_data(d);
            }
            None => {
                self.write_boolean(false);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Action {
    AddPlayer = 0,
    InitializeChat = 1,
    UpdateGameMode = 2,
    UpdateListed = 3,
    UpdateLatency = 4,
    UpdateDisplayName = 5,
    UpdateListOrder = 6,
    UpdateHat = 7,
}

impl Action {
    pub const VALUES: [Action; 8] = [
        Action::AddPlayer,
        Action::InitializeChat,
        Action::UpdateGameMode,
        Action::UpdateListed,
        Action::UpdateLatency,
        Action::UpdateDisplayName,
        Action::UpdateListOrder,
        Action::UpdateHat,
    ];

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket$Action#read
    pub fn read(
        &self,
        builder: &mut EntryBuilder,
        buf: &mut RegistryFriendlyByteBuf,
    ) -> Result<(), ProtocolError> {
        match self {
            Action::AddPlayer => {
                let name = buf.read_utf(16)?;
                let properties = buf.read_game_profile_properties()?;
                builder.profile = Some(GameProfile {
                    id: builder.profile_id,
                    name,
                    properties,
                });
            }
            Action::InitializeChat => {
                builder.chat_session = buf.read_nullable_remote_chat_session_data()?;
            }
            Action::UpdateGameMode => {
                builder.game_mode = GameType::by_id(buf.read_var_int()?);
            }
            Action::UpdateListed => {
                builder.listed = buf.read_boolean()?;
            }
            Action::UpdateLatency => {
                builder.latency = buf.read_var_int()?;
            }
            Action::UpdateDisplayName => {
                builder.display_name = buf.read_nullable_component()?;
            }
            Action::UpdateListOrder => {
                builder.list_order = buf.read_var_int()?;
            }
            Action::UpdateHat => {
                builder.show_hat = buf.read_boolean()?;
            }
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket$Action#write
    pub fn write(
        &self,
        buf: &mut RegistryFriendlyByteBuf,
        entry: &Entry,
    ) -> Result<(), ProtocolError> {
        match self {
            Action::AddPlayer => {
                let profile = entry
                    .profile
                    .as_ref()
                    .ok_or(ProtocolError::MissingField("profile"))?;
                buf.write_utf(&profile.name);
                buf.write_game_profile_properties(&profile.properties);
            }
            Action::InitializeChat => {
                buf.write_nullable_remote_chat_session_data(entry.chat_session.as_ref());
            }
            Action::UpdateGameMode => {
                buf.write_var_int(entry.game_mode.get_id());
            }
            Action::UpdateListed => {
                buf.write_boolean(entry.listed);
            }
            Action::UpdateLatency => {
                buf.write_var_int(entry.latency);
            }
            Action::UpdateDisplayName => {
                buf.write_nullable_component(entry.display_name.as_ref());
            }
            Action::UpdateListOrder => {
                buf.write_var_int(entry.list_order);
            }
            Action::UpdateHat => {
                buf.write_boolean(entry.show_hat);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub profile_id: Uuid,
    pub profile: Option<GameProfile>,
    pub listed: bool,
    pub latency: i32,
    pub game_mode: GameType,
    pub display_name: Option<Component>,
    pub show_hat: bool,
    pub list_order: i32,
    pub chat_session: Option<RemoteChatSessionData>,
}

impl Entry {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket$Entry#<init>(ServerPlayer)
    pub fn from_server_player(player: &ServerPlayer) -> Self {
        Self {
            profile_id: player.uuid,
            profile: Some(player.game_profile.clone()),
            listed: true,
            latency: player.latency,
            game_mode: player.game_mode,
            display_name: player.tab_list_display_name.clone(),
            show_hat: player.show_hat,
            list_order: player.tab_list_order,
            chat_session: player.chat_session.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryBuilder {
    pub profile_id: Uuid,
    pub profile: Option<GameProfile>,
    pub listed: bool,
    pub latency: i32,
    pub game_mode: GameType,
    pub display_name: Option<Component>,
    pub show_hat: bool,
    pub list_order: i32,
    pub chat_session: Option<RemoteChatSessionData>,
}

impl EntryBuilder {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket$EntryBuilder#<init>
    pub fn new(profile_id: Uuid) -> Self {
        Self {
            profile_id,
            profile: None,
            listed: false,
            latency: 0,
            game_mode: GameType::DEFAULT_MODE,
            display_name: None,
            show_hat: false,
            list_order: 0,
            chat_session: None,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket$EntryBuilder#build
    pub fn build(self) -> Entry {
        Entry {
            profile_id: self.profile_id,
            profile: self.profile,
            listed: self.listed,
            latency: self.latency,
            game_mode: self.game_mode,
            display_name: self.display_name,
            show_hat: self.show_hat,
            list_order: self.list_order,
            chat_session: self.chat_session,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerInfoUpdatePacket {
    pub actions: BTreeSet<Action>,
    pub entries: Vec<Entry>,
}

impl ClientboundPlayerInfoUpdatePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#<init>(EnumSet, Collection)
    pub fn new(actions: BTreeSet<Action>, players: &[ServerPlayer]) -> Self {
        let entries = players.iter().map(Entry::from_server_player).collect();
        Self { actions, entries }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#<init>(Action, ServerPlayer)
    pub fn single(action: Action, player: &ServerPlayer) -> Self {
        let mut actions = BTreeSet::new();
        actions.insert(action);
        Self {
            actions,
            entries: vec![Entry::from_server_player(player)],
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#createPlayerInitializing
    pub fn create_player_initializing(players: &[ServerPlayer]) -> Self {
        let mut actions = BTreeSet::new();
        actions.insert(Action::AddPlayer);
        actions.insert(Action::InitializeChat);
        actions.insert(Action::UpdateGameMode);
        actions.insert(Action::UpdateListed);
        actions.insert(Action::UpdateLatency);
        actions.insert(Action::UpdateDisplayName);
        actions.insert(Action::UpdateHat);
        actions.insert(Action::UpdateListOrder);
        Self::new(actions, players)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Result<Self, ProtocolError> {
        let actions = buf.read_enum_set_action()?;
        let count = buf.read_var_int()? as usize;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let profile_id = buf.read_uuid()?;
            let mut builder = EntryBuilder::new(profile_id);
            for action in &actions {
                action.read(&mut builder, buf)?;
            }
            entries.push(builder.build());
        }
        Ok(Self { actions, entries })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) -> Result<(), ProtocolError> {
        buf.write_enum_set_action(&self.actions);
        buf.write_var_int(self.entries.len() as i32);
        for entry in &self.entries {
            buf.write_uuid(entry.profile_id);
            for action in &self.actions {
                action.write(buf, entry)?;
            }
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:player_info_update"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_player_info_update(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#actions
    pub fn actions(&self) -> &BTreeSet<Action> {
        &self.actions
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#entries
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#newEntries
    pub fn new_entries(&self) -> Vec<&Entry> {
        if self.actions.contains(&Action::AddPlayer) {
            self.entries.iter().collect()
        } else {
            Vec::new()
        }
    }
}

impl fmt::Display for ClientboundPlayerInfoUpdatePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundPlayerInfoUpdatePacket{{actions={:?}, entries={:?}}}",
            self.actions, self.entries
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_uuid(a: i64, b: i64) -> Uuid {
        Uuid::new(a, b)
    }

    fn sample_player() -> ServerPlayer {
        ServerPlayer {
            uuid: sample_uuid(0x0123_4567_89ab_cdef, -1),
            game_profile: GameProfile {
                id: sample_uuid(0x0123_4567_89ab_cdef, -1),
                name: "Steve".to_string(),
                properties: PropertyMap(vec![Property {
                    name: "textures".to_string(),
                    value: "abc".to_string(),
                    signature: Some("sig".to_string()),
                }]),
            },
            latency: -42,
            game_mode: GameType::Creative,
            tab_list_display_name: Some(Component("name".to_string())),
            show_hat: true,
            tab_list_order: i32::MIN,
            chat_session: Some(RemoteChatSessionData {
                session_id: sample_uuid(7, 8),
                profile_public_key: ProfilePublicKeyData {
                    expires_at: i64::MIN,
                    key: vec![1, 2, 3],
                    key_signature: vec![9, 8],
                },
            }),
        }
    }

    #[test]
    fn enum_set_round_trip_all_actions() {
        let mut buf = RegistryFriendlyByteBuf::new();
        let mut actions = BTreeSet::new();
        for &a in &Action::VALUES {
            actions.insert(a);
        }
        buf.write_enum_set_action(&actions);
        assert_eq!(buf.as_slice(), &[0xFF]);
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = read_buf.read_enum_set_action().unwrap();
        assert_eq!(decoded, actions);
    }

    #[test]
    fn varint_round_trip_negative_and_extremes() {
        let values = [0i32, 1, -1, 127, 128, i32::MAX, i32::MIN, -42];
        for v in values {
            let mut buf = RegistryFriendlyByteBuf::new();
            buf.write_var_int(v);
            let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_vec());
            let decoded = read_buf.read_var_int().unwrap();
            assert_eq!(decoded, v, "varint mismatch for {v}");
        }
    }

    #[test]
    fn packet_round_trip_single_update_latency() {
        let player = sample_player();
        let packet = ClientboundPlayerInfoUpdatePacket::single(Action::UpdateLatency, &player);
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf).unwrap();

        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundPlayerInfoUpdatePacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.actions.len(), 1);
        assert!(decoded.actions.contains(&Action::UpdateLatency));
        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(decoded.entries[0].latency, -42);
        assert_eq!(decoded.entries[0].profile_id, player.uuid);
    }

    #[test]
    fn new_entries_only_when_add_player_present() {
        let player = sample_player();
        let with_add = ClientboundPlayerInfoUpdatePacket::single(Action::AddPlayer, &player);
        assert_eq!(with_add.new_entries().len(), 1);

        let without_add = ClientboundPlayerInfoUpdatePacket::single(Action::UpdateListed, &player);
        assert!(without_add.new_entries().is_empty());
    }

    #[test]
    fn create_player_initializing_has_expected_actions() {
        let player = sample_player();
        let packet = ClientboundPlayerInfoUpdatePacket::create_player_initializing(&[player]);
        let expected: BTreeSet<Action> = [
            Action::AddPlayer,
            Action::InitializeChat,
            Action::UpdateGameMode,
            Action::UpdateListed,
            Action::UpdateLatency,
            Action::UpdateDisplayName,
            Action::UpdateHat,
            Action::UpdateListOrder,
        ]
        .into_iter()
        .collect();
        assert_eq!(packet.actions, expected);
        assert_eq!(packet.entries.len(), 1);
        assert!(packet.entries[0].listed);
    }

    #[test]
    fn packet_round_trip_add_player_with_properties_and_chat() {
        let player = sample_player();
        let packet = ClientboundPlayerInfoUpdatePacket::create_player_initializing(&[player.clone()]);
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf).unwrap();

        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundPlayerInfoUpdatePacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded, packet);
        let entry = &decoded.entries[0];
        let profile = entry.profile.as_ref().unwrap();
        assert_eq!(profile.name, "Steve");
        assert_eq!(profile.properties.0.len(), 1);
        assert_eq!(profile.properties.0[0].signature.as_deref(), Some("sig"));
        assert_eq!(entry.list_order, i32::MIN);
        assert_eq!(
            entry.chat_session.as_ref().unwrap().profile_public_key.expires_at,
            i64::MIN
        );
    }
}
