use std::marker::PhantomData;
use std::sync::LazyLock;

/// Minimal packet flow enum matching `net.minecraft.network.protocol.PacketFlow`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

/// Minimal identifier matching `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    /// Creates an identifier in the default `minecraft` namespace.
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

/// Minimal packet marker trait matching `net.minecraft.network.protocol.Packet<T>`.
pub trait Packet<L: ?Sized> {}

/// Minimal listener marker matching
/// `net.minecraft.network.protocol.status.ClientStatusPacketListener`.
pub trait ClientStatusPacketListener {}

/// Minimal listener marker matching
/// `net.minecraft.network.protocol.status.ServerStatusPacketListener`.
pub trait ServerStatusPacketListener {}

/// Minimal packet type matching `net.minecraft.network.protocol.PacketType<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType<T> {
    pub flow: PacketFlow,
    pub id: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T> PacketType<T> {
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self {
            flow,
            id,
            _marker: PhantomData,
        }
    }
}

// Minimal self-contained packet placeholders for the status protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundStatusResponsePacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundStatusRequestPacket;

impl Packet<dyn ClientStatusPacketListener> for ClientboundStatusResponsePacket {}
impl Packet<dyn ServerStatusPacketListener> for ServerboundStatusRequestPacket {}

// JAVA: net.minecraft.network.protocol.status.StatusPacketTypes#CLIENTBOUND_STATUS_RESPONSE
pub static CLIENTBOUND_STATUS_RESPONSE: LazyLock<PacketType<ClientboundStatusResponsePacket>> =
    LazyLock::new(|| create_clientbound("status_response"));

// JAVA: net.minecraft.network.protocol.status.StatusPacketTypes#SERVERBOUND_STATUS_REQUEST
pub static SERVERBOUND_STATUS_REQUEST: LazyLock<PacketType<ServerboundStatusRequestPacket>> =
    LazyLock::new(|| create_serverbound("status_request"));

// JAVA: net.minecraft.network.protocol.status.StatusPacketTypes#createClientbound
fn create_clientbound<T: Packet<dyn ClientStatusPacketListener>>(path: &str) -> PacketType<T> {
    PacketType::new(
        PacketFlow::Clientbound,
        Identifier::with_default_namespace(path),
    )
}

// JAVA: net.minecraft.network.protocol.status.StatusPacketTypes#createServerbound
fn create_serverbound<T: Packet<dyn ServerStatusPacketListener>>(path: &str) -> PacketType<T> {
    PacketType::new(
        PacketFlow::Serverbound,
        Identifier::with_default_namespace(path),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clientbound_status_response_matches_java() {
        let pt = &*CLIENTBOUND_STATUS_RESPONSE;
        assert_eq!(pt.flow, PacketFlow::Clientbound);
        assert_eq!(pt.id.namespace, "minecraft");
        assert_eq!(pt.id.path, "status_response");
    }

    #[test]
    fn serverbound_status_request_matches_java() {
        let pt = &*SERVERBOUND_STATUS_REQUEST;
        assert_eq!(pt.flow, PacketFlow::Serverbound);
        assert_eq!(pt.id.namespace, "minecraft");
        assert_eq!(pt.id.path, "status_request");
    }

    #[test]
    fn create_functions_build_expected_packet_types() {
        let c: PacketType<ClientboundStatusResponsePacket> = create_clientbound("status_response");
        let s: PacketType<ServerboundStatusRequestPacket> = create_serverbound("status_request");
        assert_eq!(c.flow, PacketFlow::Clientbound);
        assert_eq!(s.flow, PacketFlow::Serverbound);
        assert_eq!(c.id, Identifier::with_default_namespace("status_response"));
        assert_eq!(s.id, Identifier::with_default_namespace("status_request"));
    }

    #[test]
    fn java_int_semantics_wrapping_overflow_negative() {
        let a: i32 = i32::MAX;
        assert_eq!(a.wrapping_add(1), i32::MIN);
        let b: i32 = i32::MIN;
        assert_eq!(b.wrapping_sub(1), i32::MAX);
        let c: i32 = -1;
        assert_eq!(c.wrapping_mul(2), -2);
        assert_eq!((-7_i32).wrapping_add(3), -4);
        let shift_masked: i32 = 1_i32.wrapping_shl((33_u32) & 31);
        assert_eq!(shift_masked, 2);
    }

    #[test]
    fn java_float_bits_edge_cases() {
        assert_eq!(f32::NAN.to_bits() & 0x7FC0_0000, 0x7FC0_0000);
        assert_eq!(f32::INFINITY.to_bits(), 0x7F80_0000);
        assert_eq!(f32::NEG_INFINITY.to_bits(), 0xFF80_0000);
        assert_eq!((-0.0_f32).to_bits(), 0x8000_0000);
        assert_eq!(0.0_f32.to_bits(), 0x0000_0000);
        let neg_nan_bits = (-f32::NAN).to_bits();
        assert_ne!(neg_nan_bits, f32::NAN.to_bits());
    }

    #[test]
    fn identifier_default_namespace_edge_cases() {
        let empty = Identifier::with_default_namespace("");
        assert_eq!(empty.namespace, "minecraft");
        assert_eq!(empty.path, "");
        let custom = Identifier::with_default_namespace("a/b_c-1");
        assert_eq!(custom.path, "a/b_c-1");
    }
}
