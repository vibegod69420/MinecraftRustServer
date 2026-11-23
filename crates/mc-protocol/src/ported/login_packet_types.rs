use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace(String)
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType {
    pub flow: PacketFlow,
    pub id: Identifier,
}

impl PacketType {
    // JAVA: net.minecraft.network.protocol.PacketType#<init>(PacketFlow, Identifier)
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self { flow, id }
    }
}

// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#CLIENTBOUND_CUSTOM_QUERY
pub static CLIENTBOUND_CUSTOM_QUERY: LazyLock<PacketType> =
    LazyLock::new(|| create_clientbound("custom_query"));
// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#CLIENTBOUND_LOGIN_FINISHED
pub static CLIENTBOUND_LOGIN_FINISHED: LazyLock<PacketType> =
    LazyLock::new(|| create_clientbound("login_finished"));
// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#CLIENTBOUND_HELLO
pub static CLIENTBOUND_HELLO: LazyLock<PacketType> =
    LazyLock::new(|| create_clientbound("hello"));
// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#CLIENTBOUND_LOGIN_COMPRESSION
pub static CLIENTBOUND_LOGIN_COMPRESSION: LazyLock<PacketType> =
    LazyLock::new(|| create_clientbound("login_compression"));
// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#CLIENTBOUND_LOGIN_DISCONNECT
pub static CLIENTBOUND_LOGIN_DISCONNECT: LazyLock<PacketType> =
    LazyLock::new(|| create_clientbound("login_disconnect"));
// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#SERVERBOUND_CUSTOM_QUERY_ANSWER
pub static SERVERBOUND_CUSTOM_QUERY_ANSWER: LazyLock<PacketType> =
    LazyLock::new(|| create_serverbound("custom_query_answer"));
// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#SERVERBOUND_HELLO
pub static SERVERBOUND_HELLO: LazyLock<PacketType> =
    LazyLock::new(|| create_serverbound("hello"));
// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#SERVERBOUND_KEY
pub static SERVERBOUND_KEY: LazyLock<PacketType> =
    LazyLock::new(|| create_serverbound("key"));
// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#SERVERBOUND_LOGIN_ACKNOWLEDGED
pub static SERVERBOUND_LOGIN_ACKNOWLEDGED: LazyLock<PacketType> =
    LazyLock::new(|| create_serverbound("login_acknowledged"));

// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#createClientbound(String)
fn create_clientbound(name: &str) -> PacketType {
    PacketType::new(
        PacketFlow::Clientbound,
        Identifier::with_default_namespace(name),
    )
}

// JAVA: net.minecraft.network.protocol.login.LoginPacketTypes#createServerbound(String)
fn create_serverbound(name: &str) -> PacketType {
    PacketType::new(
        PacketFlow::Serverbound,
        Identifier::with_default_namespace(name),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clientbound_packet_types() {
        let cases = [
            (&*CLIENTBOUND_CUSTOM_QUERY, "custom_query"),
            (&*CLIENTBOUND_LOGIN_FINISHED, "login_finished"),
            (&*CLIENTBOUND_HELLO, "hello"),
            (&*CLIENTBOUND_LOGIN_COMPRESSION, "login_compression"),
            (&*CLIENTBOUND_LOGIN_DISCONNECT, "login_disconnect"),
        ];
        for (pt, path) in cases {
            assert_eq!(pt.flow, PacketFlow::Clientbound);
            assert_eq!(pt.id.namespace, "minecraft");
            assert_eq!(pt.id.path, path);
        }
    }

    #[test]
    fn serverbound_packet_types() {
        let cases = [
            (&*SERVERBOUND_CUSTOM_QUERY_ANSWER, "custom_query_answer"),
            (&*SERVERBOUND_HELLO, "hello"),
            (&*SERVERBOUND_KEY, "key"),
            (&*SERVERBOUND_LOGIN_ACKNOWLEDGED, "login_acknowledged"),
        ];
        for (pt, path) in cases {
            assert_eq!(pt.flow, PacketFlow::Serverbound);
            assert_eq!(pt.id.namespace, "minecraft");
            assert_eq!(pt.id.path, path);
        }
    }

    #[test]
    fn identifier_default_namespace() {
        let id = Identifier::with_default_namespace("test");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "test");
    }

    #[test]
    fn packet_type_equality() {
        let a = create_clientbound("hello");
        let b = PacketType::new(
            PacketFlow::Clientbound,
            Identifier::with_default_namespace("hello"),
        );
        let c = create_serverbound("hello");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
