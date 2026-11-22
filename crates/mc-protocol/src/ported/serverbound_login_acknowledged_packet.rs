#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub i32);

pub struct LoginPacketTypes;

impl LoginPacketTypes {
    pub const SERVERBOUND_LOGIN_ACKNOWLEDGED: PacketType = PacketType(0);
}

pub trait ServerLoginPacketListener {
    fn handle_login_acknowledgement(&mut self, packet: &ServerboundLoginAcknowledgedPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnitStreamCodec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundLoginAcknowledgedPacket(());

impl ServerboundLoginAcknowledgedPacket {
    pub const INSTANCE: Self = Self(());
    pub const STREAM_CODEC: UnitStreamCodec = UnitStreamCodec;

    // JAVA: net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket#type
    pub fn get_type(&self) -> PacketType {
        LoginPacketTypes::SERVERBOUND_LOGIN_ACKNOWLEDGED
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket#handle
    pub fn handle(&self, listener: &mut impl ServerLoginPacketListener) {
        listener.handle_login_acknowledgement(self);
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket#isTerminal
    pub fn is_terminal(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        ack_count: i32,
        last_was_instance: bool,
    }

    impl ServerLoginPacketListener for MockListener {
        fn handle_login_acknowledgement(&mut self, packet: &ServerboundLoginAcknowledgedPacket) {
            self.ack_count = self.ack_count.wrapping_add(1);
            self.last_was_instance =
                std::ptr::eq(packet, &ServerboundLoginAcknowledgedPacket::INSTANCE)
                    || *packet == ServerboundLoginAcknowledgedPacket::INSTANCE;
        }
    }

    #[test]
    fn instance_has_expected_type() {
        let packet = ServerboundLoginAcknowledgedPacket::INSTANCE;
        assert_eq!(
            packet.get_type(),
            LoginPacketTypes::SERVERBOUND_LOGIN_ACKNOWLEDGED
        );
    }

    #[test]
    fn packet_is_terminal() {
        assert!(ServerboundLoginAcknowledgedPacket::INSTANCE.is_terminal());
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = MockListener {
            ack_count: 0,
            last_was_instance: false,
        };
        ServerboundLoginAcknowledgedPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.ack_count, 1);
        assert!(listener.last_was_instance);
    }

    #[test]
    fn handle_count_wraps_on_overflow() {
        let mut listener = MockListener {
            ack_count: i32::MAX,
            last_was_instance: false,
        };
        ServerboundLoginAcknowledgedPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.ack_count, i32::MIN);
    }

    #[test]
    fn negative_packet_type_and_float_bits_edge_cases() {
        let packet_type = PacketType(-1);
        assert_eq!(packet_type.0.wrapping_sub(1), -2);
        assert_eq!(f64::NEG_INFINITY.to_bits(), 0xfff0_0000_0000_0000_u64);
    }
}
