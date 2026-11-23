use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketType {
    ClientboundHurtAnimation,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            read_pos: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_varint(&mut self) -> io::Result<i32> {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_u8()? as i32;
            let shift = j.wrapping_mul(7) & 31;
            i |= (b0 & 127) << shift;
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if (b0 & 128) == 0 {
                break;
            }
        }
        Ok(i)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_varint(&mut self, mut value: i32) -> io::Result<()> {
        loop {
            if (value & !0x7F) == 0 {
                self.data.push(value as u8);
                return Ok(());
            }
            self.data.push(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readInt
    pub fn read_int(&mut self) -> io::Result<i32> {
        if self.data.len().saturating_sub(self.read_pos) < 4 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 4]);
        self.read_pos += 4;
        Ok(i32::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeInt
    pub fn write_int(&mut self, value: i32) -> io::Result<()> {
        self.data.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readFloat
    pub fn read_float(&mut self) -> io::Result<f32> {
        let bits = self.read_int()? as u32;
        Ok(f32::from_bits(bits))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeFloat
    pub fn write_float(&mut self, value: f32) -> io::Result<()> {
        self.write_int(value.to_bits() as i32)
    }
}

pub trait LivingEntityLike {
    fn get_id(&self) -> i32;
    fn get_hurt_dir(&self) -> f32;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundHurtAnimationPacket {
    pub id: i32,
    pub yaw: f32,
}

pub trait ClientGamePacketListener {
    fn handle_hurt_animation(&mut self, packet: &ClientboundHurtAnimationPacket);
}

impl ClientboundHurtAnimationPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket#<init>
    pub fn new(id: i32, yaw: f32) -> Self {
        Self { id, yaw }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket#<init>(LivingEntity)
    pub fn from_living_entity(entity: &impl LivingEntityLike) -> Self {
        Self::new(entity.get_id(), entity.get_hurt_dir())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket#id
    pub fn id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket#yaw
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let id = buf.read_varint()?;
        let yaw = buf.read_float()?;
        Ok(Self::new(id, yaw))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_varint(self.id)?;
        buf.write_float(self.yaw)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ClientboundHurtAnimation
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_hurt_animation(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let pkt = ClientboundHurtAnimationPacket::new(42, 90.5);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundHurtAnimationPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.id(), 42);
        assert_eq!(decoded.yaw().to_bits(), 90.5f32.to_bits());
    }

    #[test]
    fn negative_id_and_nan_bits_preserved() {
        let yaw = f32::from_bits(0x7FC0_0001);
        let pkt = ClientboundHurtAnimationPacket::new(-12345, yaw);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundHurtAnimationPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.id, -12345);
        assert_eq!(decoded.yaw.to_bits(), yaw.to_bits());
    }

    #[test]
    fn from_living_entity_constructor() {
        struct E;
        impl LivingEntityLike for E {
            fn get_id(&self) -> i32 {
                7
            }
            fn get_hurt_dir(&self) -> f32 {
                -180.0
            }
        }
        let pkt = ClientboundHurtAnimationPacket::from_living_entity(&E);
        assert_eq!(pkt.id, 7);
        assert_eq!(pkt.yaw.to_bits(), (-180.0f32).to_bits());
    }

    #[test]
    fn type_and_handle() {
        let pkt = ClientboundHurtAnimationPacket::new(1, 0.0);
        assert_eq!(pkt.packet_type(), PacketType::ClientboundHurtAnimation);

        struct Listener(Option<ClientboundHurtAnimationPacket>);
        impl ClientGamePacketListener for Listener {
            fn handle_hurt_animation(&mut self, packet: &ClientboundHurtAnimationPacket) {
                self.0 = Some(*packet);
            }
        }
        let mut listener = Listener(None);
        pkt.handle(&mut listener);
        assert_eq!(listener.0, Some(pkt));
    }
}
