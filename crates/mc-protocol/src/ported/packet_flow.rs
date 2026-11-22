//! Port of `net.minecraft.network.protocol.PacketFlow`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketFlow {
    Serverbound,
    Clientbound,
}

impl PacketFlow {
    // JAVA: net.minecraft.network.protocol.PacketFlow#getOpposite
    pub fn get_opposite(&self) -> Self {
        match self {
            Self::Clientbound => Self::Serverbound,
            Self::Serverbound => Self::Clientbound,
        }
    }

    // JAVA: net.minecraft.network.protocol.PacketFlow#id
    pub fn id(&self) -> &'static str {
        match self {
            Self::Serverbound => "serverbound",
            Self::Clientbound => "clientbound",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serverbound_id() {
        assert_eq!(PacketFlow::Serverbound.id(), "serverbound");
    }

    #[test]
    fn clientbound_id() {
        assert_eq!(PacketFlow::Clientbound.id(), "clientbound");
    }

    #[test]
    fn serverbound_opposite_is_clientbound() {
        assert_eq!(
            PacketFlow::Serverbound.get_opposite(),
            PacketFlow::Clientbound
        );
    }

    #[test]
    fn clientbound_opposite_is_serverbound() {
        assert_eq!(
            PacketFlow::Clientbound.get_opposite(),
            PacketFlow::Serverbound
        );
    }

    #[test]
    fn opposite_is_involution() {
        assert_eq!(
            PacketFlow::Serverbound.get_opposite().get_opposite(),
            PacketFlow::Serverbound
        );
        assert_eq!(
            PacketFlow::Clientbound.get_opposite().get_opposite(),
            PacketFlow::Clientbound
        );
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(PacketFlow::Serverbound, PacketFlow::Clientbound);
        assert_ne!(PacketFlow::Serverbound.id(), PacketFlow::Clientbound.id());
    }
}
