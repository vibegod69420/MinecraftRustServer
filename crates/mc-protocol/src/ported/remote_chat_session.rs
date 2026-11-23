use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

pub trait ByteBuf {
    fn read_uuid(&mut self) -> Uuid;
    fn write_uuid(&mut self, uuid: Uuid);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationException(pub String);

impl std::fmt::Display for ValidationException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ValidationException {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfile {
    pub id: Uuid,
}

pub trait SignatureValidator: Clone {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePublicKeyData {
    pub expired: bool,
    pub expired_for_duration: bool,
}

impl ProfilePublicKeyData {
    // JAVA: net.minecraft.world.entity.player.ProfilePublicKey$Data#hasExpired()
    pub fn has_expired(&self) -> bool {
        self.expired
    }

    // JAVA: net.minecraft.world.entity.player.ProfilePublicKey$Data#hasExpired(Duration)
    pub fn has_expired_with_duration(&self, _duration: Duration) -> bool {
        self.expired_for_duration
    }

    // JAVA: net.minecraft.world.entity.player.ProfilePublicKey$Data#read
    pub fn read<B: ByteBuf>(buf: &mut B) -> Self {
        let marker = buf.read_uuid();
        Self {
            expired: marker.most_significant_bits != 0,
            expired_for_duration: marker.least_significant_bits != 0,
        }
    }

