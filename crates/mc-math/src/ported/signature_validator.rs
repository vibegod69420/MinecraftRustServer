use std::fmt;

/// Minimal exception type for signature operations.
#[derive(Debug)]
pub struct SignatureException {
    pub message: String,
}

impl SignatureException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SignatureException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SignatureException {}

/// Output sink that receives signed data chunks.
///
/// Mirrors the updater output target used by `SignatureUpdater` in Java.
pub trait SignatureOutput {
    fn update(&mut self, data: &[u8]) -> Result<(), SignatureException>;
}

/// Feeds data into a [`SignatureOutput`] for signature validation.
///
/// Mirrors `net.minecraft.util.SignatureUpdater`.
pub trait SignatureUpdater {
    fn update(&self, output: &mut dyn SignatureOutput) -> Result<(), SignatureException>;
}

impl<F> SignatureUpdater for F
where
    F: Fn(&mut dyn SignatureOutput) -> Result<(), SignatureException>,
{
    fn update(&self, output: &mut dyn SignatureOutput) -> Result<(), SignatureException> {
        self(output)
    }
}

/// Minimal abstraction over `java.security.Signature`.
pub trait SignatureEngine {
    fn update(&mut self, data: &[u8]) -> Result<(), SignatureException>;
    fn verify(&self, signature: &[u8]) -> Result<bool, SignatureException>;
}

/// Minimal abstraction over `java.security.PublicKey` for signature creation.
pub trait PublicKey: Send + Sync {
    fn create_signature_engine(
        &self,
        algorithm: &str,
    ) -> Result<Box<dyn SignatureEngine>, SignatureException>;
}

/// Minimal abstraction over `com.mojang.authlib.yggdrasil.ServicesKeyInfo`.
pub trait ServicesKeyInfo: Send + Sync {
    fn signature(&self) -> Box<dyn SignatureEngine>;
}

/// Minimal abstraction over `com.mojang.authlib.yggdrasil.ServicesKeySet`.
pub trait ServicesKeySet<K: ?Sized>: Send + Sync {
    fn keys(&self, key_type: &K) -> Vec<Box<dyn ServicesKeyInfo + Send + Sync>>;
}

type ValidateFn =
    Box<dyn Fn(&dyn SignatureUpdater, &[u8]) -> bool + Send + Sync>;

/// Port of `net.minecraft.util.SignatureValidator`.
pub struct SignatureValidator {
    validate_fn: ValidateFn,
}

impl SignatureValidator {
    /// Creates a validator from a custom validation function.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&dyn SignatureUpdater, &[u8]) -> bool + Send + Sync + 'static,
    {
        Self {
            validate_fn: Box::new(f),
        }
    }

    // JAVA: net.minecraft.util.SignatureValidator#validate
    pub fn validate(&self, updater: &dyn SignatureUpdater, signature: &[u8]) -> bool {
        (self.validate_fn)(updater, signature)
    }

    // JAVA: net.minecraft.util.SignatureValidator#validate
    pub fn validate_bytes(&self, payload: &[u8], signature: &[u8]) -> bool {
        struct BytePayload<'a>(&'a [u8]);
        impl SignatureUpdater for BytePayload<'_> {
            fn update(
                &self,
                output: &mut dyn SignatureOutput,
            ) -> Result<(), SignatureException> {
                output.update(self.0)
            }
        }
        self.validate(&BytePayload(payload), signature)
    }
}

// JAVA: net.minecraft.util.SignatureValidator#NO_VALIDATION
pub fn no_validation() -> SignatureValidator {
    SignatureValidator::new(|_updater, _signature| true)
}

// JAVA: net.minecraft.util.SignatureValidator#verifySignature
fn verify_signature(
    updater: &dyn SignatureUpdater,
    signature_bytes: &[u8],
    engine: &mut dyn SignatureEngine,
) -> Result<bool, SignatureException> {
    struct EngineOutput<'a>(&'a mut dyn SignatureEngine);
    impl SignatureOutput for EngineOutput<'_> {
        fn update(&mut self, data: &[u8]) -> Result<(), SignatureException> {
            self.0.update(data)
        }
    }
    updater.update(&mut EngineOutput(engine))?;
    engine.verify(signature_bytes)
}

