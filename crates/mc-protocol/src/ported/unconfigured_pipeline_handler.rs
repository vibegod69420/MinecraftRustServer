use std::any::Any;
use std::fmt;

/// Minimal marker trait for Minecraft packet listener types.
pub trait PacketListener: Send + 'static {}

/// Minimal marker trait for protocol metadata used by packet codecs.
pub trait ProtocolInfo<T: PacketListener + ?Sized>: Send + 'static {}

/// Minimal inbound packet decoder handler placeholder.
pub struct PacketDecoder<T: PacketListener + ?Sized> {
    _protocol: Box<dyn ProtocolInfo<T>>,
}

impl<T: PacketListener + ?Sized> PacketDecoder<T> {
    pub fn new(protocol: Box<dyn ProtocolInfo<T>>) -> Self {
        Self { _protocol: protocol }
    }
}

/// Minimal outbound packet encoder handler placeholder.
pub struct PacketEncoder<T: PacketListener + ?Sized> {
    _protocol: Box<dyn ProtocolInfo<T>>,
}

impl<T: PacketListener + ?Sized> PacketEncoder<T> {
    pub fn new(protocol: Box<dyn ProtocolInfo<T>>) -> Self {
        Self { _protocol: protocol }
    }
}

/// Marker wrapper for a raw inbound byte buffer message.
#[derive(Debug)]
pub struct ByteBufMessage(pub Vec<u8>);

/// Marker wrapper for a protocol packet message.
pub struct PacketMessage {
    pub debug_name: String,
}

impl PacketMessage {
    pub fn new(debug_name: impl Into<String>) -> Self {
        Self {
            debug_name: debug_name.into(),
        }
    }
}

impl fmt::Debug for PacketMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PacketMessage")
            .field("debug_name", &self.debug_name)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoderException(pub String);

impl fmt::Display for DecoderException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DecoderException {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncoderException(pub String);

impl fmt::Display for EncoderException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EncoderException {}

/// Minimal channel promise abstraction.
pub trait ChannelPromise: Send {
    fn set_success(&mut self);
}

/// Simple promise implementation for tests and local use.
#[derive(Debug, Default)]
pub struct SimplePromise {
    pub success: bool,
}

impl SimplePromise {
    pub fn new() -> Self {
        Self { success: false }
    }
}

impl ChannelPromise for SimplePromise {
    fn set_success(&mut self) {
        self.success = true;
    }
}

/// Minimal channel handler context abstraction capturing the Netty operations
/// used by `UnconfiguredPipelineHandler`.
pub trait ChannelHandlerContext: Send {
    fn handler_name(&self) -> &str;
    fn replace_handler(&mut self, old_name: &str, new_name: &str, handler: Box<dyn Any + Send>);
    fn set_auto_read(&mut self, auto_read: bool);
    fn fire_channel_read(&mut self, msg: Box<dyn Any + Send>);
    fn write_message(&mut self, msg: Box<dyn Any + Send>, promise: &mut dyn ChannelPromise);
}

fn describe_message(msg: &(dyn Any + Send)) -> String {
    if let Some(packet) = msg.downcast_ref::<PacketMessage>() {
        return packet.debug_name.clone();
    }
    if let Some(buf) = msg.downcast_ref::<ByteBufMessage>() {
        return format!("ByteBufMessage(len={})", buf.0.len());
    }
    "UnknownMessage".to_string()
}

/// Inbound pipeline configuration task.
pub struct InboundConfigurationTask {
    task: Option<Box<dyn FnOnce(&mut dyn ChannelHandlerContext) + Send>>,
}

impl InboundConfigurationTask {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut dyn ChannelHandlerContext) + Send + 'static,
    {
        Self {
            task: Some(Box::new(f)),
        }
    }

    // JAVA: net.minecraft.network.UnconfiguredPipelineHandler$InboundConfigurationTask#run
    pub fn run(&mut self, ctx: &mut dyn ChannelHandlerContext) {
        if let Some(task) = self.task.take() {
            task(ctx);
        }
    }

