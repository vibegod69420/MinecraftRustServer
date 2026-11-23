//! Port of `net.minecraft.network.protocol.common.ClientboundShowDialogPacket`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub java_constant: &'static str,
}

pub const CLIENTBOUND_SHOW_DIALOG_TYPE: PacketType = PacketType {
    java_constant: "CommonPacketTypes.CLIENTBOUND_SHOW_DIALOG",
};

/// Minimal holder mirror for `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    value: T,
}

impl<T> Holder<T> {
    // JAVA: net.minecraft.core.Holder#direct
    pub fn direct(value: T) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.core.Holder#value
    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Placeholder for `net.minecraft.server.dialog.Dialog`.
/// The full registry/dialog codec is external to this self-contained module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dialog {
    pub encoded_data: Vec<u8>,
}

pub trait ClientCommonPacketListener {
    fn handle_show_dialog(&mut self, packet: &ClientboundShowDialogPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundShowDialogPacket {
    dialog: Holder<Dialog>,
}

impl ClientboundShowDialogPacket {
    // JAVA: net.minecraft.network.protocol.common.ClientboundShowDialogPacket#<init>
    pub fn new(dialog: Holder<Dialog>) -> Self {
        Self { dialog }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundShowDialogPacket#dialog
    pub fn dialog(&self) -> &Holder<Dialog> {
        &self.dialog
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundShowDialogPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_SHOW_DIALOG_TYPE
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundShowDialogPacket#handle
    pub fn handle<L: ClientCommonPacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_show_dialog(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        dialog_len: usize,
    }

    impl ClientCommonPacketListener for MockListener {
        fn handle_show_dialog(&mut self, packet: &ClientboundShowDialogPacket) {
            self.called = true;
            self.dialog_len = packet.dialog().value().encoded_data.len();
        }
    }

    #[test]
    fn new_and_dialog_accessor() {
        let dialog = Dialog {
            encoded_data: vec![1, 2, 3],
        };
        let packet = ClientboundShowDialogPacket::new(Holder::direct(dialog.clone()));
        assert_eq!(packet.dialog().value(), &dialog);
    }

    #[test]
    fn packet_type_constant() {
        let packet = ClientboundShowDialogPacket::new(Holder::direct(Dialog {
            encoded_data: vec![],
        }));
        assert_eq!(packet.get_type(), CLIENTBOUND_SHOW_DIALOG_TYPE);
        assert_eq!(
            packet.get_type().java_constant,
            "CommonPacketTypes.CLIENTBOUND_SHOW_DIALOG"
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ClientboundShowDialogPacket::new(Holder::direct(Dialog {
            encoded_data: vec![9; 4],
        }));
        let mut listener = MockListener {
            called: false,
            dialog_len: 0,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.dialog_len, 4);
    }

    #[test]
    fn empty_dialog_payload_edge_case() {
        let packet = ClientboundShowDialogPacket::new(Holder::direct(Dialog {
            encoded_data: Vec::new(),
        }));
        assert!(packet.dialog().value().encoded_data.is_empty());
    }

    #[test]
    fn large_dialog_payload_edge_case() {
        let data = vec![0xABu8; 16 * 1024];
        let packet = ClientboundShowDialogPacket::new(Holder::direct(Dialog {
            encoded_data: data.clone(),
        }));
        assert_eq!(packet.dialog().value().encoded_data.len(), data.len());
        assert_eq!(packet.dialog().value().encoded_data[0], 0xAB);
        assert_eq!(*packet.dialog().value().encoded_data.last().unwrap(), 0xAB);
    }
}
