use bytes::{Buf, BufMut};

#[derive(Debug)]
pub struct FriendlyByteBuf<B> {
    pub inner: B,
}

impl<B: Buf> FriendlyByteBuf<B> {
    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b = self.inner.get_u8() as i8;
            i |= ((b & 127) as i32) << ((j.wrapping_mul(7)) & 31);
            j = j.wrapping_add(1);
            if j > 5 {
                panic!("VarInt too big");
            }
            if (b & -128i8) != -128i8 {
                break;
            }
        }
        i
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarIntArray
    pub fn read_var_int_array(&mut self) -> Vec<i32> {
        let len = self.read_var_int() as usize;
        let mut arr = Vec::with_capacity(len);
        for _ in 0..len {
            arr.push(self.read_var_int());
        }
        arr
    }
}

impl<B: BufMut> FriendlyByteBuf<B> {
    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value & !0x7F) == 0 {
                self.inner.put_u8(value as u8);
                return self;
            }
            self.inner.put_u8(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarIntArray
    pub fn write_var_int_array(&mut self, array: &[i32]) -> &mut Self {
        self.write_var_int(array.len() as i32);
        for &v in array {
            self.write_var_int(v);
        }
        self
    }
}

pub trait PassengerEntity {
    fn get_id(&self) -> i32;
    fn get_passenger_ids(&self) -> Vec<i32>;
}

pub trait ClientGamePacketListener {
    fn handle_set_entity_passengers_packet(&mut self, packet: &ClientboundSetPassengersPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetPassengersPacket {
    vehicle: i32,
    passengers: Vec<i32>,
}

impl ClientboundSetPassengersPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPassengersPacket#<init>(Entity)
    pub fn from_entity<E: PassengerEntity>(entity: &E) -> Self {
        Self {
            vehicle: entity.get_id(),
            passengers: entity.get_passenger_ids(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPassengersPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut FriendlyByteBuf<B>) -> Self {
        let vehicle = buf.read_var_int();
        let passengers = buf.read_var_int_array();
        Self { vehicle, passengers }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPassengersPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut FriendlyByteBuf<B>) {
        buf.write_var_int(self.vehicle);
        buf.write_var_int_array(&self.passengers);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPassengersPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_set_entity_passengers_packet(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPassengersPacket#getPassengers
    pub fn get_passengers(&self) -> &[i32] {
        &self.passengers
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPassengersPacket#getVehicle
    pub fn get_vehicle(&self) -> i32 {
        self.vehicle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockEntity {
        id: i32,
        passengers: Vec<i32>,
    }

    impl PassengerEntity for MockEntity {
        fn get_id(&self) -> i32 {
            self.id
        }

        fn get_passenger_ids(&self) -> Vec<i32> {
            self.passengers.clone()
        }
    }

    struct MockListener(Option<(i32, Vec<i32>)>);
    impl ClientGamePacketListener for MockListener {
        fn handle_set_entity_passengers_packet(&mut self, packet: &ClientboundSetPassengersPacket) {
            self.0 = Some((packet.get_vehicle(), packet.get_passengers().to_vec()));
        }
    }

    #[test]
    fn round_trip() {
        let pkt = ClientboundSetPassengersPacket::from_entity(&MockEntity {
            id: 10,
            passengers: vec![20, 30, 40],
        });
        let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
        let decoded = ClientboundSetPassengersPacket::read(&mut read_buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn empty_passengers() {
        let pkt = ClientboundSetPassengersPacket::from_entity(&MockEntity {
            id: 1,
            passengers: vec![],
        });
        let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
        let decoded = ClientboundSetPassengersPacket::read(&mut read_buf);
        assert_eq!(decoded.get_vehicle(), 1);
        assert!(decoded.get_passengers().is_empty());
    }

    #[test]
    fn negative_and_extreme_ids() {
        let pkt = ClientboundSetPassengersPacket::from_entity(&MockEntity {
            id: -1,
            passengers: vec![i32::MIN, -7, 0, i32::MAX],
        });
        let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
        let decoded = ClientboundSetPassengersPacket::read(&mut read_buf);
        assert_eq!(decoded.get_vehicle(), -1);
        assert_eq!(decoded.get_passengers(), &[i32::MIN, -7, 0, i32::MAX]);
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ClientboundSetPassengersPacket::from_entity(&MockEntity {
            id: 5,
            passengers: vec![6, 7],
        });
        let mut listener = MockListener(None);
        pkt.handle(&mut listener);
        assert_eq!(listener.0, Some((5, vec![6, 7])));
    }
}