    // JAVA: net.minecraft.network.UnconfiguredPipelineHandler$InboundConfigurationTask#andThen
    pub fn and_then(mut self, mut next: InboundConfigurationTask) -> InboundConfigurationTask {
        InboundConfigurationTask::new(move |ctx: &mut dyn ChannelHandlerContext| {
            self.run(ctx);
            next.run(ctx);
        })
    }
}

/// Outbound pipeline configuration task.
pub struct OutboundConfigurationTask {
    task: Option<Box<dyn FnOnce(&mut dyn ChannelHandlerContext) + Send>>,
}

impl OutboundConfigurationTask {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut dyn ChannelHandlerContext) + Send + 'static,
    {
        Self {
            task: Some(Box::new(f)),
        }
    }

    // JAVA: net.minecraft.network.UnconfiguredPipelineHandler$OutboundConfigurationTask#run
    pub fn run(&mut self, ctx: &mut dyn ChannelHandlerContext) {
        if let Some(task) = self.task.take() {
            task(ctx);
        }
    }

    // JAVA: net.minecraft.network.UnconfiguredPipelineHandler$OutboundConfigurationTask#andThen
    pub fn and_then(mut self, mut next: OutboundConfigurationTask) -> OutboundConfigurationTask {
        OutboundConfigurationTask::new(move |ctx: &mut dyn ChannelHandlerContext| {
            self.run(ctx);
            next.run(ctx);
        })
    }
}

// JAVA: net.minecraft.network.UnconfiguredPipelineHandler#setupInboundProtocol
pub fn setup_inbound_protocol<T: PacketListener + 'static>(
    protocol: Box<dyn ProtocolInfo<T>>,
) -> InboundConfigurationTask {
    setup_inbound_handler(Box::new(PacketDecoder::<T>::new(protocol)))
}

// JAVA: net.minecraft.network.UnconfiguredPipelineHandler#setupInboundHandler
fn setup_inbound_handler(handler: Box<dyn Any + Send>) -> InboundConfigurationTask {
    InboundConfigurationTask::new(move |ctx: &mut dyn ChannelHandlerContext| {
        let name = ctx.handler_name().to_string();
        ctx.replace_handler(&name, "decoder", handler);
        ctx.set_auto_read(true);
    })
}

// JAVA: net.minecraft.network.UnconfiguredPipelineHandler#setupOutboundProtocol
pub fn setup_outbound_protocol<T: PacketListener + 'static>(
    protocol: Box<dyn ProtocolInfo<T>>,
) -> OutboundConfigurationTask {
    setup_outbound_handler(Box::new(PacketEncoder::<T>::new(protocol)))
}

// JAVA: net.minecraft.network.UnconfiguredPipelineHandler#setupOutboundHandler
fn setup_outbound_handler(handler: Box<dyn Any + Send>) -> OutboundConfigurationTask {
    OutboundConfigurationTask::new(move |ctx: &mut dyn ChannelHandlerContext| {
        let name = ctx.handler_name().to_string();
        ctx.replace_handler(&name, "encoder", handler);
    })
}

/// Inbound unconfigured pipeline handler.
pub struct Inbound;

impl Inbound {
    // JAVA: net.minecraft.network.UnconfiguredPipelineHandler$Inbound#channelRead
    pub fn channel_read(
        ctx: &mut dyn ChannelHandlerContext,
        msg: Box<dyn Any + Send>,
    ) -> Result<(), DecoderException> {
        let is_byte_buf = msg.downcast_ref::<ByteBufMessage>().is_some();
        let is_packet = msg.downcast_ref::<PacketMessage>().is_some();
        if !is_byte_buf && !is_packet {
            ctx.fire_channel_read(msg);
            Ok(())
        } else {
            let desc = describe_message(msg.as_ref());
            drop(msg); // ReferenceCountUtil.release
            Err(DecoderException(format!(
                "Pipeline has no inbound protocol configured, can't process packet {}",
                desc
            )))
        }
    }

    // JAVA: net.minecraft.network.UnconfiguredPipelineHandler$Inbound#write
    pub fn write(
        ctx: &mut dyn ChannelHandlerContext,
        msg: Box<dyn Any + Send>,
        promise: &mut dyn ChannelPromise,
    ) {
        if msg.is::<InboundConfigurationTask>() {
            let mut task = msg
                .downcast::<InboundConfigurationTask>()
                .expect("checked InboundConfigurationTask type");
            task.run(ctx);
            drop(task); // ReferenceCountUtil.release
            promise.set_success();
        } else {
            ctx.write_message(msg, promise);
        }
    }
}

