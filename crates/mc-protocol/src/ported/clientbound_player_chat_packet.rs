//! Port of `net.minecraft.network.protocol.game.ClientboundPlayerChatPacket`.

use std::fmt;
use std::io;

/// Minimal decode error for self-contained packet codecs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    pub message: String,
}

impl DecodeError {
    fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecodeError {}

impl From<io::Error> for DecodeError {
    fn from(e: io::Error) -> Self {
        Self::new(e.to_string())
    }
}

fn read_byte(buf: &mut &[u8]) -> Result<u8, DecodeError> {
    if buf.is_empty() {
        return Err(DecodeError::new("EOF reading byte"));
    }
    let b = buf[0];
    *buf = &buf[1..];
    Ok(b)
}

fn read_i32_be(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    if buf.len() < 4 {
        return Err(DecodeError::new("EOF reading int"));
    }
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&buf[..4]);
    *buf = &buf[4..];
    Ok(i32::from_be_bytes(bytes))
}

fn write_i32_be(buf: &mut Vec<u8>, value: i32) {
    buf.extend_from_slice(&value.to_be_bytes());
}

fn read_i64_be(buf: &mut &[u8]) -> Result<i64, DecodeError> {
    if buf.len() < 8 {
        return Err(DecodeError::new("EOF reading long"));
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buf[..8]);
    *buf = &buf[8..];
    Ok(i64::from_be_bytes(bytes))
}

fn write_i64_be(buf: &mut Vec<u8>, value: i64) {
    buf.extend_from_slice(&value.to_be_bytes());
}

/// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
fn read_varint(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    loop {
        let b = read_byte(buf)? as i8;
        i |= ((b & 127) as i32) << (j.wrapping_mul(7));
        j = j.wrapping_add(1);
        if j > 5 {
            return Err(DecodeError::new("VarInt too big"));
        }
        if (b & -128) != -128 {
            break;
        }
    }
    Ok(i)
}

/// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
fn write_varint(buf: &mut Vec<u8>, mut value: i32) {
    loop {
        let mut b = (value & 0x7F) as u8;
        let u = value as u32;
        value = (u >> 7) as i32;
        if value != 0 {
            b |= 0x80;
        }
        buf.push(b);
        if value == 0 {
            break;
        }
    }
}

/// JAVA: net.minecraft.network.FriendlyByteBuf#readUUID()
fn read_uuid(buf: &mut &[u8]) -> Result<u128, DecodeError> {
    let most = read_i64_be(buf)? as u64;
    let least = read_i64_be(buf)? as u64;
    Ok(((most as u128) << 64) | (least as u128))
}

/// JAVA: net.minecraft.network.FriendlyByteBuf#writeUUID(UUID)
fn write_uuid(buf: &mut Vec<u8>, value: u128) {
    let most = (value >> 64) as u64 as i64;
    let least = (value as u64) as i64;
    write_i64_be(buf, most);
    write_i64_be(buf, least);
}

fn read_bool(buf: &mut &[u8]) -> Result<bool, DecodeError> {
    Ok(read_byte(buf)? != 0)
}

fn write_bool(buf: &mut Vec<u8>, value: bool) {
    buf.push(if value { 1 } else { 0 });
}

fn read_bytes_fixed(buf: &mut &[u8], len: usize) -> Result<Vec<u8>, DecodeError> {
    if buf.len() < len {
        return Err(DecodeError::new("EOF reading fixed bytes"));
    }
    let out = buf[..len].to_vec();
    *buf = &buf[len..];
    Ok(out)
}

fn read_byte_array(buf: &mut &[u8]) -> Result<Vec<u8>, DecodeError> {
    let len = read_varint(buf)?;
    if len < 0 {
        return Err(DecodeError::new("Negative byte array length"));
    }
    let len = len as usize;
    if buf.len() < len {
        return Err(DecodeError::new("EOF reading byte array"));
    }
    let out = buf[..len].to_vec();
    *buf = &buf[len..];
    Ok(out)
}

fn write_byte_array(buf: &mut Vec<u8>, bytes: &[u8]) {
    write_varint(buf, bytes.len() as i32);
    buf.extend_from_slice(bytes);
}

/// JAVA: net.minecraft.network.chat.MessageSignature
/// Self-contained representation: 256-byte signature payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignature {
    pub bytes: [u8; 256],
}

impl MessageSignature {
    /// JAVA: net.minecraft.network.chat.MessageSignature#read(net.minecraft.network.FriendlyByteBuf)
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let bytes_vec = read_bytes_fixed(buf, 256)?;
        let mut bytes = [0u8; 256];
        bytes.copy_from_slice(&bytes_vec);
        Ok(Self { bytes })
    }

    /// JAVA: net.minecraft.network.chat.MessageSignature#write(net.minecraft.network.FriendlyByteBuf, net.minecraft.network.chat.MessageSignature)
    pub fn write(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.bytes);
    }
}

/// JAVA: net.minecraft.network.chat.SignedMessageBody.Packed
/// Minimal self-contained packed body: content string, timestamp, salt, lastSeen signature list.
/// This preserves wire structure for the common representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageBodyPacked {
    pub content: String,
    pub timestamp_millis: i64,
    pub salt: i64,
    pub last_seen: Vec<MessageSignature>,
}

