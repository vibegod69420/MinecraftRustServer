use std::io;

#[derive(Debug, thiserror::Error)]
pub enum PacketError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("invalid enum value: {0}")]
    InvalidEnum(String),
    #[error("illegal state: {0}")]
    IllegalState(String),
    #[error("varint too large")]
    VarIntTooLarge,
}

const METHOD_ADD: i32 = 0;
const METHOD_REMOVE: i32 = 1;
const METHOD_CHANGE: i32 = 2;
const METHOD_JOIN: i32 = 3;
const METHOD_LEAVE: i32 = 4;
const MAX_VISIBILITY_LENGTH: usize = 40;
const MAX_COLLISION_LENGTH: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Add,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ChatFormatting {
    Black = 0,
    DarkBlue = 1,
    DarkGreen = 2,
    DarkAqua = 3,
    DarkRed = 4,
    DarkPurple = 5,
    Gold = 6,
    Gray = 7,
    DarkGray = 8,
    Blue = 9,
    Green = 10,
    Aqua = 11,
    Red = 12,
    LightPurple = 13,
    Yellow = 14,
    White = 15,
    Obfuscated = 16,
    Bold = 17,
    Strikethrough = 18,
    Underline = 19,
    Italic = 20,
    Reset = 21,
}

impl ChatFormatting {
    pub fn from_ordinal(ordinal: i32) -> Result<Self, PacketError> {
        match ordinal {
            0 => Ok(Self::Black),
            1 => Ok(Self::DarkBlue),
            2 => Ok(Self::DarkGreen),
            3 => Ok(Self::DarkAqua),
            4 => Ok(Self::DarkRed),
            5 => Ok(Self::DarkPurple),
            6 => Ok(Self::Gold),
            7 => Ok(Self::Gray),
            8 => Ok(Self::DarkGray),
            9 => Ok(Self::Blue),
            10 => Ok(Self::Green),
            11 => Ok(Self::Aqua),
            12 => Ok(Self::Red),
            13 => Ok(Self::LightPurple),
            14 => Ok(Self::Yellow),
            15 => Ok(Self::White),
            16 => Ok(Self::Obfuscated),
            17 => Ok(Self::Bold),
            18 => Ok(Self::Strikethrough),
            19 => Ok(Self::Underline),
            20 => Ok(Self::Italic),
            21 => Ok(Self::Reset),
            _ => Err(PacketError::InvalidEnum(format!(
                "ChatFormatting ordinal {}",
                ordinal
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Always,
    Never,
    HideForOtherTeams,
    HideForOwnTeam,
}

impl Visibility {
    pub fn as_name(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::HideForOtherTeams => "hideForOtherTeams",
            Self::HideForOwnTeam => "hideForOwnTeam",
        }
    }

    pub fn from_name(name: &str) -> Result<Self, PacketError> {
        match name {
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            "hideForOtherTeams" => Ok(Self::HideForOtherTeams),
            "hideForOwnTeam" => Ok(Self::HideForOwnTeam),
            _ => Err(PacketError::InvalidEnum(format!("Visibility name {}", name))),
        }
    }

    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Result<Self, PacketError> {
        let name = buf.read_utf(MAX_VISIBILITY_LENGTH)?;
        Self::from_name(&name)
    }

    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) -> Result<(), PacketError> {
        buf.write_utf(self.as_name(), MAX_VISIBILITY_LENGTH)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionRule {
    Always,
    Never,
    PushOtherTeams,
    PushOwnTeam,
}

impl CollisionRule {
    pub fn as_name(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::PushOtherTeams => "pushOtherTeams",
            Self::PushOwnTeam => "pushOwnTeam",
        }
    }

    pub fn from_name(name: &str) -> Result<Self, PacketError> {
        match name {
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            "pushOtherTeams" => Ok(Self::PushOtherTeams),
            "pushOwnTeam" => Ok(Self::PushOwnTeam),
            _ => Err(PacketError::InvalidEnum(format!(
                "CollisionRule name {}",
                name
            ))),
        }
    }

    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Result<Self, PacketError> {
        let name = buf.read_utf(MAX_COLLISION_LENGTH)?;
        Self::from_name(&name)
    }

    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) -> Result<(), PacketError> {
        buf.write_utf(self.as_name(), MAX_COLLISION_LENGTH)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn read_trusted(buf: &mut RegistryFriendlyByteBuf) -> Result<Self, PacketError> {
        let text = buf.read_utf(262_144)?;
        Ok(Self { text })
    }

    pub fn write_trusted(&self, buf: &mut RegistryFriendlyByteBuf) -> Result<(), PacketError> {
        buf.write_utf(&self.text, 262_144)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn read_byte(&mut self) -> Result<i8, PacketError> {
        if self.read_pos >= self.data.len() {
            return Err(PacketError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "read_byte",
            )));
        }
        let b = self.data[self.read_pos] as i8;
        self.read_pos = self.read_pos.wrapping_add(1);
        Ok(b)
    }

    pub fn write_byte(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    pub fn read_varint(&mut self) -> Result<i32, PacketError> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.read_byte()? as u8 as u32;
            result |= (byte & 0x7F).wrapping_shl(shift);
            if (byte & 0x80) == 0 {
                return Ok(result as i32);
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                return Err(PacketError::VarIntTooLarge);
            }
        }
    }

    pub fn write_varint(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7Fu32) == 0 {
                self.data.push(val as u8);
                return;
            }
            self.data.push(((val & 0x7F) as u8) | 0x80);
            val >>= 7;
        }
    }

