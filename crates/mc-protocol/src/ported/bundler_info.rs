use std::any::Any;
use std::sync::Arc;

/// Shared packet handle used by the bundler.
///
/// Java packets are object references; [`Arc::ptr_eq`] preserves identity
/// semantics required for delimiter checks.
pub type SharedPacket = Arc<dyn Any + Send + Sync>;

// JAVA: net.minecraft.network.protocol.BundlerInfo#BUNDLE_SIZE_LIMIT
pub const BUNDLE_SIZE_LIMIT: usize = 4096;

pub trait BundlerInfo: Send + Sync {
    // JAVA: net.minecraft.network.protocol.BundlerInfo#unbundlePacket
    fn unbundle_packet(&self, packet: SharedPacket, consumer: &mut dyn FnMut(SharedPacket));

    // JAVA: net.minecraft.network.protocol.BundlerInfo#startPacketBundling
    fn start_packet_bundling(&self, packet: &SharedPacket) -> Option<Box<dyn Bundler>>;
}

pub trait Bundler: Send {
    // JAVA: net.minecraft.network.protocol.BundlerInfo$Bundler#addPacket
    fn add_packet(&mut self, packet: SharedPacket) -> Option<SharedPacket>;
}

// JAVA: net.minecraft.network.protocol.BundlerInfo#createForPacket
pub fn create_for_packet<BF, SE>(
    bundle_factory: BF,
    sub_packets_extractor: SE,
    delimiter: SharedPacket,
) -> Box<dyn BundlerInfo>
where
    BF: Fn(Vec<SharedPacket>) -> SharedPacket + Send + Sync + 'static,
    SE: Fn(&SharedPacket) -> Option<Vec<SharedPacket>> + Send + Sync + 'static,
{
    Box::new(CreatedBundlerInfo {
        bundle_factory: Arc::new(bundle_factory),
        sub_packets_extractor,
        delimiter,
    })
}

struct CreatedBundlerInfo<BF, SE> {
    bundle_factory: Arc<BF>,
    sub_packets_extractor: SE,
    delimiter: SharedPacket,
}

impl<BF, SE> BundlerInfo for CreatedBundlerInfo<BF, SE>
where
    BF: Fn(Vec<SharedPacket>) -> SharedPacket + Send + Sync + 'static,
    SE: Fn(&SharedPacket) -> Option<Vec<SharedPacket>> + Send + Sync + 'static,
{
    // JAVA: net.minecraft.network.protocol.BundlerInfo#unbundlePacket
    fn unbundle_packet(&self, packet: SharedPacket, consumer: &mut dyn FnMut(SharedPacket)) {
        if let Some(sub_packets) = (self.sub_packets_extractor)(&packet) {
            consumer(Arc::clone(&self.delimiter));
            for sub_packet in sub_packets {
                consumer(sub_packet);
            }
            consumer(Arc::clone(&self.delimiter));
        } else {
            consumer(packet);
        }
    }

    // JAVA: net.minecraft.network.protocol.BundlerInfo#startPacketBundling
    fn start_packet_bundling(&self, packet: &SharedPacket) -> Option<Box<dyn Bundler>> {
        if Arc::ptr_eq(packet, &self.delimiter) {
            Some(Box::new(CreatedBundler {
                delimiter: Arc::clone(&self.delimiter),
                bundle_factory: Arc::clone(&self.bundle_factory),
                bundle_packets: Vec::new(),
            }))
        } else {
            None
        }
    }
}

struct CreatedBundler<BF> {
    delimiter: SharedPacket,
    bundle_factory: Arc<BF>,
    bundle_packets: Vec<SharedPacket>,
}

