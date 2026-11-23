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

    pub fn read_utf(&mut self, max_len: usize) -> io::Result<String> {
        let byte_len = self.read_var_int()? as usize;
        let max_bytes = max_len.saturating_mul(3);
        if byte_len > max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "String too long",
            ));
        }
        if self.remaining() < byte_len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF reading string",
            ));
        }
        let bytes = &self.data[self.pos..self.pos + byte_len];
        self.pos += byte_len;
        let s = String::from_utf8(bytes.to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if s.chars().count() > max_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "String too long",
            ));
        }
        Ok(s)
    }

    pub fn write_utf(&mut self, value: &str, max_len: usize) -> io::Result<()> {
        if value.chars().count() > max_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "String too long",
            ));
        }
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
        Ok(())
    }

    pub fn read_collection<T, F>(
        &mut self,
        max_size: usize,
        mut reader: F,
    ) -> io::Result<Vec<T>>
    where
        F: FnMut(&mut Self) -> io::Result<T>,
    {
        let len = self.read_var_int()? as usize;
        if len > max_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Collection too large",
            ));
        }
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(reader(self)?);
        }
        Ok(out)
    }

    pub fn write_collection<T, F>(
        &mut self,
        items: &[T],
        max_size: usize,
        mut writer: F,
    ) -> io::Result<()>
    where
        F: FnMut(&mut Self, &T) -> io::Result<()>,
    {
        if items.len() > max_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Collection too large",
            ));
        }
        self.write_var_int(items.len() as i32);
        for item in items {
            writer(self, item)?;
        }
        Ok(())
    }

    pub fn read_optional<T, F>(&mut self, mut reader: F) -> io::Result<Option<T>>
    where
        F: FnMut(&mut Self) -> io::Result<T>,
    {
        let present = self.read_boolean()?;
        if present {
            Ok(Some(reader(self)?))
        } else {
            Ok(None)
        }
    }

    pub fn write_optional<T, F>(
        &mut self,
        value: Option<&T>,
        mut writer: F,
    ) -> io::Result<()>
    where
        F: FnMut(&mut Self, &T) -> io::Result<()>,
    {
        match value {
            Some(v) => {
                self.write_boolean(true);
                writer(self, v)?;
            }
            None => self.write_boolean(false),
        }
        Ok(())
    }
}

pub trait ServerGamePacketListener {
    fn handle_edit_book(&mut self, packet: &ServerboundEditBookPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundEditBookPacket {
    pub slot: i32,
    pub pages: Vec<String>,
    pub title: Option<String>,
}

impl ServerboundEditBookPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundEditBookPacket#<init>
    pub fn new(slot: i32, pages: Vec<String>, title: Option<String>) -> Self {
        Self { slot, pages, title }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEditBookPacket#STREAM_CODEC
    pub fn decode(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let slot = buf.read_var_int()?;
        let pages = buf.read_collection(100, |b| b.read_utf(1024))?;
        let title = buf.read_optional(|b| b.read_utf(32))?;
        Ok(Self::new(slot, pages, title))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEditBookPacket#STREAM_CODEC
    pub fn encode(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_var_int(self.slot);
        buf.write_collection(&self.pages, 100, |b, p| b.write_utf(p, 1024))?;
        buf.write_optional(self.title.as_ref(), |b, t| b.write_utf(t, 32))?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEditBookPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:edit_book")
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEditBookPacket#handle
    pub fn handle<H: ServerGamePacketListener>(&self, handler: &mut H) {
        handler.handle_edit_book(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_with_title() {
        let pkt = ServerboundEditBookPacket::new(
            5,
            vec!["hello".to_string(), "world".to_string()],
            Some("title".to_string()),
        );
        let mut buf = FriendlyByteBuf::new(Vec::new());
        pkt.encode(&mut buf).unwrap();
        let mut reader = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundEditBookPacket::decode(&mut reader).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_no_title_empty_pages() {
        let pkt = ServerboundEditBookPacket::new(0, vec![], None);
        let mut buf = FriendlyByteBuf::new(Vec::new());
        pkt.encode(&mut buf).unwrap();
        let mut reader = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundEditBookPacket::decode(&mut reader).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn negative_and_extreme_slot_roundtrip() {
        for slot in [i32::MIN, -1, 0, i32::MAX] {
            let pkt = ServerboundEditBookPacket::new(slot, vec!["p".to_string()], None);
            let mut buf = FriendlyByteBuf::new(Vec::new());
            pkt.encode(&mut buf).unwrap();
            let mut reader = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ServerboundEditBookPacket::decode(&mut reader).unwrap();
            assert_eq!(decoded.slot, slot);
        }
    }

    #[test]
    fn type_and_handle() {
        let pkt = ServerboundEditBookPacket::new(1, vec!["a".to_string()], Some("t".to_string()));
        assert_eq!(pkt.packet_type(), PacketType("minecraft:edit_book"));

        struct H {
            called: bool,
            slot: i32,
        }
        impl ServerGamePacketListener for H {
            fn handle_edit_book(&mut self, packet: &ServerboundEditBookPacket) {
                self.called = true;
                self.slot = packet.slot;
            }
        }

        let mut handler = H {
            called: false,
            slot: 0,
        };
        pkt.handle(&mut handler);
        assert!(handler.called);
        assert_eq!(handler.slot, 1);
    }
}
