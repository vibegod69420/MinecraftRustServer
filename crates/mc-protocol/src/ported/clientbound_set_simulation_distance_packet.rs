use bytes::{BufMut, BytesMut};
use std::io;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    buf: BytesMut,
    read_pos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self {
            buf: BytesMut::from(&data[..]),
            read_pos: 0,
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf.to_vec()
    }

    fn read_byte(&mut self) -> io::Result<i8> {
        if self.read_pos >= self.buf.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.buf[self.read_pos] as i8;
        self.read_pos += 1;
        Ok(b)
    }

    fn write_byte(&mut self, value: i8) {
        self.buf.put_i8(value);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_byte()?;
            i |= ((b0 as i32) & 127) << j.wrapping_mul(7);
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if ((b0 as i32) & 128) != 128 {
                break;
            }
        }
        Ok(i)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & -128) != 0 {
                self.write_byte(((value & 127) | 128) as i8);
                value = ((value as u32) >> 7) as i32;
            } else {
                self.write_byte(value as i8);
                break;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType<T>(PhantomData<T>);

pub trait ClientGamePacketListener {
    fn handle_set_simulation_distance(
        &mut self,
        packet: &ClientboundSetSimulationDistancePacket,
    );
}

pub const CLIENTBOUND_SET_SIMULATION_DISTANCE: PacketType<ClientboundSetSimulationDistancePacket> =
    PacketType(PhantomData);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetSimulationDistancePacket {
    pub simulation_distance: i32,
}

impl ClientboundSetSimulationDistancePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetSimulationDistancePacket#<init>
    pub fn new(simulation_distance: i32) -> Self {
        Self { simulation_distance }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetSimulationDistancePacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        Ok(Self::new(buf.read_var_int()?))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetSimulationDistancePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.simulation_distance);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetSimulationDistancePacket#type
    pub fn get_type(&self) -> PacketType<Self> {
        CLIENTBOUND_SET_SIMULATION_DISTANCE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetSimulationDistancePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_set_simulation_distance(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_simulation_distance() {
        let packet = ClientboundSetSimulationDistancePacket::new(12);
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundSetSimulationDistancePacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn roundtrip_negative_and_extremes() {
        for value in [0, -1, i32::MIN, i32::MAX, 128, -128] {
            let packet = ClientboundSetSimulationDistancePacket::new(value);
            let mut buf = FriendlyByteBuf::new();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
            let decoded = ClientboundSetSimulationDistancePacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.simulation_distance, value);
        }
    }

    #[test]
    fn varint_encoding_known_bytes() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(0);
        assert_eq!(buf.into_vec(), vec![0]);

        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(1);
        assert_eq!(buf.into_vec(), vec![1]);

        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(128);
        assert_eq!(buf.into_vec(), vec![0x80, 0x01]);
    }

    #[test]
    fn varint_negative_one_encoding() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        let bytes = buf.into_vec();
        assert_eq!(bytes, vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        let mut read_buf = FriendlyByteBuf::from_vec(bytes);
        assert_eq!(read_buf.read_var_int().unwrap(), -1);
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener {
            seen: Option<i32>,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_set_simulation_distance(
                &mut self,
                packet: &ClientboundSetSimulationDistancePacket,
            ) {
                self.seen = Some(packet.simulation_distance);
            }
        }
        let packet = ClientboundSetSimulationDistancePacket::new(7);
        let mut listener = MockListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen, Some(7));
    }
}