    pub fn read_utf(&mut self, max_len: usize) -> Result<String, PacketError> {
        let max_bytes = max_len.saturating_mul(4).saturating_add(3);
        let byte_len_i32 = self.read_varint()?;
        if byte_len_i32 < 0 {
            return Err(PacketError::IllegalState(
                "The received encoded string buffer length is less than zero! Weird string!"
                    .to_string(),
            ));
        }
        let byte_len = byte_len_i32 as usize;
        if byte_len > max_bytes {
            return Err(PacketError::IllegalState(format!(
                "The received encoded string buffer length is longer than maximum allowed ({} > {})",
                byte_len, max_bytes
            )));
        }
        let end = self.read_pos.checked_add(byte_len).ok_or_else(|| {
            PacketError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "read_utf overflow",
            ))
        })?;
        if end > self.data.len() {
            return Err(PacketError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "read_utf",
            )));
        }
        let bytes = self.data[self.read_pos..end].to_vec();
        let s = String::from_utf8(bytes)?;
        if s.encode_utf16().count() > max_len {
            return Err(PacketError::IllegalState(format!(
                "The received string length is longer than maximum allowed ({} > {})",
                s.encode_utf16().count(),
                max_len
            )));
        }
        self.read_pos = end;
        Ok(s)
    }

    pub fn write_utf(&mut self, value: &str, max_len: usize) -> Result<(), PacketError> {
        let char_count = value.encode_utf16().count();
        if char_count > max_len {
            return Err(PacketError::IllegalState(format!(
                "String too big (was {} characters, max {})",
                char_count, max_len
            )));
        }
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
        Ok(())
    }

    pub fn read_list<T, F>(&mut self, mut reader: F) -> Result<Vec<T>, PacketError>
    where
        F: FnMut(&mut Self) -> Result<T, PacketError>,
    {
        let len = self.read_varint()?;
        if len < 0 {
            return Err(PacketError::IllegalState(format!(
                "Negative list size: {}",
                len
            )));
        }
        let mut out = Vec::with_capacity(len as usize);
        let mut i: i32 = 0;
        while i < len {
            out.push(reader(self)?);
            i = i.wrapping_add(1);
        }
        Ok(out)
    }

    pub fn write_collection<T, F>(&mut self, values: &[T], mut writer: F) -> Result<(), PacketError>
    where
        F: FnMut(&mut Self, &T) -> Result<(), PacketError>,
    {
        self.write_varint(values.len() as i32);
        for v in values {
            writer(self, v)?;
        }
        Ok(())
    }

    pub fn read_enum_chat_formatting(&mut self) -> Result<ChatFormatting, PacketError> {
        let ordinal = self.read_varint()?;
        ChatFormatting::from_ordinal(ordinal)
    }

    pub fn write_enum_chat_formatting(&mut self, value: ChatFormatting) {
        self.write_varint(value as i32);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerTeamData {
    pub name: String,
    pub display_name: Component,
    pub player_prefix: Component,
    pub player_suffix: Component,
    pub name_tag_visibility: Visibility,
    pub collision_rule: CollisionRule,
    pub color: ChatFormatting,
    pub options: i32,
    pub players: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub trait ClientGamePacketListener {
    fn handle_set_player_team_packet(&mut self, packet: &ClientboundSetPlayerTeamPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameters {
    display_name: Component,
    player_prefix: Component,
    player_suffix: Component,
    nametag_visibility: Visibility,
    collision_rule: CollisionRule,
    color: ChatFormatting,
    options: i32,
}

impl Parameters {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#<init>
    pub fn from_team(team: &PlayerTeamData) -> Self {
        Self {
            display_name: team.display_name.clone(),
            options: team.options,
            nametag_visibility: team.name_tag_visibility,
            collision_rule: team.collision_rule,
            color: team.color,
            player_prefix: team.player_prefix.clone(),
            player_suffix: team.player_suffix.clone(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#<init>
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Result<Self, PacketError> {
        let display_name = Component::read_trusted(buf)?;
        let options = buf.read_byte()? as i32;
        let nametag_visibility = Visibility::read(buf)?;
        let collision_rule = CollisionRule::read(buf)?;
        let color = buf.read_enum_chat_formatting()?;
        let player_prefix = Component::read_trusted(buf)?;
        let player_suffix = Component::read_trusted(buf)?;
        Ok(Self {
            display_name,
            player_prefix,
            player_suffix,
            nametag_visibility,
            collision_rule,
            color,
            options,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#getDisplayName
    pub fn get_display_name(&self) -> &Component {
        &self.display_name
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#getOptions
    pub fn get_options(&self) -> i32 {
        self.options
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#getColor
    pub fn get_color(&self) -> ChatFormatting {
        self.color
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#getNametagVisibility
    pub fn get_nametag_visibility(&self) -> Visibility {
        self.nametag_visibility
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#getCollisionRule
    pub fn get_collision_rule(&self) -> CollisionRule {
        self.collision_rule
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#getPlayerPrefix
    pub fn get_player_prefix(&self) -> &Component {
        &self.player_prefix
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#getPlayerSuffix
    pub fn get_player_suffix(&self) -> &Component {
        &self.player_suffix
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket$Parameters#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) -> Result<(), PacketError> {
        self.display_name.write_trusted(buf)?;
        buf.write_byte(self.options as i8);
        self.nametag_visibility.write(buf)?;
        self.collision_rule.write(buf)?;
        buf.write_enum_chat_formatting(self.color);
        self.player_prefix.write_trusted(buf)?;
        self.player_suffix.write_trusted(buf)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetPlayerTeamPacket {
    method: i32,
    name: String,
    players: Vec<String>,
    parameters: Option<Parameters>,
}

impl ClientboundSetPlayerTeamPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#<init>
    fn new(
        name: String,
        method: i32,
        parameters: Option<Parameters>,
        players: Vec<String>,
    ) -> Self {
        Self {
            name,
            method,
            parameters,
            players,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#createAddOrModifyPacket
    pub fn create_add_or_modify_packet(team: &PlayerTeamData, add: bool) -> Self {
        Self::new(
            team.name.clone(),
            if add { METHOD_ADD } else { METHOD_CHANGE },
            Some(Parameters::from_team(team)),
            if add {
                team.players.clone()
            } else {
                Vec::new()
            },
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#createRemovePacket
    pub fn create_remove_packet(team: &PlayerTeamData) -> Self {
        Self::new(team.name.clone(), METHOD_REMOVE, None, Vec::new())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#createPlayerPacket
    pub fn create_player_packet(team: &PlayerTeamData, player: String, action: Action) -> Self {
        Self::new(
            team.name.clone(),
            if action == Action::Add {
                METHOD_JOIN
            } else {
                METHOD_LEAVE
            },
            None,
            vec![player],
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#<init>
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Result<Self, PacketError> {
        let name = buf.read_utf(32767)?;
        let method = buf.read_byte()? as i32;
        let parameters = if Self::should_have_parameters(method) {
            Some(Parameters::read(buf)?)
        } else {
            None
        };
        let players = if Self::should_have_player_list(method) {
            buf.read_list(|b| b.read_utf(32767))?
        } else {
            Vec::new()
        };
        Ok(Self {
            method,
            name,
            players,
            parameters,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) -> Result<(), PacketError> {
        buf.write_utf(&self.name, 32767)?;
        buf.write_byte(self.method as i8);
        if Self::should_have_parameters(self.method) {
            let params = self.parameters.as_ref().ok_or_else(|| {
                PacketError::IllegalState(format!(
                    "Parameters not present, but method is{}",
                    self.method
                ))
            })?;
            params.write(buf)?;
        }
        if Self::should_have_player_list(self.method) {
            buf.write_collection(&self.players, |b, p| b.write_utf(p, 32767))?;
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#shouldHavePlayerList
    fn should_have_player_list(method: i32) -> bool {
        method == METHOD_ADD || method == METHOD_JOIN || method == METHOD_LEAVE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#shouldHaveParameters
    fn should_have_parameters(method: i32) -> bool {
        method == METHOD_ADD || method == METHOD_CHANGE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#getPlayerAction
    pub fn get_player_action(&self) -> Option<Action> {
        match self.method {
            METHOD_ADD | METHOD_JOIN => Some(Action::Add),
            METHOD_LEAVE => Some(Action::Remove),
            _ => None,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#getTeamAction
    pub fn get_team_action(&self) -> Option<Action> {
        match self.method {
            METHOD_ADD => Some(Action::Add),
            METHOD_REMOVE => Some(Action::Remove),
            _ => None,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_set_player_team_packet(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#getName
    pub fn get_name(&self) -> &str {
        &self.name
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#getPlayers
    pub fn get_players(&self) -> &[String] {
        &self.players
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket#getParameters
    pub fn get_parameters(&self) -> Option<&Parameters> {
        self.parameters.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_team() -> PlayerTeamData {
        PlayerTeamData {
            name: "red".to_string(),
            display_name: Component::literal("Red Team"),
            player_prefix: Component::literal("[R] "),
            player_suffix: Component::literal(" !"),
            name_tag_visibility: Visibility::HideForOtherTeams,
            collision_rule: CollisionRule::PushOwnTeam,
            color: ChatFormatting::Red,
            options: 3,
            players: vec!["Alice".to_string(), "Bob".to_string()],
        }
    }

    fn round_trip(pkt: &ClientboundSetPlayerTeamPacket) -> ClientboundSetPlayerTeamPacket {
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        ClientboundSetPlayerTeamPacket::read(&mut read_buf).unwrap()
    }

    #[test]
    fn round_trip_add_packet() {
        let team = sample_team();
        let pkt = ClientboundSetPlayerTeamPacket::create_add_or_modify_packet(&team, true);
        let decoded = round_trip(&pkt);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_player_action(), Some(Action::Add));
        assert_eq!(decoded.get_team_action(), Some(Action::Add));
        assert_eq!(decoded.get_name(), "red");
        assert_eq!(
            decoded.get_players(),
            &["Alice".to_string(), "Bob".to_string()]
        );
        let params = decoded.get_parameters().unwrap();
        assert_eq!(params.get_display_name(), &Component::literal("Red Team"));
        assert_eq!(params.get_options(), 3);
        assert_eq!(params.get_color(), ChatFormatting::Red);
        assert_eq!(
            params.get_nametag_visibility(),
            Visibility::HideForOtherTeams
        );
        assert_eq!(params.get_collision_rule(), CollisionRule::PushOwnTeam);
        assert_eq!(params.get_player_prefix(), &Component::literal("[R] "));
        assert_eq!(params.get_player_suffix(), &Component::literal(" !"));
        assert_eq!(decoded.packet_type(), PacketType);

        struct NoopListener;
        impl ClientGamePacketListener for NoopListener {
            fn handle_set_player_team_packet(
                &mut self,
                _packet: &ClientboundSetPlayerTeamPacket,
            ) {
            }
        }
        let mut listener = NoopListener;
        decoded.handle(&mut listener);
    }

    #[test]
    fn round_trip_change_remove_join_leave() {
        let team = sample_team();

        let change = ClientboundSetPlayerTeamPacket::create_add_or_modify_packet(&team, false);
        assert_eq!(change.get_team_action(), None);
        assert_eq!(change.get_player_action(), None);
        assert!(change.get_players().is_empty());
        assert_eq!(change, round_trip(&change));

        let remove = ClientboundSetPlayerTeamPacket::create_remove_packet(&team);
        assert_eq!(remove.get_team_action(), Some(Action::Remove));
        assert_eq!(remove.get_player_action(), None);
        assert!(remove.get_players().is_empty());
        assert!(remove.get_parameters().is_none());
        assert_eq!(remove, round_trip(&remove));

        let join = ClientboundSetPlayerTeamPacket::create_player_packet(
            &team,
            "Steve".to_string(),
            Action::Add,
        );
        assert_eq!(join.get_player_action(), Some(Action::Add));
        assert_eq!(join.get_team_action(), None);
        assert_eq!(join.get_players(), &["Steve".to_string()]);
        assert_eq!(join, round_trip(&join));

        let leave = ClientboundSetPlayerTeamPacket::create_player_packet(
            &team,
            "Alex".to_string(),
            Action::Remove,
        );
        assert_eq!(leave.get_player_action(), Some(Action::Remove));
        assert_eq!(leave.get_team_action(), None);
        assert_eq!(leave, round_trip(&leave));
    }

    #[test]
    fn negative_options_and_byte_edges() {
        let mut team = sample_team();
        team.options = -128;
        let decoded =
            round_trip(&ClientboundSetPlayerTeamPacket::create_add_or_modify_packet(
                &team, false,
            ));
        assert_eq!(decoded.get_parameters().unwrap().get_options(), -128);

        team.options = 127;
        let decoded =
            round_trip(&ClientboundSetPlayerTeamPacket::create_add_or_modify_packet(
                &team, false,
            ));
        assert_eq!(decoded.get_parameters().unwrap().get_options(), 127);

        team.options = -1;
        let decoded =
            round_trip(&ClientboundSetPlayerTeamPacket::create_add_or_modify_packet(
                &team, false,
            ));
        assert_eq!(decoded.get_parameters().unwrap().get_options(), -1);

        team.options = 0;
        let decoded =
            round_trip(&ClientboundSetPlayerTeamPacket::create_add_or_modify_packet(
                &team, false,
            ));
        assert_eq!(decoded.get_parameters().unwrap().get_options(), 0);
    }

    #[test]
    fn varint_negative_and_extreme_bits() {
        let mut buf = RegistryFriendlyByteBuf::new();
        let values: [i32; 8] = [0, -1, i32::MIN, i32::MAX, 1, -2, 128, -128];
        for &v in &values {
            buf.write_varint(v);
        }
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        for &v in &values {
            let got = read_buf.read_varint().unwrap();
            assert_eq!(got, v, "varint round-trip failed for {}", v);
        }
    }

    #[test]
    fn method_flags_unknown_and_missing_params_error() {
        assert!(ClientboundSetPlayerTeamPacket::should_have_parameters(
            METHOD_ADD
        ));
        assert!(!ClientboundSetPlayerTeamPacket::should_have_parameters(
            METHOD_REMOVE
        ));
        assert!(ClientboundSetPlayerTeamPacket::should_have_parameters(
            METHOD_CHANGE
        ));
        assert!(!ClientboundSetPlayerTeamPacket::should_have_parameters(
            METHOD_JOIN
        ));
        assert!(!ClientboundSetPlayerTeamPacket::should_have_parameters(
            METHOD_LEAVE
        ));

        assert!(ClientboundSetPlayerTeamPacket::should_have_player_list(
            METHOD_ADD
        ));
        assert!(!ClientboundSetPlayerTeamPacket::should_have_player_list(
            METHOD_REMOVE
        ));
        assert!(!ClientboundSetPlayerTeamPacket::should_have_player_list(
            METHOD_CHANGE
        ));
        assert!(ClientboundSetPlayerTeamPacket::should_have_player_list(
            METHOD_JOIN
        ));
        assert!(ClientboundSetPlayerTeamPacket::should_have_player_list(
            METHOD_LEAVE
        ));

        let pkt = ClientboundSetPlayerTeamPacket {
            method: 99,
            name: "x".to_string(),
            players: Vec::new(),
            parameters: None,
        };
        assert_eq!(pkt.get_player_action(), None);
        assert_eq!(pkt.get_team_action(), None);

        let pkt_neg = ClientboundSetPlayerTeamPacket {
            method: -1,
            name: "x".to_string(),
            players: Vec::new(),
            parameters: None,
        };
        assert_eq!(pkt_neg.get_player_action(), None);
        assert_eq!(pkt_neg.get_team_action(), None);

        let bad = ClientboundSetPlayerTeamPacket {
            method: METHOD_ADD,
            name: "team".to_string(),
            players: vec!["p".to_string()],
            parameters: None,
        };
        let mut buf = RegistryFriendlyByteBuf::new();
        let err = bad.write(&mut buf).unwrap_err();
        match err {
            PacketError::IllegalState(msg) => {
                assert_eq!(msg, "Parameters not present, but method is0");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn float_bit_patterns_preserved() {
        let f32_bits: [u32; 4] = [0x0000_0000, 0x8000_0000, 0x7FC0_0001, 0xFFC0_1234];
        for bits in f32_bits {
            let f = f32::from_bits(bits);
            let be = f.to_be_bytes();
            let got = f32::from_be_bytes(be);
            assert_eq!(got.to_bits(), bits);
        }
        let f64_bits: [u64; 4] = [
            0x0000_0000_0000_0000,
            0x8000_0000_0000_0000,
            0x7FF8_0000_0000_0001,
            0xFFF0_0000_0000_0001,
        ];
        for bits in f64_bits {
            let d = f64::from_bits(bits);
            let be = d.to_be_bytes();
            let got = f64::from_be_bytes(be);
            assert_eq!(got.to_bits(), bits);
        }
    }
}
