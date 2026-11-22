/// Placeholder for `net.minecraft.network.chat.Component`.
/// The real type is unported; this preserves the packet field shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub value: String,
}

/// Placeholder for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

/// Placeholder for `net.minecraft.network.protocol.game.GamePacketTypes`.
pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_SET_SUBTITLE_TEXT: PacketType =
        PacketType("minecraft:clientbound_set_subtitle_text");
}

/// Placeholder for `net.minecraft.network.protocol.game.ClientGamePacketListener`.
pub trait ClientGamePacketListener {
    fn set_subtitle_text(&mut self, packet: &ClientboundSetSubtitleTextPacket);
}

/// Port of `net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetSubtitleTextPacket {
    text: Component,
}

impl ClientboundSetSubtitleTextPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket#<init>
    pub fn new(text: Component) -> Self {
        Self { text }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket#text
    pub fn text(&self) -> &Component {
        &self.text
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket#type
    pub fn get_type(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_SET_SUBTITLE_TEXT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.set_subtitle_text(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        received: Option<ClientboundSetSubtitleTextPacket>,
    }

    impl ClientGamePacketListener for MockListener {
        fn set_subtitle_text(&mut self, packet: &ClientboundSetSubtitleTextPacket) {
            self.received = Some(packet.clone());
        }
    }

    #[test]
    fn new_and_text_accessor() {
        let component = Component {
            value: "hello".to_string(),
        };
        let packet = ClientboundSetSubtitleTextPacket::new(component.clone());
        assert_eq!(packet.text(), &component);
    }

    #[test]
    fn packet_type_matches() {
        let packet = ClientboundSetSubtitleTextPacket::new(Component {
            value: String::new(),
        });
        assert_eq!(
            packet.get_type(),
            GamePacketTypes::CLIENTBOUND_SET_SUBTITLE_TEXT
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ClientboundSetSubtitleTextPacket::new(Component {
            value: "subtitle".to_string(),
        });
        let mut listener = MockListener { received: None };
        packet.handle(&mut listener);
        assert_eq!(listener.received.as_ref(), Some(&packet));
    }

    #[test]
    fn clone_and_eq() {
        let packet = ClientboundSetSubtitleTextPacket::new(Component {
            value: "x".to_string(),
        });
        let cloned = packet.clone();
        assert_eq!(packet, cloned);
    }
}
