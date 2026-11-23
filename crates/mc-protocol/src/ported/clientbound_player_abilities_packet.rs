#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Abilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub mayfly: bool,
    pub instabuild: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

impl Abilities {
    // JAVA: net.minecraft.world.entity.player.Abilities#getFlyingSpeed
    pub fn get_flying_speed(&self) -> f32 {
        self.flying_speed
    }

    // JAVA: net.minecraft.world.entity.player.Abilities#getWalkingSpeed
    pub fn get_walking_speed(&self) -> f32 {
        self.walking_speed
    }
}

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn read_byte(&mut self) -> i8 {
        let v = self.data[self.read_pos] as i8;
        self.read_pos += 1;
        v
    }

    pub fn write_byte(&mut self, v: i8) {
        self.data.push(v as u8);
    }

    pub fn read_int(&mut self) -> i32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 4]);
        self.read_pos += 4;
        i32::from_be_bytes(bytes)
    }

    pub fn write_int(&mut self, v: i32) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    pub fn read_float(&mut self) -> f32 {
        f32::from_bits(self.read_int() as u32)
    }

    pub fn write_float(&mut self, v: f32) {
        self.write_int(v.to_bits() as i32);
    }
}

pub const FLAG_INVULNERABLE: i8 = 1;
pub const FLAG_FLYING: i8 = 2;
pub const FLAG_CAN_FLY: i8 = 4;
pub const FLAG_INSTABUILD: i8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ClientGamePacketListener {
    fn handle_player_abilities(&mut self, packet: &ClientboundPlayerAbilitiesPacket);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundPlayerAbilitiesPacket {
    invulnerable: bool,
    is_flying: bool,
    can_fly: bool,
    instabuild: bool,
    flying_speed: f32,
    walking_speed: f32,
}

impl ClientboundPlayerAbilitiesPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#<init>
    pub fn from_abilities(abilities: &Abilities) -> Self {
        Self {
            invulnerable: abilities.invulnerable,
            is_flying: abilities.flying,
            can_fly: abilities.mayfly,
            instabuild: abilities.instabuild,
            flying_speed: abilities.get_flying_speed(),
            walking_speed: abilities.get_walking_speed(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        let b = buf.read_byte();
        let invulnerable = (b & FLAG_INVULNERABLE) != 0;
        let is_flying = (b & FLAG_FLYING) != 0;
        let can_fly = (b & FLAG_CAN_FLY) != 0;
        let instabuild = (b & FLAG_INSTABUILD) != 0;
        let flying_speed = buf.read_float();
        let walking_speed = buf.read_float();
        Self {
            invulnerable,
            is_flying,
            can_fly,
            instabuild,
            flying_speed,
            walking_speed,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        let mut b: i8 = 0;
        if self.invulnerable {
            b |= FLAG_INVULNERABLE;
        }
        if self.is_flying {
            b |= FLAG_FLYING;
        }
        if self.can_fly {
            b |= FLAG_CAN_FLY;
        }
        if self.instabuild {
            b |= FLAG_INSTABUILD;
        }
        buf.write_byte(b);
        buf.write_float(self.flying_speed);
        buf.write_float(self.walking_speed);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:player_abilities")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_player_abilities(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#isInvulnerable
    pub fn is_invulnerable(&self) -> bool {
        self.invulnerable
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#isFlying
    pub fn is_flying(&self) -> bool {
        self.is_flying
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#canFly
    pub fn can_fly(&self) -> bool {
        self.can_fly
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#canInstabuild
    pub fn can_instabuild(&self) -> bool {
        self.instabuild
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#getFlyingSpeed
    pub fn get_flying_speed(&self) -> f32 {
        self.flying_speed
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket#getWalkingSpeed
    pub fn get_walking_speed(&self) -> f32 {
        self.walking_speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_roundtrip() {
        let abilities = Abilities {
            invulnerable: true,
            flying: false,
            mayfly: true,
            instabuild: true,
            flying_speed: 0.05,
            walking_speed: 0.1,
        };
        let pkt = ClientboundPlayerAbilitiesPacket::from_abilities(&abilities);
        assert!(pkt.is_invulnerable());
        assert!(!pkt.is_flying());
        assert!(pkt.can_fly());
        assert!(pkt.can_instabuild());

        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundPlayerAbilitiesPacket::from_buf(&mut read_buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn all_flags_set_and_float_bits() {
        let abilities = Abilities {
            invulnerable: true,
            flying: true,
            mayfly: true,
            instabuild: true,
            flying_speed: f32::from_bits(0x7F7F_FFFF),
            walking_speed: f32::from_bits(0xFF7F_FFFF),
        };
        let pkt = ClientboundPlayerAbilitiesPacket::from_abilities(&abilities);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        assert_eq!(buf.data[0], 0x0F);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundPlayerAbilitiesPacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_flying_speed().to_bits(), 0x7F7F_FFFF);
        assert_eq!(decoded.get_walking_speed().to_bits(), 0xFF7F_FFFF);
    }

    #[test]
    fn no_flags_and_negative_zero() {
        let abilities = Abilities {
            invulnerable: false,
            flying: false,
            mayfly: false,
            instabuild: false,
            flying_speed: 0.0,
            walking_speed: -0.0,
        };
        let pkt = ClientboundPlayerAbilitiesPacket::from_abilities(&abilities);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        assert_eq!(buf.data[0], 0x00);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundPlayerAbilitiesPacket::from_buf(&mut read_buf);
        assert_eq!(decoded, pkt);
        assert_eq!(decoded.get_walking_speed().to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn negative_speed_and_nan_bits() {
        let pkt = ClientboundPlayerAbilitiesPacket::from_abilities(&Abilities {
            invulnerable: false,
            flying: true,
            mayfly: false,
            instabuild: false,
            flying_speed: -1.5,
            walking_speed: f32::NAN,
        });
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundPlayerAbilitiesPacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_flying_speed().to_bits(), (-1.5f32).to_bits());
        assert!(decoded.get_walking_speed().is_nan());
        assert!(decoded.is_flying());
        assert!(!decoded.can_fly());
    }
}
