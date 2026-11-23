use bytes::{Buf, BufMut};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PacketDecodeError {
    #[error("unexpected end of buffer")]
    UnexpectedEof,
    #[error("varint too big")]
    VarIntTooBig,
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int<B: Buf>(buf: &mut B) -> Result<i32, PacketDecodeError> {
    let mut value: i32 = 0;
    let mut position: i32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(PacketDecodeError::UnexpectedEof);
        }
        let b0 = buf.get_u8() as i8;
        value |= ((b0 & 127) as i32) << (position * 7);
        position += 1;
        if position > 5 {
            return Err(PacketDecodeError::VarIntTooBig);
        }
        if (b0 & -128i8) == 0 {
            break;
        }
    }
    Ok(value)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value & -128i32) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value & 127) | 128) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readContainerId
fn read_container_id<B: Buf>(buf: &mut B) -> Result<i32, PacketDecodeError> {
    read_var_int(buf)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeContainerId
fn write_container_id<B: BufMut>(buf: &mut B, value: i32) {
    write_var_int(buf, value);
}

pub trait ClientGamePacketListener {
    fn handle_mount_screen_open(&mut self, packet: &ClientboundMountScreenOpenPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundMountScreenOpenPacket {
    container_id: i32,
    inventory_columns: i32,
    entity_id: i32,
}

impl ClientboundMountScreenOpenPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundMountScreenOpenPacket#<init>
    pub fn new(container_id: i32, inventory_columns: i32, entity_id: i32) -> Self {
        Self {
            container_id,
            inventory_columns,
            entity_id,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMountScreenOpenPacket#<init>
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, PacketDecodeError> {
        let container_id = read_container_id(buf)?;
        let inventory_columns = read_var_int(buf)?;
        if buf.remaining() < 4 {
            return Err(PacketDecodeError::UnexpectedEof);
        }
        let entity_id = buf.get_i32();
        Ok(Self {
            container_id,
            inventory_columns,
            entity_id,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMountScreenOpenPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_container_id(buf, self.container_id);
        write_var_int(buf, self.inventory_columns);
        buf.put_i32(self.entity_id);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMountScreenOpenPacket#type
    pub fn type_name(&self) -> &'static str {
        "minecraft:mount_screen_open"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMountScreenOpenPacket#handle
    pub fn handle<H: ClientGamePacketListener>(&self, handler: &mut H) {
        handler.handle_mount_screen_open(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMountScreenOpenPacket#getContainerId
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMountScreenOpenPacket#getInventoryColumns
    pub fn get_inventory_columns(&self) -> i32 {
        self.inventory_columns
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMountScreenOpenPacket#getEntityId
    pub fn get_entity_id(&self) -> i32 {
        self.entity_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockHandler {
        called: bool,
        container_id: i32,
    }

    impl ClientGamePacketListener for MockHandler {
        fn handle_mount_screen_open(&mut self, packet: &ClientboundMountScreenOpenPacket) {
            self.called = true;
            self.container_id = packet.get_container_id();
        }
    }

    #[test]
    fn roundtrip_normal() {
        let pkt = ClientboundMountScreenOpenPacket::new(42, 9, 12345);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundMountScreenOpenPacket::read(&mut buf.freeze()).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_container_id(), 42);
        assert_eq!(decoded.get_inventory_columns(), 9);
        assert_eq!(decoded.get_entity_id(), 12345);
    }

    #[test]
    fn roundtrip_negative_and_extreme_ints() {
        let pkt = ClientboundMountScreenOpenPacket::new(-1, 0, i32::MIN);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundMountScreenOpenPacket::read(&mut buf.freeze()).unwrap();
        assert_eq!(pkt, decoded);

        let pkt2 = ClientboundMountScreenOpenPacket::new(i32::MAX, i32::MAX, i32::MAX);
        let mut buf2 = BytesMut::new();
        pkt2.write(&mut buf2);
        let decoded2 = ClientboundMountScreenOpenPacket::read(&mut buf2.freeze()).unwrap();
        assert_eq!(pkt2, decoded2);
    }

    #[test]
    fn varint_encoding_matches_java() {
        let mut buf = BytesMut::new();
        write_var_int(&mut buf, 300);
        assert_eq!(&buf[..], &[0xAC, 0x02]);
        let mut read_buf = buf.freeze();
        assert_eq!(read_var_int(&mut read_buf).unwrap(), 300);

        let mut buf_neg = BytesMut::new();
        write_var_int(&mut buf_neg, -1);
        assert_eq!(buf_neg.len(), 5);
        let mut read_neg = buf_neg.freeze();
        assert_eq!(read_var_int(&mut read_neg).unwrap(), -1);
    }

    #[test]
    fn decode_eof_and_handle() {
        let mut empty = bytes::Bytes::new();
        assert_eq!(
            ClientboundMountScreenOpenPacket::read(&mut empty),
            Err(PacketDecodeError::UnexpectedEof)
        );

        let pkt = ClientboundMountScreenOpenPacket::new(7, 3, 99);
        let mut handler = MockHandler {
            called: false,
            container_id: 0,
        };
        pkt.handle(&mut handler);
        assert!(handler.called);
        assert_eq!(handler.container_id, 7);
        assert_eq!(pkt.type_name(), "minecraft:mount_screen_open");
    }
}