// JAVA: net.minecraft.util.SignatureValidator#from
pub fn from_public_key(key: Box<dyn PublicKey>, algorithm: String) -> SignatureValidator {
    SignatureValidator::new(move |updater, signature_bytes| {
        let mut engine = match key.create_signature_engine(&algorithm) {
            Ok(engine) => engine,
            Err(e) => {
                eprintln!("Failed to verify signature: {e}");
                return false;
            }
        };
        match verify_signature(updater, signature_bytes, engine.as_mut()) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to verify signature: {e}");
                false
            }
        }
    })
}

// JAVA: net.minecraft.util.SignatureValidator#from
pub fn from_services_key_set<K: ?Sized>(
    key_set: &dyn ServicesKeySet<K>,
    key_type: &K,
) -> Option<SignatureValidator> {
    let keys = key_set.keys(key_type);
    if keys.is_empty() {
        return None;
    }
    Some(SignatureValidator::new(move |updater, signature_bytes| {
        keys.iter().any(|key_info| {
            let mut engine = key_info.signature();
            match verify_signature(updater, signature_bytes, engine.as_mut()) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to verify Services signature: {e}");
                    false
                }
            }
        })
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEngine {
        received: Vec<u8>,
        expected: Vec<u8>,
        valid_sig: Vec<u8>,
        fail_update: bool,
        fail_verify: bool,
    }

    impl SignatureEngine for MockEngine {
        fn update(&mut self, data: &[u8]) -> Result<(), SignatureException> {
            if self.fail_update {
                return Err(SignatureException::new("update fail"));
            }
            self.received.extend_from_slice(data);
            Ok(())
        }

        fn verify(&self, signature: &[u8]) -> Result<bool, SignatureException> {
            if self.fail_verify {
                return Err(SignatureException::new("verify fail"));
            }
            Ok(self.received == self.expected && signature == self.valid_sig)
        }
    }

    struct MockPublicKey {
        expected: Vec<u8>,
        valid_sig: Vec<u8>,
        fail_create: bool,
        fail_update: bool,
        fail_verify: bool,
    }

    impl PublicKey for MockPublicKey {
        fn create_signature_engine(
            &self,
            _algorithm: &str,
        ) -> Result<Box<dyn SignatureEngine>, SignatureException> {
            if self.fail_create {
                return Err(SignatureException::new("create fail"));
            }
            Ok(Box::new(MockEngine {
                received: Vec::new(),
                expected: self.expected.clone(),
                valid_sig: self.valid_sig.clone(),
                fail_update: self.fail_update,
                fail_verify: self.fail_verify,
            }))
        }
    }

    struct MockKeyInfo {
        expected: Vec<u8>,
        valid_sig: Vec<u8>,
        fail_verify: bool,
    }

    impl ServicesKeyInfo for MockKeyInfo {
        fn signature(&self) -> Box<dyn SignatureEngine> {
            Box::new(MockEngine {
                received: Vec::new(),
                expected: self.expected.clone(),
                valid_sig: self.valid_sig.clone(),
                fail_update: false,
                fail_verify: self.fail_verify,
            })
        }
    }

    struct KeyType;

    struct SimpleKeySet {
        entries: Vec<(Vec<u8>, Vec<u8>, bool)>,
    }

    impl ServicesKeySet<KeyType> for SimpleKeySet {
        fn keys(&self, _key_type: &KeyType) -> Vec<Box<dyn ServicesKeyInfo + Send + Sync>> {
            self.entries
                .iter()
                .map(|(expected, valid_sig, fail_verify)| {
                    Box::new(MockKeyInfo {
                        expected: expected.clone(),
                        valid_sig: valid_sig.clone(),
                        fail_verify: *fail_verify,
                    }) as Box<dyn ServicesKeyInfo + Send + Sync>
                })
                .collect()
        }
    }

    #[test]
    fn no_validation_accepts_anything() {
        let v = no_validation();
        let updater =
            |_out: &mut dyn SignatureOutput| -> Result<(), SignatureException> { Ok(()) };
        assert!(v.validate(&updater, &[]));
        assert!(v.validate(&updater, &[1u8, 2, 3, 0xFF, 0x80]));
        assert!(v.validate_bytes(&[0x00, 0xFF], &[0xDE, 0xAD]));
        assert!(v.validate_bytes(&[], &[]));
    }

    #[test]
    fn validate_bytes_feeds_payload_to_updater() {
        let validator = SignatureValidator::new(|updater, sig| {
            struct Capture(Vec<u8>);
            impl SignatureOutput for Capture {
                fn update(&mut self, data: &[u8]) -> Result<(), SignatureException> {
                    self.0.extend_from_slice(data);
                    Ok(())
                }
            }
            let mut cap = Capture(Vec::new());
            if updater.update(&mut cap).is_err() {
                return false;
            }
            cap.0 == b"payload" && sig == b"ok"
        });

        assert!(validator.validate_bytes(b"payload", b"ok"));
        assert!(!validator.validate_bytes(b"payload", b"bad"));
        assert!(!validator.validate_bytes(b"other", b"ok"));
    }

    #[test]
    fn from_public_key_verifies_and_handles_errors() {
        let key = Box::new(MockPublicKey {
            expected: b"data".to_vec(),
            valid_sig: b"sig".to_vec(),
            fail_create: false,
            fail_update: false,
            fail_verify: false,
        });
        let v = from_public_key(key, "TEST".to_string());
        assert!(v.validate_bytes(b"data", b"sig"));
        assert!(!v.validate_bytes(b"data", b"bad"));
        assert!(!v.validate_bytes(b"other", b"sig"));

        let create_fail = Box::new(MockPublicKey {
            expected: b"data".to_vec(),
            valid_sig: b"sig".to_vec(),
            fail_create: true,
            fail_update: false,
            fail_verify: false,
        });
        assert!(!from_public_key(create_fail, "TEST".to_string()).validate_bytes(b"data", b"sig"));

        let update_fail = Box::new(MockPublicKey {
            expected: b"data".to_vec(),
            valid_sig: b"sig".to_vec(),
            fail_create: false,
            fail_update: true,
            fail_verify: false,
        });
        assert!(!from_public_key(update_fail, "TEST".to_string()).validate_bytes(b"data", b"sig"));

        let verify_fail = Box::new(MockPublicKey {
            expected: b"data".to_vec(),
            valid_sig: b"sig".to_vec(),
            fail_create: false,
            fail_update: false,
            fail_verify: true,
        });
        assert!(!from_public_key(verify_fail, "TEST".to_string()).validate_bytes(b"data", b"sig"));
    }

    #[test]
    fn from_services_key_set_empty_and_any_match() {
        let empty = SimpleKeySet { entries: vec![] };
        assert!(from_services_key_set(&empty, &KeyType).is_none());

        let set = SimpleKeySet {
            entries: vec![
                (b"data".to_vec(), b"wrong".to_vec(), false),
                (b"data".to_vec(), b"sig".to_vec(), false),
            ],
        };
        let v = from_services_key_set(&set, &KeyType).expect("non-empty set");
        assert!(v.validate_bytes(b"data", b"sig"));
        assert!(!v.validate_bytes(b"data", b"none"));

        let error_then_ok = SimpleKeySet {
            entries: vec![
                (b"data".to_vec(), b"sig".to_vec(), true),
                (b"data".to_vec(), b"sig".to_vec(), false),
            ],
        };
        let v2 = from_services_key_set(&error_then_ok, &KeyType).expect("non-empty set");
        assert!(v2.validate_bytes(b"data", b"sig"));
    }

    #[test]
    fn updater_error_and_edge_case_bytes() {
        let payload: Vec<u8> = (0u8..=255).collect();
        let sig: Vec<u8> = vec![0x80, 0xFF, 0x00, 0x7F, 0x01];
        let key = Box::new(MockPublicKey {
            expected: payload.clone(),
            valid_sig: sig.clone(),
            fail_create: false,
            fail_update: false,
            fail_verify: false,
        });
        let v = from_public_key(key, "EDGE".to_string());
        assert!(v.validate_bytes(&payload, &sig));
        assert!(!v.validate_bytes(&payload, &[0x00]));

        let bad_updater =
            |_out: &mut dyn SignatureOutput| -> Result<(), SignatureException> {
                Err(SignatureException::new("boom"))
            };
        assert!(!v.validate(&bad_updater, &sig));
    }
}
