use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PacketType(pub &'static str);

pub mod game_packet_types {
    use super::PacketType;

    pub const SERVERBOUND_CONTAINER_BUTTON_CLICK: PacketType =
        PacketType("serverbound_container_button_click");
}

pub trait ServerGamePacketListener {
    fn handle_container_button_click(&mut self, packet: &ServerboundContainerButtonClickPacket);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
pub fn read_var_int(buf: &mut impl Buf) -> i32 {
    let mut i: i32 = 0;
    let mut j: u32 = 0;
    loop {
        let b0 = buf.get_u8();
        i |= ((b0 & 0x7F) as i32) << (j.wrapping_mul(7) & 31);
        j = j.wrapping_add(1);
        if j > 5 {
            panic!("VarInt too big");
        }
        if (b0 & 0x80) == 0 {
            return i;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
pub fn write_var_int(buf: &mut impl BufMut, mut value: i32) {
    loop {
        if (value & !0x7F) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundContainerButtonClickPacket {
    pub container_id: i32,
    pub button_id: i32,
}

impl ServerboundContainerButtonClickPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerButtonClickPacket#<init>
    pub fn new(container_id: i32, button_id: i32) -> Self {
        Self {
            container_id,
            button_id,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerButtonClickPacket#read
    pub fn read(buf: &mut impl Buf) -> Self {
        let container_id = read_var_int(buf);
        let button_id = read_var_int(buf);
        Self {
            container_id,
            button_id,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerButtonClickPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.container_id);
        write_var_int(buf, self.button_id);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerButtonClickPacket#type
    pub fn packet_type(&self) -> PacketType {
        game_packet_types::SERVERBOUND_CONTAINER_BUTTON_CLICK
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerButtonClickPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_container_button_click(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_basic() {
        let pkt = ServerboundContainerButtonClickPacket::new(1, 2);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundContainerButtonClickPacket::read(&mut buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_negative_and_extreme() {
        let cases = [
            ServerboundContainerButtonClickPacket::new(-1, i32::MIN),
            ServerboundContainerButtonClickPacket::new(i32::MAX, -123_456),
            ServerboundContainerButtonClickPacket::new(0, 0),
        ];
        for pkt in cases {
            let mut buf = BytesMut::new();
            pkt.write(&mut buf);
            let decoded = ServerboundContainerButtonClickPacket::read(&mut buf);
            assert_eq!(pkt, decoded);
        }
    }

    #[test]
    fn varint_wire_format_and_edges() {
        let mut buf = BytesMut::new();
        write_var_int(&mut buf, -1);
        assert_eq!(&buf[..], &[0xff, 0xff, 0xff, 0xff, 0x0f]);
        let decoded = read_var_int(&mut buf);
        assert_eq!(decoded, -1);

        let mut buf2 = BytesMut::new();
        write_var_int(&mut buf2, 128);
        assert_eq!(&buf2[..], &[0x80, 0x01]);
        let decoded2 = read_var_int(&mut buf2);
        assert_eq!(decoded2, 128);

        for val in [0i32, 127, 255, i32::MIN, i32::MAX] {
            let mut b = BytesMut::new();
            write_var_int(&mut b, val);
            assert_eq!(read_var_int(&mut b), val);
        }
    }

    #[test]
    fn packet_type_and_handle() {
        struct MockListener {
            id: i32,
            btn: i32,
        }
        impl ServerGamePacketListener for MockListener {
            fn handle_container_button_click(
                &mut self,
                packet: &ServerboundContainerButtonClickPacket,
            ) {
                self.id = packet.container_id;
                self.btn = packet.button_id;
            }
        }

        let pkt = ServerboundContainerButtonClickPacket::new(5, 3);
        assert_eq!(
            pkt.packet_type(),
            game_packet_types::SERVERBOUND_CONTAINER_BUTTON_CLICK
        );
        let mut listener = MockListener { id: 0, btn: 0 };
        pkt.handle(&mut listener);
        assert_eq!(listener.id, 5);
        assert_eq!(listener.btn, 3);
    }
}
