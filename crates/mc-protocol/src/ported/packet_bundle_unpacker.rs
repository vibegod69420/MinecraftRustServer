pub trait Packet {
    fn is_terminal(&self) -> bool;
}

pub trait BundlerInfo<P: ?Sized, O> {
    fn unbundle_packet<F: FnMut(O)>(&self, packet: &P, add: F);
}

pub trait ChannelPipeline {
    fn remove(&mut self, name: &str);
}

pub trait ChannelHandlerContext {
    type Pipeline: ChannelPipeline;
    fn name(&self) -> &str;
    fn pipeline(&mut self) -> &mut Self::Pipeline;
}

pub struct PacketBundleUnpacker<B> {
    bundler_info: B,
}

impl<B> PacketBundleUnpacker<B> {
    // JAVA: net.minecraft.network.PacketBundleUnpacker#<init>
    pub fn new(bundler_info: B) -> Self {
        Self { bundler_info }
    }

    // JAVA: net.minecraft.network.PacketBundleUnpacker#encode
    pub fn encode<P, O, C>(&self, ctx: &mut C, packet: &P, out: &mut Vec<O>)
    where
        P: Packet,
        B: BundlerInfo<P, O>,
        C: ChannelHandlerContext,
    {
        self.bundler_info
            .unbundle_packet(packet, |p| out.push(p));
        if packet.is_terminal() {
            let name = ctx.name().to_string();
            ctx.pipeline().remove(&name);
        }
    }

    pub fn bundler_info(&self) -> &B {
        &self.bundler_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockPacket {
        id: i32,
        terminal: bool,
    }

    impl Packet for MockPacket {
        fn is_terminal(&self) -> bool {
            self.terminal
        }
    }

    struct MockBundler;

    impl BundlerInfo<MockPacket, MockPacket> for MockBundler {
        fn unbundle_packet<F: FnMut(MockPacket)>(&self, packet: &MockPacket, mut add: F) {
            add(MockPacket {
                id: packet.id,
                terminal: false,
            });
            add(MockPacket {
                id: packet.id.wrapping_add(1),
                terminal: false,
            });
        }
    }

    #[derive(Default)]
    struct MockPipeline {
        removed: Vec<String>,
    }

    impl ChannelPipeline for MockPipeline {
        fn remove(&mut self, name: &str) {
            self.removed.push(name.to_string());
        }
    }

    struct MockCtx {
        name: String,
        pipeline: MockPipeline,
    }

    impl MockCtx {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                pipeline: MockPipeline::default(),
            }
        }
    }

    impl ChannelHandlerContext for MockCtx {
        type Pipeline = MockPipeline;
        fn name(&self) -> &str {
            &self.name
        }
        fn pipeline(&mut self) -> &mut Self::Pipeline {
            &mut self.pipeline
        }
    }

    #[test]
    fn encode_unbundles_and_keeps_pipeline_for_non_terminal() {
        let unpacker = PacketBundleUnpacker::new(MockBundler);
        let mut ctx = MockCtx::new("bundler");
        let packet = MockPacket {
            id: 10,
            terminal: false,
        };
        let mut out = Vec::new();
        unpacker.encode(&mut ctx, &packet, &mut out);
        assert_eq!(
            out,
            vec![
                MockPacket {
                    id: 10,
                    terminal: false
                },
                MockPacket {
                    id: 11,
                    terminal: false
                }
            ]
        );
        assert!(ctx.pipeline.removed.is_empty());
    }

    #[test]
    fn encode_removes_pipeline_for_terminal_packet() {
        let unpacker = PacketBundleUnpacker::new(MockBundler);
        let mut ctx = MockCtx::new("bundle_unpacker");
        let packet = MockPacket {
            id: -1,
            terminal: true,
        };
        let mut out = Vec::new();
        unpacker.encode(&mut ctx, &packet, &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(ctx.pipeline.removed, vec!["bundle_unpacker"]);
    }

    #[test]
    fn encode_handles_i32_edge_ids_with_wrapping() {
        let unpacker = PacketBundleUnpacker::new(MockBundler);
        let mut ctx = MockCtx::new("h");
        let packet = MockPacket {
            id: i32::MAX,
            terminal: false,
        };
        let mut out = Vec::new();
        unpacker.encode(&mut ctx, &packet, &mut out);
        assert_eq!(out[0].id, i32::MAX);
        assert_eq!(out[1].id, i32::MIN);
    }

    #[test]
    fn encode_terminal_still_unbundles_before_removal() {
        struct TrackingBundler;
        impl BundlerInfo<MockPacket, i32> for TrackingBundler {
            fn unbundle_packet<F: FnMut(i32)>(&self, packet: &MockPacket, mut add: F) {
                add(packet.id);
            }
        }
        let unpacker = PacketBundleUnpacker::new(TrackingBundler);
        let mut ctx = MockCtx::new("x");
        let packet = MockPacket {
            id: 7,
            terminal: true,
        };
        let mut out = Vec::new();
        unpacker.encode(&mut ctx, &packet, &mut out);
        assert_eq!(out, vec![7]);
        assert_eq!(ctx.pipeline.removed, vec!["x"]);
    }

    #[test]
    fn encode_empty_unbundle_terminal_removes_pipeline() {
        struct EmptyBundler;
        impl BundlerInfo<MockPacket, MockPacket> for EmptyBundler {
            fn unbundle_packet<F: FnMut(MockPacket)>(&self, _packet: &MockPacket, _add: F) {}
        }
        let unpacker = PacketBundleUnpacker::new(EmptyBundler);
        let mut ctx = MockCtx::new("n");
        let packet = MockPacket {
            id: 0,
            terminal: true,
        };
        let mut out = Vec::new();
        unpacker.encode(&mut ctx, &packet, &mut out);
        assert!(out.is_empty());
        assert_eq!(ctx.pipeline.removed, vec!["n"]);
    }
}
