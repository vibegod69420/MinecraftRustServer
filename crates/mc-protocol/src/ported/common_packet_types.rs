use std::marker::PhantomData;
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
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType<T> {
    pub flow: PacketFlow,
    pub id: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T> PacketType<T> {
    // JAVA: net.minecraft.network.protocol.PacketType#<init>
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self {
            flow,
            id,
            _marker: PhantomData,
        }
    }
}

// Marker types for referenced packet classes (self-contained stubs).
pub struct ClientboundClearDialogPacket;
pub struct ClientboundCustomPayloadPacket;
pub struct ClientboundCustomReportDetailsPacket;
pub struct ClientboundDisconnectPacket;
pub struct ClientboundKeepAlivePacket;
pub struct ClientboundPingPacket;
pub struct ClientboundResourcePackPopPacket;
pub struct ClientboundResourcePackPushPacket;
pub struct ClientboundServerLinksPacket;
pub struct ClientboundShowDialogPacket;
pub struct ClientboundStoreCookiePacket;
pub struct ClientboundTransferPacket;
pub struct ClientboundUpdateTagsPacket;
pub struct ServerboundClientInformationPacket;
pub struct ServerboundCustomPayloadPacket;
pub struct ServerboundKeepAlivePacket;
pub struct ServerboundPongPacket;
pub struct ServerboundResourcePackPacket;
pub struct ServerboundCustomClickActionPacket;

// JAVA: net.minecraft.network.protocol.common.CommonPacketTypes#createClientbound
fn create_clientbound<T>(name: &str) -> PacketType<T> {
    PacketType::new(PacketFlow::Clientbound, Identifier::with_default_namespace(name))
}

// JAVA: net.minecraft.network.protocol.common.CommonPacketTypes#createServerbound
fn create_serverbound<T>(name: &str) -> PacketType<T> {
    PacketType::new(PacketFlow::Serverbound, Identifier::with_default_namespace(name))
}

pub static CLIENTBOUND_CLEAR_DIALOG: LazyLock<PacketType<ClientboundClearDialogPacket>> =
    LazyLock::new(|| create_clientbound("clear_dialog"));
pub static CLIENTBOUND_CUSTOM_PAYLOAD: LazyLock<PacketType<ClientboundCustomPayloadPacket>> =
    LazyLock::new(|| create_clientbound("custom_payload"));
pub static CLIENTBOUND_CUSTOM_REPORT_DETAILS: LazyLock<
    PacketType<ClientboundCustomReportDetailsPacket>,
> = LazyLock::new(|| create_clientbound("custom_report_details"));
pub static CLIENTBOUND_DISCONNECT: LazyLock<PacketType<ClientboundDisconnectPacket>> =
    LazyLock::new(|| create_clientbound("disconnect"));
pub static CLIENTBOUND_KEEP_ALIVE: LazyLock<PacketType<ClientboundKeepAlivePacket>> =
    LazyLock::new(|| create_clientbound("keep_alive"));
pub static CLIENTBOUND_PING: LazyLock<PacketType<ClientboundPingPacket>> =
    LazyLock::new(|| create_clientbound("ping"));
pub static CLIENTBOUND_RESOURCE_PACK_POP: LazyLock<PacketType<ClientboundResourcePackPopPacket>> =
    LazyLock::new(|| create_clientbound("resource_pack_pop"));
pub static CLIENTBOUND_RESOURCE_PACK_PUSH: LazyLock<PacketType<ClientboundResourcePackPushPacket>> =
    LazyLock::new(|| create_clientbound("resource_pack_push"));
pub static CLIENTBOUND_SERVER_LINKS: LazyLock<PacketType<ClientboundServerLinksPacket>> =
    LazyLock::new(|| create_clientbound("server_links"));
pub static CLIENTBOUND_SHOW_DIALOG: LazyLock<PacketType<ClientboundShowDialogPacket>> =
    LazyLock::new(|| create_clientbound("show_dialog"));
pub static CLIENTBOUND_STORE_COOKIE: LazyLock<PacketType<ClientboundStoreCookiePacket>> =
    LazyLock::new(|| create_clientbound("store_cookie"));
