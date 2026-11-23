//! Port of `net.minecraft.network.protocol.status.ServerStatus` (Java 1.21.11).
//!
//! This module provides a self-contained data model and the concrete behavioral
//! methods from the Java record. Mojang `Codec`/`DataResult` infrastructure is
//! out of scope for this crate, so codec statics are represented by explicit
//! encode/decode helpers where Java contained behavior.

use std::fmt;

/// Minimal stand-in for `net.minecraft.network.chat.Component`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Component {
    pub text: String,
}

impl Component {
    /// Convenience constructor for an empty component, analogous to
    /// `CommonComponents.EMPTY` usage in the source record defaults.
    pub fn empty() -> Self {
        Self {
            text: String::new(),
        }
    }
}

/// Minimal stand-in for `java.util.UUID` as used by `NameAndId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct JavaUuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

/// Minimal stand-in for `net.minecraft.server.players.NameAndId`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameAndId {
    pub name: String,
    pub id: JavaUuid,
}

/// Minimal stand-in for `net.minecraft.WorldVersion`, used by
/// `ServerStatus.Version#current`.
pub trait WorldVersionInfo {
    fn name(&self) -> &str;
    fn protocol_version(&self) -> i32;
}

/// Error returned when decoding a favicon data URI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaviconDecodeError {
    pub message: String,
}

impl fmt::Display for FaviconDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FaviconDecodeError {}

/// Port of `net.minecraft.network.protocol.status.ServerStatus`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerStatus {
    pub description: Component,
    pub players: Option<Players>,
    pub version: Option<Version>,
    pub favicon: Option<Favicon>,
    pub enforces_secure_chat: bool,
}

impl ServerStatus {
    // JAVA: net.minecraft.network.protocol.status.ServerStatus#ServerStatus
    pub fn new(
        description: Component,
        players: Option<Players>,
        version: Option<Version>,
        favicon: Option<Favicon>,
        enforces_secure_chat: bool,
    ) -> Self {
        Self {
            description,
            players,
            version,
            favicon,
            enforces_secure_chat,
        }
    }
}

/// Port of `net.minecraft.network.protocol.status.ServerStatus$Favicon`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Favicon {
    pub icon_bytes: Vec<u8>,
}

impl Favicon {
    pub const PREFIX: &'static str = "data:image/png;base64,";

    // JAVA: net.minecraft.network.protocol.status.ServerStatus$Favicon#Favicon
    pub fn new(icon_bytes: Vec<u8>) -> Self {
        Self { icon_bytes }
    }

    // JAVA: net.minecraft.network.protocol.status.ServerStatus$Favicon#decode
    pub fn decode(input: &str) -> Result<Self, FaviconDecodeError> {
        if !input.starts_with(Self::PREFIX) {
            return Err(FaviconDecodeError {
                message: "Unknown format".to_string(),
            });
        }

        let payload = input[Self::PREFIX.len()..].replace('\n', "");
        match b64_decode_standard(&payload) {
            Ok(icon_bytes) => Ok(Self { icon_bytes }),
            Err(()) => Err(FaviconDecodeError {
                message: "Malformed base64 server icon".to_string(),
            }),
        }
    }

    // JAVA: net.minecraft.network.protocol.status.ServerStatus$Favicon#encode
    pub fn encode(&self) -> String {
        let mut out = String::with_capacity(Self::PREFIX.len() + b64_encoded_len(self.icon_bytes.len()));
        out.push_str(Self::PREFIX);
        out.push_str(&b64_encode_standard(&self.icon_bytes));
        out
    }
}

/// Port of `net.minecraft.network.protocol.status.ServerStatus$Players`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<NameAndId>,
}

impl Players {
    // JAVA: net.minecraft.network.protocol.status.ServerStatus$Players#Players
    pub fn new(max: i32, online: i32, sample: Vec<NameAndId>) -> Self {
        Self { max, online, sample }
    }
}

/// Port of `net.minecraft.network.protocol.status.ServerStatus$Version`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

impl Version {
    // JAVA: net.minecraft.network.protocol.status.ServerStatus$Version#Version
    pub fn new(name: String, protocol: i32) -> Self {
        Self { name, protocol }
    }

    // JAVA: net.minecraft.network.protocol.status.ServerStatus$Version#current
    pub fn current(world_version: &impl WorldVersionInfo) -> Self {
        Self {
            name: world_version.name().to_string(),
            protocol: world_version.protocol_version(),
        }
    }
}

fn b64_encoded_len(input_len: usize) -> usize {
    ((input_len + 2) / 3) * 4
}

fn b64_encode_standard(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = String::with_capacity(b64_encoded_len(input.len()));
    let mut i = 0usize;
    while i < input.len() {
        let b0 = input[i] as u32;
        let remaining = input.len() - i;

        if remaining == 1 {
            let triple = b0 << 16;
            out.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
            out.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
            out.push('=');
            out.push('=');
        } else if remaining == 2 {
            let b1 = input[i + 1] as u32;
            let triple = (b0 << 16) | (b1 << 8);
            out.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
            out.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
            out.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
            out.push('=');
        } else {
            let b1 = input[i + 1] as u32;
            let b2 = input[i + 2] as u32;
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
            out.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
            out.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
            out.push(ALPHABET[(triple & 0x3F) as usize] as char);
        }

        i += 3;
    }

    out
}

