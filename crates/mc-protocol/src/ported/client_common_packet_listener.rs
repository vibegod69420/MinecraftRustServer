// Placeholder packet types for self-contained compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundKeepAlivePacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPingPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomPayloadPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDisconnectPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundResourcePackPushPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundResourcePackPopPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateTagsPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundStoreCookiePacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTransferPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomReportDetailsPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundServerLinksPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundClearDialogPacket;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundShowDialogPacket;

/// Minimal supertrait stub for self-contained compilation.
pub trait ClientCookiePacketListener {}

pub trait ClientCommonPacketListener: ClientCookiePacketListener {
    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleKeepAlive
    fn handle_keep_alive(&mut self, packet: &ClientboundKeepAlivePacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handlePing
    fn handle_ping(&mut self, packet: &ClientboundPingPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleCustomPayload
    fn handle_custom_payload(&mut self, packet: &ClientboundCustomPayloadPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleDisconnect
    fn handle_disconnect(&mut self, packet: &ClientboundDisconnectPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleResourcePackPush
    fn handle_resource_pack_push(&mut self, packet: &ClientboundResourcePackPushPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleResourcePackPop
    fn handle_resource_pack_pop(&mut self, packet: &ClientboundResourcePackPopPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleUpdateTags
    fn handle_update_tags(&mut self, packet: &ClientboundUpdateTagsPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleStoreCookie
    fn handle_store_cookie(&mut self, packet: &ClientboundStoreCookiePacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleTransfer
    fn handle_transfer(&mut self, packet: &ClientboundTransferPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleCustomReportDetails
    fn handle_custom_report_details(&mut self, packet: &ClientboundCustomReportDetailsPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleServerLinks
    fn handle_server_links(&mut self, packet: &ClientboundServerLinksPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleClearDialog
    fn handle_clear_dialog(&mut self, packet: &ClientboundClearDialogPacket);

    // JAVA: net.minecraft.network.protocol.common.ClientCommonPacketListener#handleShowDialog
    fn handle_show_dialog(&mut self, packet: &ClientboundShowDialogPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct DummyListener {
        keep_alive: bool,
        ping: bool,
        transfer: bool,
        disconnect: bool,
    }

    impl ClientCookiePacketListener for DummyListener {}

    impl ClientCommonPacketListener for DummyListener {
        fn handle_keep_alive(&mut self, _packet: &ClientboundKeepAlivePacket) {
            self.keep_alive = true;
        }
        fn handle_ping(&mut self, _packet: &ClientboundPingPacket) {
            self.ping = true;
        }
        fn handle_custom_payload(&mut self, _packet: &ClientboundCustomPayloadPacket) {}
        fn handle_disconnect(&mut self, _packet: &ClientboundDisconnectPacket) {
            self.disconnect = true;
        }
        fn handle_resource_pack_push(&mut self, _packet: &ClientboundResourcePackPushPacket) {}
        fn handle_resource_pack_pop(&mut self, _packet: &ClientboundResourcePackPopPacket) {}
        fn handle_update_tags(&mut self, _packet: &ClientboundUpdateTagsPacket) {}
        fn handle_store_cookie(&mut self, _packet: &ClientboundStoreCookiePacket) {}
        fn handle_transfer(&mut self, _packet: &ClientboundTransferPacket) {
            self.transfer = true;
        }
        fn handle_custom_report_details(
            &mut self,
            _packet: &ClientboundCustomReportDetailsPacket,
        ) {
        }
        fn handle_server_links(&mut self, _packet: &ClientboundServerLinksPacket) {}
        fn handle_clear_dialog(&mut self, _packet: &ClientboundClearDialogPacket) {}
        fn handle_show_dialog(&mut self, _packet: &ClientboundShowDialogPacket) {}
    }

    #[test]
    fn dispatch_keep_alive() {
        let mut l = DummyListener::default();
        l.handle_keep_alive(&ClientboundKeepAlivePacket);
        assert!(l.keep_alive);
    }

    #[test]
    fn dispatch_ping_and_transfer() {
        let mut l = DummyListener::default();
        l.handle_ping(&ClientboundPingPacket);
        l.handle_transfer(&ClientboundTransferPacket);
        assert!(l.ping);
        assert!(l.transfer);
    }

    #[test]
    fn dispatch_disconnect() {
        let mut l = DummyListener::default();
        l.handle_disconnect(&ClientboundDisconnectPacket);
        assert!(l.disconnect);
    }

    #[test]
    fn implements_cookie_supertrait() {
        fn assert_cookie<L: ClientCookiePacketListener>() {}
        assert_cookie::<DummyListener>();
    }
}
