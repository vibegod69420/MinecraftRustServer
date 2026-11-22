use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub struct PacketType<T: ?Sized> {
    pub id: u32,
    pub name: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> PacketType<T> {
    pub const fn new(id: u32, name: &'static str) -> Self {
        Self {
            id,
            name,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Clone for PacketType<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for PacketType<T> {}

impl<T: ?Sized> PartialEq for PacketType<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl<T: ?Sized> Eq for PacketType<T> {}

impl<T: ?Sized> Hash for PacketType<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl<T: ?Sized> fmt::Debug for PacketType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PacketType")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

pub trait ClientboundPacket: fmt::Debug {
    fn handle(&self, listener: &mut dyn ClientGamePacketListener);
}

pub trait ClientGamePacketListener {
    fn handle_bundle_packet(&mut self, packet: &ClientboundBundlePacket);
}

pub struct BundlePacket<L: ?Sized> {
    packets: Vec<Box<dyn ClientboundPacket>>,
    _marker: PhantomData<fn() -> L>,
}

impl<L: ?Sized> BundlePacket<L> {
    // JAVA: net.minecraft.network.protocol.BundlePacket#<init>
    pub fn new(packets: impl IntoIterator<Item = Box<dyn ClientboundPacket>>) -> Self {
        Self {
            packets: packets.into_iter().collect(),
            _marker: PhantomData,
        }
    }

    pub fn packets(&self) -> &[Box<dyn ClientboundPacket>] {
        &self.packets
    }
}

impl<L: ?Sized> fmt::Debug for BundlePacket<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BundlePacket")
            .field("packets_len", &self.packets.len())
            .finish()
    }
}

pub struct ClientboundBundlePacket {
    base: BundlePacket<dyn ClientGamePacketListener>,
}

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_BUNDLE: PacketType<ClientboundBundlePacket> =
        PacketType::new(0, "clientbound_bundle");
}

impl ClientboundBundlePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBundlePacket#<init>
    pub fn new(packets: impl IntoIterator<Item = Box<dyn ClientboundPacket>>) -> Self {
        Self {
            base: BundlePacket::new(packets),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBundlePacket#type
    pub fn type_(&self) -> PacketType<ClientboundBundlePacket> {
        GamePacketTypes::CLIENTBOUND_BUNDLE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBundlePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_bundle_packet(self);
    }

    pub fn packets(&self) -> &[Box<dyn ClientboundPacket>] {
        self.base.packets()
    }
}

impl fmt::Debug for ClientboundBundlePacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientboundBundlePacket")
            .field("packets_len", &self.base.packets.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyPacket {
        id: u32,
    }

    impl ClientboundPacket for DummyPacket {
        fn handle(&self, _listener: &mut dyn ClientGamePacketListener) {}
    }

    struct MockListener {
        handled: usize,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_bundle_packet(&mut self, _packet: &ClientboundBundlePacket) {
            self.handled += 1;
        }
    }

    #[test]
    fn type_returns_bundle() {
        let pkt = ClientboundBundlePacket::new(Vec::new());
        let t = pkt.type_();
        assert_eq!(t.id, 0);
        assert_eq!(t.name, "clientbound_bundle");
        assert_eq!(t, GamePacketTypes::CLIENTBOUND_BUNDLE);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let pkt = ClientboundBundlePacket::new(Vec::new());
        let mut listener = MockListener { handled: 0 };
        pkt.handle(&mut listener);
        assert_eq!(listener.handled, 1);
        pkt.handle(&mut listener);
        assert_eq!(listener.handled, 2);
    }

    #[test]
    fn constructor_stores_packets() {
        let packets: Vec<Box<dyn ClientboundPacket>> = vec![
            Box::new(DummyPacket { id: 1 }),
            Box::new(DummyPacket { id: 2 }),
        ];
        let pkt = ClientboundBundlePacket::new(packets);
        assert_eq!(pkt.packets().len(), 2);
        let dbg = format!("{:?}", pkt);
        assert!(dbg.contains("packets_len"));
    }

    #[test]
    fn empty_bundle() {
        let pkt = ClientboundBundlePacket::new(std::iter::empty());
        assert_eq!(pkt.packets().len(), 0);
        let mut listener = MockListener { handled: 0 };
        pkt.handle(&mut listener);
        assert_eq!(listener.handled, 1);
    }

    #[test]
    fn packet_type_equality_and_copy() {
        let a = PacketType::<ClientboundBundlePacket>::new(1, "a");
        let b = a;
        let c = PacketType::<ClientboundBundlePacket>::new(2, "b");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
