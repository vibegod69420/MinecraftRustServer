use std::io::{self, Read, Write};

/// Placeholder for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

/// Placeholder for `net.minecraft.network.protocol.game.GamePacketTypes`.
pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_BLOCK_CHANGED_ACK: PacketType =
        PacketType("minecraft:clientbound_block_changed_ack");
}

/// Placeholder for `net.minecraft.network.protocol.game.ClientGamePacketListener`.
pub trait ClientGamePacketListener {
    fn handle_block_changed_ack(&mut self, packet: &ClientboundBlockChangedAckPacket);
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

/// Port of `net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundBlockChangedAckPacket {
    sequence: i32,
}

impl ClientboundBlockChangedAckPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket#<init>
    pub fn new(sequence: i32) -> Self {
        Self { sequence }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket#sequence
    pub fn sequence(&self) -> i32 {
        self.sequence
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket#<init>
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let sequence = read_var_int(reader)?;
        Ok(Self { sequence })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket#write
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_int(writer, self.sequence)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket#type
    pub fn get_type(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_BLOCK_CHANGED_ACK
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_block_changed_ack(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct MockListener {
        received: Option<ClientboundBlockChangedAckPacket>,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_block_changed_ack(&mut self, packet: &ClientboundBlockChangedAckPacket) {
            self.received = Some(*packet);
        }
    }

    #[test]
    fn round_trip_basic() {
        let packet = ClientboundBlockChangedAckPacket::new(5);
        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();
        let decoded = ClientboundBlockChangedAckPacket::read(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn round_trip_negative_and_extremes() {
        for seq in [i32::MIN, -1, 0, 1, 127, 128, i32::MAX] {
            let packet = ClientboundBlockChangedAckPacket::new(seq);
            let mut buf = Vec::new();
            packet.write(&mut buf).unwrap();
            let decoded =
                ClientboundBlockChangedAckPacket::read(&mut Cursor::new(&buf)).unwrap();
            assert_eq!(decoded.sequence(), seq);
        }
    }

    #[test]
    fn var_int_encoding_known_values() {
        let mut buf = Vec::new();
        ClientboundBlockChangedAckPacket::new(0)
            .write(&mut buf)
            .unwrap();
        assert_eq!(buf, vec![0x00]);

        buf.clear();
        ClientboundBlockChangedAckPacket::new(127)
            .write(&mut buf)
            .unwrap();
        assert_eq!(buf, vec![0x7f]);

        buf.clear();
        ClientboundBlockChangedAckPacket::new(128)
            .write(&mut buf)
            .unwrap();
        assert_eq!(buf, vec![0x80, 0x01]);

        buf.clear();
        ClientboundBlockChangedAckPacket::new(-1)
            .write(&mut buf)
            .unwrap();
        assert_eq!(buf, vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
    }

    #[test]
    fn handle_dispatches() {
        let packet = ClientboundBlockChangedAckPacket::new(42);
        let mut listener = MockListener { received: None };
        packet.handle(&mut listener);
        assert_eq!(listener.received, Some(packet));
    }

    #[test]
    fn packet_type_matches() {
        let packet = ClientboundBlockChangedAckPacket::new(0);
        assert_eq!(
            packet.get_type(),
            GamePacketTypes::CLIENTBOUND_BLOCK_CHANGED_ACK
        );
    }
}
