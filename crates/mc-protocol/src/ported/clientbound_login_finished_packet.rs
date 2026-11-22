#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub i32);

pub struct LoginPacketTypes;

impl LoginPacketTypes {
    pub const CLIENTBOUND_LOGIN_FINISHED: PacketType = PacketType(1);
}

/// Minimal self-contained stub for `com.mojang.authlib.GameProfile`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfile {
    pub id: [u8; 16],
    pub name: String,
}

pub trait ClientLoginPacketListener {
    fn handle_login_finished(&mut self, packet: &ClientboundLoginFinishedPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnitStreamCodec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginFinishedPacket {
    game_profile: GameProfile,
}

impl ClientboundLoginFinishedPacket {
    pub const STREAM_CODEC: UnitStreamCodec = UnitStreamCodec;

    pub fn new(game_profile: GameProfile) -> Self {
        Self { game_profile }
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket#gameProfile
    pub fn game_profile(&self) -> &GameProfile {
        &self.game_profile
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket#type
    pub fn get_type(&self) -> PacketType {
        LoginPacketTypes::CLIENTBOUND_LOGIN_FINISHED
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket#handle
    pub fn handle(&self, listener: &mut impl ClientLoginPacketListener) {
        listener.handle_login_finished(self);
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket#isTerminal
    pub fn is_terminal(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        finished_name: Option<String>,
        count: i32,
    }

    impl ClientLoginPacketListener for MockListener {
        fn handle_login_finished(&mut self, packet: &ClientboundLoginFinishedPacket) {
            self.finished_name = Some(packet.game_profile().name.clone());
            self.count = self.count.wrapping_add(1);
        }
    }

    fn sample_profile() -> GameProfile {
        GameProfile {
            id: [1_u8; 16],
            name: "Steve".to_string(),
        }
    }

    #[test]
    fn game_profile_accessor_returns_profile() {
        let packet = ClientboundLoginFinishedPacket::new(sample_profile());
        assert_eq!(packet.game_profile().name, "Steve");
        assert_eq!(packet.game_profile().id, [1_u8; 16]);
    }

    #[test]
    fn type_and_terminal_match_java() {
        let packet = ClientboundLoginFinishedPacket::new(sample_profile());
        assert_eq!(
            packet.get_type(),
            LoginPacketTypes::CLIENTBOUND_LOGIN_FINISHED
        );
        assert!(packet.is_terminal());
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = MockListener {
            finished_name: None,
            count: 0,
        };
        let packet = ClientboundLoginFinishedPacket::new(sample_profile());
        packet.handle(&mut listener);
        assert_eq!(listener.finished_name.as_deref(), Some("Steve"));
        assert_eq!(listener.count, 1);
    }

    #[test]
    fn handle_count_wraps_on_overflow() {
        let mut listener = MockListener {
            finished_name: None,
            count: i32::MAX,
        };
        let packet = ClientboundLoginFinishedPacket::new(sample_profile());
        packet.handle(&mut listener);
        assert_eq!(listener.count, i32::MIN);
    }

    #[test]
    fn negative_id_byte_and_float_bits_edge_cases() {
        let mut profile = sample_profile();
        profile.id[0] = 0xFF;
        profile.id[15] = 0x80;
        let packet = ClientboundLoginFinishedPacket::new(profile);

        assert_eq!(packet.game_profile().id[0] as i8, -1);
        assert_eq!(packet.game_profile().id[15] as i8, -128);
        assert_eq!(f32::NEG_INFINITY.to_bits(), 0xFF80_0000_u32);
    }
}