impl SignedMessageBodyPacked {
    /// JAVA: net.minecraft.network.chat.SignedMessageBody$Packed#<init>(net.minecraft.network.FriendlyByteBuf)
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let content_bytes = read_byte_array(buf)?;
        let content = String::from_utf8(content_bytes)
            .map_err(|e| DecodeError::new(format!("Invalid UTF-8 content: {e}")))?;
        let timestamp_millis = read_i64_be(buf)?;
        let salt = read_i64_be(buf)?;
        let last_seen_count = read_varint(buf)?;
        if last_seen_count < 0 {
            return Err(DecodeError::new("Negative last seen count"));
        }
        let mut last_seen = Vec::with_capacity(last_seen_count as usize);
        for _ in 0..last_seen_count {
            let has_signature = read_bool(buf)?;
            if has_signature {
                last_seen.push(MessageSignature::read(buf)?);
            } else {
                let id = read_varint(buf)?;
                // Represent referenced signature id as a zeroed signature marker is not faithful;
                // store as a deterministic placeholder with id encoded in first 4 bytes and a marker.
                let mut bytes = [0u8; 256];
                bytes[0] = 0xFE;
                bytes[1..5].copy_from_slice(&id.to_be_bytes());
                last_seen.push(MessageSignature { bytes });
            }
        }
        Ok(Self {
            content,
            timestamp_millis,
            salt,
            last_seen,
        })
    }

    /// JAVA: net.minecraft.network.chat.SignedMessageBody$Packed#write(net.minecraft.network.FriendlyByteBuf)
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_byte_array(buf, self.content.as_bytes());
        write_i64_be(buf, self.timestamp_millis);
        write_i64_be(buf, self.salt);
        write_varint(buf, self.last_seen.len() as i32);
        for sig in &self.last_seen {
            // Always write full signature form for self-contained fidelity of this representation.
            write_bool(buf, true);
            sig.write(buf);
        }
    }
}

/// JAVA: net.minecraft.network.chat.Component
/// Self-contained opaque component representation using encoded bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub encoded: Vec<u8>,
}

/// JAVA: net.minecraft.network.chat.FilterMask
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterMask {
    pub mask: Option<Vec<i64>>,
}

impl FilterMask {
    /// JAVA: net.minecraft.network.chat.FilterMask#read(net.minecraft.network.FriendlyByteBuf)
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let has_mask = read_bool(buf)?;
        if !has_mask {
            return Ok(Self { mask: None });
        }
        let len = read_varint(buf)?;
        if len < 0 {
            return Err(DecodeError::new("Negative filter mask length"));
        }
        let mut mask = Vec::with_capacity(len as usize);
        for _ in 0..len {
            mask.push(read_i64_be(buf)?);
        }
        Ok(Self { mask: Some(mask) })
    }

    /// JAVA: net.minecraft.network.chat.FilterMask#write(net.minecraft.network.FriendlyByteBuf, net.minecraft.network.chat.FilterMask)
    pub fn write(buf: &mut Vec<u8>, mask: &FilterMask) {
        match &mask.mask {
            None => write_bool(buf, false),
            Some(words) => {
                write_bool(buf, true);
                write_varint(buf, words.len() as i32);
                for &w in words {
                    write_i64_be(buf, w);
                }
            }
        }
    }
}

/// JAVA: net.minecraft.network.chat.ChatType.Bound
/// Self-contained representation preserving key fields as encoded bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTypeBound {
    pub chat_type_id: i32,
    pub name: Component,
    pub target_name: Option<Component>,
}

impl ChatTypeBound {
    fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let chat_type_id = read_varint(buf)?;
        let name = read_component(buf)?;
        let has_target = read_bool(buf)?;
        let target_name = if has_target {
            Some(read_component(buf)?)
        } else {
            None
        };
        Ok(Self {
            chat_type_id,
            name,
            target_name,
        })
    }

    fn write(&self, buf: &mut Vec<u8>) {
        write_varint(buf, self.chat_type_id);
        write_component(buf, &self.name);
        match &self.target_name {
            Some(target) => {
                write_bool(buf, true);
                write_component(buf, target);
            }
            None => write_bool(buf, false),
        }
    }
}

fn read_component(buf: &mut &[u8]) -> Result<Component, DecodeError> {
    // Self-contained opaque component codec: length-prefixed bytes.
    let encoded = read_byte_array(buf)?;
    Ok(Component { encoded })
}

