// JAVA: net.minecraft.network.ProtocolInfo
pub trait PacketListener {}

/// Placeholder packet type used to preserve the ProtocolInfo API shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

/// Placeholder packet marker used to preserve the ProtocolInfo API shape.
pub trait Packet {}

/// Placeholder bundle metadata type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BundlerInfo;

/// Minimal stream codec placeholder to keep this module self-contained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DummyCodec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketFlow {
    Serverbound,
    Clientbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

// JAVA: net.minecraft.network.ProtocolInfo$Details$PacketVisitor
pub trait DetailsPacketVisitor {
    // JAVA: net.minecraft.network.ProtocolInfo$Details$PacketVisitor#accept
    fn accept(&self, packet_type: &PacketType, id: i32);
}

// JAVA: net.minecraft.network.ProtocolInfo$Details
pub trait Details {
    // JAVA: net.minecraft.network.ProtocolInfo$Details#id
    fn id(&self) -> ConnectionProtocol;

    // JAVA: net.minecraft.network.ProtocolInfo$Details#flow
    fn flow(&self) -> PacketFlow;

    // JAVA: net.minecraft.network.ProtocolInfo$Details#listPackets
    fn list_packets(&self, visitor: &dyn DetailsPacketVisitor);
}

// JAVA: net.minecraft.network.ProtocolInfo$DetailsProvider
pub trait DetailsProvider {
    // JAVA: net.minecraft.network.ProtocolInfo$DetailsProvider#details
    fn details(&self) -> &dyn Details;
}

pub trait ProtocolInfo<T: PacketListener> {
    // JAVA: net.minecraft.network.ProtocolInfo#id
    fn id(&self) -> ConnectionProtocol;

    // JAVA: net.minecraft.network.ProtocolInfo#flow
    fn flow(&self) -> PacketFlow;

    // JAVA: net.minecraft.network.ProtocolInfo#codec
    fn codec(&self) -> DummyCodec;

    // JAVA: net.minecraft.network.ProtocolInfo#bundlerInfo
    fn bundler_info(&self) -> Option<BundlerInfo>;
}

#[cfg(test)]
mod tests {
    use super::{
        BundlerInfo, ConnectionProtocol, Details, DetailsPacketVisitor, DummyCodec, PacketFlow,
        PacketListener, PacketType, ProtocolInfo,
    };

    struct TestListener;

    impl PacketListener for TestListener {}

    struct TestVisitor {
        count: std::cell::Cell<i32>,
    }

    impl DetailsPacketVisitor for TestVisitor {
        fn accept(&self, packet_type: &PacketType, id: i32) {
            assert!(!packet_type.name.is_empty());
            assert!(id >= 0);
            self.count.set(self.count.get() + 1);
        }
    }

    struct TestDetails;

    impl Details for TestDetails {
        fn id(&self) -> ConnectionProtocol {
            ConnectionProtocol::Play
        }

        fn flow(&self) -> PacketFlow {
            PacketFlow::Serverbound
        }

        fn list_packets(&self, visitor: &dyn DetailsPacketVisitor) {
            visitor.accept(&PacketType { name: "test_a" }, 0);
            visitor.accept(&PacketType { name: "test_b" }, 1);
        }
    }

    struct TestProtocolInfo;

    impl ProtocolInfo<TestListener> for TestProtocolInfo {
        fn id(&self) -> ConnectionProtocol {
            ConnectionProtocol::Play
        }

        fn flow(&self) -> PacketFlow {
            PacketFlow::Clientbound
        }

        fn codec(&self) -> DummyCodec {
            DummyCodec
        }

        fn bundler_info(&self) -> Option<BundlerInfo> {
            Some(BundlerInfo)
        }
    }

    #[test]
    fn test_enums() {
        assert_eq!(ConnectionProtocol::Handshake, ConnectionProtocol::Handshake);
        assert_eq!(PacketFlow::Serverbound, PacketFlow::Serverbound);
        assert_ne!(PacketFlow::Serverbound, PacketFlow::Clientbound);
    }

    #[test]
    fn test_details_list_packets() {
        let details = TestDetails;
        let visitor = TestVisitor {
            count: std::cell::Cell::new(0),
        };
        details.list_packets(&visitor);
        assert_eq!(visitor.count.get(), 2);
    }

    #[test]
    fn test_protocol_info_defaults_shape() {
        let info = TestProtocolInfo;
        assert_eq!(info.id(), ConnectionProtocol::Play);
        assert_eq!(info.flow(), PacketFlow::Clientbound);
        assert_eq!(info.codec(), DummyCodec);
        assert_eq!(info.bundler_info(), Some(BundlerInfo));
    }
}
