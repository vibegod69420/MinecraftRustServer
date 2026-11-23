use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoderException {
    pub message: String,
}

impl fmt::Display for DecoderException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecoderException {}

pub trait BundlePacket {
    fn is_terminal(&self) -> bool;
}

pub trait Bundler<P: BundlePacket> {
    fn add_packet(&mut self, packet: P) -> Option<P>;
}

pub trait BundlerInfo<P: BundlePacket> {
    type Bundler: Bundler<P>;
    fn start_packet_bundling(&mut self, packet: &P) -> Option<Self::Bundler>;
}

pub trait PacketBundleContext {
    fn remove_handler(&mut self);
}

pub struct PacketBundlePacker<P: BundlePacket, I: BundlerInfo<P>> {
    bundler_info: I,
    current_bundler: Option<I::Bundler>,
}

impl<P: BundlePacket, I: BundlerInfo<P>> PacketBundlePacker<P, I> {
    // JAVA: net.minecraft.network.PacketBundlePacker#<init>
    pub fn new(bundler_info: I) -> Self {
        Self {
            bundler_info,
            current_bundler: None,
        }
    }

    // JAVA: net.minecraft.network.PacketBundlePacker#decode
    pub fn decode<C: PacketBundleContext>(
        &mut self,
        ctx: &mut C,
        packet: P,
        out: &mut Vec<P>,
    ) -> Result<(), DecoderException> {
        if let Some(bundler) = self.current_bundler.as_mut() {
            Self::verify_non_terminal_packet(&packet)?;
            if let Some(bundled) = bundler.add_packet(packet) {
                self.current_bundler = None;
                out.push(bundled);
            }
        } else if let Some(bundler) = self.bundler_info.start_packet_bundling(&packet) {
            Self::verify_non_terminal_packet(&packet)?;
            self.current_bundler = Some(bundler);
        } else {
            let terminal = packet.is_terminal();
            out.push(packet);
            if terminal {
                ctx.remove_handler();
            }
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.PacketBundlePacker#verifyNonTerminalPacket
    fn verify_non_terminal_packet(packet: &P) -> Result<(), DecoderException> {
        if packet.is_terminal() {
            return Err(DecoderException {
                message: "Terminal message received in bundle".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestPacket {
        id: i32,
        terminal: bool,
    }

    impl BundlePacket for TestPacket {
        fn is_terminal(&self) -> bool {
            self.terminal
        }
    }

    struct TestBundler {
        finish_on: i32,
    }

    impl Bundler<TestPacket> for TestBundler {
        fn add_packet(&mut self, packet: TestPacket) -> Option<TestPacket> {
            if packet.id == self.finish_on {
                Some(TestPacket {
                    id: -1,
                    terminal: false,
                })
            } else {
                None
            }
        }
    }

    struct TestBundlerInfo {
        start_on: i32,
        finish_on: i32,
    }

    impl BundlerInfo<TestPacket> for TestBundlerInfo {
        type Bundler = TestBundler;
        fn start_packet_bundling(&mut self, packet: &TestPacket) -> Option<TestBundler> {
            if packet.id == self.start_on {
                Some(TestBundler {
                    finish_on: self.finish_on,
                })
            } else {
                None
            }
        }
    }

    #[derive(Default)]
    struct TestCtx {
        removed: bool,
    }

    impl PacketBundleContext for TestCtx {
        fn remove_handler(&mut self) {
            self.removed = true;
        }
    }

    fn pkt(id: i32, terminal: bool) -> TestPacket {
        TestPacket { id, terminal }
    }

    #[test]
    fn passthrough_non_terminal_packet() {
        let mut packer = PacketBundlePacker::new(TestBundlerInfo {
            start_on: 10,
            finish_on: 20,
        });
        let mut ctx = TestCtx::default();
        let mut out = Vec::new();
        packer.decode(&mut ctx, pkt(1, false), &mut out).unwrap();
        assert_eq!(out, vec![pkt(1, false)]);
        assert!(!ctx.removed);
    }

    #[test]
    fn terminal_passthrough_removes_handler() {
        let mut packer = PacketBundlePacker::new(TestBundlerInfo {
            start_on: 10,
            finish_on: 20,
        });
        let mut ctx = TestCtx::default();
        let mut out = Vec::new();
        packer.decode(&mut ctx, pkt(1, true), &mut out).unwrap();
        assert_eq!(out, vec![pkt(1, true)]);
        assert!(ctx.removed);
    }

    #[test]
    fn bundle_start_add_and_complete() {
        let mut packer = PacketBundlePacker::new(TestBundlerInfo {
            start_on: 10,
            finish_on: 20,
        });
        let mut ctx = TestCtx::default();
        let mut out = Vec::new();

        packer.decode(&mut ctx, pkt(10, false), &mut out).unwrap();
        assert!(out.is_empty());
        assert!(!ctx.removed);

        packer.decode(&mut ctx, pkt(11, false), &mut out).unwrap();
        assert!(out.is_empty());

        packer.decode(&mut ctx, pkt(20, false), &mut out).unwrap();
        assert_eq!(out, vec![pkt(-1, false)]);

        // After completion, normal passthrough resumes.
        let mut out2 = Vec::new();
        packer.decode(&mut ctx, pkt(5, false), &mut out2).unwrap();
        assert_eq!(out2, vec![pkt(5, false)]);
    }

    #[test]
    fn terminal_packet_inside_bundle_throws() {
        let mut packer = PacketBundlePacker::new(TestBundlerInfo {
            start_on: 10,
            finish_on: 20,
        });
        let mut ctx = TestCtx::default();
        let mut out = Vec::new();
        packer.decode(&mut ctx, pkt(10, false), &mut out).unwrap();

        let err = packer
            .decode(&mut ctx, pkt(99, true), &mut out)
            .unwrap_err();
        assert_eq!(err.message, "Terminal message received in bundle");
    }

    #[test]
    fn terminal_bundle_start_packet_throws() {
        let mut packer = PacketBundlePacker::new(TestBundlerInfo {
            start_on: 10,
            finish_on: 20,
        });
        let mut ctx = TestCtx::default();
        let mut out = Vec::new();
        let err = packer
            .decode(&mut ctx, pkt(10, true), &mut out)
            .unwrap_err();
        assert_eq!(err.message, "Terminal message received in bundle");
        assert!(out.is_empty());
        assert!(!ctx.removed);
    }
}
