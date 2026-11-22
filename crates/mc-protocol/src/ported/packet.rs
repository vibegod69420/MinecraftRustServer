//! Port of `net.minecraft.network.protocol.Packet`.

use std::fmt;
use std::marker::PhantomData;

/// Placeholder for `net.minecraft.network.PacketListener`.
pub trait PacketListener {}

/// Placeholder for Netty `ByteBuf`.
pub trait ByteBuf {}

/// Placeholder for `net.minecraft.network.protocol.PacketType`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PacketType<P: ?Sized> {
    id: i32,
    _marker: PhantomData<fn() -> P>,
}

impl<P: ?Sized> PacketType<P> {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}

impl<P: ?Sized> fmt::Debug for PacketType<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PacketType").field("id", &self.id).finish()
    }
}

/// Placeholder for `net.minecraft.network.codec.StreamMemberEncoder`.
pub trait StreamMemberEncoder<B: ?Sized, T: ?Sized> {
    fn encode(&self, value: &T, buf: &mut B);
}

impl<B: ?Sized, T: ?Sized, F: Fn(&T, &mut B)> StreamMemberEncoder<B, T> for F {
    fn encode(&self, value: &T, buf: &mut B) {
        (self)(value, buf);
    }
}

/// Placeholder for `net.minecraft.network.codec.StreamDecoder`.
pub trait StreamDecoder<B: ?Sized, T> {
    fn decode(&self, buf: &mut B) -> T;
}

impl<B: ?Sized, T, F: Fn(&mut B) -> T> StreamDecoder<B, T> for F {
    fn decode(&self, buf: &mut B) -> T {
        (self)(buf)
    }
}

/// Placeholder for `net.minecraft.network.codec.StreamCodec`.
pub struct StreamCodec<B: ?Sized, T> {
    encoder: Box<dyn Fn(&T, &mut B)>,
    decoder: Box<dyn Fn(&mut B) -> T>,
}

impl<B: ?Sized + 'static, T: 'static> StreamCodec<B, T> {
    /// Placeholder for `StreamCodec.ofMember`.
    pub fn of_member(
        encoder: impl Fn(&T, &mut B) + 'static,
        decoder: impl Fn(&mut B) -> T + 'static,
    ) -> Self {
        Self {
            encoder: Box::new(encoder),
            decoder: Box::new(decoder),
        }
    }

    pub fn encode(&self, value: &T, buf: &mut B) {
        (self.encoder)(value, buf);
    }

    pub fn decode(&self, buf: &mut B) -> T {
        (self.decoder)(buf)
    }
}

/// Port of `net.minecraft.network.protocol.Packet`.
pub trait Packet<T: PacketListener> {
    // JAVA: net.minecraft.network.protocol.Packet#type
    fn packet_type(&self) -> PacketType<Self>
    where
        Self: Sized;

    // JAVA: net.minecraft.network.protocol.Packet#handle
    fn handle(&self, listener: &mut T);

    // JAVA: net.minecraft.network.protocol.Packet#isSkippable
    fn is_skippable(&self) -> bool {
        false
    }

    // JAVA: net.minecraft.network.protocol.Packet#isTerminal
    fn is_terminal(&self) -> bool {
        false
    }
}