pub static CLIENTBOUND_TRANSFER: LazyLock<PacketType<ClientboundTransferPacket>> =
    LazyLock::new(|| create_clientbound("transfer"));
pub static CLIENTBOUND_UPDATE_TAGS: LazyLock<PacketType<ClientboundUpdateTagsPacket>> =
    LazyLock::new(|| create_clientbound("update_tags"));

pub static SERVERBOUND_CLIENT_INFORMATION: LazyLock<PacketType<ServerboundClientInformationPacket>> =
    LazyLock::new(|| create_serverbound("client_information"));
pub static SERVERBOUND_CUSTOM_PAYLOAD: LazyLock<PacketType<ServerboundCustomPayloadPacket>> =
    LazyLock::new(|| create_serverbound("custom_payload"));
pub static SERVERBOUND_KEEP_ALIVE: LazyLock<PacketType<ServerboundKeepAlivePacket>> =
    LazyLock::new(|| create_serverbound("keep_alive"));
pub static SERVERBOUND_PONG: LazyLock<PacketType<ServerboundPongPacket>> =
    LazyLock::new(|| create_serverbound("pong"));
pub static SERVERBOUND_RESOURCE_PACK: LazyLock<PacketType<ServerboundResourcePackPacket>> =
    LazyLock::new(|| create_serverbound("resource_pack"));
pub static SERVERBOUND_CUSTOM_CLICK_ACTION: LazyLock<
    PacketType<ServerboundCustomClickActionPacket>,
> = LazyLock::new(|| create_serverbound("custom_click_action"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_clientbound_sets_flow_and_namespace() {
        let pt: PacketType<()> = create_clientbound("test_packet");
        assert_eq!(pt.flow, PacketFlow::Clientbound);
        assert_eq!(pt.id.namespace, "minecraft");
        assert_eq!(pt.id.path, "test_packet");
    }

    #[test]
    fn create_serverbound_sets_flow_and_namespace() {
        let pt: PacketType<()> = create_serverbound("my_packet");
        assert_eq!(pt.flow, PacketFlow::Serverbound);
        assert_eq!(pt.id, Identifier::with_default_namespace("my_packet"));
    }

    #[test]
    fn clientbound_constants_have_expected_paths() {
        assert_eq!(CLIENTBOUND_CUSTOM_PAYLOAD.flow, PacketFlow::Clientbound);
        assert_eq!(CLIENTBOUND_CUSTOM_PAYLOAD.id.path, "custom_payload");
        assert_eq!(CLIENTBOUND_KEEP_ALIVE.id.path, "keep_alive");
        assert_eq!(CLIENTBOUND_DISCONNECT.id.path, "disconnect");
        assert_eq!(CLIENTBOUND_UPDATE_TAGS.id.path, "update_tags");
    }

    #[test]
    fn serverbound_constants_have_expected_paths() {
        assert_eq!(SERVERBOUND_KEEP_ALIVE.flow, PacketFlow::Serverbound);
        assert_eq!(SERVERBOUND_KEEP_ALIVE.id.path, "keep_alive");
        assert_eq!(SERVERBOUND_PONG.id.path, "pong");
        assert_eq!(SERVERBOUND_CUSTOM_CLICK_ACTION.id.path, "custom_click_action");
        assert_eq!(SERVERBOUND_CUSTOM_PAYLOAD.id.path, "custom_payload");
    }

    #[test]
    fn same_path_different_flow_is_distinguishable() {
        assert_eq!(CLIENTBOUND_KEEP_ALIVE.id.path, SERVERBOUND_KEEP_ALIVE.id.path);
        assert_ne!(CLIENTBOUND_KEEP_ALIVE.flow, SERVERBOUND_KEEP_ALIVE.flow);
        assert_eq!(CLIENTBOUND_CUSTOM_PAYLOAD.id.path, SERVERBOUND_CUSTOM_PAYLOAD.id.path);
        assert_ne!(CLIENTBOUND_CUSTOM_PAYLOAD.flow, SERVERBOUND_CUSTOM_PAYLOAD.flow);
    }
}
