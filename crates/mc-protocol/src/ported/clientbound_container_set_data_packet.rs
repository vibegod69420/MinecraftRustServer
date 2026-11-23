use bytes::{Buf, BufMut};

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int<B: Buf>(buf: &mut B) -> i32 {
    let mut val: u32 = 0;
    let mut pos: u32 = 0;
    loop {
        let b = buf.get_u8();
        val |= ((b & 0x7F) as u32).wrapping_shl(pos.wrapping_mul(7));
        pos = pos.wrapping_add(1);
        if pos > 5 {
            panic!("VarInt too big");
        }
        if (b & 0x80) == 0 {
            break;
        }
    }
    val as i32
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value as u32 & !0x7F_u32) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value as u8) & 0x7F) | 0x80);
        value = ((value as u32) >> 7) as i32;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readContainerId
fn read_container_id<B: Buf>(buf: &mut B) -> i32 {
    read_var_int(buf)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeContainerId
fn write_container_id<B: BufMut>(buf: &mut B, value: i32) {
    write_var_int(buf, value);
}

pub trait ClientGamePacketListener {
    fn handle_container_set_data(&mut self, packet: &ClientboundContainerSetDataPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundContainerSetDataPacket {
    container_id: i32,
    id: i32,
    value: i32,
}

impl ClientboundContainerSetDataPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket#<init>(int, int, int)
    pub fn new(container_id: i32, id: i32, value: i32) -> Self {
        Self {
            container_id,
            id,
            value,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let container_id = read_container_id(buf);
        let id = buf.get_i16() as i32;
        let value = buf.get_i16() as i32;
        Self {
            container_id,
            id,
            value,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_container_id(buf, self.container_id);
        buf.put_i16(self.id as i16);
        buf.put_i16(self.value as i16);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:container_set_data"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_container_set_data(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket#getContainerId
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket#getValue
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn write_read_roundtrip() {
        let packet = ClientboundContainerSetDataPacket::new(1, 2, 3);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundContainerSetDataPacket::read(&mut reader);
        assert_eq!(packet, decoded);
    }

    #[test]
    fn short_sign_extension() {
        let packet = ClientboundContainerSetDataPacket::new(0, -1, -32768);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundContainerSetDataPacket::read(&mut reader);
        assert_eq!(decoded.get_id(), -1);
        assert_eq!(decoded.get_value(), -32768);
    }

    #[test]
    fn short_truncation_on_write() {
        let packet = ClientboundContainerSetDataPacket::new(5, 0x1_0001, -32769);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundContainerSetDataPacket::read(&mut reader);
        assert_eq!(decoded.get_id(), 1);
        assert_eq!(decoded.get_value(), 32767);
    }

    #[test]
    fn negative_container_id_roundtrip() {
        let packet = ClientboundContainerSetDataPacket::new(-1, 0, 0);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundContainerSetDataPacket::read(&mut reader);
        assert_eq!(decoded.get_container_id(), -1);
    }

    #[test]
    fn handle_calls_listener() {
        struct L {
            cid: i32,
            id: i32,
            val: i32,
        }
        impl ClientGamePacketListener for L {
            fn handle_container_set_data(&mut self, packet: &ClientboundContainerSetDataPacket) {
                self.cid = packet.get_container_id();
                self.id = packet.get_id();
                self.val = packet.get_value();
            }
        }
        let packet = ClientboundContainerSetDataPacket::new(3, 4, 5);
        let mut l = L { cid: 0, id: 0, val: 0 };
        packet.handle(&mut l);
        assert_eq!((l.cid, l.id, l.val), (3, 4, 5));
    }
}