fn write_component(buf: &mut Vec<u8>, component: &Component) {
    write_byte_array(buf, &component.encoded);
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerChatPacket {
    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#globalIndex
    pub global_index: i32,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#sender
    pub sender: u128,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#index
    pub index: i32,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#signature
    pub signature: Option<MessageSignature>,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#body
    pub body: SignedMessageBodyPacked,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#unsignedContent
    pub unsigned_content: Option<Component>,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#filterMask
    pub filter_mask: FilterMask,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#chatType
    pub chat_type: ChatTypeBound,
}

impl ClientboundPlayerChatPacket {
    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#<init>(net.minecraft.network.RegistryFriendlyByteBuf)
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let global_index = read_varint(buf)?;
        let sender = read_uuid(buf)?;
        let index = read_varint(buf)?;
        let has_signature = read_bool(buf)?;
        let signature = if has_signature {
            Some(MessageSignature::read(buf)?)
        } else {
            None
        };
        let body = SignedMessageBodyPacked::read(buf)?;
        let has_unsigned = read_bool(buf)?;
        let unsigned_content = if has_unsigned {
            Some(read_component(buf)?)
        } else {
            None
        };
        let filter_mask = FilterMask::read(buf)?;
        let chat_type = ChatTypeBound::read(buf)?;
        Ok(Self {
            global_index,
            sender,
            index,
            signature,
            body,
            unsigned_content,
            filter_mask,
            chat_type,
        })
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#write(net.minecraft.network.RegistryFriendlyByteBuf)
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_varint(buf, self.global_index);
        write_uuid(buf, self.sender);
        write_varint(buf, self.index);
        match &self.signature {
            Some(sig) => {
                write_bool(buf, true);
                sig.write(buf);
            }
            None => write_bool(buf, false),
        }
        self.body.write(buf);
        match &self.unsigned_content {
            Some(component) => {
                write_bool(buf, true);
                write_component(buf, component);
            }
            None => write_bool(buf, false),
        }
        FilterMask::write(buf, &self.filter_mask);
        self.chat_type.write(buf);
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#type()
    pub fn type_id(&self) -> &'static str {
        "minecraft:player_chat"
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#handle(net.minecraft.network.protocol.game.ClientGamePacketListener)
    pub fn handle<H: FnOnce(&Self)>(&self, handler: H) {
        handler(self);
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerChatPacket#isSkippable()
    pub fn is_skippable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_packet() -> ClientboundPlayerChatPacket {
        ClientboundPlayerChatPacket {
            global_index: 10,
            sender: 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210u128,
            index: -1,
            signature: Some(MessageSignature { bytes: [0xAB; 256] }),
            body: SignedMessageBodyPacked {
                content: "hello".to_string(),
                timestamp_millis: 1_700_000_000_000i64,
                salt: -42i64,
                last_seen: vec![],
            },
            unsigned_content: Some(Component {
                encoded: vec![1, 2, 3],
            }),
            filter_mask: FilterMask {
                mask: Some(vec![0i64, -1i64]),
            },
            chat_type: ChatTypeBound {
                chat_type_id: 3,
                name: Component {
                    encoded: vec![9, 8],
                },
                target_name: None,
            },
        }
    }

    #[test]
    fn roundtrip_player_chat_packet() {
        let packet = sample_packet();
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundPlayerChatPacket::read(&mut slice).expect("read");
        assert!(slice.is_empty());
        assert_eq!(decoded, packet);
        assert!(decoded.is_skippable());
        assert_eq!(decoded.type_id(), "minecraft:player_chat");
    }

    #[test]
    fn roundtrip_with_nullables_absent() {
        let packet = ClientboundPlayerChatPacket {
            global_index: 0,
            sender: 0,
            index: 0,
            signature: None,
            body: SignedMessageBodyPacked {
                content: String::new(),
                timestamp_millis: 0,
                salt: 0,
                last_seen: vec![],
            },
            unsigned_content: None,
            filter_mask: FilterMask { mask: None },
            chat_type: ChatTypeBound {
                chat_type_id: 0,
                name: Component { encoded: vec![] },
                target_name: Some(Component { encoded: vec![7] }),
            },
        };
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundPlayerChatPacket::read(&mut slice).expect("read");
        assert_eq!(decoded, packet);
    }

    #[test]
    fn uuid_roundtrip_extremes() {
        for value in [0u128, u128::MAX, 1u128 << 127, 0xFFFF_FFFF_FFFF_FFFFu128] {
            let mut buf = Vec::new();
            write_uuid(&mut buf, value);
            let mut slice: &[u8] = &buf;
            let decoded = read_uuid(&mut slice).expect("read uuid");
            assert!(slice.is_empty());
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn varint_roundtrip_negative_and_extremes() {
        for value in [0i32, -1, i32::MIN, i32::MAX, 128, -128] {
            let mut buf = Vec::new();
            write_varint(&mut buf, value);
            let mut slice: &[u8] = &buf;
            let decoded = read_varint(&mut slice).expect("read varint");
            assert!(slice.is_empty());
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn filter_mask_none_and_some_wire_shape() {
        let mut buf = Vec::new();
        FilterMask::write(&mut buf, &FilterMask { mask: None });
        assert_eq!(buf, vec![0x00]);

        let mut buf2 = Vec::new();
        FilterMask::write(&mut buf2, &FilterMask { mask: Some(vec![1i64]) });
        let mut slice: &[u8] = &buf2;
        let decoded = FilterMask::read(&mut slice).expect("read mask");
        assert_eq!(decoded.mask, Some(vec![1i64]));
    }
}
