use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    ServerboundPlayerCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Action {
    StopSleeping = 0,
    StartSprinting = 1,
    StopSprinting = 2,
    StartRidingJump = 3,
    StopRidingJump = 4,
    OpenInventory = 5,
    StartFallFlying = 6,
}

impl Action {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket$Action#values
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::StopSleeping),
            1 => Some(Self::StartSprinting),
            2 => Some(Self::StopSprinting),
            3 => Some(Self::StartRidingJump),
            4 => Some(Self::StopRidingJump),
            5 => Some(Self::OpenInventory),
            6 => Some(Self::StartFallFlying),
            _ => None,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket$Action#ordinal
    pub fn id(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    pub id: i32,
}

impl Entity {
    // JAVA: net.minecraft.world.entity.Entity#<init>
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.world.entity.Entity#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int(buf: &mut impl Buf) -> i32 {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    loop {
        let b = buf.get_u8() as u32;
        result |= (b & 0x7F) << shift;
        if (b & 0x80) == 0 {
            return result as i32;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return result as i32;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int(buf: &mut impl BufMut, value: i32) {
    let mut val = value as u32;
    loop {
        if (val & !0x7F) == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8(((val & 0x7F) as u8) | 0x80);
        val >>= 7;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readEnum
fn read_action(buf: &mut impl Buf) -> Action {
    let id = read_var_int(buf);
    Action::from_id(id).unwrap_or(Action::StopSleeping)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeEnum
fn write_action(buf: &mut impl BufMut, action: Action) {
    write_var_int(buf, action.id());
}

pub trait ServerGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ServerGamePacketListener#handlePlayerCommand
    fn handle_player_command(&mut self, packet: &ServerboundPlayerCommandPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerCommandPacket {
    id: i32,
    action: Action,
    data: i32,
}

impl ServerboundPlayerCommandPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket#<init>(Entity,ServerboundPlayerCommandPacket$Action)
    pub fn new(entity: &Entity, action: Action) -> Self {
        Self::new_with_data(entity, action, 0)
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket#<init>(Entity,ServerboundPlayerCommandPacket$Action,int)
    pub fn new_with_data(entity: &Entity, action: Action, data: i32) -> Self {
        Self {
            id: entity.get_id(),
            action,
            data,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let id = read_var_int(buf);
        let action = read_action(buf);
        let data = read_var_int(buf);
        Self { id, action, data }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.id);
        write_action(buf, self.action);
        write_var_int(buf, self.data);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ServerboundPlayerCommand
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket#handle
    pub fn handle(&self, listener: &mut impl ServerGamePacketListener) {
        listener.handle_player_command(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket#getAction
    pub fn get_action(&self) -> Action {
        self.action
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket#getData
    pub fn get_data(&self) -> i32 {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestListener {
        called: bool,
        action: Option<Action>,
    }

    impl ServerGamePacketListener for TestListener {
        fn handle_player_command(&mut self, packet: &ServerboundPlayerCommandPacket) {
            self.called = true;
            self.action = Some(packet.get_action());
        }
    }

    #[test]
    fn round_trip_basic() {
        let entity = Entity::new(123);
        let packet = ServerboundPlayerCommandPacket::new_with_data(&entity, Action::StartRidingJump, 7);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ServerboundPlayerCommandPacket::read(&mut reader);
        assert_eq!(decoded, packet);
        assert_eq!(decoded.get_id(), 123);
        assert_eq!(decoded.get_data(), 7);
    }

    #[test]
    fn constructor_defaults_data_to_zero() {
        let packet = ServerboundPlayerCommandPacket::new(&Entity::new(5), Action::OpenInventory);
        assert_eq!(packet.get_data(), 0);
        assert_eq!(packet.get_action(), Action::OpenInventory);
    }

    #[test]
    fn negative_and_large_varint_values_round_trip() {
        let packet = ServerboundPlayerCommandPacket::new_with_data(&Entity::new(-1), Action::StartFallFlying, i32::MIN);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ServerboundPlayerCommandPacket::read(&mut reader);
        assert_eq!(decoded.get_id(), -1);
        assert_eq!(decoded.get_data(), i32::MIN);
    }

    #[test]
    fn handle_calls_listener_and_type_matches() {
        let packet = ServerboundPlayerCommandPacket::new(&Entity::new(9), Action::StopSprinting);
        let mut listener = TestListener { called: false, action: None };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.action, Some(Action::StopSprinting));
        assert_eq!(packet.packet_type(), PacketType::ServerboundPlayerCommand);
    }
}
