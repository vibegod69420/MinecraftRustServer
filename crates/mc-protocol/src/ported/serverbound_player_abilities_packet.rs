use bytes::{Buf, BufMut};

pub const FLAG_FLYING: i8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Abilities {
    pub flying: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerAbilitiesPacket {
    is_flying: bool,
}

pub trait ServerGamePacketListener {
    fn handle_player_abilities(&mut self, packet: &ServerboundPlayerAbilitiesPacket);
}

impl ServerboundPlayerAbilitiesPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket#<init>(Abilities)
    pub fn new(abilities: &Abilities) -> Self {
        Self {
            is_flying: abilities.flying,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let b = buf.get_i8();
        let is_flying = (b & FLAG_FLYING) != 0;
        Self { is_flying }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket#write(FriendlyByteBuf)
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        let mut b: i8 = 0;
        if self.is_flying {
            b = (b | FLAG_FLYING) as i8;
        }
        buf.put_i8(b);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket#type()
    pub fn type_name(&self) -> &'static str {
        "SERVERBOUND_PLAYER_ABILITIES"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket#handle(ServerGamePacketListener)
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_player_abilities(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket#isFlying()
    pub fn is_flying(&self) -> bool {
        self.is_flying
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_flying_true() {
        let pkt = ServerboundPlayerAbilitiesPacket::new(&Abilities { flying: true });
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        assert_eq!(buf.as_ref(), &[2u8]);
        let decoded = ServerboundPlayerAbilitiesPacket::read(&mut buf.freeze());
        assert!(decoded.is_flying());
        assert_eq!(decoded.type_name(), "SERVERBOUND_PLAYER_ABILITIES");
    }

    #[test]
    fn roundtrip_flying_false() {
        let pkt = ServerboundPlayerAbilitiesPacket::new(&Abilities { flying: false });
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        assert_eq!(buf.as_ref(), &[0u8]);
        let decoded = ServerboundPlayerAbilitiesPacket::read(&mut buf.freeze());
        assert!(!decoded.is_flying());
    }

    #[test]
    fn read_ignores_other_bits() {
        let mut all_bits: &[u8] = &[0xFF];
        let decoded = ServerboundPlayerAbilitiesPacket::read(&mut all_bits);
        assert!(decoded.is_flying());

        let mut no_flying_bit: &[u8] = &[0xFD];
        let decoded = ServerboundPlayerAbilitiesPacket::read(&mut no_flying_bit);
        assert!(!decoded.is_flying());
    }

    #[test]
    fn read_negative_byte_preserves_flying_bit() {
        let mut data: &[u8] = &[0xFE]; // -2 as i8, bit 2 set
        let decoded = ServerboundPlayerAbilitiesPacket::read(&mut data);
        assert!(decoded.is_flying());
    }

    #[test]
    fn handle_dispatches_listener() {
        struct L {
            flying: Option<bool>,
        }
        impl ServerGamePacketListener for L {
            fn handle_player_abilities(&mut self, packet: &ServerboundPlayerAbilitiesPacket) {
                self.flying = Some(packet.is_flying());
            }
        }
        let pkt = ServerboundPlayerAbilitiesPacket::new(&Abilities { flying: true });
        let mut l = L { flying: None };
        pkt.handle(&mut l);
        assert_eq!(l.flying, Some(true));
    }
}
