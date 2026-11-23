use std::fmt;
use std::io;

use bytes::Buf;

/// JAVA: net.minecraft.network.SkipPacketException
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipPacketException;

impl fmt::Display for SkipPacketException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SkipPacketException")
    }
}

impl std::error::Error for SkipPacketException {}

/// Minimal protocol id abstraction for the decoder.
/// JAVA: net.minecraft.network.ProtocolInfo#id (shape)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtocolId {
    pub id: &'static str,
}

impl ProtocolId {
    pub const fn new(id: &'static str) -> Self {
        Self { id }
    }
}

/// Marker for a decoded packet type.
/// JAVA: net.minecraft.network.protocol.PacketType
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PacketType {
    pub name: &'static str,
}

impl PacketType {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// Decoded packet value.
/// JAVA: net.minecraft.network.protocol.Packet
pub trait Packet: fmt::Debug {
    fn packet_type(&self) -> PacketType;
    fn class_name(&self) -> &'static str;

    /// Hook equivalent to `ProtocolSwapHandler.handleInboundTerminalPacket`.
    /// Default is no-op.
    fn handle_inbound_terminal(&self) {}
}

/// Packet listener marker.
/// JAVA: net.minecraft.network.PacketListener
pub trait PacketListener {}

/// Codec for decoding packets from a byte buffer.
/// JAVA: net.minecraft.network.codec.StreamCodec / ProtocolInfo.codec()
pub trait PacketCodec {
    type Listener: PacketListener;
    type Packet: Packet;

    fn decode(&self, buf: &mut dyn Buf) -> Result<Self::Packet, Box<dyn std::error::Error>>;
}

/// Protocol information required by the decoder.
/// JAVA: net.minecraft.network.ProtocolInfo
pub trait ProtocolInfo {
    type Listener: PacketListener;
    type Packet: Packet;
    type Codec: PacketCodec<Listener = Self::Listener, Packet = Self::Packet>;

    fn id(&self) -> ProtocolId;
    fn codec(&self) -> &Self::Codec;
}

/// Decode event observer, used to model `JvmProfiler.INSTANCE.onPacketReceived(...)`.
pub trait PacketDecodeObserver {
    fn on_packet_received(
        &self,
        protocol_id: ProtocolId,
        packet_type: &PacketType,
        remote_address: Option<&str>,
        readable_bytes: usize,
    );
}

/// No-op observer.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopPacketDecodeObserver;

impl PacketDecodeObserver for NoopPacketDecodeObserver {
    fn on_packet_received(
        &self,
        _protocol_id: ProtocolId,
        _packet_type: &PacketType,
        _remote_address: Option<&str>,
        _readable_bytes: usize,
    ) {
    }
}

/// Logger hook equivalent to SLF4J debug logging in Java.
pub trait PacketDecodeLogger {
    fn is_debug_enabled(&self) -> bool;
    fn debug_packet_received(
        &self,
        protocol_id: ProtocolId,
        packet_type: &PacketType,
        packet_class_name: &str,
        readable_bytes: usize,
    );
}

/// No-op logger.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopPacketDecodeLogger;

impl PacketDecodeLogger for NoopPacketDecodeLogger {
    fn is_debug_enabled(&self) -> bool {
        false
    }

    fn debug_packet_received(
        &self,
        _protocol_id: ProtocolId,
        _packet_type: &PacketType,
        _packet_class_name: &str,
        _readable_bytes: usize,
    ) {
    }
}

/// Errors produced by packet decoding.
#[derive(Debug)]
pub enum PacketDecodeError {
    Decode(Box<dyn std::error::Error>),
    ExtraBytes {
        protocol_id: &'static str,
        packet_type: String,
        packet_class_name: &'static str,
        extra_bytes: usize,
    },
}

impl fmt::Display for PacketDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketDecodeError::Decode(e) => write!(f, "{e}"),
            PacketDecodeError::ExtraBytes {
                protocol_id,
                packet_type,
                packet_class_name,
                extra_bytes,
            } => write!(
                f,
                "Packet {protocol_id}/{packet_type} ({packet_class_name}) was larger than I expected, found {extra_bytes} bytes extra whilst reading packet {packet_type}"
            ),
        }
    }
}

