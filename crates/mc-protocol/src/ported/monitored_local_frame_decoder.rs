pub trait BandwidthDebugMonitor {
    // JAVA: net.minecraft.network.BandwidthDebugMonitor#onReceive
    fn on_receive(&mut self, bytes: i32);
}

pub trait ByteBuf {
    fn readable_bytes(&self) -> i32;
}

pub trait ChannelHandlerContext<M> {
    fn fire_channel_read(&mut self, msg: M);
}

/// Minimal placeholder for `net.minecraft.network.HiddenByteBuf`.
pub struct HiddenByteBuf;

impl HiddenByteBuf {
    // JAVA: net.minecraft.network.HiddenByteBuf#unpack
    pub fn unpack<B, O>(msg: InboundMessage<B, O>) -> InboundMessage<B, O> {
        msg
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundMessage<B, O> {
    ByteBuf(B),
    Other(O),
}

pub struct MonitoredLocalFrameDecoder<M> {
    monitor: M,
}

impl<M: BandwidthDebugMonitor> MonitoredLocalFrameDecoder<M> {
    // JAVA: net.minecraft.network.MonitoredLocalFrameDecoder#<init>
    pub fn new(monitor: M) -> Self {
        Self { monitor }
    }

    // JAVA: net.minecraft.network.MonitoredLocalFrameDecoder#channelRead
    pub fn channel_read<B: ByteBuf, O, C: ChannelHandlerContext<InboundMessage<B, O>>>(
        &mut self,
        ctx: &mut C,
        msg: InboundMessage<B, O>,
    ) {
        let msg = HiddenByteBuf::unpack(msg);
        if let InboundMessage::ByteBuf(ref buf) = msg {
            self.monitor.on_receive(buf.readable_bytes());
        }
        ctx.fire_channel_read(msg);
    }

    pub fn monitor(&self) -> &M {
        &self.monitor
    }

    pub fn monitor_mut(&mut self) -> &mut M {
        &mut self.monitor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockMonitor {
        received: Vec<i32>,
    }

    impl BandwidthDebugMonitor for MockMonitor {
        fn on_receive(&mut self, bytes: i32) {
            self.received.push(bytes);
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockBuf {
        readable: i32,
    }

    impl ByteBuf for MockBuf {
        fn readable_bytes(&self) -> i32 {
            self.readable
        }
    }

    #[derive(Default)]
    struct MockCtx {
        fired: Vec<InboundMessage<MockBuf, String>>,
    }

    impl ChannelHandlerContext<InboundMessage<MockBuf, String>> for MockCtx {
        fn fire_channel_read(&mut self, msg: InboundMessage<MockBuf, String>) {
            self.fired.push(msg);
        }
    }

    #[test]
    fn channel_read_reports_bytebuf_size() {
        let mut decoder = MonitoredLocalFrameDecoder::new(MockMonitor::default());
        let mut ctx = MockCtx::default();
        decoder.channel_read(
            &mut ctx,
            InboundMessage::ByteBuf(MockBuf { readable: 42 }),
        );
        assert_eq!(decoder.monitor().received, vec![42]);
        assert_eq!(ctx.fired.len(), 1);
        assert_eq!(
            ctx.fired[0],
            InboundMessage::ByteBuf(MockBuf { readable: 42 })
        );
    }

    #[test]
    fn channel_read_ignores_non_bytebuf() {
        let mut decoder = MonitoredLocalFrameDecoder::new(MockMonitor::default());
        let mut ctx = MockCtx::default();
        decoder.channel_read(&mut ctx, InboundMessage::Other("hello".to_string()));
        assert!(decoder.monitor().received.is_empty());
        assert_eq!(ctx.fired, vec![InboundMessage::Other("hello".to_string())]);
    }

    #[test]
    fn channel_read_handles_zero_and_negative_readable_bytes() {
        let mut decoder = MonitoredLocalFrameDecoder::new(MockMonitor::default());
        let mut ctx = MockCtx::default();
        decoder.channel_read(&mut ctx, InboundMessage::ByteBuf(MockBuf { readable: 0 }));
        decoder.channel_read(&mut ctx, InboundMessage::ByteBuf(MockBuf { readable: -1 }));
        decoder.channel_read(
            &mut ctx,
            InboundMessage::ByteBuf(MockBuf {
                readable: i32::MIN,
            }),
        );
        assert_eq!(decoder.monitor().received, vec![0, -1, i32::MIN]);
        assert_eq!(ctx.fired.len(), 3);
    }

    #[test]
    fn unpack_is_identity() {
        let msg = InboundMessage::<MockBuf, String>::Other("x".to_string());
        let unpacked = HiddenByteBuf::unpack(msg.clone());
        assert_eq!(unpacked, msg);
    }
}
