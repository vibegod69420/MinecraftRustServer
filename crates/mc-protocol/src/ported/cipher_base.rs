use bytes::{Buf, BufMut, BytesMut};
use std::fmt;

#[derive(Debug)]
pub struct ShortBufferException;

impl fmt::Display for ShortBufferException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShortBufferException")
    }
}

impl std::error::Error for ShortBufferException {}

pub trait Cipher {
    fn get_output_size(&self, input_size: usize) -> usize;
    fn update(
        &mut self,
        input: &[u8],
        input_offset: usize,
        input_len: usize,
        output: &mut [u8],
        output_offset: usize,
    ) -> Result<usize, ShortBufferException>;
}

pub struct CipherBase<C: Cipher> {
    cipher: C,
    heap_in: Vec<u8>,
    heap_out: Vec<u8>,
}

impl<C: Cipher> CipherBase<C> {
    // JAVA: net.minecraft.network.CipherBase#<init>
    pub fn new(cipher: C) -> Self {
        Self {
            cipher,
            heap_in: Vec::new(),
            heap_out: Vec::new(),
        }
    }

    // JAVA: net.minecraft.network.CipherBase#bufToByte
    fn buf_to_byte(&mut self, buf: &mut BytesMut) -> usize {
        let len = buf.remaining();
        if self.heap_in.len() < len {
            self.heap_in = vec![0u8; len];
        }
        buf.copy_to_slice(&mut self.heap_in[..len]);
        len
    }

    // JAVA: net.minecraft.network.CipherBase#decipher
    pub fn decipher(&mut self, input: &mut BytesMut) -> Result<BytesMut, ShortBufferException> {
        let len = self.buf_to_byte(input);
        let out_size = self.cipher.get_output_size(len);
        let mut out_buf = BytesMut::zeroed(out_size);
        let heap_in = &self.heap_in[..len];
        let out_slice = &mut out_buf[..];
        let written = self.cipher.update(heap_in, 0, len, out_slice, 0)?;
        out_buf.truncate(written);
        Ok(out_buf)
    }

    // JAVA: net.minecraft.network.CipherBase#encipher
    pub fn encipher(
        &mut self,
        input: &mut BytesMut,
        output: &mut BytesMut,
    ) -> Result<(), ShortBufferException> {
        let len = self.buf_to_byte(input);
        let out_size = self.cipher.get_output_size(len);
        if self.heap_out.len() < out_size {
            self.heap_out = vec![0u8; out_size];
        }
        let cipher = &mut self.cipher;
        let heap_in = &self.heap_in[..len];
        let heap_out = &mut self.heap_out[..];
        let written = cipher.update(heap_in, 0, len, heap_out, 0)?;
        output.put_slice(&heap_out[..written]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IdentityCipher;

    impl Cipher for IdentityCipher {
        fn get_output_size(&self, input_size: usize) -> usize {
            input_size
        }

        fn update(
            &mut self,
            input: &[u8],
            input_offset: usize,
            input_len: usize,
            output: &mut [u8],
            output_offset: usize,
        ) -> Result<usize, ShortBufferException> {
            output[output_offset..output_offset + input_len]
                .copy_from_slice(&input[input_offset..input_offset + input_len]);
            Ok(input_len)
        }
    }

    struct XorCipher(u8);

    impl Cipher for XorCipher {
        fn get_output_size(&self, input_size: usize) -> usize {
            input_size
        }

        fn update(
            &mut self,
            input: &[u8],
            input_offset: usize,
            input_len: usize,
            output: &mut [u8],
            output_offset: usize,
        ) -> Result<usize, ShortBufferException> {
            for i in 0..input_len {
                output[output_offset + i] = input[input_offset + i] ^ self.0;
            }
            Ok(input_len)
        }
    }

    #[test]
    fn test_decipher_identity() {
        let mut cb = CipherBase::new(IdentityCipher);
        let mut input = BytesMut::from(&b"hello"[..]);
        let out = cb.decipher(&mut input).unwrap();
        assert_eq!(&out[..], b"hello");
        assert!(!input.has_remaining());
    }

    #[test]
    fn test_encipher_identity() {
        let mut cb = CipherBase::new(IdentityCipher);
        let mut input = BytesMut::from(&b"world"[..]);
        let mut output = BytesMut::new();
        cb.encipher(&mut input, &mut output).unwrap();
        assert_eq!(&output[..], b"world");
    }

    #[test]
    fn test_xor_round_trip() {
        let mut enc = CipherBase::new(XorCipher(0x5A));
        let mut dec = CipherBase::new(XorCipher(0x5A));
        let original = b"secret data";
        let mut input = BytesMut::from(&original[..]);
        let mut encrypted = BytesMut::new();
        enc.encipher(&mut input, &mut encrypted).unwrap();
        assert_ne!(&encrypted[..], &original[..]);

        let mut enc_buf = encrypted;
        let decrypted = dec.decipher(&mut enc_buf).unwrap();
        assert_eq!(&decrypted[..], &original[..]);
    }

    #[test]
    fn test_heap_grows() {
        let mut cb = CipherBase::new(IdentityCipher);
        let mut input = BytesMut::from(&[1u8, 2, 3][..]);
        let _ = cb.decipher(&mut input).unwrap();

        let mut input2 = BytesMut::from(&[0u8; 16][..]);
        let out = cb.decipher(&mut input2).unwrap();
        assert_eq!(out.len(), 16);
    }

    #[test]
    fn test_empty() {
        let mut cb = CipherBase::new(IdentityCipher);
        let mut input = BytesMut::new();
        let out = cb.decipher(&mut input).unwrap();
        assert!(out.is_empty());

        let mut input = BytesMut::new();
        let mut output = BytesMut::new();
        cb.encipher(&mut input, &mut output).unwrap();
        assert!(output.is_empty());
    }
}