impl std::error::Error for PacketDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PacketDecodeError::Decode(e) => Some(e.as_ref()),
            PacketDecodeError::ExtraBytes { .. } => None,
        }
    }
}

/// Rust port of Netty `PacketDecoder`.
///
/// Netty-specific channel/pipeline behavior is represented through explicit
/// inputs/outputs: decode reads from a `Buf`, emits packets into a `Vec`,
/// and invokes observer/logger/hooks.
/// JAVA: net.minecraft.network.PacketDecoder
#[derive(Debug)]
pub struct PacketDecoder<P: ProtocolInfo> {
    protocol_info: P,
}

impl<P: ProtocolInfo> PacketDecoder<P> {
    // JAVA: net.minecraft.network.PacketDecoder#<init>
    pub fn new(protocol_info: P) -> Self {
        Self { protocol_info }
    }

    pub fn protocol_info(&self) -> &P {
        &self.protocol_info
    }

    /// Decode one packet from `buf`, mirroring Java `decode`.
    ///
    /// - `remote_address`: corresponds to `ctx.channel().remoteAddress()`.
    /// - `observer`: corresponds to `JvmProfiler.INSTANCE`.
    /// - `logger`: corresponds to `LOGGER`.
    ///
    /// Returns `Ok(true)` if a packet was decoded and added to `out`.
    /// Returns `Ok(false)` if there were no readable bytes.
    // JAVA: net.minecraft.network.PacketDecoder#decode
    pub fn decode(
        &self,
        buf: &mut dyn Buf,
        out: &mut Vec<P::Packet>,
        remote_address: Option<&str>,
        observer: &dyn PacketDecodeObserver,
        logger: &dyn PacketDecodeLogger,
    ) -> Result<bool, PacketDecodeError> {
        let readable_bytes: usize = buf.remaining();
        if readable_bytes == 0 {
            return Ok(false);
        }

        let packet: P::Packet = match self.protocol_info.codec().decode(buf) {
            Ok(p) => p,
            Err(e) => {
                if is_skip_packet_exception(e.as_ref()) {
                    let remaining = buf.remaining();
                    buf.advance(remaining);
                }
                return Err(PacketDecodeError::Decode(e));
            }
        };

        let packet_type = packet.packet_type();
        observer.on_packet_received(
            self.protocol_info.id(),
            &packet_type,
            remote_address,
            readable_bytes,
        );

        let extra = buf.remaining();
        if extra > 0 {
            return Err(PacketDecodeError::ExtraBytes {
                protocol_id: self.protocol_info.id().id,
                packet_type: packet_type.to_string(),
                packet_class_name: packet.class_name(),
                extra_bytes: extra,
            });
        }

        if logger.is_debug_enabled() {
            logger.debug_packet_received(
                self.protocol_info.id(),
                &packet_type,
                packet.class_name(),
                readable_bytes,
            );
        }

        packet.handle_inbound_terminal();
        out.push(packet);
        Ok(true)
    }
}

