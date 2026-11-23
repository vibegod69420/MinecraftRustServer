#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

pub const GAME_PACKET_TYPES_CLIENTBOUND_SET_HEALTH: PacketType = PacketType {
    name: "minecraft:set_health",
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_index: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            read_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_index)
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.data[self.read_index];
        self.read_index += 1;
        b
    }

    fn write_byte(&mut self, value: u8) {
        self.data.push(value);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readFloat()
    pub fn read_float(&mut self) -> f32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.data[self.read_index..self.read_index + 4]);
        self.read_index += 4;
        f32::from_be_bytes(bytes)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeFloat(float)
    pub fn write_float(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_byte();
            value |= ((current_byte & 0x7F) as i32) << position;
            if (current_byte & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                panic!("VarInt too big");
            }
        }
        value
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F) == 0 {
                self.write_byte(value as u8);
                return;
            }
            self.write_byte(((value & 0x7F) as u8) | 0x80);
            value = ((value as u32) >> 7) as i32;
        }
    }
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleSetHealth(ClientboundSetHealthPacket)
    fn handle_set_health(&mut self, packet: &ClientboundSetHealthPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetHealthPacket {
    health: f32,
    food: i32,
    saturation: f32,
}

impl ClientboundSetHealthPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetHealthPacket#<init>(float, int, float)
    pub fn new(health: f32, food: i32, saturation: f32) -> Self {
        Self {
            health,
            food,
            saturation,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetHealthPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let health = buf.read_float();
        let food = buf.read_var_int();
        let saturation = buf.read_float();
        Self {
            health,
            food,
            saturation,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetHealthPacket#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_float(self.health);
        buf.write_var_int(self.food);
        buf.write_float(self.saturation);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetHealthPacket#type()
    pub fn type_(&self) -> PacketType {
        GAME_PACKET_TYPES_CLIENTBOUND_SET_HEALTH
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetHealthPacket#handle(ClientGamePacketListener)
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_set_health(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetHealthPacket#getHealth()
    pub fn get_health(&self) -> f32 {
        self.health
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetHealthPacket#getFood()
    pub fn get_food(&self) -> i32 {
        self.food
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetHealthPacket#getSaturation()
    pub fn get_saturation(&self) -> f32 {
        self.saturation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_normal_values() {
        let packet = ClientboundSetHealthPacket::new(20.0_f32, 20, 5.0_f32);
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundSetHealthPacket::read(&mut read_buf);
        assert_eq!(decoded.get_health().to_bits(), 20.0_f32.to_bits());
        assert_eq!(decoded.get_food(), 20);
        assert_eq!(decoded.get_saturation().to_bits(), 5.0_f32.to_bits());
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_negative_and_extreme_float_bits() {
        let values = [
            (0.0_f32, 0_i32, 0.0_f32),
            (-1.0_f32, -1_i32, -0.0_f32),
            (f32::NAN, i32::MIN, f32::INFINITY),
            (f32::NEG_INFINITY, i32::MAX, f32::MIN_POSITIVE),
        ];
        for (health, food, saturation) in values {
            let packet = ClientboundSetHealthPacket::new(health, food, saturation);
            let mut buf = FriendlyByteBuf::default();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ClientboundSetHealthPacket::read(&mut read_buf);
            assert_eq!(decoded.get_health().to_bits(), health.to_bits());
            assert_eq!(decoded.get_food(), food);
            assert_eq!(decoded.get_saturation().to_bits(), saturation.to_bits());
            assert_eq!(read_buf.remaining(), 0);
        }
    }

    #[test]
    fn var_int_round_trip_edge_cases() {
        let mut buf = FriendlyByteBuf::default();
        let values = [0_i32, 1, -1, 127, 128, i32::MAX, i32::MIN];
        for &v in &values {
            buf.write_var_int(v);
        }
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        for &v in &values {
            assert_eq!(read_buf.read_var_int(), v);
        }
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn handle_calls_listener_and_type_matches() {
        struct TestListener {
            called: bool,
            health: f32,
            food: i32,
            saturation: f32,
        }
        impl ClientGamePacketListener for TestListener {
            fn handle_set_health(&mut self, packet: &ClientboundSetHealthPacket) {
                self.called = true;
                self.health = packet.get_health();
                self.food = packet.get_food();
                self.saturation = packet.get_saturation();
            }
        }

        let packet = ClientboundSetHealthPacket::new(7.5_f32, 3, 1.25_f32);
        let mut listener = TestListener {
            called: false,
            health: 0.0,
            food: 0,
            saturation: 0.0,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.health.to_bits(), 7.5_f32.to_bits());
        assert_eq!(listener.food, 3);
        assert_eq!(listener.saturation.to_bits(), 1.25_f32.to_bits());
        assert_eq!(packet.type_(), GAME_PACKET_TYPES_CLIENTBOUND_SET_HEALTH);
    }
}
