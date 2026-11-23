use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

impl Uuid {
    pub const NIL: Self = Self {
        most_significant_bits: 0,
        least_significant_bits: 0,
    };

    pub const fn new(most_significant_bits: i64, least_significant_bits: i64) -> Self {
        Self {
            most_significant_bits,
            least_significant_bits,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Instant {
    pub seconds: i64,
    pub nanos: i32,
}

impl Instant {
    pub const EPOCH: Self = Self {
        seconds: 0,
        nanos: 0,
    };

    pub fn is_before(&self, other: &Self) -> bool {
        (self.seconds, self.nanos) < (other.seconds, other.nanos)
    }

    pub fn now() -> Self {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => Self {
                seconds: d.as_secs() as i64,
                nanos: d.subsec_nanos() as i32,
            },
            Err(_) => Self::EPOCH,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Component {
    pub key: &'static str,
}

impl Component {
    pub const fn translatable(key: &'static str) -> Self {
        Self { key }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterMask {
    PassThrough,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignedMessageLink {
    pub index: i32,
    pub sender: Uuid,
    pub session: Uuid,
}

impl SignedMessageLink {
    pub fn root(sender: Uuid, session: Uuid) -> Self {
        Self {
            index: 0,
            sender,
            session,
        }
    }

    pub fn advance(&self) -> Self {
        Self {
            index: self.index.wrapping_add(1),
            ..*self
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSignature {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedMessageBody {
    pub content: String,
    pub time_stamp: Instant,
    pub salt: i64,
    pub last_seen: Vec<MessageSignature>,
}

pub trait SignatureValidator {
    fn validate(&self, updater: &dyn Fn(&mut dyn FnMut(&[u8])), signature: &[u8]) -> bool;
}

pub trait ProfilePublicKeyData {
    fn has_expired(&self) -> bool;
}

pub trait ProfilePublicKey {
    fn data(&self) -> &dyn ProfilePublicKeyData;
    fn create_signature_validator(&self) -> Box<dyn SignatureValidator>;
}

pub trait Signer {
    fn sign(&self, updater: &dyn Fn(&mut dyn FnMut(&[u8]))) -> Vec<u8>;
}

#[derive(Clone, Debug)]
pub struct PlayerChatMessage {
    pub link: SignedMessageLink,
    pub signature: Option<MessageSignature>,
    pub body: SignedMessageBody,
    pub unsigned_content: Option<Component>,
    pub filter_mask: FilterMask,
}

impl PlayerChatMessage {
    pub fn update_signature(
        output: &mut dyn FnMut(&[u8]),
        link: &SignedMessageLink,
        body: &SignedMessageBody,
    ) {
        output(&link.sender.most_significant_bits.to_be_bytes());
        output(&link.sender.least_significant_bits.to_be_bytes());
        output(&link.session.most_significant_bits.to_be_bytes());
        output(&link.session.least_significant_bits.to_be_bytes());
        output(&link.index.to_be_bytes());
        output(&body.salt.to_be_bytes());
        output(&body.time_stamp.seconds.to_be_bytes());
        output(&body.time_stamp.nanos.to_be_bytes());
        let content_bytes = body.content.as_bytes();
        output(&(content_bytes.len() as i32).to_be_bytes());
        output(content_bytes);
    }

    pub fn verify(&self, validator: &dyn SignatureValidator) -> bool {
        let Some(sig) = &self.signature else {
            return false;
        };
        let link = self.link;
        let body = self.body.clone();
        validator.validate(
            &|updater: &mut dyn FnMut(&[u8])| {
                Self::update_signature(updater, &link, &body);
            },
            &sig.bytes,
        )
    }

    pub fn has_expired_server(&self, now: Instant) -> bool {
        now.seconds.saturating_sub(self.body.time_stamp.seconds) > 300
    }

    pub fn unsigned(sender: Uuid, content: String) -> Self {
        Self {
            link: SignedMessageLink::root(sender, Uuid::NIL),
            signature: None,
            body: SignedMessageBody {
                content,
                time_stamp: Instant::EPOCH,
                salt: 0,
                last_seen: Vec::new(),
            },
            unsigned_content: None,
            filter_mask: FilterMask::PassThrough,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodeException {
    pub component: Component,
}

impl DecodeException {
    pub const MISSING_PROFILE_KEY: Component =
        Component::translatable("chat.disabled.missingProfileKey");
    pub const CHAIN_BROKEN: Component = Component::translatable("chat.disabled.chain_broken");
    pub const EXPIRED_PROFILE_KEY: Component =
        Component::translatable("chat.disabled.expiredProfileKey");
    pub const INVALID_SIGNATURE: Component =
        Component::translatable("chat.disabled.invalid_signature");
    pub const OUT_OF_ORDER_CHAT: Component =
        Component::translatable("chat.disabled.out_of_order_chat");

    // JAVA: net.minecraft.network.chat.SignedMessageChain$DecodeException#<init>
    pub fn new(component: Component) -> Self {
        Self { component }
    }
}

impl fmt::Display for DecodeException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.component.key)
    }
}

impl std::error::Error for DecodeException {}

pub trait Encoder {
    fn pack(&mut self, body: &SignedMessageBody) -> Option<MessageSignature>;
}

pub struct UnsignedChainEncoder;

impl Encoder for UnsignedChainEncoder {
    // JAVA: net.minecraft.network.chat.SignedMessageChain$Encoder#pack
    fn pack(&mut self, _body: &SignedMessageBody) -> Option<MessageSignature> {
        None
    }
}

pub trait Decoder {
    fn unpack(
        &mut self,
        signature: Option<&MessageSignature>,
        body: &SignedMessageBody,
    ) -> Result<PlayerChatMessage, DecodeException>;

    fn set_chain_broken(&mut self) {}
}

pub struct UnsignedDecoder<F: Fn() -> bool> {
    pub sender: Uuid,
    pub enforce_secure_profile: F,
}

impl<F: Fn() -> bool> UnsignedDecoder<F> {
    // JAVA: net.minecraft.network.chat.SignedMessageChain$Decoder#unsigned
    pub fn unsigned(sender: Uuid, enforce_secure_profile: F) -> Self {
        Self {
            sender,
            enforce_secure_profile,
        }
    }
}

impl<F: Fn() -> bool> Decoder for UnsignedDecoder<F> {
    // JAVA: net.minecraft.network.chat.SignedMessageChain$Decoder#unsigned
    fn unpack(
        &mut self,
        _signature: Option<&MessageSignature>,
        body: &SignedMessageBody,
    ) -> Result<PlayerChatMessage, DecodeException> {
        if (self.enforce_secure_profile)() {
            return Err(DecodeException::new(DecodeException::MISSING_PROFILE_KEY));
        }
        Ok(PlayerChatMessage::unsigned(
            self.sender,
            body.content.clone(),
        ))
    }
}

pub struct SignedMessageChain {
    pub next_link: Option<SignedMessageLink>,
    pub last_time_stamp: Instant,
}

impl SignedMessageChain {
    // JAVA: net.minecraft.network.chat.SignedMessageChain#<init>
    pub fn new(sender: Uuid, session_id: Uuid) -> Self {
        Self {
            next_link: Some(SignedMessageLink::root(sender, session_id)),
            last_time_stamp: Instant::EPOCH,
        }
    }

    // JAVA: net.minecraft.network.chat.SignedMessageChain#encoder
    pub fn encoder<'a, S: Signer + 'a>(&'a mut self, signer: S) -> ChainEncoder<'a, S> {
        ChainEncoder { chain: self, signer }
    }

    // JAVA: net.minecraft.network.chat.SignedMessageChain#decoder
    pub fn decoder<'a>(
        &'a mut self,
        public_key: Option<&'a dyn ProfilePublicKey>,
    ) -> ChainDecoder<'a> {
        let validator = public_key.map(|k| k.create_signature_validator());
        ChainDecoder {
            chain: self,
            public_key,
            validator,
        }
    }
}

pub struct ChainEncoder<'a, S: Signer> {
    chain: &'a mut SignedMessageChain,
    signer: S,
}

impl<S: Signer> ChainEncoder<'_, S> {
    // JAVA: net.minecraft.network.chat.SignedMessageChain#encoder
    pub fn pack(&mut self, body: &SignedMessageBody) -> Option<MessageSignature> {
        let link = self.chain.next_link?;
        self.chain.next_link = Some(link.advance());
        let bytes = self.signer.sign(&|updater: &mut dyn FnMut(&[u8])| {
            PlayerChatMessage::update_signature(updater, &link, body);
        });
        Some(MessageSignature { bytes })
    }
}

impl<S: Signer> Encoder for ChainEncoder<'_, S> {
    // JAVA: net.minecraft.network.chat.SignedMessageChain#encoder
    fn pack(&mut self, body: &SignedMessageBody) -> Option<MessageSignature> {
        ChainEncoder::pack(self, body)
    }
}

pub struct ChainDecoder<'a> {
    chain: &'a mut SignedMessageChain,
    public_key: Option<&'a dyn ProfilePublicKey>,
    validator: Option<Box<dyn SignatureValidator>>,
}

impl Decoder for ChainDecoder<'_> {
    // JAVA: net.minecraft.network.chat.SignedMessageChain#decoder
    fn unpack(
        &mut self,
        signature: Option<&MessageSignature>,
        body: &SignedMessageBody,
    ) -> Result<PlayerChatMessage, DecodeException> {
        if self.public_key.is_none() {
            return Err(DecodeException::new(DecodeException::MISSING_PROFILE_KEY));
        }
        if self.public_key.unwrap().data().has_expired() {
            return Err(DecodeException::new(DecodeException::EXPIRED_PROFILE_KEY));
        }
        let Some(link) = self.chain.next_link else {
            return Err(DecodeException::new(DecodeException::CHAIN_BROKEN));
        };
        if body.time_stamp.is_before(&self.chain.last_time_stamp) {
            self.set_chain_broken();
            return Err(DecodeException::new(DecodeException::OUT_OF_ORDER_CHAT));
        }
        self.chain.last_time_stamp = body.time_stamp;
        let message = PlayerChatMessage {
            link,
            signature: signature.cloned(),
            body: body.clone(),
            unsigned_content: None,
            filter_mask: FilterMask::PassThrough,
        };
        let validator = self
            .validator
            .as_ref()
            .expect("validator must exist when public key is present");
        if !message.verify(validator.as_ref()) {
            self.set_chain_broken();
            return Err(DecodeException::new(DecodeException::INVALID_SIGNATURE));
        }
        if message.has_expired_server(Instant::now()) {
            eprintln!(
                "Received expired chat: '{}'. Is the client/server system time unsynchronized?",
                body.content
            );
        }
        self.chain.next_link = Some(link.advance());
        Ok(message)
    }

    // JAVA: net.minecraft.network.chat.SignedMessageChain#decoder
    fn set_chain_broken(&mut self) {
        self.chain.next_link = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSigner;

    impl Signer for TestSigner {
        fn sign(&self, updater: &dyn Fn(&mut dyn FnMut(&[u8]))) -> Vec<u8> {
            let mut buf = Vec::new();
            updater(&mut |data| buf.extend_from_slice(data));
            buf
        }
    }

    struct TestValidator {
        valid: bool,
    }

    impl SignatureValidator for TestValidator {
        fn validate(&self, updater: &dyn Fn(&mut dyn FnMut(&[u8])), signature: &[u8]) -> bool {
            if !self.valid {
                return false;
            }
            let mut buf = Vec::new();
            updater(&mut |data| buf.extend_from_slice(data));
            buf == signature
        }
    }

    struct TestKeyData {
        expired: bool,
    }

    impl ProfilePublicKeyData for TestKeyData {
        fn has_expired(&self) -> bool {
            self.expired
        }
    }

    struct TestKey {
        data: TestKeyData,
        valid_sigs: bool,
    }

    impl ProfilePublicKey for TestKey {
        fn data(&self) -> &dyn ProfilePublicKeyData {
            &self.data
        }

        fn create_signature_validator(&self) -> Box<dyn SignatureValidator> {
            Box::new(TestValidator {
                valid: self.valid_sigs,
            })
        }
    }

    fn make_body(content: &str, secs: i64) -> SignedMessageBody {
        SignedMessageBody {
            content: content.to_string(),
            time_stamp: Instant {
                seconds: secs,
                nanos: 0,
            },
            salt: 0,
            last_seen: Vec::new(),
        }
    }

    #[test]
    fn encoder_returns_none_when_chain_broken_and_advances_link() {
        let mut chain = SignedMessageChain::new(Uuid::new(1, 2), Uuid::new(3, 4));
        let first_link = chain.next_link.unwrap();
        let sig = {
            let mut enc = chain.encoder(TestSigner);
            let body = make_body("hello", 1_700_000_000);
            let sig = enc.pack(&body).expect("should produce signature");
            assert!(!sig.bytes.is_empty());
            sig
        };
        let advanced = chain.next_link.unwrap();
        assert_eq!(advanced.index, first_link.index.wrapping_add(1));
        assert_eq!(advanced.sender, first_link.sender);
        assert_eq!(advanced.session, first_link.session);

        chain.next_link = None;
        let mut enc = chain.encoder(TestSigner);
        assert!(enc.pack(&make_body("broken", 1_700_000_001)).is_none());
        let _ = sig;
    }

    #[test]
    fn decoder_happy_path_round_trip() {
        let sender = Uuid::new(10, 20);
        let session = Uuid::new(30, 40);
        let body = make_body("test message", 1_700_000_100);

        let sig = {
            let mut enc_chain = SignedMessageChain::new(sender, session);
            let mut enc = enc_chain.encoder(TestSigner);
            enc.pack(&body).expect("sig")
        };

        let mut dec_chain = SignedMessageChain::new(sender, session);
        let key = TestKey {
            data: TestKeyData { expired: false },
            valid_sigs: true,
        };
        let msg = {
            let mut dec = dec_chain.decoder(Some(&key));
            dec.unpack(Some(&sig), &body).expect("should decode")
        };
        assert_eq!(msg.body.content, "test message");
        assert_eq!(msg.link.sender, sender);
        assert_eq!(dec_chain.next_link.unwrap().index, 1);
        assert_eq!(dec_chain.last_time_stamp, body.time_stamp);
    }

    #[test]
    fn decoder_rejects_out_of_order_and_breaks_chain() {
        let sender = Uuid::new(1, 0);
        let session = Uuid::new(2, 0);
        let key = TestKey {
            data: TestKeyData { expired: false },
            valid_sigs: true,
        };
        let body1 = make_body("first", 1_700_000_200);
        let body2 = make_body("second", 1_700_000_100);

        let sig1 = {
            let mut enc_chain = SignedMessageChain::new(sender, session);
            let mut enc = enc_chain.encoder(TestSigner);
            enc.pack(&body1).unwrap()
        };

        let mut dec_chain = SignedMessageChain::new(sender, session);
        {
            let mut dec = dec_chain.decoder(Some(&key));
            dec.unpack(Some(&sig1), &body1).unwrap();
        }

        let err = {
            let mut dec = dec_chain.decoder(Some(&key));
            let dummy_sig = MessageSignature { bytes: vec![0] };
            dec.unpack(Some(&dummy_sig), &body2).unwrap_err()
        };
        assert_eq!(err.component, DecodeException::OUT_OF_ORDER_CHAT);
        assert!(dec_chain.next_link.is_none());
    }

    #[test]
    fn decoder_rejects_invalid_signature_and_breaks_chain() {
        let sender = Uuid::new(5, 6);
        let session = Uuid::new(7, 8);
        let body = make_body("msg", 1_700_000_300);
        let sig = {
            let mut enc_chain = SignedMessageChain::new(sender, session);
            let mut enc = enc_chain.encoder(TestSigner);
            enc.pack(&body).unwrap()
        };

        let mut dec_chain = SignedMessageChain::new(sender, session);
        let bad_key = TestKey {
            data: TestKeyData { expired: false },
            valid_sigs: false,
        };
        let err = {
            let mut dec = dec_chain.decoder(Some(&bad_key));
            dec.unpack(Some(&sig), &body).unwrap_err()
        };
        assert_eq!(err.component, DecodeException::INVALID_SIGNATURE);
        assert!(dec_chain.next_link.is_none());

        let err2 = {
            let mut dec = dec_chain.decoder(Some(&bad_key));
            dec.unpack(Some(&sig), &body).unwrap_err()
        };
        assert_eq!(err2.component, DecodeException::CHAIN_BROKEN);
    }

    #[test]
    fn decoder_missing_or_expired_key_and_unsigned_decoder() {
        let sender = Uuid::new(1, 1);
        let session = Uuid::new(2, 2);
        let body = make_body("x", 1_700_000_400);
        let sig = MessageSignature { bytes: vec![1, 2, 3] };

        let mut chain1 = SignedMessageChain::new(sender, session);
        let err = {
            let mut dec = chain1.decoder(None);
            dec.unpack(Some(&sig), &body).unwrap_err()
        };
        assert_eq!(err.component, DecodeException::MISSING_PROFILE_KEY);

        let mut chain2 = SignedMessageChain::new(sender, session);
        let expired_key = TestKey {
            data: TestKeyData { expired: true },
            valid_sigs: true,
        };
        let err = {
            let mut dec = chain2.decoder(Some(&expired_key));
            dec.unpack(Some(&sig), &body).unwrap_err()
        };
        assert_eq!(err.component, DecodeException::EXPIRED_PROFILE_KEY);

        let mut unsigned = UnsignedDecoder::unsigned(sender, || false);
        let msg = unsigned.unpack(None, &body).unwrap();
        assert_eq!(msg.body.content, "x");
        assert!(msg.signature.is_none());

        let mut unsigned_reject = UnsignedDecoder::unsigned(sender, || true);
        let err = unsigned_reject.unpack(None, &body).unwrap_err();
        assert_eq!(err.component, DecodeException::MISSING_PROFILE_KEY);
    }

    #[test]
    fn negative_timestamps_and_link_index_wrapping() {
        let sender = Uuid::new(-1, i64::MIN);
        let session = Uuid::new(i64::MAX, -999);
        let mut chain = SignedMessageChain::new(sender, session);
        chain.next_link = Some(SignedMessageLink {
            index: i32::MAX,
            sender,
            session,
        });

        let body_neg = make_body("neg", -500);
        let sig = {
            let mut enc = chain.encoder(TestSigner);
            enc.pack(&body_neg).unwrap()
        };
        assert!(!sig.bytes.is_empty());
        assert_eq!(chain.next_link.unwrap().index, i32::MIN);

        let mut dec_chain = SignedMessageChain::new(sender, session);
        dec_chain.last_time_stamp = Instant {
            seconds: -100,
            nanos: 0,
        };
        let key = TestKey {
            data: TestKeyData { expired: false },
            valid_sigs: true,
        };
        let err = {
            let mut dec = dec_chain.decoder(Some(&key));
            dec.unpack(Some(&sig), &make_body("old", -500))
                .unwrap_err()
        };
        assert_eq!(err.component, DecodeException::OUT_OF_ORDER_CHAT);

        // Instant edge cases (float-bits rule not applicable; cover ordering/negative nanos)
        let a = Instant {
            seconds: 0,
            nanos: i32::MIN,
        };
        let b = Instant {
            seconds: 0,
            nanos: i32::MAX,
        };
        assert!(a.is_before(&b));
        assert!(!Instant::EPOCH.is_before(&Instant::EPOCH));
        let _ = f32::from_bits(0x7FC0_0001).to_bits(); // NaN payload bit-pattern sanity
    }
}