fn is_skip_packet_exception(err: &(dyn std::error::Error + 'static)) -> bool {
    let mut cur: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(e) = cur {
        if e.is::<SkipPacketException>() {
            return true;
        }
        if let Some(ioe) = e.downcast_ref::<io::Error>() {
            if let Some(inner) = ioe.get_ref() {
                let inner_err: &(dyn std::error::Error + 'static) = inner;
                if inner_err.is::<SkipPacketException>() {
                    return true;
                }
            }
        }
        cur = e.source();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::io::Cursor;

    #[derive(Debug)]
    struct TestListener;
    impl PacketListener for TestListener {}

    #[derive(Debug, Clone)]
    struct TestPacket {
        ptype: PacketType,
        class_name: &'static str,
        terminal_calls: std::rc::Rc<Cell<u32>>,
    }

    impl TestPacket {
        fn new(name: &'static str, terminal_calls: std::rc::Rc<Cell<u32>>) -> Self {
            Self {
                ptype: PacketType::new(name),
                class_name: "TestPacket",
                terminal_calls,
            }
        }
    }

    impl Packet for TestPacket {
        fn packet_type(&self) -> PacketType {
            self.ptype.clone()
        }
        fn class_name(&self) -> &'static str {
            self.class_name
        }
        fn handle_inbound_terminal(&self) {
            self.terminal_calls
                .set(self.terminal_calls.get().wrapping_add(1));
        }
    }

    struct TestCodec {
        packet_name: &'static str,
        consume: usize,
        fail: Option<Box<dyn std::error::Error>>,
        terminal_calls: std::rc::Rc<Cell<u32>>,
    }

    impl PacketCodec for TestCodec {
        type Listener = TestListener;
        type Packet = TestPacket;

        fn decode(&self, buf: &mut dyn Buf) -> Result<Self::Packet, Box<dyn std::error::Error>> {
            if let Some(_) = &self.fail {
                // Recreate error by matching kind; tests construct specific instances.
                if self.packet_name == "skip" {
                    return Err(Box::new(SkipPacketException));
                }
                return Err(Box::new(io::Error::other("decode failure")));
            }
            let to_consume = std::cmp::min(self.consume, buf.remaining());
            buf.advance(to_consume);
            Ok(TestPacket::new(self.packet_name, self.terminal_calls.clone()))
        }
    }

    struct TestProtocolInfo {
        id: ProtocolId,
        codec: TestCodec,
    }

    impl ProtocolInfo for TestProtocolInfo {
        type Listener = TestListener;
        type Packet = TestPacket;
        type Codec = TestCodec;

        fn id(&self) -> ProtocolId {
            self.id
        }

        fn codec(&self) -> &Self::Codec {
            &self.codec
        }
    }

    #[derive(Default)]
    struct RecordingObserver {
        calls: RefCell<Vec<(ProtocolId, PacketType, Option<String>, usize)>>,
    }

    impl PacketDecodeObserver for RecordingObserver {
        fn on_packet_received(
            &self,
            protocol_id: ProtocolId,
            packet_type: &PacketType,
            remote_address: Option<&str>,
            readable_bytes: usize,
        ) {
            self.calls.borrow_mut().push((
                protocol_id,
                packet_type.clone(),
                remote_address.map(|s| s.to_string()),
                readable_bytes,
            ));
        }
    }

    #[derive(Default)]
    struct RecordingLogger {
        enabled: bool,
        calls: RefCell<Vec<(ProtocolId, PacketType, String, usize)>>,
    }

    impl PacketDecodeLogger for RecordingLogger {
        fn is_debug_enabled(&self) -> bool {
            self.enabled
        }

        fn debug_packet_received(
            &self,
            protocol_id: ProtocolId,
            packet_type: &PacketType,
            packet_class_name: &str,
            readable_bytes: usize,
        ) {
            self.calls.borrow_mut().push((
                protocol_id,
                packet_type.clone(),
                packet_class_name.to_string(),
                readable_bytes,
            ));
        }
    }

    fn make_decoder(
        packet_name: &'static str,
        consume: usize,
        fail: Option<Box<dyn std::error::Error>>,
    ) -> (PacketDecoder<TestProtocolInfo>, std::rc::Rc<Cell<u32>>) {
        let terminal_calls = std::rc::Rc::new(Cell::new(0u32));
        let decoder = PacketDecoder::new(TestProtocolInfo {
            id: ProtocolId::new("play"),
            codec: TestCodec {
                packet_name,
                consume,
                fail,
                terminal_calls: terminal_calls.clone(),
            },
        });
        (decoder, terminal_calls)
    }

    #[test]
    fn decode_success_no_extra_bytes() {
        let (decoder, terminal_calls) = make_decoder("test_packet", 4, None);
        let mut buf = Cursor::new(vec![1u8, 2, 3, 4]);
        let mut out = Vec::new();
        let obs = RecordingObserver::default();
        let log = RecordingLogger {
            enabled: true,
            ..Default::default()
        };

        let ok = decoder
            .decode(&mut buf, &mut out, Some("127.0.0.1:1"), &obs, &log)
            .expect("decode should succeed");
        assert!(ok);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].packet_type(), PacketType::new("test_packet"));
        assert_eq!(terminal_calls.get(), 1);
        assert_eq!(obs.calls.borrow().len(), 1);
        assert_eq!(log.calls.borrow().len(), 1);
        assert_eq!(buf.remaining(), 0);
    }

    #[test]
    fn decode_empty_buffer_returns_false() {
        let (decoder, terminal_calls) = make_decoder("test_packet", 0, None);
        let mut buf = Cursor::new(Vec::<u8>::new());
        let mut out = Vec::new();
        let ok = decoder
            .decode(
                &mut buf,
                &mut out,
                None,
                &NoopPacketDecodeObserver,
                &NoopPacketDecodeLogger,
            )
            .expect("empty decode");
        assert!(!ok);
        assert!(out.is_empty());
        assert_eq!(terminal_calls.get(), 0);
    }

    #[test]
    fn decode_extra_bytes_returns_error() {
        let (decoder, terminal_calls) = make_decoder("test_packet", 2, None);
        let mut buf = Cursor::new(vec![10u8, 20, 30, 40, 50]);
        let mut out = Vec::new();
        let obs = RecordingObserver::default();

        let err = decoder
            .decode(
                &mut buf,
                &mut out,
                None,
                &obs,
                &NoopPacketDecodeLogger,
            )
            .expect_err("should fail on extra bytes");

        match err {
            PacketDecodeError::ExtraBytes {
                protocol_id,
                packet_type,
                packet_class_name,
                extra_bytes,
            } => {
                assert_eq!(protocol_id, "play");
                assert_eq!(packet_type, "test_packet");
                assert_eq!(packet_class_name, "TestPacket");
                assert_eq!(extra_bytes, 3);
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(out.is_empty());
        assert_eq!(terminal_calls.get(), 0);
        assert_eq!(obs.calls.borrow().len(), 1); // profiler called before extra-bytes check
    }

    #[test]
    fn decode_skip_packet_exception_skips_remaining() {
        let (decoder, _) = make_decoder("skip", 0, Some(Box::new(SkipPacketException)));
        let mut buf = Cursor::new(vec![1u8, 2, 3, 4, 5, 6]);
        let mut out = Vec::new();

        let err = decoder
            .decode(
                &mut buf,
                &mut out,
                None,
                &NoopPacketDecodeObserver,
                &NoopPacketDecodeLogger,
            )
            .expect_err("should propagate decode error");
        assert!(matches!(err, PacketDecodeError::Decode(_)));
        assert_eq!(buf.remaining(), 0, "SkipPacketException should skip remaining bytes");
        assert!(out.is_empty());
    }

    #[test]
    fn decode_non_skip_error_preserves_remaining() {
        let (decoder, _) = make_decoder("other", 0, Some(Box::new(io::Error::other("boom"))));
        let mut buf = Cursor::new(vec![9u8, 8, 7]);
        let mut out = Vec::new();

        let err = decoder
            .decode(
                &mut buf,
                &mut out,
                None,
                &NoopPacketDecodeObserver,
                &NoopPacketDecodeLogger,
            )
            .expect_err("should propagate decode error");
        assert!(matches!(err, PacketDecodeError::Decode(_)));
        assert_eq!(buf.remaining(), 3, "non-skip errors must not skip bytes");
    }

    #[test]
    fn debug_logger_not_called_when_disabled() {
        let (decoder, _) = make_decoder("pkt", 1, None);
        let mut buf = Cursor::new(vec![42u8]);
        let mut out = Vec::new();
        let log = RecordingLogger {
            enabled: false,
            ..Default::default()
        };
        decoder
            .decode(&mut buf, &mut out, None, &NoopPacketDecodeObserver, &log)
            .expect("decode");
        assert!(log.calls.borrow().is_empty());
    }
}
