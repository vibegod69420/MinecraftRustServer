use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub mod game_packet_types {
    use super::PacketType;

    pub const CLIENTBOUND_DEBUG_ENTITY_VALUE: PacketType =
        PacketType("clientbound_debug_entity_value");
}

/// Self-contained placeholder for the unported
/// `net.minecraft.util.debug.DebugSubscription.Update`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSubscriptionUpdate {
    pub type_id: i32,
}

impl DebugSubscriptionUpdate {
    pub fn stream_decode(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let type_id = buf.read_var_int()?;
        Ok(Self { type_id })
    }

    pub fn stream_encode(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_var_int(self.type_id);
    }
}

/// Minimal self-contained buffer stub for `RegistryFriendlyByteBuf`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn writer() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_byte()? as i32;
            result |= (current_byte & 127) << position;
            if (current_byte & 128) == 0 {
                break;
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt is too big",
                ));
            }
        }
        Ok(result)
    }

    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F) == 0 {
                self.data.push(value as u8);
                return;
            }
            self.data.push(((value & 0x7F) as u8) | 0x80);
            value = ((value as u32) >> 7) as i32;
        }
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }
}

pub trait ClientGamePacketListener {
    fn handle_debug_entity_value(&mut self, packet: &ClientboundDebugEntityValuePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugEntityValuePacket {
    entity_id: i32,
    update: DebugSubscriptionUpdate,
}

impl ClientboundDebugEntityValuePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugEntityValuePacket#ClientboundDebugEntityValuePacket
    pub fn new(entity_id: i32, update: DebugSubscriptionUpdate) -> Self {
        Self { entity_id, update }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugEntityValuePacket#entityId
    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugEntityValuePacket#update
    pub fn update(&self) -> &DebugSubscriptionUpdate {
        &self.update
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugEntityValuePacket#type
    pub fn get_type(&self) -> PacketType {
        game_packet_types::CLIENTBOUND_DEBUG_ENTITY_VALUE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugEntityValuePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_debug_entity_value(self);
    }

    pub fn stream_decode(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let entity_id = buf.read_var_int()?;
        let update = DebugSubscriptionUpdate::stream_decode(buf)?;
        Ok(Self::new(entity_id, update))
    }

    pub fn stream_encode(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        self.update.stream_encode(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen: Option<(i32, i32)>,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_debug_entity_value(&mut self, packet: &ClientboundDebugEntityValuePacket) {
            self.seen = Some((packet.entity_id(), packet.update().type_id));
        }
    }

    #[test]
    fn packet_accessors_and_type() {
        let packet = ClientboundDebugEntityValuePacket::new(42, DebugSubscriptionUpdate { type_id: 7 });
        assert_eq!(packet.entity_id(), 42);
        assert_eq!(packet.update(), &DebugSubscriptionUpdate { type_id: 7 });
        assert_eq!(
            packet.get_type(),
            game_packet_types::CLIENTBOUND_DEBUG_ENTITY_VALUE
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ClientboundDebugEntityValuePacket::new(-5, DebugSubscriptionUpdate { type_id: 3 });
        let mut listener = TestListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen, Some((-5, 3)));
    }

    #[test]
    fn stream_codec_roundtrip() {
        let packet = ClientboundDebugEntityValuePacket::new(
            123456,
            DebugSubscriptionUpdate { type_id: -99 },
        );
        let mut buf = RegistryFriendlyByteBuf::writer();
        packet.stream_encode(&mut buf);
        let encoded = buf.into_inner();
        let mut read_buf = RegistryFriendlyByteBuf::new(encoded);
        let decoded = ClientboundDebugEntityValuePacket::stream_decode(&mut read_buf).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn varint_edge_cases_and_negative_values() {
        let mut buf = RegistryFriendlyByteBuf::writer();
        buf.write_var_int(0);
        buf.write_var_int(-1);
        buf.write_var_int(i32::MIN);
        buf.write_var_int(i32::MAX);
        let data = buf.into_inner();
        let mut read_buf = RegistryFriendlyByteBuf::new(data);
        assert_eq!(read_buf.read_var_int().unwrap(), 0);
        assert_eq!(read_buf.read_var_int().unwrap(), -1);
        assert_eq!(read_buf.read_var_int().unwrap(), i32::MIN);
        assert_eq!(read_buf.read_var_int().unwrap(), i32::MAX);
    }

    #[test]
    fn varint_too_big_returns_error() {
        // 5 bytes with continuation bit set on the 5th byte is too big for Minecraft VarInt
        let data = vec![0x80, 0x80, 0x80, 0x80, 0x80];
        let mut buf = RegistryFriendlyByteBuf::new(data);
        let err = buf.read_var_int().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