// JAVA: net.minecraft.network.protocol.Packet#codec
pub fn codec<B, T, E, D>(encoder: E, decoder: D) -> StreamCodec<B, T>
where
    B: ?Sized + 'static,
    T: 'static,
    E: StreamMemberEncoder<B, T> + 'static,
    D: StreamDecoder<B, T> + 'static,
{
    StreamCodec::of_member(
        move |value: &T, buf: &mut B| encoder.encode(value, buf),
        move |buf: &mut B| decoder.decode(buf),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBuf {
        data: Vec<u8>,
        pos: usize,
    }

    impl ByteBuf for TestBuf {}

    impl TestBuf {
        fn new() -> Self {
            Self {
                data: Vec::new(),
                pos: 0,
            }
        }

        fn write_i32(&mut self, v: i32) {
            self.data.extend_from_slice(&v.to_be_bytes());
        }

        fn read_i32(&mut self) -> i32 {
            let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into().unwrap();
            self.pos += 4;
            i32::from_be_bytes(bytes)
        }

        fn write_f32(&mut self, v: f32) {
            self.data.extend_from_slice(&v.to_bits().to_be_bytes());
        }

        fn read_f32(&mut self) -> f32 {
            let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into().unwrap();
            self.pos += 4;
            f32::from_bits(u32::from_be_bytes(bytes))
        }

        fn write_f64(&mut self, v: f64) {
            self.data.extend_from_slice(&v.to_bits().to_be_bytes());
        }

        fn read_f64(&mut self) -> f64 {
            let bytes: [u8; 8] = self.data[self.pos..self.pos + 8].try_into().unwrap();
            self.pos += 8;
            f64::from_bits(u64::from_be_bytes(bytes))
        }
    }

    struct TestListener {
        handled: i32,
    }

    impl PacketListener for TestListener {}

    #[derive(Debug, Clone, PartialEq)]
    struct TestPacket {
        value: i32,
    }

    impl Packet<TestListener> for TestPacket {
        fn packet_type(&self) -> PacketType<Self> {
            PacketType::new(42)
        }

        fn handle(&self, listener: &mut TestListener) {
            listener.handled = listener.handled.wrapping_add(self.value);
        }
    }

    #[test]
    fn defaults_are_false() {
        let p = TestPacket { value: 0 };
        assert!(!p.is_skippable());
        assert!(!p.is_terminal());
    }

    #[test]
    fn packet_type_and_handle() {
        let p = TestPacket { value: 7 };
        assert_eq!(p.packet_type().id(), 42);
        let mut listener = TestListener { handled: 0 };
        p.handle(&mut listener);
        assert_eq!(listener.handled, 7);
    }

    #[test]
    fn codec_round_trip_i32_edge_cases() {
        let c = codec::<TestBuf, i32, _, _>(
            |v: &i32, buf: &mut TestBuf| buf.write_i32(*v),
            |buf: &mut TestBuf| buf.read_i32(),
        );
        for &v in &[i32::MIN, i32::MAX, -1, 0, 1] {
            let mut buf = TestBuf::new();
            c.encode(&v, &mut buf);
            buf.pos = 0;
            let decoded = c.decode(&mut buf);
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn codec_round_trip_float_bits() {
        let cf = codec::<TestBuf, f32, _, _>(
            |v: &f32, buf: &mut TestBuf| buf.write_f32(*v),
            |buf: &mut TestBuf| buf.read_f32(),
        );
        let cd = codec::<TestBuf, f64, _, _>(
            |v: &f64, buf: &mut TestBuf| buf.write_f64(*v),
            |buf: &mut TestBuf| buf.read_f64(),
        );

        let f32_cases = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.0_f32, 1.5_f32];
        for &v in &f32_cases {
            let mut buf = TestBuf::new();
            cf.encode(&v, &mut buf);
            buf.pos = 0;
            let decoded = cf.decode(&mut buf);
            assert_eq!(decoded.to_bits(), v.to_bits());
        }

        let f64_cases = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0_f64, f64::MIN_POSITIVE];
        for &v in &f64_cases {
            let mut buf = TestBuf::new();
            cd.encode(&v, &mut buf);
            buf.pos = 0;
            let decoded = cd.decode(&mut buf);
            assert_eq!(decoded.to_bits(), v.to_bits());
        }
    }

    #[test]
    fn codec_uses_stream_member_traits() {
        struct Enc;
        impl StreamMemberEncoder<TestBuf, i32> for Enc {
            fn encode(&self, value: &i32, buf: &mut TestBuf) {
                buf.write_i32(value.wrapping_mul(2));
            }
        }
        struct Dec;
        impl StreamDecoder<TestBuf, i32> for Dec {
            fn decode(&self, buf: &mut TestBuf) -> i32 {
                buf.read_i32().wrapping_div(2)
            }
        }

        let c = codec::<TestBuf, i32, _, _>(Enc, Dec);
        let mut buf = TestBuf::new();
        c.encode(&-7_i32, &mut buf);
        buf.pos = 0;
        assert_eq!(c.decode(&mut buf), -7_i32);
    }
}
