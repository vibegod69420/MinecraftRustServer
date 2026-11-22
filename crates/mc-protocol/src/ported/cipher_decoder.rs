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

    pub fn decipher(
        &mut self,
        _ctx: &mut ChannelHandlerContext,
        input: &mut BytesMut,
    ) -> Result<BytesMut, CipherError> {
        let in_len = input.remaining();
        if in_len == 0 {
            return Ok(BytesMut::new());
        }
        let input_bytes = input.copy_to_bytes(in_len);
        let out_len = self.cipher.get_output_size(in_len);
        let mut out_buf = vec![0u8; out_len];
        let written = self.cipher.update(&input_bytes, &mut out_buf)?;
        Ok(BytesMut::from(&out_buf[..written]))
    }
}

pub struct CipherDecoder {
    cipher: CipherBase,
}

impl CipherDecoder {
    // JAVA: net.minecraft.network.CipherDecoder#<init>
    pub fn new(cipher: Box<dyn JavaCipher>) -> Self {
        Self {
            cipher: CipherBase::new(cipher),
        }
    }

    // JAVA: net.minecraft.network.CipherDecoder#decode
    pub fn decode(
        &mut self,
        ctx: &mut ChannelHandlerContext,
        msg: &mut BytesMut,
        out: &mut Vec<BytesMut>,
    ) -> Result<(), CipherError> {
        let decoded = self.cipher.decipher(ctx, msg)?;
        out.push(decoded);
        Ok(())
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

    #[test]
    fn decode_identity_produces_single_output() {
        let mut dec = CipherDecoder::new(Box::new(IdentityCipher));
        let mut ctx = ChannelHandlerContext;
        let mut msg = BytesMut::from(&[10u8, 20, 30][..]);
        let mut out = Vec::new();
        dec.decode(&mut ctx, &mut msg, &mut out).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(&out[0][..], &[10, 20, 30]);
        assert!(!msg.has_remaining());
    }

    #[test]
    fn decode_empty_input_pushes_empty_buffer() {
        let mut dec = CipherDecoder::new(Box::new(IdentityCipher));
        let mut ctx = ChannelHandlerContext;
        let mut msg = BytesMut::new();
        let mut out = Vec::new();
        dec.decode(&mut ctx, &mut msg, &mut out).unwrap();
        assert_eq!(out.len(), 1);
        assert!(out[0].is_empty());
    }

    #[test]
    fn decode_propagates_cipher_error() {
        struct FailCipher;
        impl JavaCipher for FailCipher {
            fn get_output_size(&self, input_len: usize) -> usize {
                input_len
            }
            fn update(&mut self, _: &[u8], _: &mut [u8]) -> Result<usize, CipherError> {
                Err(CipherError {
                    message: "decrypt fail".into(),
                })
            }
        }
        let mut dec = CipherDecoder::new(Box::new(FailCipher));
        let mut ctx = ChannelHandlerContext;
        let mut msg = BytesMut::from(&[1u8, 2][..]);
        let mut out = Vec::new();
        let err = dec.decode(&mut ctx, &mut msg, &mut out).unwrap_err();
        assert_eq!(err.to_string(), "decrypt fail");
        assert!(out.is_empty());
    }

    #[test]
    fn decode_multiple_calls_append_outputs() {
        let mut dec = CipherDecoder::new(Box::new(IdentityCipher));
        let mut ctx = ChannelHandlerContext;
        let mut out = Vec::new();
        let mut msg1 = BytesMut::from(&[1u8][..]);
        let mut msg2 = BytesMut::from(&[2u8][..]);
        dec.decode(&mut ctx, &mut msg1, &mut out).unwrap();
        dec.decode(&mut ctx, &mut msg2, &mut out).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(&out[0][..], &[1]);
        assert_eq!(&out[1][..], &[2]);
    }
}