fn b64_val(b: u8) -> Option<u32> {
    match b {
        b'A'..=b'Z' => Some((b - b'A') as u32),
        b'a'..=b'z' => Some((b - b'a' + 26) as u32),
        b'0'..=b'9' => Some((b - b'0' + 52) as u32),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn b64_decode_standard(input: &str) -> Result<Vec<u8>, ()> {
    let filtered: Vec<u8> = input.bytes().filter(|&b| b != b'\n').collect();
    if filtered.is_empty() {
        return Ok(Vec::new());
    }
    if filtered.len() % 4 != 0 {
        return Err(());
    }

    let mut out = Vec::with_capacity((filtered.len() / 4) * 3);
    let mut chunk_idx = 0usize;
    let chunk_count = filtered.len() / 4;
    while chunk_idx < chunk_count {
        let base = chunk_idx * 4;
        let c0 = filtered[base];
        let c1 = filtered[base + 1];
        let c2 = filtered[base + 2];
        let c3 = filtered[base + 3];

        let v0 = b64_val(c0).ok_or(())?;
        let v1 = b64_val(c1).ok_or(())?;

        let is_last = chunk_idx + 1 == chunk_count;
        if is_last {
            if c2 == b'=' && c3 == b'=' {
                out.push(((v0 << 2) | (v1 >> 4)) as u8);
            } else if c3 == b'=' {
                let v2 = b64_val(c2).ok_or(())?;
                out.push(((v0 << 2) | (v1 >> 4)) as u8);
                out.push((((v1 & 0x0F) << 4) | (v2 >> 2)) as u8);
            } else {
                let v2 = b64_val(c2).ok_or(())?;
                let v3 = b64_val(c3).ok_or(())?;
                out.push(((v0 << 2) | (v1 >> 4)) as u8);
                out.push((((v1 & 0x0F) << 4) | (v2 >> 2)) as u8);
                out.push((((v2 & 0x03) << 6) | v3) as u8);
            }
        } else {
            if c2 == b'=' || c3 == b'=' {
                return Err(());
            }
            let v2 = b64_val(c2).ok_or(())?;
            let v3 = b64_val(c3).ok_or(())?;
            out.push(((v0 << 2) | (v1 >> 4)) as u8);
            out.push((((v1 & 0x0F) << 4) | (v2 >> 2)) as u8);
            out.push((((v2 & 0x03) << 6) | v3) as u8);
        }

        chunk_idx += 1;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockWorldVersion {
        name: String,
        protocol: i32,
    }

    impl WorldVersionInfo for MockWorldVersion {
        fn name(&self) -> &str {
            &self.name
        }

        fn protocol_version(&self) -> i32 {
            self.protocol
        }
    }

    #[test]
    fn favicon_roundtrip_and_prefix() {
        let bytes = vec![0u8, 1, 2, 3, 250, 255];
        let favicon = Favicon::new(bytes.clone());
        let encoded = favicon.encode();
        assert!(encoded.starts_with(Favicon::PREFIX));

        let decoded = Favicon::decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.icon_bytes, bytes);
        assert_eq!(decoded, favicon);
    }

    #[test]
    fn favicon_decode_errors_and_newline_handling() {
        let err = Favicon::decode("image/png;base64,AAAA").unwrap_err();
        assert_eq!(err.message, "Unknown format");

        let malformed = format!("{}not-base64!", Favicon::PREFIX);
        let err = Favicon::decode(&malformed).unwrap_err();
        assert_eq!(err.message, "Malformed base64 server icon");

        let with_newlines = format!("{}SGVs\nbG8=", Favicon::PREFIX);
        let decoded = Favicon::decode(&with_newlines).expect("newline-stripped decode");
        assert_eq!(decoded.icon_bytes, b"Hello");
    }

    #[test]
    fn players_and_version_negative_and_overflow_edges() {
        let sample = vec![NameAndId {
            name: "Player".to_string(),
            id: JavaUuid {
                most_significant_bits: i64::MIN,
                least_significant_bits: -1i64,
            },
        }];

        let players = Players::new(i32::MAX, -1i32, sample.clone());
        assert_eq!(players.max, i32::MAX);
        assert_eq!(players.online, -1i32);
        assert_eq!(players.sample, sample);

        let protocol = i32::MIN;
        let wrapped = protocol.wrapping_add(-1i32);
        assert_eq!(wrapped, i32::MAX);

        let version = Version::new("snapshot".to_string(), protocol);
        assert_eq!(version.protocol, i32::MIN);
    }

    #[test]
    fn version_current_uses_world_version_info() {
        let mock = MockWorldVersion {
            name: "1.21.11".to_string(),
            protocol: 767i32,
        };
        let version = Version::current(&mock);
        assert_eq!(
            version,
            Version {
                name: "1.21.11".to_string(),
                protocol: 767i32,
            }
        );
    }

    #[test]
    fn server_status_construction_and_float_bits_edge_case() {
        let status = ServerStatus::new(
            Component::empty(),
            Some(Players::new(20, 0, Vec::new())),
            Some(Version::new("test".to_string(), 0)),
            None,
            true,
        );

        assert_eq!(status.description, Component::empty());
        assert!(status.enforces_secure_chat);
        assert_eq!(status.players.as_ref().map(|p| p.max), Some(20));

        let neg_one_f32_bits: u32 = 0xBF80_0000;
        let f = f32::from_bits(neg_one_f32_bits);
        assert_eq!(f.to_bits(), neg_one_f32_bits);
        assert_eq!(f, -1.0f32);

        let nan_bits: u64 = 0x7FF8_0000_0000_0001u64;
        let d = f64::from_bits(nan_bits);
        assert_eq!(d.to_bits(), nan_bits);
        assert!(d.is_nan());
    }
}