/// Outbound unconfigured pipeline handler.
pub struct Outbound;

impl Outbound {
    // JAVA: net.minecraft.network.UnconfiguredPipelineHandler$Outbound#write
    pub fn write(
        ctx: &mut dyn ChannelHandlerContext,
        msg: Box<dyn Any + Send>,
        promise: &mut dyn ChannelPromise,
    ) -> Result<(), EncoderException> {
        if msg.downcast_ref::<PacketMessage>().is_some() {
            let desc = describe_message(msg.as_ref());
            drop(msg); // ReferenceCountUtil.release
            return Err(EncoderException(format!(
                "Pipeline has no outbound protocol configured, can't process packet {}",
                desc
            )));
        }

        if msg.is::<OutboundConfigurationTask>() {
            let mut task = msg
                .downcast::<OutboundConfigurationTask>()
                .expect("checked OutboundConfigurationTask type");
            task.run(ctx);
            drop(task); // ReferenceCountUtil.release
            promise.set_success();
            return Ok(());
        }

        ctx.write_message(msg, promise);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct DummyListener;
    impl PacketListener for DummyListener {}

    struct DummyProtocol;
    impl ProtocolInfo<DummyListener> for DummyProtocol {}

    #[derive(Default)]
    struct MockCtx {
        name: String,
        replacements: Vec<(String, String, &'static str)>,
        auto_read: Option<bool>,
        fired: Vec<String>,
        writes: usize,
    }

    impl MockCtx {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                ..Default::default()
            }
        }
    }

    impl ChannelHandlerContext for MockCtx {
        fn handler_name(&self) -> &str {
            &self.name
        }

        fn replace_handler(
            &mut self,
            old_name: &str,
            new_name: &str,
            handler: Box<dyn Any + Send>,
        ) {
            let kind = if handler.is::<PacketDecoder<DummyListener>>() {
                "decoder_handler"
            } else if handler.is::<PacketEncoder<DummyListener>>() {
                "encoder_handler"
            } else {
                "other"
            };
            self.replacements
                .push((old_name.to_string(), new_name.to_string(), kind));
        }

        fn set_auto_read(&mut self, auto_read: bool) {
            self.auto_read = Some(auto_read);
        }

        fn fire_channel_read(&mut self, msg: Box<dyn Any + Send>) {
            self.fired.push(describe_message(msg.as_ref()));
        }

        fn write_message(&mut self, _msg: Box<dyn Any + Send>, _promise: &mut dyn ChannelPromise) {
            self.writes += 1;
        }
    }

    #[test]
    fn test_inbound_channel_read_rejects_bytebuf_and_packet() {
        let mut ctx = MockCtx::new("inbound");
        let packet_err = Inbound::channel_read(
            &mut ctx,
            Box::new(PacketMessage::new("TestPacket")),
        )
        .unwrap_err();
        assert_eq!(
            packet_err.0,
            "Pipeline has no inbound protocol configured, can't process packet TestPacket"
        );

        let buf_err = Inbound::channel_read(
            &mut ctx,
            Box::new(ByteBufMessage(vec![1, 2, 3])),
        )
        .unwrap_err();
        assert_eq!(
            buf_err.0,
            "Pipeline has no inbound protocol configured, can't process packet ByteBufMessage(len=3)"
        );

        Inbound::channel_read(&mut ctx, Box::new(String::from("passthrough"))).unwrap();
        assert_eq!(ctx.fired, vec!["UnknownMessage".to_string()]);
    }

    #[test]
    fn test_inbound_write_runs_config_task_and_forwards_other() {
        let mut ctx = MockCtx::new("inbound");
        let mut promise = SimplePromise::new();
        let ran = Arc::new(Mutex::new(false));
        let ran_clone = ran.clone();
        let task = InboundConfigurationTask::new(move |_ctx| {
            *ran_clone.lock().unwrap() = true;
        });

        Inbound::write(&mut ctx, Box::new(task), &mut promise);
        assert!(*ran.lock().unwrap());
        assert!(promise.success);
        assert_eq!(ctx.writes, 0);

        let mut promise2 = SimplePromise::new();
        Inbound::write(&mut ctx, Box::new(42i32), &mut promise2);
        assert!(!promise2.success);
        assert_eq!(ctx.writes, 1);
    }

