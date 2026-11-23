use bytes::{Buf, BufMut};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub const SERVERBOUND_CLIENT_COMMAND: PacketType = PacketType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    PerformRespawn,
    RequestStats,
}

impl Action {
    fn from_varint(value: i32) -> Result<Self, io::Error> {
        match value {
            0 => Ok(Self::PerformRespawn),
            1 => Ok(Self::RequestStats),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid ServerboundClientCommandPacket.Action",
            )),
        }
    }

    fn to_varint(self) -> i32 {
        match self {
            Self::PerformRespawn => 0,
            Self::RequestStats => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundClientCommandPacket {
    action: Action,
}

pub trait ServerGamePacketListener {
    fn handle_client_command(&mut self, packet: &ServerboundClientCommandPacket);
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

impl ServerboundClientCommandPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundClientCommandPacket#<init>(net.minecraft.network.protocol.game.ServerboundClientCommandPacket$Action)
    pub fn new(action: Action) -> Self {
        Self { action }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundClientCommandPacket#<init>(net.minecraft.network.FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, io::Error> {
        let action_id = read_varint(buf)?;
        let action = Action::from_varint(action_id)?;
        Ok(Self { action })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundClientCommandPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_varint(buf, self.action.to_varint());
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundClientCommandPacket#type
    pub fn get_type(&self) -> PacketType {
        SERVERBOUND_CLIENT_COMMAND
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundClientCommandPacket#handle
    pub fn handle<L: ServerGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_client_command(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundClientCommandPacket#getAction
    pub fn get_action(&self) -> Action {
        self.action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        action: Option<Action>,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_client_command(&mut self, packet: &ServerboundClientCommandPacket) {
            self.called = true;
            self.action = Some(packet.get_action());
        }
    }

    #[test]
    fn roundtrip_perform_respawn() {
        let packet = ServerboundClientCommandPacket::new(Action::PerformRespawn);
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundClientCommandPacket::read(&mut slice).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(decoded.get_action(), Action::PerformRespawn);
    }

    #[test]
    fn roundtrip_request_stats() {
        let packet = ServerboundClientCommandPacket::new(Action::RequestStats);
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundClientCommandPacket::read(&mut slice).unwrap();
        assert_eq!(decoded.get_action(), Action::RequestStats);
    }

    #[test]
    fn invalid_action_returns_error() {
        let mut slice: &[u8] = &[2];
        let err = ServerboundClientCommandPacket::read(&mut slice);
        assert!(err.is_err());
    }

    #[test]
    fn negative_varint_action_returns_error() {
        let mut slice: &[u8] = &[0xff, 0xff, 0xff, 0xff, 0x0f];
        let err = ServerboundClientCommandPacket::read(&mut slice);
        assert!(err.is_err());
    }

    #[test]
    fn varint_too_big_returns_error() {
        let mut slice: &[u8] = &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        let err = ServerboundClientCommandPacket::read(&mut slice);
        assert!(err.is_err());
    }

    #[test]
    fn handle_dispatches_and_type_is_stable() {
        let packet = ServerboundClientCommandPacket::new(Action::PerformRespawn);
        let mut listener = MockListener {
            called: false,
            action: None,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.action, Some(Action::PerformRespawn));
        assert_eq!(packet.get_type(), SERVERBOUND_CLIENT_COMMAND);
    }
}
