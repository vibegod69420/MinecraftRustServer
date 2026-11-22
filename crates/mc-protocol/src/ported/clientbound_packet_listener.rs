//! Port of `net.minecraft.network.ClientboundPacketListener`.

/// Minimal self-contained stub for `net.minecraft.network.protocol.PacketFlow`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

/// Minimal self-contained stub for `net.minecraft.network.PacketListener`.
pub trait PacketListener {
    fn flow(&self) -> PacketFlow;
}

/// Port of `net.minecraft.network.ClientboundPacketListener`.
pub trait ClientboundPacketListener {
    // JAVA: net.minecraft.network.ClientboundPacketListener#flow
    fn flow(&self) -> PacketFlow {
        PacketFlow::Clientbound
    }
}

impl<T: ClientboundPacketListener + ?Sized> PacketListener for T {
    fn flow(&self) -> PacketFlow {
        ClientboundPacketListener::flow(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyClientListener;
    impl ClientboundPacketListener for DummyClientListener {}

    struct AnotherClientListener {
        _id: i32,
    }
    impl ClientboundPacketListener for AnotherClientListener {}

    #[test]
    fn flow_returns_clientbound() {
        let listener = DummyClientListener;
        assert_eq!(ClientboundPacketListener::flow(&listener), PacketFlow::Clientbound);
    }

    #[test]
    fn packet_listener_blanket_impl_returns_clientbound() {
        let listener = DummyClientListener;
        assert_eq!(PacketListener::flow(&listener), PacketFlow::Clientbound);
    }

    #[test]
    fn different_implementor_returns_clientbound() {
        let listener = AnotherClientListener { _id: -1 };
        assert_eq!(PacketListener::flow(&listener), PacketFlow::Clientbound);
    }

    #[test]
    fn packet_flow_enum_distinguishes_directions() {
        assert_ne!(PacketFlow::Clientbound, PacketFlow::Serverbound);
        let flow = PacketFlow::Clientbound;
        assert_eq!(flow, PacketFlow::Clientbound);
    }

    #[test]
    fn flow_is_consistent_across_calls() {
        let listener = DummyClientListener;
        let first = ClientboundPacketListener::flow(&listener);
        let second = PacketListener::flow(&listener);
        assert_eq!(first, second);
        assert_eq!(first, PacketFlow::Clientbound);
    }
}
