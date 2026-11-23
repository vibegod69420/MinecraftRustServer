use std::error::Error;

pub type SignError = Box<dyn Error + Send + Sync>;

// JAVA: net.minecraft.util.SignatureUpdater
pub trait SignatureUpdater {
    // JAVA: net.minecraft.util.SignatureUpdater#update
    fn update(
        &self,
        output: &mut dyn FnMut(&[u8]) -> Result<(), SignError>,
    ) -> Result<(), SignError>;
}

impl<F> SignatureUpdater for F
where
    F: Fn(&mut dyn FnMut(&[u8]) -> Result<(), SignError>) -> Result<(), SignError>,
{
    fn update(
        &self,
        output: &mut dyn FnMut(&[u8]) -> Result<(), SignError>,
    ) -> Result<(), SignError> {
        (self)(output)
    }
}

// JAVA: net.minecraft.util.Signer
pub trait Signer {
    // JAVA: net.minecraft.util.Signer#sign
    fn sign(&self, updater: &dyn SignatureUpdater) -> Result<Vec<u8>, SignError>;

    // JAVA: net.minecraft.util.Signer#sign
    fn sign_bytes(&self, data: &[u8]) -> Result<Vec<u8>, SignError> {
        struct BytesUpdater<'a>(&'a [u8]);
        impl SignatureUpdater for BytesUpdater<'_> {
            fn update(
                &self,
                output: &mut dyn FnMut(&[u8]) -> Result<(), SignError>,
            ) -> Result<(), SignError> {
                output(self.0)
            }
        }
        self.sign(&BytesUpdater(data))
    }
}

/// Rust adaptation of `Signer.from(PrivateKey, String)`.
/// The Java version constructs a `java.security.Signature` with the given key/algorithm;
/// here the caller supplies the signing implementation that receives the updater.
pub struct FromFn<F>(F);

// JAVA: net.minecraft.util.Signer#from
pub fn from<F>(sign_fn: F) -> FromFn<F>
where
    F: Fn(&dyn SignatureUpdater) -> Result<Vec<u8>, SignError>,
{
    FromFn(sign_fn)
}

impl<F> Signer for FromFn<F>
where
    F: Fn(&dyn SignatureUpdater) -> Result<Vec<u8>, SignError>,
{
    // JAVA: net.minecraft.util.Signer#sign
    fn sign(&self, updater: &dyn SignatureUpdater) -> Result<Vec<u8>, SignError> {
        (self.0)(updater).map_err(|e| -> SignError {
            format!("Failed to sign message: {e}").into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{from, SignError, SignatureUpdater, Signer};

    struct CollectSigner;

    impl Signer for CollectSigner {
        fn sign(&self, updater: &dyn SignatureUpdater) -> Result<Vec<u8>, SignError> {
            let mut buf = Vec::new();
            updater.update(&mut |data| {
                buf.extend_from_slice(data);
                Ok(())
            })?;
            Ok(buf)
        }
    }

    #[test]
    fn sign_bytes_uses_default_updater() {
        let signer = CollectSigner;
        let out = signer.sign_bytes(b"hello").expect("sign");
        assert_eq!(out, b"hello");
    }

    #[test]
    fn from_wraps_error_message() {
        let signer = from(|_updater: &dyn SignatureUpdater| -> Result<Vec<u8>, SignError> {
            Err("boom".into())
        });
        let err = signer.sign_bytes(b"data").unwrap_err();
        assert_eq!(err.to_string(), "Failed to sign message: boom");
    }

    #[test]
    fn from_passes_updater_data() {
        let signer = from(|updater: &dyn SignatureUpdater| -> Result<Vec<u8>, SignError> {
            let mut buf = Vec::new();
            updater.update(&mut |data| {
                buf.extend_from_slice(data);
                Ok(())
            })?;
            buf.reverse();
            Ok(buf)
        });
        let out = signer.sign_bytes(&[1, 2, 3]).expect("sign");
        assert_eq!(out, vec![3, 2, 1]);
    }

    #[test]
    fn closure_signature_updater_impl_works() {
        let signer = CollectSigner;
        let updater = |output: &mut dyn FnMut(&[u8]) -> Result<(), SignError>| -> Result<(), SignError> {
            output(b"ab")?;
            output(b"cd")?;
            Ok(())
        };
        let out = signer.sign(&updater).expect("sign");
        assert_eq!(out, b"abcd");
    }
}
