use std::io::{self, Read, Write};

/// Placeholder for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

/// Placeholder for `net.minecraft.network.protocol.game.GamePacketTypes`.
pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const SERVERBOUND_PICK_ITEM_FROM_ENTITY: PacketType =
        PacketType("minecraft:serverbound_pick_item_from_entity");
}

/// Placeholder for `net.minecraft.network.protocol.game.ServerGamePacketListener`.
pub trait ServerGamePacketListener {
    fn handle_pick_item_from_entity(&mut self, packet: &ServerboundPickItemFromEntityPacket);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int<W: Write>(writer: &mut W, value: i32) -> io::Result<()> {
    let mut val = value as u32;
    loop {
        if (val & !0x7F) == 0 {
            writer.write_all(&[val as u8])?;
            return Ok(());
        }
        writer.write_all(&[((val & 0x7F) | 0x80) as u8])?;
        val >>= 7;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let b = byte[0];
        result |= (((b & 0x7F) as u32) << shift) as i32;
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt is too big",
            ));
        }
    }
}

/// Port of `net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPickItemFromEntityPacket {
    id: i32,
    include_data: bool,
}

impl ServerboundPickItemFromEntityPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket#<init>
    pub fn new(id: i32, include_data: bool) -> Self {
        Self { id, include_data }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket#id
    pub fn id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket#includeData
    pub fn include_data(&self) -> bool {
        self.include_data
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket#type
    pub fn get_type(&self) -> PacketType {
        GamePacketTypes::SERVERBOUND_PICK_ITEM_FROM_ENTITY
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket#handle
    pub fn handle<L: ServerGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_pick_item_from_entity(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket#write
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_int(writer, self.id)?;
        writer.write_all(&[if self.include_data { 1u8 } else { 0u8 }])?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket#<init>
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let id = read_var_int(reader)?;
        let mut bool_buf = [0u8; 1];
        reader.read_exact(&mut bool_buf)?;
        let include_data = bool_buf[0] != 0;
        Ok(Self { id, include_data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct MockListener {
        received: Option<ServerboundPickItemFromEntityPacket>,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_pick_item_from_entity(&mut self, packet: &ServerboundPickItemFromEntityPacket) {
            self.received = Some(*packet);
        }
    }

    #[test]
    fn round_trip_basic() {
        let packet = ServerboundPickItemFromEntityPacket::new(42, true);
        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();
        let decoded =
            ServerboundPickItemFromEntityPacket::read(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn round_trip_negative_id() {
        let packet = ServerboundPickItemFromEntityPacket::new(-1, false);
        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();
        let decoded =
            ServerboundPickItemFromEntityPacket::read(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded.id(), -1);
        assert!(!decoded.include_data());
    }

    #[test]
    fn round_trip_integer_extremes() {
        for id in [i32::MIN, i32::MAX, 0, -123_456] {
            for include_data in [false, true] {
                let packet = ServerboundPickItemFromEntityPacket::new(id, include_data);
                let mut buf = Vec::new();
                packet.write(&mut buf).unwrap();
                let decoded =
                    ServerboundPickItemFromEntityPacket::read(&mut Cursor::new(&buf)).unwrap();
                assert_eq!(packet, decoded);
            }
        }
    }

    #[test]
    fn handle_dispatches() {
        let packet = ServerboundPickItemFromEntityPacket::new(7, true);
        let mut listener = MockListener { received: None };
        packet.handle(&mut listener);
        assert_eq!(listener.received, Some(packet));
    }

    #[test]
    fn packet_type_matches() {
        let packet = ServerboundPickItemFromEntityPacket::new(0, false);
        assert_eq!(
            packet.get_type(),
            GamePacketTypes::SERVERBOUND_PICK_ITEM_FROM_ENTITY
        );
    }
}
