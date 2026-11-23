use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketType {
    pub flow: PacketFlow,
    pub id: Identifier,
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#createClientbound
fn create_clientbound(name: &str) -> PacketType {
    PacketType {
        flow: PacketFlow::Clientbound,
        id: Identifier::with_default_namespace(name),
    }
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#createServerbound
fn create_serverbound(name: &str) -> PacketType {
    PacketType {
        flow: PacketFlow::Serverbound,
        id: Identifier::with_default_namespace(name),
    }
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#CLIENTBOUND_CODE_OF_CONDUCT
pub fn clientbound_code_of_conduct() -> PacketType {
    create_clientbound("code_of_conduct")
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#CLIENTBOUND_FINISH_CONFIGURATION
pub fn clientbound_finish_configuration() -> PacketType {
    create_clientbound("finish_configuration")
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#CLIENTBOUND_REGISTRY_DATA
pub fn clientbound_registry_data() -> PacketType {
    create_clientbound("registry_data")
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#CLIENTBOUND_RESET_CHAT
pub fn clientbound_reset_chat() -> PacketType {
    create_clientbound("reset_chat")
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#CLIENTBOUND_SELECT_KNOWN_PACKS
pub fn clientbound_select_known_packs() -> PacketType {
    create_clientbound("select_known_packs")
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#CLIENTBOUND_UPDATE_ENABLED_FEATURES
pub fn clientbound_update_enabled_features() -> PacketType {
    create_clientbound("update_enabled_features")
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#SERVERBOUND_ACCEPT_CODE_OF_CONDUCT
pub fn serverbound_accept_code_of_conduct() -> PacketType {
    create_serverbound("accept_code_of_conduct")
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#SERVERBOUND_FINISH_CONFIGURATION
pub fn serverbound_finish_configuration() -> PacketType {
    create_serverbound("finish_configuration")
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationPacketTypes#SERVERBOUND_SELECT_KNOWN_PACKS
pub fn serverbound_select_known_packs() -> PacketType {
    create_serverbound("select_known_packs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clientbound_types_have_correct_flow_and_ids() {
        let cases = [
            (clientbound_code_of_conduct(), "code_of_conduct"),
            (clientbound_finish_configuration(), "finish_configuration"),
            (clientbound_registry_data(), "registry_data"),
            (clientbound_reset_chat(), "reset_chat"),
            (clientbound_select_known_packs(), "select_known_packs"),
            (
                clientbound_update_enabled_features(),
                "update_enabled_features",
            ),
        ];
        for (ty, name) in cases {
            assert_eq!(ty.flow, PacketFlow::Clientbound);
            assert_eq!(ty.id, Identifier::with_default_namespace(name));
        }
    }

    #[test]
    fn serverbound_types_have_correct_flow_and_ids() {
        let cases = [
            (
                serverbound_accept_code_of_conduct(),
                "accept_code_of_conduct",
            ),
            (
                serverbound_finish_configuration(),
                "finish_configuration",
            ),
            (serverbound_select_known_packs(), "select_known_packs"),
        ];
        for (ty, name) in cases {
            assert_eq!(ty.flow, PacketFlow::Serverbound);
            assert_eq!(ty.id, Identifier::with_default_namespace(name));
        }
    }

    #[test]
    fn identifier_default_namespace() {
        let id = Identifier::with_default_namespace("test_path");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "test_path");
        assert_eq!(id.to_string(), "minecraft:test_path");
    }

    #[test]
    fn packet_type_equality_and_clone() {
        let a = clientbound_registry_data();
        let b = create_clientbound("registry_data");
        assert_eq!(a, b);
        let c = a.clone();
        assert_eq!(a, c);
        assert_ne!(a, serverbound_finish_configuration());
    }
}
