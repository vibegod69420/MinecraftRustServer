use std::marker::PhantomData;

/// Minimal placeholder for `net.minecraft.network.PacketListener`.
pub trait PacketListener {}

/// Minimal placeholder for `io.netty.buffer.ByteBuf`.
pub trait ByteBuf {}

/// Minimal placeholder for `net.minecraft.network.ProtocolInfo`.
pub struct ProtocolInfo<T: ?Sized + PacketListener> {
    pub id: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized + PacketListener> ProtocolInfo<T> {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            _marker: PhantomData,
        }
    }
}

/// Minimal placeholder for `net.minecraft.network.ProtocolInfo.DetailsProvider`.
pub trait ProtocolInfoDetailsProvider {}

/// JAVA: net.minecraft.network.protocol.SimpleUnboundProtocol
pub trait SimpleUnboundProtocol<T: PacketListener, B: ByteBuf>: ProtocolInfoDetailsProvider {
    // JAVA: net.minecraft.network.protocol.SimpleUnboundProtocol#bind
    fn bind<F>(&self, buf_factory: F) -> ProtocolInfo<T>
    where
        F: Fn(Box<dyn ByteBuf>) -> B;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener;
    impl PacketListener for TestListener {}

    struct TestBuf {
        data: Vec<u8>,
    }
    impl ByteBuf for TestBuf {}

    struct RawBuf;
    impl ByteBuf for RawBuf {}

    struct TestProtocol {
        name: &'static str,
    }
    impl ProtocolInfoDetailsProvider for TestProtocol {}

    impl SimpleUnboundProtocol<TestListener, TestBuf> for TestProtocol {
        // JAVA: net.minecraft.network.protocol.SimpleUnboundProtocol#bind
        fn bind<F>(&self, buf_factory: F) -> ProtocolInfo<TestListener>
        where
            F: Fn(Box<dyn ByteBuf>) -> TestBuf,
        {
            let raw: Box<dyn ByteBuf> = Box::new(RawBuf);
            let converted = buf_factory(raw);
            ProtocolInfo::new(format!("{}:{}", self.name, converted.data.len()))
        }
    }

    #[test]
    fn bind_returns_protocol_info_with_expected_id() {
        let proto = TestProtocol { name: "play" };
        let info = proto.bind(|_b| TestBuf { data: vec![1, 2, 3] });
        assert_eq!(info.id, "play:3");
    }

    #[test]
    fn bind_with_empty_buffer() {
        let proto = TestProtocol { name: "empty" };
        let info = proto.bind(|_b| TestBuf { data: vec![] });
        assert_eq!(info.id, "empty:0");
    }

    #[test]
    fn bind_with_negative_like_byte_values() {
        let proto = TestProtocol { name: "bytes" };
        let info = proto.bind(|_b| TestBuf {
            data: vec![0xFF, 0x80, 0x00],
        });
        assert_eq!(info.id, "bytes:3");
    }

    #[test]
    fn bind_factory_receives_input() {
        struct CountingProtocol;
        impl ProtocolInfoDetailsProvider for CountingProtocol {}
        impl SimpleUnboundProtocol<TestListener, TestBuf> for CountingProtocol {
            // JAVA: net.minecraft.network.protocol.SimpleUnboundProtocol#bind
            fn bind<F>(&self, buf_factory: F) -> ProtocolInfo<TestListener>
            where
                F: Fn(Box<dyn ByteBuf>) -> TestBuf,
            {
                let raw: Box<dyn ByteBuf> = Box::new(RawBuf);
                let buf = buf_factory(raw);
                let sum = buf.data.iter().fold(0i32, |acc, &b| acc.wrapping_add(b as i32));
                ProtocolInfo::new(format!("sum={sum}"))
            }
        }
        let proto = CountingProtocol;
        let info = proto.bind(|_b| TestBuf { data: vec![10, 20, u8::MAX] });
        assert_eq!(info.id, "sum=285");
    }
}
