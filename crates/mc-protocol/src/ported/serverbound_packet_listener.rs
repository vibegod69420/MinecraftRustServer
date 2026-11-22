//! Port of `net.minecraft.network.ServerboundPacketListener`.

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

/// Port of `net.minecraft.network.ServerboundPacketListener`.
pub trait ServerboundPacketListener {
    // JAVA: net.minecraft.network.ServerboundPacketListener#flow
    fn flow(&self) -> PacketFlow {
        PacketFlow::Serverbound
    }
}

impl<T: ServerboundPacketListener + ?Sized> PacketListener for T {
    fn flow(&self) -> PacketFlow {
        ServerboundPacketListener::flow(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyServerListener;
    impl ServerboundPacketListener for DummyServerListener {}

    struct AnotherServerListener {
        _port: u16,
    }
    impl ServerboundPacketListener for AnotherServerListener {}

    #[test]
    fn flow_returns_serverbound() {
        let listener = DummyServerListener;
        assert_eq!(ServerboundPacketListener::flow(&listener), PacketFlow::Serverbound);
    }

    #[test]
    fn packet_listener_blanket_impl_returns_serverbound() {
        let listener = DummyServerListener;
        assert_eq!(PacketListener::flow(&listener), PacketFlow::Serverbound);
    }

    #[test]
    fn different_implementor_returns_serverbound() {
        let listener = AnotherServerListener { _port: u16::MAX };
        assert_eq!(PacketListener::flow(&listener), PacketFlow::Serverbound);
    }

    #[test]
    fn packet_flow_enum_distinguishes_directions() {
        assert_ne!(PacketFlow::Serverbound, PacketFlow::Clientbound);
        let flow = PacketFlow::Serverbound;
        assert_eq!(flow, PacketFlow::Serverbound);
    }

    #[test]
    fn flow_is_consistent_across_calls() {
        let listener = DummyServerListener;
        let first = ServerboundPacketListener::flow(&listener);
        let second = PacketListener::flow(&listener);
        assert_eq!(first, second);
        assert_eq!(first, PacketFlow::Serverbound);
    }
}
