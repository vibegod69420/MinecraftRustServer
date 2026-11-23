use bytes::{Buf, BufMut};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub const CLIENTBOUND_SET_BORDER_WARNING_DISTANCE: PacketType = PacketType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetBorderWarningDistancePacket {
    warning_blocks: i32,
}

pub trait WorldBorderLike {
    fn get_warning_blocks(&self) -> i32;
}

pub trait ClientGamePacketListener {
    fn handle_set_border_warning_distance(
        &mut self,
        packet: &ClientboundSetBorderWarningDistancePacket,
    );
}

fn read_varint<B: Buf>(buf: &mut B) -> Result<i32, io::Error> {
    let mut value: i32 = 0;
    let mut position: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "VarInt too short",
            ));
        }
        let byte = buf.get_u8();
        value |= ((byte & 0x7f) as i32) << ((position * 7) & 31);
        position += 1;
        if position > 5 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt too big",
            ));
        }
        if (byte & 0x80) == 0 {
            break;
        }
    }
    Ok(value)
}

fn write_varint<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value & !0x7f_i32) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value & 0x7f) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

impl ClientboundSetBorderWarningDistancePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDistancePacket#<init>(net.minecraft.world.level.border.WorldBorder)
    pub fn from_world_border(border: &impl WorldBorderLike) -> Self {
        Self {
            warning_blocks: border.get_warning_blocks(),
        }
    }

    pub fn new(warning_blocks: i32) -> Self {
        Self { warning_blocks }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDistancePacket#<init>(net.minecraft.network.FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, io::Error> {
        let warning_blocks = read_varint(buf)?;
        Ok(Self { warning_blocks })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDistancePacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_varint(buf, self.warning_blocks);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDistancePacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_SET_BORDER_WARNING_DISTANCE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDistancePacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_set_border_warning_distance(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDistancePacket#getWarningBlocks
    pub fn get_warning_blocks(&self) -> i32 {
        self.warning_blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Border(i32);
    impl WorldBorderLike for Border {
        fn get_warning_blocks(&self) -> i32 {
            self.0
        }
    }

    struct MockListener {
        called: bool,
        warning_blocks: Option<i32>,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_set_border_warning_distance(
            &mut self,
            packet: &ClientboundSetBorderWarningDistancePacket,
        ) {
            self.called = true;
            self.warning_blocks = Some(packet.get_warning_blocks());
        }
    }

    #[test]
    fn from_world_border_and_getter() {
        let packet = ClientboundSetBorderWarningDistancePacket::from_world_border(&Border(5));
        assert_eq!(packet.get_warning_blocks(), 5);
    }

    #[test]
    fn roundtrip_positive_value() {
        let packet = ClientboundSetBorderWarningDistancePacket::new(42);
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSetBorderWarningDistancePacket::read(&mut slice).unwrap();
        assert_eq!(decoded.get_warning_blocks(), 42);
    }

    #[test]
    fn roundtrip_negative_value() {
        let packet = ClientboundSetBorderWarningDistancePacket::new(-1);
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSetBorderWarningDistancePacket::read(&mut slice).unwrap();
        assert_eq!(decoded.get_warning_blocks(), -1);
    }

    #[test]
    fn roundtrip_large_value() {
        let packet = ClientboundSetBorderWarningDistancePacket::new(i32::MAX);
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSetBorderWarningDistancePacket::read(&mut slice).unwrap();
        assert_eq!(decoded.get_warning_blocks(), i32::MAX);
    }

    #[test]
    fn invalid_varint_and_handle() {
        let mut bad: &[u8] = &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        assert!(ClientboundSetBorderWarningDistancePacket::read(&mut bad).is_err());

        let packet = ClientboundSetBorderWarningDistancePacket::new(7);
        let mut listener = MockListener {
            called: false,
            warning_blocks: None,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.warning_blocks, Some(7));
        assert_eq!(packet.get_type(), CLIENTBOUND_SET_BORDER_WARNING_DISTANCE);
    }
}
