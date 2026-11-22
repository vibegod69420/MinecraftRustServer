use bytes::{Buf, BytesMut};
use std::fmt;

/// Placeholder for Netty's `ChannelHandlerContext`.
#[derive(Debug, Default)]
pub struct ChannelHandlerContext;

#[derive(Debug)]
pub struct CipherError {
    pub message: String,
}

impl fmt::Display for CipherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CipherError {}

/// Minimal abstraction representing `javax.crypto.Cipher` for self-contained porting.
pub trait JavaCipher {
    fn get_output_size(&self, input_len: usize) -> usize;
    fn update(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, CipherError>;
}

/// Minimal local stand-in for `net.minecraft.network.CipherBase`.
pub struct CipherBase {
    cipher: Box<dyn JavaCipher>,
}

impl CipherBase {
    pub fn new(cipher: Box<dyn JavaCipher>) -> Self {
        Self { cipher }
    }

    pub fn encipher(&mut self, input: &mut BytesMut, output: &mut BytesMut) -> Result<(), CipherError> {
        let in_len = input.remaining();
        if in_len == 0 {
            return Ok(());
        }
        let input_bytes = input.copy_to_bytes(in_len);
        let out_len = self.cipher.get_output_size(in_len);
        let mut out_buf = vec![0u8; out_len];
        let written = self.cipher.update(&input_bytes, &mut out_buf)?;
        output.extend_from_slice(&out_buf[..written]);
        Ok(())
    }
}

pub struct CipherEncoder {
    cipher: CipherBase,
}

impl CipherEncoder {
    // JAVA: net.minecraft.network.CipherEncoder#<init>
    pub fn new(cipher: Box<dyn JavaCipher>) -> Self {
        Self {
            cipher: CipherBase::new(cipher),
        }
    }

    // JAVA: net.minecraft.network.CipherEncoder#encode
    pub fn encode(
        &mut self,
        _ctx: &mut ChannelHandlerContext,
        msg: &mut BytesMut,
        out: &mut BytesMut,
    ) -> Result<(), CipherError> {
        self.cipher.encipher(msg, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IdentityCipher;
    impl JavaCipher for IdentityCipher {
        fn get_output_size(&self, input_len: usize) -> usize {
            input_len
        }
        fn update(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, CipherError> {
            output[..input.len()].copy_from_slice(input);
            Ok(input.len())
        }
    }

    struct XorCipher(u8);
    impl JavaCipher for XorCipher {
        fn get_output_size(&self, input_len: usize) -> usize {
            input_len
        }
        fn update(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, CipherError> {
            for (i, b) in input.iter().enumerate() {
                output[i] = b ^ self.0;
            }
            Ok(input.len())
        }
    }

    #[test]
    fn encode_identity_copies_bytes() {
        let mut enc = CipherEncoder::new(Box::new(IdentityCipher));
        let mut ctx = ChannelHandlerContext;
        let mut msg = BytesMut::from(&[1u8, 2, 3, 4][..]);
        let mut out = BytesMut::new();
        enc.encode(&mut ctx, &mut msg, &mut out).unwrap();
        assert_eq!(&out[..], &[1, 2, 3, 4]);
        assert!(!msg.has_remaining());
    }

    #[test]
    fn encode_xor_transforms_bytes() {
        let mut enc = CipherEncoder::new(Box::new(XorCipher(0xFF)));
        let mut ctx = ChannelHandlerContext;
        let mut msg = BytesMut::from(&[0x00u8, 0x0F, 0xF0][..]);
        let mut out = BytesMut::new();
        enc.encode(&mut ctx, &mut msg, &mut out).unwrap();
        assert_eq!(&out[..], &[0xFF, 0xF0, 0x0F]);
    }

    #[test]
    fn encode_empty_input_produces_no_output() {
        let mut enc = CipherEncoder::new(Box::new(IdentityCipher));
        let mut ctx = ChannelHandlerContext;
        let mut msg = BytesMut::new();
        let mut out = BytesMut::new();
        enc.encode(&mut ctx, &mut msg, &mut out).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn encode_propagates_cipher_error() {
        struct FailCipher;
        impl JavaCipher for FailCipher {
            fn get_output_size(&self, input_len: usize) -> usize {
                input_len
            }
            fn update(&mut self, _: &[u8], _: &mut [u8]) -> Result<usize, CipherError> {
                Err(CipherError {
                    message: "boom".into(),
                })
            }
        }
        let mut enc = CipherEncoder::new(Box::new(FailCipher));
        let mut ctx = ChannelHandlerContext;
        let mut msg = BytesMut::from(&[1u8][..]);
        let mut out = BytesMut::new();
        let err = enc.encode(&mut ctx, &mut msg, &mut out).unwrap_err();
        assert_eq!(err.to_string(), "boom");
    }
}
