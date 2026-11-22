use std::time::{SystemTime, UNIX_EPOCH};

/// Minimal UUID stub for `java.util.UUID`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub bits: u128,
}

impl Uuid {
    pub fn from_u128(bits: u128) -> Self {
        Self { bits }
    }

    pub fn random_uuid() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let ptr = (&nanos as *const u128 as usize) as u128;
        let mut x = nanos ^ ptr.rotate_left(32) ^ 0x9e3779b97f4a7c15u128;
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58476d1ce4e5b9u128);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94d049bb133111ebu128);
        x ^= x >> 31;

        let mut bytes = x.to_be_bytes();
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Self::from_u128(u128::from_be_bytes(bytes))
    }
}

/// Minimal stub for a profile public key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfilePublicKey {
    pub data: Vec<u8>,
}

/// Minimal stub for a profile private key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfilePrivateKey {
    pub data: Vec<u8>,
}

/// Minimal stub for `net.minecraft.world.entity.player.ProfileKeyPair`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileKeyPair {
    pub public_key: ProfilePublicKey,
    pub private_key: ProfilePrivateKey,
}

impl ProfileKeyPair {
    pub fn public_key(&self) -> &ProfilePublicKey {
        &self.public_key
    }

    pub fn private_key(&self) -> &ProfilePrivateKey {
        &self.private_key
    }
}

/// Minimal stub for `net.minecraft.util.Signer`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signer {
    pub algorithm: String,
    pub key: ProfilePrivateKey,
}

impl Signer {
    pub fn from(key: &ProfilePrivateKey, algorithm: &str) -> Self {
        Self {
            algorithm: algorithm.to_string(),
            key: key.clone(),
        }
    }
}

/// Minimal stub for `net.minecraft.network.chat.SignedMessageChain.Encoder`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedMessageChainEncoder {
    pub sender: Uuid,
    pub session_id: Uuid,
    pub signer: Signer,
}

/// Minimal stub for `net.minecraft.network.chat.SignedMessageChain`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedMessageChain {
    pub sender: Uuid,
    pub session_id: Uuid,
}

impl SignedMessageChain {
    pub fn new(sender: Uuid, session_id: Uuid) -> Self {
        Self { sender, session_id }
    }

    pub fn encoder(&self, signer: Signer) -> SignedMessageChainEncoder {
        SignedMessageChainEncoder {
            sender: self.sender,
            session_id: self.session_id,
            signer,
        }
    }
}

/// Minimal stub for `net.minecraft.network.chat.RemoteChatSession`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteChatSession {
    pub session_id: Uuid,
    pub public_key: ProfilePublicKey,
}

/// Port of `net.minecraft.network.chat.LocalChatSession`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalChatSession {
    pub session_id: Uuid,
    pub key_pair: ProfileKeyPair,
}

impl LocalChatSession {
    // JAVA: net.minecraft.network.chat.LocalChatSession#create
    pub fn create(key_pair: ProfileKeyPair) -> Self {
        Self {
            session_id: Uuid::random_uuid(),
            key_pair,
        }
    }

    // JAVA: net.minecraft.network.chat.LocalChatSession#createMessageEncoder
    pub fn create_message_encoder(&self, sender: Uuid) -> SignedMessageChainEncoder {
        SignedMessageChain::new(sender, self.session_id).encoder(Signer::from(
            self.key_pair.private_key(),
            "SHA256withRSA",
        ))
    }

    // JAVA: net.minecraft.network.chat.LocalChatSession#asRemote
    pub fn as_remote(&self) -> RemoteChatSession {
        RemoteChatSession {
            session_id: self.session_id,
            public_key: self.key_pair.public_key().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LocalChatSession, ProfileKeyPair, ProfilePrivateKey, ProfilePublicKey, Uuid,
    };

    fn key_pair(pub_byte: u8, priv_byte: u8) -> ProfileKeyPair {
        ProfileKeyPair {
            public_key: ProfilePublicKey {
                data: vec![pub_byte],
            },
            private_key: ProfilePrivateKey {
                data: vec![priv_byte],
            },
        }
    }

    #[test]
    fn create_sets_key_pair() {
        let kp = key_pair(1, 2);
        let session = LocalChatSession::create(kp.clone());
        assert_eq!(session.key_pair, kp);
    }

    #[test]
    fn create_message_encoder_uses_sender_session_and_sha256withrsa() {
        let session = LocalChatSession {
            session_id: Uuid::from_u128(0x1234),
            key_pair: key_pair(10, 20),
        };
        let sender = Uuid::from_u128(0x5678);
        let encoder = session.create_message_encoder(sender);
        assert_eq!(encoder.sender, sender);
        assert_eq!(encoder.session_id, Uuid::from_u128(0x1234));
        assert_eq!(encoder.signer.algorithm, "SHA256withRSA");
        assert_eq!(encoder.signer.key.data, vec![20]);
    }

    #[test]
    fn as_remote_preserves_session_id_and_public_key() {
        let session = LocalChatSession {
            session_id: Uuid::from_u128(99),
            key_pair: key_pair(7, 8),
        };
        let remote = session.as_remote();
        assert_eq!(remote.session_id, Uuid::from_u128(99));
        assert_eq!(remote.public_key.data, vec![7]);
    }

    #[test]
    fn random_uuid_has_version_and_variant_bits() {
        let uuid = Uuid::random_uuid();
        let bytes = uuid.bits.to_be_bytes();
        assert_eq!(bytes[6] & 0xf0, 0x40);
        assert_eq!(bytes[8] & 0xc0, 0x80);
    }
}
