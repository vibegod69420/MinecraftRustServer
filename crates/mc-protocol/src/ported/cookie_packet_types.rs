//! Port of `net.minecraft.network.protocol.cookie.CookiePacketTypes`.

use std::marker::PhantomData;

/// Minimal local stand-in for `net.minecraft.network.protocol.PacketFlow`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

/// Minimal local stand-in for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

/// Minimal local stand-in for `net.minecraft.network.protocol.Packet`.
pub trait Packet<L: ?Sized> {}

/// Minimal local stand-in for `net.minecraft.network.protocol.cookie.ClientCookiePacketListener`.
pub trait ClientCookiePacketListener {}

/// Minimal local stand-in for `net.minecraft.network.protocol.cookie.ServerCookiePacketListener`.
pub trait ServerCookiePacketListener {}

/// Minimal local stand-in for packet classes referenced by the cookie packet types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundCookieRequestPacket;

/// Minimal local stand-in for packet classes referenced by the cookie packet types.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundCookieResponsePacket;

impl<L: ?Sized + ClientCookiePacketListener> Packet<L> for ClientboundCookieRequestPacket {}
impl<L: ?Sized + ServerCookiePacketListener> Packet<L> for ServerboundCookieResponsePacket {}

/// Minimal local stand-in for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PacketType<T: ?Sized> {
    flow: PacketFlow,
    id: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> PacketType<T> {
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self {
            flow,
            id,
            _marker: PhantomData,
        }
    }

    pub fn flow(&self) -> PacketFlow {
        self.flow
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }
}

/// Port of `net.minecraft.network.protocol.cookie.CookiePacketTypes`.
pub struct CookiePacketTypes;

impl CookiePacketTypes {
    // JAVA: net.minecraft.network.protocol.cookie.CookiePacketTypes#createClientbound
    fn create_clientbound<T: ?Sized + Packet<dyn ClientCookiePacketListener>>(
        path: &str,
    ) -> PacketType<T> {
        PacketType::new(
            PacketFlow::Clientbound,
            Identifier::with_default_namespace(path),
        )
    }

    // JAVA: net.minecraft.network.protocol.cookie.CookiePacketTypes#createServerbound
    fn create_serverbound<T: ?Sized + Packet<dyn ServerCookiePacketListener>>(
        path: &str,
    ) -> PacketType<T> {
        PacketType::new(
            PacketFlow::Serverbound,
            Identifier::with_default_namespace(path),
        )
    }

    // JAVA: net.minecraft.network.protocol.cookie.CookiePacketTypes#CLIENTBOUND_COOKIE_REQUEST
    pub fn clientbound_cookie_request() -> PacketType<ClientboundCookieRequestPacket> {
        Self::create_clientbound("cookie_request")
    }

    // JAVA: net.minecraft.network.protocol.cookie.CookiePacketTypes#SERVERBOUND_COOKIE_RESPONSE
    pub fn serverbound_cookie_response() -> PacketType<ServerboundCookieResponsePacket> {
        Self::create_serverbound("cookie_response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clientbound_cookie_request_type() {
        let ty = CookiePacketTypes::clientbound_cookie_request();
        assert_eq!(ty.flow(), PacketFlow::Clientbound);
        assert_eq!(ty.id().namespace(), "minecraft");
        assert_eq!(ty.id().path(), "cookie_request");
    }

    #[test]
    fn serverbound_cookie_response_type() {
        let ty = CookiePacketTypes::serverbound_cookie_response();
        assert_eq!(ty.flow(), PacketFlow::Serverbound);
        assert_eq!(ty.id().namespace(), "minecraft");
        assert_eq!(ty.id().path(), "cookie_response");
    }

    #[test]
    fn identifier_default_namespace() {
        let id = Identifier::with_default_namespace("test_path");
        assert_eq!(id.namespace(), "minecraft");
        assert_eq!(id.path(), "test_path");
    }

    #[test]
    fn packet_type_equality_and_fields() {
        let a = PacketType::<ClientboundCookieRequestPacket>::new(
            PacketFlow::Clientbound,
            Identifier::with_default_namespace("cookie_request"),
        );
        let b = CookiePacketTypes::clientbound_cookie_request();
        let c = CookiePacketTypes::serverbound_cookie_response();
        assert_eq!(a, b);
        assert_ne!(a.id(), c.id());
        assert_ne!(a.flow(), c.flow());
    }

    #[test]
    fn packet_type_is_distinct_by_generic() {
        let a = PacketType::<ClientboundCookieRequestPacket>::new(
            PacketFlow::Clientbound,
            Identifier::with_default_namespace("same"),
        );
        let b = PacketType::<ServerboundCookieResponsePacket>::new(
            PacketFlow::Clientbound,
            Identifier::with_default_namespace("same"),
        );
        // Different Rust types; verify their runtime fields match as expected.
        assert_eq!(a.flow(), b.flow());
        assert_eq!(a.id(), b.id());
    }
}