    #[test]
    fn test_outbound_write_rejects_packet_and_runs_config_task() {
        let mut ctx = MockCtx::new("outbound");
        let mut promise = SimplePromise::new();
        let err = Outbound::write(
            &mut ctx,
            Box::new(PacketMessage::new("OutboundPacket")),
            &mut promise,
        )
        .unwrap_err();
        assert_eq!(
            err.0,
            "Pipeline has no outbound protocol configured, can't process packet OutboundPacket"
        );
        assert!(!promise.success);

        let ran = Arc::new(Mutex::new(false));
        let ran_clone = ran.clone();
        let task = OutboundConfigurationTask::new(move |_ctx| {
            *ran_clone.lock().unwrap() = true;
        });
        let mut promise2 = SimplePromise::new();
        Outbound::write(&mut ctx, Box::new(task), &mut promise2).unwrap();
        assert!(*ran.lock().unwrap());
        assert!(promise2.success);

        let mut promise3 = SimplePromise::new();
        Outbound::write(&mut ctx, Box::new(String::from("other")), &mut promise3).unwrap();
        assert_eq!(ctx.writes, 1);
    }

    #[test]
    fn test_config_task_and_then_chaining_with_edge_case_values() {
        let mut ctx = MockCtx::new("pipeline");
        let log: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));

        let log1 = log.clone();
        let task1 = InboundConfigurationTask::new(move |_ctx| {
            let mut g = log1.lock().unwrap();
            let prev = g.last().copied().unwrap_or(0i32);
            g.push(prev.wrapping_add(i32::MAX));
        });

        let log2 = log.clone();
        let task2 = InboundConfigurationTask::new(move |_ctx| {
            let mut g = log2.lock().unwrap();
            let prev = g.last().copied().unwrap_or(0i32);
            g.push(prev.wrapping_sub(-1i32));
        });

        let mut chained = task1.and_then(task2);
        chained.run(&mut ctx);

        let g = log.lock().unwrap();
        assert_eq!(g.len(), 2);
        assert_eq!(g[0], i32::MAX);
        assert_eq!(g[1], i32::MAX.wrapping_sub(-1i32)); // overflow edge case

        // Running again should be a no-op because FnOnce tasks were taken.
        drop(g);
        chained.run(&mut ctx);
        assert_eq!(log.lock().unwrap().len(), 2);
    }

    #[test]
    fn test_setup_handlers_configure_pipeline_and_float_bits_state() {
        let mut ctx = MockCtx::new("handler0");

        let mut inbound_task = setup_inbound_protocol::<DummyListener>(Box::new(DummyProtocol));
        inbound_task.run(&mut ctx);
        assert_eq!(
            ctx.replacements,
            vec![("handler0".to_string(), "decoder".to_string(), "decoder_handler")]
        );
        assert_eq!(ctx.auto_read, Some(true));

        let mut ctx2 = MockCtx::new("handler1");
        let mut outbound_task = setup_outbound_protocol::<DummyListener>(Box::new(DummyProtocol));
        outbound_task.run(&mut ctx2);
        assert_eq!(
            ctx2.replacements,
            vec![("handler1".to_string(), "encoder".to_string(), "encoder_handler")]
        );
        assert_eq!(ctx2.auto_read, None);

        // Float bits edge case: preserve exact bit pattern through task-captured state.
        let bits = Arc::new(Mutex::new(0u32));
        let bits_clone = bits.clone();
        let mut task = OutboundConfigurationTask::new(move |_ctx| {
            let f: f32 = -0.0_f32;
            let mut g = bits_clone.lock().unwrap();
            *g = f.to_bits();
        });
        task.run(&mut ctx2);
        assert_eq!(*bits.lock().unwrap(), 0x8000_0000u32);
        let f = f32::from_bits(*bits.lock().unwrap());
        assert!(f.is_sign_negative());
        assert_eq!(f, -0.0_f32);
    }
}
