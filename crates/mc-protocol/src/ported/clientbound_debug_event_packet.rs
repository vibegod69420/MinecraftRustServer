#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub &'static str);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_DEBUG_EVENT: PacketType = PacketType("CLIENTBOUND_DEBUG_EVENT");
}

/// Placeholder for `net.minecraft.util.debug.DebugSubscription.Event<?>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSubscriptionEvent {
    pub id: i32,
    pub data: Vec<u8>,
}

/// Placeholder for `net.minecraft.network.protocol.game.ClientGamePacketListener`.
pub trait ClientGamePacketListener {
    fn handle_debug_event(&mut self, packet: &ClientboundDebugEventPacket);
}

/// Port of `net.minecraft.network.protocol.game.ClientboundDebugEventPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugEventPacket {
    event: DebugSubscriptionEvent,
}

impl ClientboundDebugEventPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugEventPacket#<init>
    pub fn new(event: DebugSubscriptionEvent) -> Self {
        Self { event }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugEventPacket#event
    pub fn event(&self) -> &DebugSubscriptionEvent {
        &self.event
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugEventPacket#type
    pub fn type_(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_DEBUG_EVENT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugEventPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_debug_event(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        handled: bool,
        event_id: i32,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_debug_event(&mut self, packet: &ClientboundDebugEventPacket) {
            self.handled = true;
            self.event_id = packet.event().id;
        }
    }

    #[test]
    fn new_and_event_accessor() {
        let event = DebugSubscriptionEvent {
            id: 42,
            data: vec![1, 2, 3],
        };
        let packet = ClientboundDebugEventPacket::new(event.clone());
        assert_eq!(packet.event(), &event);
    }

    #[test]
    fn packet_type_is_debug_event() {
        let packet = ClientboundDebugEventPacket::new(DebugSubscriptionEvent {
            id: 0,
            data: vec![],
        });
        assert_eq!(packet.type_(), GamePacketTypes::CLIENTBOUND_DEBUG_EVENT);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ClientboundDebugEventPacket::new(DebugSubscriptionEvent {
            id: -7,
            data: vec![],
        });
        let mut listener = MockListener {
            handled: false,
            event_id: 0,
        };
        packet.handle(&mut listener);
        assert!(listener.handled);
        assert_eq!(listener.event_id, -7);
    }

    #[test]
    fn clone_eq_and_negative_edge_id() {
        let packet = ClientboundDebugEventPacket::new(DebugSubscriptionEvent {
            id: i32::MIN,
            data: vec![0xFF],
        });
        let cloned = packet.clone();
        assert_eq!(packet, cloned);
        assert_eq!(packet.event().id, i32::MIN);
    }
}