impl<BF> Bundler for CreatedBundler<BF>
where
    BF: Fn(Vec<SharedPacket>) -> SharedPacket + Send + Sync + 'static,
{
    // JAVA: net.minecraft.network.protocol.BundlerInfo$Bundler#addPacket
    fn add_packet(&mut self, packet: SharedPacket) -> Option<SharedPacket> {
        if Arc::ptr_eq(&packet, &self.delimiter) {
            let packets = std::mem::take(&mut self.bundle_packets);
            return Some((self.bundle_factory)(packets));
        }

        if self.bundle_packets.len() >= BUNDLE_SIZE_LIMIT {
            panic!("Too many packets in a bundle");
        }

        self.bundle_packets.push(packet);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestPacket(i32);

    #[derive(Debug)]
    struct FloatBitsPacket(u32);

    #[derive(Debug)]
    struct TestBundlePacket {
        subs: Vec<SharedPacket>,
    }

    #[derive(Debug)]
    struct TestDelimiter;

    fn make_delimiter() -> SharedPacket {
        Arc::new(TestDelimiter)
    }

    fn make_info(delimiter: SharedPacket) -> Box<dyn BundlerInfo> {
        create_for_packet(
            |subs: Vec<SharedPacket>| -> SharedPacket { Arc::new(TestBundlePacket { subs }) },
            |packet: &SharedPacket| {
                Arc::downcast::<TestBundlePacket>(Arc::clone(packet))
                    .ok()
                    .map(|b| b.subs.clone())
            },
            delimiter,
        )
    }

    #[test]
    fn unbundle_non_bundle_passes_through() {
        let delimiter = make_delimiter();
        let info = make_info(delimiter);
        let packet: SharedPacket = Arc::new(TestPacket(-7));

        let mut out = Vec::new();
        info.unbundle_packet(Arc::clone(&packet), &mut |p| out.push(p));

        assert_eq!(out.len(), 1);
        assert!(Arc::ptr_eq(&out[0], &packet));
    }

    #[test]
    fn unbundle_bundle_emits_delimiter_subs_delimiter() {
        let delimiter = make_delimiter();
        let info = make_info(Arc::clone(&delimiter));
        let sub1: SharedPacket = Arc::new(TestPacket(1));
        let sub2: SharedPacket = Arc::new(TestPacket(i32::MIN));
        let bundle: SharedPacket = Arc::new(TestBundlePacket {
            subs: vec![Arc::clone(&sub1), Arc::clone(&sub2)],
        });

        let mut out = Vec::new();
        info.unbundle_packet(bundle, &mut |p| out.push(p));

        assert_eq!(out.len(), 4);
        assert!(Arc::ptr_eq(&out[0], &delimiter));
        assert!(Arc::ptr_eq(&out[1], &sub1));
        assert!(Arc::ptr_eq(&out[2], &sub2));
        assert!(Arc::ptr_eq(&out[3], &delimiter));
    }

    #[test]
    fn start_bundling_only_on_delimiter_identity() {
        let delimiter = make_delimiter();
        let info = make_info(Arc::clone(&delimiter));

        let other: SharedPacket = Arc::new(TestPacket(0));
        assert!(info.start_packet_bundling(&other).is_none());
        assert!(info.start_packet_bundling(&delimiter).is_some());

        // Same delimiter type but different instance must not start bundling.
        let fake_delimiter: SharedPacket = Arc::new(TestDelimiter);
        assert!(info.start_packet_bundling(&fake_delimiter).is_none());
    }

    #[test]
    fn bundler_collects_and_emits_on_delimiter() {
        let delimiter = make_delimiter();
        let info = make_info(Arc::clone(&delimiter));
        let mut bundler = info.start_packet_bundling(&delimiter).expect("bundler");

        let p1: SharedPacket = Arc::new(TestPacket(i32::MAX));
        let p2: SharedPacket = Arc::new(FloatBitsPacket(f32::NAN.to_bits()));
        let p3: SharedPacket = Arc::new(TestPacket(-1));

        assert!(bundler.add_packet(Arc::clone(&p1)).is_none());
        assert!(bundler.add_packet(Arc::clone(&p2)).is_none());
        assert!(bundler.add_packet(Arc::clone(&p3)).is_none());

        let bundled = bundler
            .add_packet(Arc::clone(&delimiter))
            .expect("bundle on delimiter");
        let bundle = Arc::downcast::<TestBundlePacket>(bundled).expect("bundle type");

        assert_eq!(bundle.subs.len(), 3);
        assert!(Arc::ptr_eq(&bundle.subs[0], &p1));
        assert!(Arc::ptr_eq(&bundle.subs[1], &p2));
        assert!(Arc::ptr_eq(&bundle.subs[2], &p3));

        let float_packet =
            Arc::downcast::<FloatBitsPacket>(Arc::clone(&bundle.subs[1])).expect("float packet");
        assert_eq!(float_packet.0, f32::NAN.to_bits());
    }

    #[test]
    fn bundler_treats_non_identity_delimiter_as_regular_packet() {
        let delimiter = make_delimiter();
        let info = make_info(Arc::clone(&delimiter));
        let mut bundler = info.start_packet_bundling(&delimiter).expect("bundler");

        let fake_delimiter: SharedPacket = Arc::new(TestDelimiter);
        let payload: SharedPacket = Arc::new(TestPacket(-128));

        assert!(bundler.add_packet(Arc::clone(&payload)).is_none());
        assert!(bundler.add_packet(Arc::clone(&fake_delimiter)).is_none());

        let bundled = bundler
            .add_packet(Arc::clone(&delimiter))
            .expect("bundle on real delimiter");
        let bundle = Arc::downcast::<TestBundlePacket>(bundled).expect("bundle type");

        assert_eq!(bundle.subs.len(), 2);
        assert!(Arc::ptr_eq(&bundle.subs[0], &payload));
        assert!(Arc::ptr_eq(&bundle.subs[1], &fake_delimiter));
    }

    #[test]
    #[should_panic(expected = "Too many packets in a bundle")]
    fn bundler_rejects_packets_over_limit() {
        let delimiter = make_delimiter();
        let info = make_info(Arc::clone(&delimiter));
        let mut bundler = info.start_packet_bundling(&delimiter).expect("bundler");

        for i in 0..BUNDLE_SIZE_LIMIT {
            let value = if i % 2 == 0 { i as i32 } else { -(i as i32) };
            let packet: SharedPacket = Arc::new(TestPacket(value));
            assert!(bundler.add_packet(packet).is_none());
        }

        // BUNDLE_SIZE_LIMIT packets are allowed; the next one must panic.
        let _ = bundler.add_packet(Arc::new(TestPacket(i32::MAX)));
    }
}
