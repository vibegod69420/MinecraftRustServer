use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    UnexpectedEof,
    VarIntTooBig,
    InvalidEnumOrdinal(i32),
    InvalidUtf8,
    StringTooLong(usize),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of buffer"),
            Self::VarIntTooBig => write!(f, "VarInt too big"),
            Self::InvalidEnumOrdinal(v) => write!(f, "invalid enum ordinal: {v}"),
            Self::InvalidUtf8 => write!(f, "invalid utf8"),
            Self::StringTooLong(len) => write!(f, "string too long: {len}"),
        }
    }
}

impl std::error::Error for DecodeError {}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_i32(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut value: u32 = 0;
    let mut position: u32 = 0;
    loop {
        if buf.is_empty() {
            return Err(DecodeError::UnexpectedEof);
        }
        let byte = buf[0];
        *buf = &buf[1..];
        value |= ((byte & 0x7F) as u32) << position;
        if (byte & 0x80) == 0 {
            break;
        }
        position += 7;
        if position >= 32 {
            return Err(DecodeError::VarIntTooBig);
        }
    }
    Ok(value as i32)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_i32(buf: &mut Vec<u8>, mut value: i32) {
    loop {
        if (value & !0x7F) == 0 {
            buf.push(value as u8);
            return;
        }
        buf.push(((value & 0x7F) as u8) | 0x80);
        value = ((value as u32) >> 7) as i32;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
fn read_utf(buf: &mut &[u8], max_length: usize) -> Result<String, DecodeError> {
    let max_bytes = max_length.saturating_mul(3);
    let len = read_var_i32(buf)? as usize;
    if len > max_bytes {
        return Err(DecodeError::StringTooLong(len));
    }
    if buf.len() < len {
        return Err(DecodeError::UnexpectedEof);
    }
    let bytes = &buf[..len];
    *buf = &buf[len..];
    let s = std::str::from_utf8(bytes)
        .map_err(|_| DecodeError::InvalidUtf8)?
        .to_owned();
    if s.chars().count() > max_length {
        return Err(DecodeError::StringTooLong(s.chars().count()));
    }
    Ok(s)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
fn write_utf(buf: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    write_var_i32(buf, bytes.len() as i32);
    buf.extend_from_slice(bytes);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(pub String);

// JAVA: net.minecraft.network.FriendlyByteBuf#readIdentifier
fn read_identifier(buf: &mut &[u8]) -> Result<Identifier, DecodeError> {
    Ok(Identifier(read_utf(buf, 32767)?))
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeIdentifier
fn write_identifier(buf: &mut Vec<u8>, id: &Identifier) {
    write_utf(buf, &id.0);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementHolder {
    pub id: Identifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeenAdvancementsAction {
    OpenedTab,
    ClosedScreen,
}

impl SeenAdvancementsAction {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket$Action#ordinal
    pub fn ordinal(self) -> i32 {
        match self {
            Self::OpenedTab => 0,
            Self::ClosedScreen => 1,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket$Action#values
    pub fn from_ordinal(ordinal: i32) -> Result<Self, DecodeError> {
        match ordinal {
            0 => Ok(Self::OpenedTab),
            1 => Ok(Self::ClosedScreen),
            _ => Err(DecodeError::InvalidEnumOrdinal(ordinal)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSeenAdvancementsPacket {
    pub action: SeenAdvancementsAction,
    pub tab: Option<Identifier>,
}

impl ServerboundSeenAdvancementsPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket#<init>
    pub fn new(action: SeenAdvancementsAction, tab: Option<Identifier>) -> Self {
        Self { action, tab }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket#openedTab
    pub fn opened_tab(advancement: &AdvancementHolder) -> Self {
        Self::new(
            SeenAdvancementsAction::OpenedTab,
            Some(advancement.id.clone()),
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket#closedScreen
    pub fn closed_screen() -> Self {
        Self::new(SeenAdvancementsAction::ClosedScreen, None)
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let action = SeenAdvancementsAction::from_ordinal(read_var_i32(buf)?)?;
        let tab = if action == SeenAdvancementsAction::OpenedTab {
            Some(read_identifier(buf)?)
        } else {
            None
        };
        Ok(Self { action, tab })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket#write
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_var_i32(buf, self.action.ordinal());
        if self.action == SeenAdvancementsAction::OpenedTab {
            if let Some(tab) = &self.tab {
                write_identifier(buf, tab);
            }
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket#getAction
    pub fn get_action(&self) -> SeenAdvancementsAction {
        self.action
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket#getTab
    pub fn get_tab(&self) -> Option<&Identifier> {
        self.tab.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_opened_tab() {
        let pkt = ServerboundSeenAdvancementsPacket::opened_tab(&AdvancementHolder {
            id: Identifier("minecraft:story/root".to_owned()),
        });
        let mut buf = Vec::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundSeenAdvancementsPacket::read(&mut slice).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_action(), SeenAdvancementsAction::OpenedTab);
        assert_eq!(
            decoded.get_tab(),
            Some(&Identifier("minecraft:story/root".to_owned()))
        );
        assert!(slice.is_empty());
    }

    #[test]
    fn roundtrip_closed_screen() {
        let pkt = ServerboundSeenAdvancementsPacket::closed_screen();
        let mut buf = Vec::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundSeenAdvancementsPacket::read(&mut slice).unwrap();
        assert_eq!(decoded.get_action(), SeenAdvancementsAction::ClosedScreen);
        assert_eq!(decoded.get_tab(), None);
        assert!(slice.is_empty());
    }

    #[test]
    fn invalid_enum_ordinal() {
        let mut buf = Vec::new();
        write_var_i32(&mut buf, 99);
        let mut slice: &[u8] = &buf;
        let err = ServerboundSeenAdvancementsPacket::read(&mut slice).unwrap_err();
        assert_eq!(err, DecodeError::InvalidEnumOrdinal(99));
    }

    #[test]
    fn negative_enum_ordinal() {
        let mut buf = Vec::new();
        write_var_i32(&mut buf, -1);
        let mut slice: &[u8] = &buf;
        let err = ServerboundSeenAdvancementsPacket::read(&mut slice).unwrap_err();
        assert_eq!(err, DecodeError::InvalidEnumOrdinal(-1));
    }

    #[test]
    fn unexpected_eof() {
        let mut slice: &[u8] = &[];
        let err = ServerboundSeenAdvancementsPacket::read(&mut slice).unwrap_err();
        assert_eq!(err, DecodeError::UnexpectedEof);
    }

    #[test]
    fn varint_negative_and_edge_values() {
        for v in [0i32, 1, -1, i32::MIN, i32::MAX, 127, 128, -128, 255, -256] {
            let mut buf = Vec::new();
            write_var_i32(&mut buf, v);
            let mut slice: &[u8] = &buf;
            let decoded = read_var_i32(&mut slice).unwrap();
            assert_eq!(decoded, v);
            assert!(slice.is_empty());
        }
    }
}