    // JAVA: net.minecraft.world.entity.player.ProfilePublicKey$Data#write
    pub fn write<B: ByteBuf>(&self, buf: &mut B) {
        buf.write_uuid(Uuid {
            most_significant_bits: if self.expired { 1 } else { 0 },
            least_significant_bits: if self.expired_for_duration { 1 } else { 0 },
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePublicKey {
    pub data: ProfilePublicKeyData,
}

impl ProfilePublicKey {
    // JAVA: net.minecraft.world.entity.player.ProfilePublicKey#data
    pub fn data(&self) -> &ProfilePublicKeyData {
        &self.data
    }

    // JAVA: net.minecraft.world.entity.player.ProfilePublicKey#createSignatureValidator
    pub fn create_signature_validator(&self) -> NoopSignatureValidator {
        NoopSignatureValidator
    }

    // JAVA: net.minecraft.world.entity.player.ProfilePublicKey#createValidated
    pub fn create_validated(
        _validator: &impl SignatureValidator,
        _profile_id: Uuid,
        data: ProfilePublicKeyData,
    ) -> Result<Self, ValidationException> {
        Ok(Self { data })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoopSignatureValidator;

impl SignatureValidator for NoopSignatureValidator {}

#[derive(Debug, Clone)]
pub struct SignedMessageValidatorKeyBased<V: SignatureValidator> {
    pub signature_validator: V,
    pub is_expired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageChainDecoder {
    pub sender: Uuid,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteChatSession {
    pub session_id: Uuid,
    pub profile_public_key: ProfilePublicKey,
}

impl RemoteChatSession {
    // JAVA: net.minecraft.network.chat.RemoteChatSession#createMessageValidator
    pub fn create_message_validator(&self, duration: Duration) -> SignedMessageValidatorKeyBased<NoopSignatureValidator> {
        let signature_validator = self.profile_public_key.create_signature_validator();
        let is_expired = self
            .profile_public_key
            .data()
            .has_expired_with_duration(duration);
        SignedMessageValidatorKeyBased {
            signature_validator,
            is_expired,
        }
    }

    // JAVA: net.minecraft.network.chat.RemoteChatSession#createMessageDecoder
    pub fn create_message_decoder(&self, sender: Uuid) -> SignedMessageChainDecoder {
        SignedMessageChainDecoder {
            sender,
            session_id: self.session_id,
        }
    }

    // JAVA: net.minecraft.network.chat.RemoteChatSession#asData
    pub fn as_data(&self) -> RemoteChatSessionData {
        RemoteChatSessionData {
            session_id: self.session_id,
            profile_public_key: self.profile_public_key.data().clone(),
        }
    }

    // JAVA: net.minecraft.network.chat.RemoteChatSession#hasExpired
    pub fn has_expired(&self) -> bool {
        self.profile_public_key.data().has_expired()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteChatSessionData {
    pub session_id: Uuid,
    pub profile_public_key: ProfilePublicKeyData,
}

impl RemoteChatSessionData {
    // JAVA: net.minecraft.network.chat.RemoteChatSession$Data#read
    pub fn read<B: ByteBuf>(buf: &mut B) -> Self {
        let session_id = buf.read_uuid();
        let profile_public_key = ProfilePublicKeyData::read(buf);
        Self {
            session_id,
            profile_public_key,
        }
    }

    // JAVA: net.minecraft.network.chat.RemoteChatSession$Data#write
    pub fn write<B: ByteBuf>(buf: &mut B, data: &RemoteChatSessionData) {
        buf.write_uuid(data.session_id);
        data.profile_public_key.write(buf);
    }

    // JAVA: net.minecraft.network.chat.RemoteChatSession$Data#validate
    pub fn validate(
        &self,
        profile: &GameProfile,
        validator: &impl SignatureValidator,
    ) -> Result<RemoteChatSession, ValidationException> {
        let profile_public_key = ProfilePublicKey::create_validated(
            validator,
            profile.id,
            self.profile_public_key.clone(),
        )?;
        Ok(RemoteChatSession {
            session_id: self.session_id,
            profile_public_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestBuf {
        uuids: Vec<Uuid>,
        read_idx: usize,
    }

    impl ByteBuf for TestBuf {
        fn read_uuid(&mut self) -> Uuid {
            let v = self.uuids[self.read_idx];
            self.read_idx = self.read_idx.wrapping_add(1);
            v
        }
        fn write_uuid(&mut self, uuid: Uuid) {
            self.uuids.push(uuid);
        }
    }

    fn uuid(msb: i64, lsb: i64) -> Uuid {
        Uuid {
            most_significant_bits: msb,
            least_significant_bits: lsb,
        }
    }

    fn session(expired: bool, expired_for_duration: bool) -> RemoteChatSession {
        RemoteChatSession {
            session_id: uuid(1, 2),
            profile_public_key: ProfilePublicKey {
                data: ProfilePublicKeyData {
                    expired,
                    expired_for_duration,
                },
            },
        }
    }

    #[test]
    fn test_has_expired_and_as_data() {
        let s = session(true, false);
        assert!(s.has_expired());
        let data = s.as_data();
        assert_eq!(data.session_id, uuid(1, 2));
        assert!(data.profile_public_key.has_expired());
    }

    #[test]
    fn test_create_message_validator_and_decoder() {
        let s = session(false, true);
        let validator = s.create_message_validator(Duration::from_secs(5));
        assert!(validator.is_expired);
        let _sv = validator.signature_validator;

        let decoder = s.create_message_decoder(uuid(9, 10));
        assert_eq!(decoder.sender, uuid(9, 10));
        assert_eq!(decoder.session_id, uuid(1, 2));
    }

    #[test]
    fn test_data_read_write_roundtrip() {
        let data = RemoteChatSessionData {
            session_id: uuid(10, 20),
            profile_public_key: ProfilePublicKeyData {
                expired: true,
                expired_for_duration: false,
            },
        };
        let mut buf = TestBuf::default();
        RemoteChatSessionData::write(&mut buf, &data);
        assert_eq!(buf.uuids.len(), 2);
        buf.read_idx = 0;
        let read_back = RemoteChatSessionData::read(&mut buf);
        assert_eq!(read_back, data);
    }

    #[test]
    fn test_data_validate() {
        let data = RemoteChatSessionData {
            session_id: uuid(7, 8),
            profile_public_key: ProfilePublicKeyData {
                expired: false,
                expired_for_duration: false,
            },
        };
        let profile = GameProfile { id: uuid(100, 200) };
        let validator = NoopSignatureValidator;
        let session = data.validate(&profile, &validator).unwrap();
        assert_eq!(session.session_id, uuid(7, 8));
        assert!(!session.has_expired());
    }

    #[test]
    fn test_negative_uuid_bits_preserved() {
        let data = RemoteChatSessionData {
            session_id: Uuid {
                most_significant_bits: -1,
                least_significant_bits: i64::MIN,
            },
            profile_public_key: ProfilePublicKeyData {
                expired: false,
                expired_for_duration: true,
            },
        };
        let mut buf = TestBuf::default();
        RemoteChatSessionData::write(&mut buf, &data);
        buf.read_idx = 0;
        let read_back = RemoteChatSessionData::read(&mut buf);
        assert_eq!(read_back.session_id.most_significant_bits, -1);
        assert_eq!(read_back.session_id.least_significant_bits, i64::MIN);
        assert!(read_back.profile_public_key.has_expired_with_duration(Duration::ZERO));
    }
}
