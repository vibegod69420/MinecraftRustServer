use bytes::BytesMut;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketType {
    pub name: String,
}

impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolId {
    pub id: String,
}

pub trait PacketListener {}

pub trait Packet<T: PacketListener + ?Sized>: fmt::Debug {
    fn packet_type(&self) -> PacketType;
    fn is_skippable(&self) -> bool;
    fn class_name(&self) -> &'static str;
}

pub trait ProtocolCodec<T: PacketListener + ?Sized> {
    fn encode(
        &self,
        buf: &mut BytesMut,
        packet: &dyn Packet<T>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub trait ProtocolInfo<T: PacketListener + ?Sized> {
    fn codec(&self) -> &dyn ProtocolCodec<T>;
    fn id(&self) -> &ProtocolId;
}

#[derive(Clone, Debug)]
pub struct ChannelContext {
    pub remote_address: String,
}

#[derive(Debug)]
pub struct SkipPacketEncoderException {
    pub cause: String,
}

impl fmt::Display for SkipPacketEncoderException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SkipPacketEncoderException: {}", self.cause)
    }
}

impl Error for SkipPacketEncoderException {}

pub struct JvmProfiler;

impl JvmProfiler {
    pub const INSTANCE: JvmProfiler = JvmProfiler;

    pub fn on_packet_sent(
        &self,
        _protocol_id: &ProtocolId,
        _packet_type: &PacketType,
        _remote: &str,
        _bytes: usize,
    ) {
    }
}

pub struct ProtocolSwapHandler;

impl ProtocolSwapHandler {
    pub fn handle_outbound_terminal_packet<T: PacketListener + ?Sized>(
        _ctx: &ChannelContext,
        _packet: &dyn Packet<T>,
    ) {
    }
}

pub struct PacketEncoder<T: PacketListener + ?Sized, PI: ProtocolInfo<T>> {
    protocol_info: PI,
    _marker: PhantomData<fn() -> T>,
}

impl<T: PacketListener + ?Sized, PI: ProtocolInfo<T>> PacketEncoder<T, PI> {
    // JAVA: net.minecraft.network.PacketEncoder#<init>
    pub fn new(protocol_info: PI) -> Self {
        Self {
            protocol_info,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.PacketEncoder#encode
    pub fn encode(
        &self,
        ctx: &ChannelContext,
        packet: &dyn Packet<T>,
        buf: &mut BytesMut,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let packet_type = packet.packet_type();

        let result: Result<(), Box<dyn Error + Send + Sync>> = (|| {
            self.protocol_info.codec().encode(buf, packet)?;
            let readable_bytes = buf.len();
            JvmProfiler::INSTANCE.on_packet_sent(
                self.protocol_info.id(),
                &packet_type,
                &ctx.remote_address,
                readable_bytes,
            );
            Ok(())
        })();

        ProtocolSwapHandler::handle_outbound_terminal_packet(ctx, packet);

        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                if packet.is_skippable() {
                    Err(Box::new(SkipPacketEncoderException {
                        cause: e.to_string(),
                    }))
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestListener;
    impl PacketListener for TestListener {}

    #[derive(Debug)]
    struct TestPacket {
        ptype: PacketType,
        skippable: bool,
    }

    impl Packet<TestListener> for TestPacket {
        fn packet_type(&self) -> PacketType {
            self.ptype.clone()
        }
        fn is_skippable(&self) -> bool {
            self.skippable
        }
        fn class_name(&self) -> &'static str {
            "TestPacket"
        }
    }

    struct TestCodec {
        fail: bool,
        encoded_count: Arc<AtomicUsize>,
    }

    impl ProtocolCodec<TestListener> for TestCodec {
        fn encode(
            &self,
            buf: &mut BytesMut,
            _packet: &dyn Packet<TestListener>,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            if self.fail {
                return Err("encode fail".into());
            }
            buf.extend_from_slice(&[1, 2, 3, 4]);
            self.encoded_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct TestInfo {
        codec: TestCodec,
        id: ProtocolId,
    }

    impl ProtocolInfo<TestListener> for TestInfo {
        fn codec(&self) -> &dyn ProtocolCodec<TestListener> {
            &self.codec
        }
        fn id(&self) -> &ProtocolId {
            &self.id
        }
    }

    fn make_info(fail: bool, counter: Arc<AtomicUsize>) -> TestInfo {
        TestInfo {
            codec: TestCodec {
                fail,
                encoded_count: counter,
            },
            id: ProtocolId {
                id: "play".to_string(),
            },
        }
    }

    fn make_packet(skippable: bool) -> TestPacket {
        TestPacket {
            ptype: PacketType {
                name: "test_packet".to_string(),
            },
            skippable,
        }
    }

    fn make_ctx() -> ChannelContext {
        ChannelContext {
            remote_address: "127.0.0.1:25565".to_string(),
        }
    }

    #[test]
    fn encode_success_writes_bytes() {
        let counter = Arc::new(AtomicUsize::new(0));
        let encoder = PacketEncoder::new(make_info(false, counter.clone()));
        let packet = make_packet(false);
        let ctx = make_ctx();
        let mut buf = BytesMut::new();

        let res = encoder.encode(&ctx, &packet, &mut buf);
        assert!(res.is_ok());
        assert_eq!(buf.len(), 4);
        assert_eq!(&buf[..], &[1, 2, 3, 4]);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn encode_skippable_error_returns_skip_exception() {
        let counter = Arc::new(AtomicUsize::new(0));
        let encoder = PacketEncoder::new(make_info(true, counter));
        let packet = make_packet(true);
        let ctx = make_ctx();
        let mut buf = BytesMut::new();

        let err = encoder
            .encode(&ctx, &packet, &mut buf)
            .expect_err("should fail");
        assert!(err.downcast_ref::<SkipPacketEncoderException>().is_some());
    }

    #[test]
    fn encode_non_skippable_error_propagates_original() {
        let counter = Arc::new(AtomicUsize::new(0));
        let encoder = PacketEncoder::new(make_info(true, counter));
        let packet = make_packet(false);
        let ctx = make_ctx();
        let mut buf = BytesMut::new();

        let err = encoder
            .encode(&ctx, &packet, &mut buf)
            .expect_err("should fail");
        assert!(err.downcast_ref::<SkipPacketEncoderException>().is_none());
        assert_eq!(err.to_string(), "encode fail");
    }

    #[test]
    fn encode_empty_buffer_readable_bytes_is_zero_on_precheck() {
        // Edge case: a codec that writes nothing still succeeds; readable bytes is 0.
        struct NoWriteCodec;
        impl ProtocolCodec<TestListener> for NoWriteCodec {
            fn encode(
                &self,
                _buf: &mut BytesMut,
                _packet: &dyn Packet<TestListener>,
            ) -> Result<(), Box<dyn Error + Send + Sync>> {
                Ok(())
            }
        }
        struct NoWriteInfo {
            codec: NoWriteCodec,
            id: ProtocolId,
        }
        impl ProtocolInfo<TestListener> for NoWriteInfo {
            fn codec(&self) -> &dyn ProtocolCodec<TestListener> {
                &self.codec
            }
            fn id(&self) -> &ProtocolId {
                &self.id
            }
        }

        let encoder = PacketEncoder::new(NoWriteInfo {
            codec: NoWriteCodec,
            id: ProtocolId {
                id: "play".to_string(),
            },
        });
        let packet = make_packet(false);
        let ctx = make_ctx();
        let mut buf = BytesMut::new();
        assert!(encoder.encode(&ctx, &packet, &mut buf).is_ok());
        assert_eq!(buf.len(), 0);
    }
}
