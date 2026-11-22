use std::marker::PhantomData;

pub trait ClientGamePacketListener {}

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

impl<T: ?Sized> std::hash::Hash for PacketType<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl<T: ?Sized> std::fmt::Debug for PacketType<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketType")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

pub struct BundleDelimiterPacket<L: ?Sized>(PhantomData<fn() -> L>);

impl<L: ?Sized> std::fmt::Debug for BundleDelimiterPacket<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BundleDelimiterPacket").finish()
    }
}

impl<L: ?Sized> Clone for BundleDelimiterPacket<L> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: ?Sized> Copy for BundleDelimiterPacket<L> {}

impl<L: ?Sized> Default for BundleDelimiterPacket<L> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<L: ?Sized> PartialEq for BundleDelimiterPacket<L> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<L: ?Sized> Eq for BundleDelimiterPacket<L> {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClientboundBundleDelimiterPacket {
    _base: BundleDelimiterPacket<dyn ClientGamePacketListener>,
}

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_BUNDLE_DELIMITER: PacketType<ClientboundBundleDelimiterPacket> =
        PacketType::new(0, "clientbound_bundle_delimiter");
}

impl ClientboundBundleDelimiterPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBundleDelimiterPacket#type
    pub fn type_(&self) -> PacketType<ClientboundBundleDelimiterPacket> {
        GamePacketTypes::CLIENTBOUND_BUNDLE_DELIMITER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_returns_bundle_delimiter() {
        let pkt = ClientboundBundleDelimiterPacket::default();
        let t = pkt.type_();
        assert_eq!(t.id, 0);
        assert_eq!(t.name, "clientbound_bundle_delimiter");
        assert_eq!(t, GamePacketTypes::CLIENTBOUND_BUNDLE_DELIMITER);
    }

    #[test]
    fn default_copy_and_equality() {
        let a = ClientboundBundleDelimiterPacket::default();
        let b = ClientboundBundleDelimiterPacket::default();
        assert_eq!(a, b);
        let c = a;
        assert_eq!(a, c);
    }

    #[test]
    fn base_default_with_dyn_trait() {
        let base = BundleDelimiterPacket::<dyn ClientGamePacketListener>::default();
        let base2 = BundleDelimiterPacket::<dyn ClientGamePacketListener>::default();
        assert_eq!(base, base2);
        let _ = format!("{:?}", base);
    }

    #[test]
    fn packet_type_properties() {
        let pt = PacketType::<ClientboundBundleDelimiterPacket>::new(42, "test");
        assert_eq!(pt.id, 42);
        assert_eq!(pt.name, "test");
        let pt2 = pt;
        assert_eq!(pt, pt2);
    }
}
