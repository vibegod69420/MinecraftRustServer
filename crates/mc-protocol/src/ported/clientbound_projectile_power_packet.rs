use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("unexpected end of buffer")]
    UnexpectedEof,
    #[error("varint too big")]
    VarIntTooBig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType {
    pub id: &'static str,
}

pub mod game_packet_types {
    use super::PacketType;

    pub const CLIENTBOUND_PROJECTILE_POWER: PacketType = PacketType {
        id: "minecraft:clientbound_projectile_power",
    };
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_index: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self {
            data,
            read_index: 0,
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_index)
    }

    fn read_u8(&mut self) -> Result<u8, ProtocolError> {
        if self.read_index >= self.data.len() {
            return Err(ProtocolError::UnexpectedEof);
        }
        let value = self.data[self.read_index];
        self.read_index = self.read_index.wrapping_add(1);
        Ok(value)
    }

    pub fn read_var_int(&mut self) -> Result<i32, ProtocolError> {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b = self.read_u8()? as i32;
            i |= (b & 127).wrapping_shl(((j.wrapping_mul(7)) & 31) as u32);
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(ProtocolError::VarIntTooBig);
            }
            if (b & 128) != 128 {
                break;
            }
        }
        Ok(i)
    }

    pub fn write_var_int(&mut self, mut value: i32) {
        while (value & -128i32) != 0 {
            self.data.push(((value & 127) | 128) as u8);
            value = ((value as u32) >> 7) as i32;
        }
        self.data.push(value as u8);
    }

    pub fn read_double(&mut self) -> Result<f64, ProtocolError> {
        if self.remaining() < 8 {
            return Err(ProtocolError::UnexpectedEof);
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.read_index..self.read_index + 8]);
        self.read_index = self.read_index.wrapping_add(8);
        Ok(f64::from_be_bytes(bytes))
    }

    pub fn write_double(&mut self, value: f64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }
}

pub trait ClientGamePacketListener {
    fn handle_projectile_power_packet(&mut self, packet: &ClientboundProjectilePowerPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundProjectilePowerPacket {
    id: i32,
    acceleration_power: f64,
}

impl ClientboundProjectilePowerPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket#<init>
    pub fn new(id: i32, acceleration_power: f64) -> Self {
        Self {
            id,
            acceleration_power,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, ProtocolError> {
        let id = buf.read_var_int()?;
        let acceleration_power = buf.read_double()?;
        Ok(Self {
            id,
            acceleration_power,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_double(self.acceleration_power);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket#type
    pub fn packet_type(&self) -> PacketType {
        game_packet_types::CLIENTBOUND_PROJECTILE_POWER
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_projectile_power_packet(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket#getAccelerationPower
    pub fn get_acceleration_power(&self) -> f64 {
        self.acceleration_power
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_type_and_handle() {
        struct MockListener {
            seen: Option<(i32, u64)>,
        }

        impl ClientGamePacketListener for MockListener {
            fn handle_projectile_power_packet(
                &mut self,
                packet: &ClientboundProjectilePowerPacket,
            ) {
                self.seen = Some((packet.get_id(), packet.get_acceleration_power().to_bits()));
            }
        }

        let packet = ClientboundProjectilePowerPacket::new(42, 1.5);
        assert_eq!(
            packet.packet_type(),
            game_packet_types::CLIENTBOUND_PROJECTILE_POWER
        );

        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundProjectilePowerPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded, packet);

        let mut listener = MockListener { seen: None };
        decoded.handle(&mut listener);
        assert_eq!(listener.seen, Some((42, 1.5f64.to_bits())));
    }

    #[test]
    fn negative_and_extreme_integers() {
        let cases = [0i32, -1, i32::MIN, i32::MAX, -123_456, 123_456];
        for id in cases {
            let packet = ClientboundProjectilePowerPacket::new(id, -0.25);
            let mut buf = FriendlyByteBuf::new();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
            let decoded = ClientboundProjectilePowerPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.get_id(), id);
            assert_eq!(
                decoded.get_acceleration_power().to_bits(),
                (-0.25f64).to_bits()
            );
        }
    }

    #[test]
    fn float_special_bits_round_trip() {
        let values: [f64; 6] = [
            0.0,
            -0.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::from_bits(0x7FF8_0000_0000_0001),
        ];
        for value in values {
            let packet = ClientboundProjectilePowerPacket::new(7, value);
            let mut buf = FriendlyByteBuf::new();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
            let decoded = ClientboundProjectilePowerPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.get_acceleration_power().to_bits(), value.to_bits());
        }
    }

    #[test]
    fn wire_format_encoding() {
        let packet = ClientboundProjectilePowerPacket::new(1, 1.0);
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut expected = Vec::new();
        expected.push(0x01); // VarInt 1
        expected.extend_from_slice(&1.0f64.to_be_bytes());
        assert_eq!(buf.as_slice(), expected.as_slice());

        let packet_neg = ClientboundProjectilePowerPacket::new(-1, 0.0);
        let mut buf_neg = FriendlyByteBuf::new();
        packet_neg.write(&mut buf_neg);
        let bytes = buf_neg.as_slice();
        assert_eq!(&bytes[..5], &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        assert_eq!(&bytes[5..], &0.0f64.to_be_bytes());
    }

    #[test]
    fn read_errors_on_underflow() {
        let mut empty = FriendlyByteBuf::from_vec(Vec::new());
        assert_eq!(
            ClientboundProjectilePowerPacket::read(&mut empty),
            Err(ProtocolError::UnexpectedEof)
        );

        let mut partial_buf = FriendlyByteBuf::new();
        partial_buf.write_var_int(5);
        let mut partial = FriendlyByteBuf::from_vec(partial_buf.into_vec());
        assert_eq!(
            ClientboundProjectilePowerPacket::read(&mut partial),
            Err(ProtocolError::UnexpectedEof)
        );
    }
}
