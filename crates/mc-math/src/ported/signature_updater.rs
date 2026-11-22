use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
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
        write!(f, "{}", self.message)
    }
}

impl Error for SignatureException {}

pub trait Output {
    // JAVA: net.minecraft.util.SignatureUpdater$Output#update
    fn update(&mut self, data: &[u8]) -> Result<(), SignatureException>;
}

impl<F> Output for F
where
    F: FnMut(&[u8]) -> Result<(), SignatureException>,
{
    fn update(&mut self, data: &[u8]) -> Result<(), SignatureException> {
        self(data)
    }
}

pub trait SignatureUpdater {
    // JAVA: net.minecraft.util.SignatureUpdater#update
    fn update(&mut self, output: &mut dyn Output) -> Result<(), SignatureException>;
}

impl<F> SignatureUpdater for F
where
    F: FnMut(&mut dyn Output) -> Result<(), SignatureException>,
{
    fn update(&mut self, output: &mut dyn Output) -> Result<(), SignatureException> {
        self(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_receives_bytes() {
        let mut collected: Vec<u8> = Vec::new();
        let mut out = |data: &[u8]| -> Result<(), SignatureException> {
            collected.extend_from_slice(data);
            Ok(())
        };
        Output::update(&mut out, &[1, 2, 3]).unwrap();
        Output::update(&mut out, &[4]).unwrap();
        assert_eq!(collected, vec![1, 2, 3, 4]);
    }

    #[test]
    fn output_propagates_error() {
        let mut out = |_data: &[u8]| -> Result<(), SignatureException> {
            Err(SignatureException::new("bad signature"))
        };
        let err = Output::update(&mut out, &[0]).unwrap_err();
        assert_eq!(err.message, "bad signature");
        assert_eq!(err.to_string(), "bad signature");
    }

    #[test]
    fn signature_updater_feeds_output() {
        let mut updater = |output: &mut dyn Output| -> Result<(), SignatureException> {
            output.update(b"hello")?;
            output.update(b" world")
        };
        let mut collected: Vec<u8> = Vec::new();
        let mut out = |data: &[u8]| -> Result<(), SignatureException> {
            collected.extend_from_slice(data);
            Ok(())
        };
        SignatureUpdater::update(&mut updater, &mut out).unwrap();
        assert_eq!(collected, b"hello world");
    }

    #[test]
    fn signature_updater_propagates_output_error() {
        let mut updater = |output: &mut dyn Output| -> Result<(), SignatureException> {
            output.update(b"fail")
        };
        let mut out = |_data: &[u8]| -> Result<(), SignatureException> {
            Err(SignatureException::new("output error"))
        };
        let err = SignatureUpdater::update(&mut updater, &mut out).unwrap_err();
        assert_eq!(err.message, "output error");
    }

    #[test]
    fn signature_exception_display_and_debug() {
        let err = SignatureException::new("msg");
        assert_eq!(format!("{}", err), "msg");
        let dbg = format!("{:?}", err);
        assert!(dbg.contains("SignatureException"));
        assert!(dbg.contains("msg"));
    }
}
