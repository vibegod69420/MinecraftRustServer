#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

pub const CLIENTBOUND_PLAYER_COMBAT_KILL: PacketType = PacketType {
    name: "minecraft:player_combat_kill",
};

pub trait ClientGamePacketListener {
    fn handle_player_combat_kill(&mut self, packet: &ClientboundPlayerCombatKillPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerCombatKillPacket {
    player_id: i32,
    message: Component,
}

impl ClientboundPlayerCombatKillPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatKillPacket#<init>
    pub fn new(player_id: i32, message: Component) -> Self {
        Self { player_id, message }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatKillPacket#playerId
    pub fn player_id(&self) -> i32 {
        self.player_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatKillPacket#message
    pub fn message(&self) -> &Component {
        &self.message
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatKillPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_PLAYER_COMBAT_KILL
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatKillPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_player_combat_kill(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatKillPacket#isSkippable
    pub fn is_skippable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        player_id: i32,
        message: String,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_player_combat_kill(&mut self, packet: &ClientboundPlayerCombatKillPacket) {
            self.called = true;
            self.player_id = packet.player_id();
            self.message = packet.message().text.clone();
        }
    }

    #[test]
    fn test_new_and_accessors() {
        let pkt = ClientboundPlayerCombatKillPacket::new(7, Component::literal("died"));
        assert_eq!(pkt.player_id(), 7);
        assert_eq!(pkt.message().text, "died");
    }

    #[test]
    fn test_negative_player_id() {
        let pkt = ClientboundPlayerCombatKillPacket::new(-1, Component::literal("x"));
        assert_eq!(pkt.player_id(), -1);
    }

    #[test]
    fn test_extreme_player_ids() {
        let min_pkt =
            ClientboundPlayerCombatKillPacket::new(i32::MIN, Component::literal("min"));
        let max_pkt =
            ClientboundPlayerCombatKillPacket::new(i32::MAX, Component::literal("max"));
        assert_eq!(min_pkt.player_id(), i32::MIN);
        assert_eq!(max_pkt.player_id(), i32::MAX);
    }

    #[test]
    fn test_is_skippable_and_type() {
        let pkt = ClientboundPlayerCombatKillPacket::new(0, Component::literal(""));
        assert!(pkt.is_skippable());
        assert_eq!(pkt.get_type(), CLIENTBOUND_PLAYER_COMBAT_KILL);
    }

    #[test]
    fn test_handle_dispatch() {
        let pkt =
            ClientboundPlayerCombatKillPacket::new(42, Component::literal("farewell"));
        let mut listener = MockListener {
            called: false,
            player_id: 0,
            message: String::new(),
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.player_id, 42);
        assert_eq!(listener.message, "farewell");
    }
}
