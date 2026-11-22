pub trait Packet {
    fn is_terminal(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnconfiguredPipelineHandler {
    Inbound,
    Outbound,
}

pub trait ChannelConfig {
    fn set_auto_read(&mut self, auto_read: bool);
}

pub trait Channel {
    type Config: ChannelConfig;
    fn config(&mut self) -> &mut Self::Config;
}

pub trait ChannelPipeline {
    fn add_before(
        &mut self,
        base_name: &str,
        name: &str,
        handler: UnconfiguredPipelineHandler,
    );
    fn add_after(
        &mut self,
        base_name: &str,
        name: &str,
        handler: UnconfiguredPipelineHandler,
    );
    fn remove(&mut self, name: &str);
}

pub trait ChannelHandlerContext {
    type ChannelTy: Channel;
    type PipelineTy: ChannelPipeline;
    fn channel(&mut self) -> &mut Self::ChannelTy;
    fn pipeline(&mut self) -> &mut Self::PipelineTy;
    fn name(&self) -> &str;
}

pub struct ProtocolSwapHandler;

impl ProtocolSwapHandler {
    // JAVA: net.minecraft.network.ProtocolSwapHandler#handleInboundTerminalPacket
    pub fn handle_inbound_terminal_packet<C: ChannelHandlerContext, P: Packet>(
        ctx: &mut C,
        packet: &P,
    ) {
        if packet.is_terminal() {
            ctx.channel().config().set_auto_read(false);
            let name = ctx.name().to_string();
            ctx.pipeline().add_before(
                &name,
                "inbound_config",
                UnconfiguredPipelineHandler::Inbound,
            );
            ctx.pipeline().remove(&name);
        }
    }

    // JAVA: net.minecraft.network.ProtocolSwapHandler#handleOutboundTerminalPacket
    pub fn handle_outbound_terminal_packet<C: ChannelHandlerContext, P: Packet>(
        ctx: &mut C,
        packet: &P,
    ) {
        if packet.is_terminal() {
            let name = ctx.name().to_string();
            ctx.pipeline().add_after(
                &name,
                "outbound_config",
                UnconfiguredPipelineHandler::Outbound,
            );
            ctx.pipeline().remove(&name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockPacket {
        terminal: bool,
    }

    impl Packet for MockPacket {
        fn is_terminal(&self) -> bool {
            self.terminal
        }
    }

    #[derive(Default)]
    struct MockConfig {
        auto_read: Option<bool>,
    }

    impl ChannelConfig for MockConfig {
        fn set_auto_read(&mut self, auto_read: bool) {
            self.auto_read = Some(auto_read);
        }
    }

    #[derive(Default)]
    struct MockChannel {
        config: MockConfig,
    }

    impl Channel for MockChannel {
        type Config = MockConfig;
        fn config(&mut self) -> &mut Self::Config {
            &mut self.config
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum PipeOp {
        AddBefore(String, String, UnconfiguredPipelineHandler),
        AddAfter(String, String, UnconfiguredPipelineHandler),
        Remove(String),
    }

    #[derive(Default)]
    struct MockPipeline {
        ops: Vec<PipeOp>,
    }

    impl ChannelPipeline for MockPipeline {
        fn add_before(
            &mut self,
            base_name: &str,
            name: &str,
            handler: UnconfiguredPipelineHandler,
        ) {
            self.ops.push(PipeOp::AddBefore(
                base_name.to_string(),
                name.to_string(),
                handler,
            ));
        }

        fn add_after(
            &mut self,
            base_name: &str,
            name: &str,
            handler: UnconfiguredPipelineHandler,
        ) {
            self.ops.push(PipeOp::AddAfter(
                base_name.to_string(),
                name.to_string(),
                handler,
            ));
        }

        fn remove(&mut self, name: &str) {
            self.ops.push(PipeOp::Remove(name.to_string()));
        }
    }

    struct MockCtx {
        handler_name: String,
        channel: MockChannel,
        pipeline: MockPipeline,
    }

    impl MockCtx {
        fn new(name: &str) -> Self {
            Self {
                handler_name: name.to_string(),
                channel: MockChannel::default(),
                pipeline: MockPipeline::default(),
            }
        }
    }

    impl ChannelHandlerContext for MockCtx {
        type ChannelTy = MockChannel;
        type PipelineTy = MockPipeline;
        fn channel(&mut self) -> &mut Self::ChannelTy {
            &mut self.channel
        }
        fn pipeline(&mut self) -> &mut Self::PipelineTy {
            &mut self.pipeline
        }
        fn name(&self) -> &str {
            &self.handler_name
        }
    }

    #[test]
    fn inbound_terminal_swaps_pipeline_and_disables_auto_read() {
        let mut ctx = MockCtx::new("handler");
        let packet = MockPacket { terminal: true };
        ProtocolSwapHandler::handle_inbound_terminal_packet(&mut ctx, &packet);
        assert_eq!(ctx.channel.config.auto_read, Some(false));
        assert_eq!(
            ctx.pipeline.ops,
            vec![
                PipeOp::AddBefore(
                    "handler".into(),
                    "inbound_config".into(),
                    UnconfiguredPipelineHandler::Inbound
                ),
                PipeOp::Remove("handler".into()),
            ]
        );
    }

    #[test]
    fn inbound_non_terminal_does_nothing() {
        let mut ctx = MockCtx::new("handler");
        let packet = MockPacket { terminal: false };
        ProtocolSwapHandler::handle_inbound_terminal_packet(&mut ctx, &packet);
        assert_eq!(ctx.channel.config.auto_read, None);
        assert!(ctx.pipeline.ops.is_empty());
    }

    #[test]
    fn outbound_terminal_swaps_pipeline() {
        let mut ctx = MockCtx::new("out");
        let packet = MockPacket { terminal: true };
        ProtocolSwapHandler::handle_outbound_terminal_packet(&mut ctx, &packet);
        assert_eq!(
            ctx.pipeline.ops,
            vec![
                PipeOp::AddAfter(
                    "out".into(),
                    "outbound_config".into(),
                    UnconfiguredPipelineHandler::Outbound
                ),
                PipeOp::Remove("out".into()),
            ]
        );
    }

    #[test]
    fn outbound_non_terminal_does_nothing() {
        let mut ctx = MockCtx::new("out");
        let packet = MockPacket { terminal: false };
        ProtocolSwapHandler::handle_outbound_terminal_packet(&mut ctx, &packet);
        assert!(ctx.pipeline.ops.is_empty());
        assert_eq!(ctx.channel.config.auto_read, None);
    }
}
