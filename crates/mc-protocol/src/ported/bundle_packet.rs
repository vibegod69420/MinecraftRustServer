//! Port of `net.minecraft.network.protocol.BundlePacket`.

/// Minimal placeholder for `net.minecraft.network.PacketListener`.
pub trait PacketListener {}

/// Minimal placeholder for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType {
    pub id: u32,
}

/// Minimal placeholder for `net.minecraft.network.protocol.Packet`.
pub trait Packet<T: PacketListener + ?Sized> {
    fn packet_type(&self) -> PacketType;
}

/// Provides the abstract `type()` for a `BundlePacket` subtype.
pub trait BundlePacketKind<T: PacketListener + ?Sized> {
    // JAVA: net.minecraft.network.protocol.BundlePacket#type
    fn bundle_type(&self) -> PacketType;
}

/// Port of abstract class `BundlePacket<T extends PacketListener> implements Packet<T>`.
pub struct BundlePacket<T: PacketListener + ?Sized, K: BundlePacketKind<T>> {
    packets: Vec<Box<dyn Packet<T>>>,
    kind: K,
}

impl<T: PacketListener + ?Sized, K: BundlePacketKind<T> + Default> BundlePacket<T, K> {
    // JAVA: net.minecraft.network.protocol.BundlePacket#<init>
    pub fn new(packets: Vec<Box<dyn Packet<T>>>) -> Self {
        Self {
            packets,
            kind: K::default(),
        }
    }
}

impl<T: PacketListener + ?Sized, K: BundlePacketKind<T>> BundlePacket<T, K> {
    // JAVA: net.minecraft.network.protocol.BundlePacket#subPackets
    pub fn sub_packets(&self) -> &[Box<dyn Packet<T>>] {
        &self.packets
    }
}

impl<T: PacketListener + ?Sized, K: BundlePacketKind<T>> Packet<T> for BundlePacket<T, K> {
    fn packet_type(&self) -> PacketType {
        self.kind.bundle_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener;
    impl PacketListener for TestListener {}

    struct DummyPacket {
        id: u32,
    }

    impl Packet<TestListener> for DummyPacket {
        fn packet_type(&self) -> PacketType {
            PacketType { id: self.id }
        }
    }

    #[derive(Default)]
    struct TestBundleKind;

    impl BundlePacketKind<TestListener> for TestBundleKind {
        fn bundle_type(&self) -> PacketType {
            PacketType { id: 999 }
        }
    }

    #[derive(Default)]
    struct AltBundleKind;

    impl BundlePacketKind<TestListener> for AltBundleKind {
        fn bundle_type(&self) -> PacketType {
            PacketType { id: u32::MAX }
        }
    }

    #[test]
    fn new_and_sub_packets_len() {
        let packets: Vec<Box<dyn Packet<TestListener>>> = vec![
            Box::new(DummyPacket { id: 1 }),
            Box::new(DummyPacket { id: 2 }),
        ];
        let bundle = BundlePacket::<TestListener, TestBundleKind>::new(packets);
        assert_eq!(bundle.sub_packets().len(), 2);
    }

    #[test]
    fn sub_packets_preserves_order() {
        let packets: Vec<Box<dyn Packet<TestListener>>> = vec![
            Box::new(DummyPacket { id: 10 }),
            Box::new(DummyPacket { id: 20 }),
            Box::new(DummyPacket { id: 30 }),
        ];
        let bundle = BundlePacket::<TestListener, TestBundleKind>::new(packets);
        let ids: Vec<u32> = bundle
            .sub_packets()
            .iter()
            .map(|p| p.packet_type().id)
            .collect();
        assert_eq!(ids, vec![10, 20, 30]);
    }

    #[test]
    fn bundle_packet_type_from_kind() {
        let bundle = BundlePacket::<TestListener, TestBundleKind>::new(Vec::new());
        assert_eq!(bundle.packet_type(), PacketType { id: 999 });
    }

    #[test]
    fn empty_bundle_has_no_sub_packets() {
        let bundle = BundlePacket::<TestListener, TestBundleKind>::new(Vec::new());
        assert!(bundle.sub_packets().is_empty());
        assert_eq!(bundle.packet_type(), PacketType { id: 999 });
    }

    #[test]
    fn different_kind_produces_different_type() {
        let bundle = BundlePacket::<TestListener, AltBundleKind>::new(Vec::new());
        assert_eq!(bundle.packet_type(), PacketType { id: u32::MAX });
    }

    #[test]
    fn sub_packets_with_negative_like_ids_via_wrapping() {
        let id = 0_u32.wrapping_sub(1);
        let packets: Vec<Box<dyn Packet<TestListener>>> =
            vec![Box::new(DummyPacket { id })];
        let bundle = BundlePacket::<TestListener, TestBundleKind>::new(packets);
        assert_eq!(bundle.sub_packets()[0].packet_type().id, u32::MAX);
    }
}
