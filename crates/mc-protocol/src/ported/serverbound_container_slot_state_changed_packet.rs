use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_byte(&mut self) -> io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    pub fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    pub fn read_boolean(&mut self) -> io::Result<bool> {
        Ok(self.read_byte()? != 0)
    }

    pub fn write_boolean(&mut self, value: bool) {
        self.write_byte(if value { 1 } else { 0 });
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        for shift in (0..32).step_by(7) {
            let b = self.read_byte()?;
            result |= ((b & 0x7F) as i32) << shift;
            if (b & 0x80) == 0 {
                return Ok(result);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VarInt too big",
        ))
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7F) == 0 {
                self.write_byte(val as u8);
                return;
            }
            self.write_byte(((val & 0x7F) as u8) | 0x80);
            val >>= 7;
        }
    }

    pub fn read_container_id(&mut self) -> io::Result<i32> {
        self.read_var_int()
    }

    pub fn write_container_id(&mut self, value: i32) {
        self.write_var_int(value);
    }
}

pub trait ServerGamePacketListener {
    fn handle_container_slot_state_changed(
        &mut self,
        packet: &ServerboundContainerSlotStateChangedPacket,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundContainerSlotStateChangedPacket {
    pub slot_id: i32,
    pub container_id: i32,
    pub new_state: bool,
}

impl ServerboundContainerSlotStateChangedPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerSlotStateChangedPacket#<init>
    pub fn new(slot_id: i32, container_id: i32, new_state: bool) -> Self {
        Self {
            slot_id,
            container_id,
            new_state,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerSlotStateChangedPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let slot_id = buf.read_var_int()?;
        let container_id = buf.read_container_id()?;
        let new_state = buf.read_boolean()?;
        Ok(Self::new(slot_id, container_id, new_state))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerSlotStateChangedPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.slot_id);
        buf.write_container_id(self.container_id);
        buf.write_boolean(self.new_state);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerSlotStateChangedPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:container_slot_state_changed")
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerSlotStateChangedPacket#handle
    pub fn handle<H: ServerGamePacketListener>(&self, handler: &mut H) {
        handler.handle_container_slot_state_changed(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let pkt = ServerboundContainerSlotStateChangedPacket::new(3, 7, true);
        let mut buf = FriendlyByteBuf::new(Vec::new());
        pkt.write(&mut buf);
        let mut reader = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundContainerSlotStateChangedPacket::read(&mut reader).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_false_and_negative_ids() {
        let pkt = ServerboundContainerSlotStateChangedPacket::new(-1, -100, false);
        let mut buf = FriendlyByteBuf::new(Vec::new());
        pkt.write(&mut buf);
        let mut reader = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundContainerSlotStateChangedPacket::read(&mut reader).unwrap();
        assert_eq!(decoded.slot_id, -1);
        assert_eq!(decoded.container_id, -100);
        assert!(!decoded.new_state);
    }

    #[test]
    fn varint_extremes_roundtrip() {
        for v in [i32::MIN, i32::MAX, 0, -1] {
            let pkt = ServerboundContainerSlotStateChangedPacket::new(v, v.wrapping_add(1), true);
            let mut buf = FriendlyByteBuf::new(Vec::new());
            pkt.write(&mut buf);
            let mut reader = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ServerboundContainerSlotStateChangedPacket::read(&mut reader).unwrap();
            assert_eq!(decoded.slot_id, v);
            assert_eq!(decoded.container_id, v.wrapping_add(1));
        }
    }

    #[test]
    fn handle_and_type() {
        let pkt = ServerboundContainerSlotStateChangedPacket::new(1, 2, true);
        assert_eq!(
            pkt.packet_type(),
            PacketType("minecraft:container_slot_state_changed")
        );

        struct H {
            seen: Option<(i32, i32, bool)>,
        }
        impl ServerGamePacketListener for H {
            fn handle_container_slot_state_changed(
                &mut self,
                p: &ServerboundContainerSlotStateChangedPacket,
            ) {
                self.seen = Some((p.slot_id, p.container_id, p.new_state));
            }
        }

        let mut handler = H { seen: None };
        pkt.handle(&mut handler);
        assert_eq!(handler.seen, Some((1, 2, true)));
    }
}
