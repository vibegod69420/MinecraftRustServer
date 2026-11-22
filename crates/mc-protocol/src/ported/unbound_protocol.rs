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

/// JAVA: net.minecraft.network.protocol.UnboundProtocol
pub trait UnboundProtocol<T: PacketListener, B: ByteBuf, C>: ProtocolInfoDetailsProvider {
    // JAVA: net.minecraft.network.protocol.UnboundProtocol#bind
    fn bind<F>(&self, buf_factory: F, context: C) -> ProtocolInfo<T>
    where
        F: Fn(Box<dyn ByteBuf>) -> B;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener;
    impl PacketListener for TestListener {}

    struct TestBuf {
        bytes: Vec<u8>,
    }
    impl ByteBuf for TestBuf {}

    struct RawBuf;
    impl ByteBuf for RawBuf {}

    struct TestProtocol;
    impl ProtocolInfoDetailsProvider for TestProtocol {}

    impl UnboundProtocol<TestListener, TestBuf, i32> for TestProtocol {
        // JAVA: net.minecraft.network.protocol.UnboundProtocol#bind
        fn bind<F>(&self, buf_factory: F, context: i32) -> ProtocolInfo<TestListener>
        where
            F: Fn(Box<dyn ByteBuf>) -> TestBuf,
        {
            let raw: Box<dyn ByteBuf> = Box::new(RawBuf);
            let buf = buf_factory(raw);
            ProtocolInfo::new(format!("ctx={};len={}", context, buf.bytes.len()))
        }
    }

    #[test]
    fn bind_uses_context_and_factory() {
        let proto = TestProtocol;
        let info = proto.bind(|_b| TestBuf { bytes: vec![1, 2] }, 7);
        assert_eq!(info.id, "ctx=7;len=2");
    }

    #[test]
    fn bind_with_negative_context() {
        let proto = TestProtocol;
        let info = proto.bind(|_b| TestBuf { bytes: vec![] }, -1_i32);
        assert_eq!(info.id, "ctx=-1;len=0");
    }

    #[test]
    fn bind_with_zero_context_and_bytes() {
        let proto = TestProtocol;
        let info = proto.bind(|_b| TestBuf { bytes: vec![0xFF, 0x80] }, 0);
        assert_eq!(info.id, "ctx=0;len=2");
    }

    #[test]
    fn bind_with_large_context_value() {
        let proto = TestProtocol;
        let ctx = i32::MAX;
        let info = proto.bind(|_b| TestBuf { bytes: vec![1] }, ctx);
        assert_eq!(info.id, format!("ctx={};len=1", i32::MAX));
    }

    #[test]
    fn bind_with_min_context_value() {
        let proto = TestProtocol;
        let ctx = i32::MIN;
        let info = proto.bind(|_b| TestBuf { bytes: vec![] }, ctx);
        assert_eq!(info.id, format!("ctx={};len=0", i32::MIN));
    }
}
