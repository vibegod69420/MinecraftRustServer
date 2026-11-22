use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_SERVER_DATA_TYPE: PacketType =
    PacketType("minecraft:clientbound_server_data");

pub trait ClientGamePacketListener {
    fn handle_server_data(&mut self, packet: &ClientboundServerDataPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    buf: Vec<u8>,
    rpos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            rpos: 0,
        }
    }

    pub fn from_vec(buf: Vec<u8>) -> Self {
        Self { buf, rpos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.rpos)
    }

    fn ensure(&self, n: usize) -> io::Result<()> {
        if self.remaining() < n {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "buffer underflow"))
        } else {
            Ok(())
        }
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.ensure(1)?;
        let v = self.buf[self.rpos];
        self.rpos += 1;
        Ok(v)
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn write_bool(&mut self, v: bool) {
        self.write_u8(if v { 1 } else { 0 });
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        self.ensure(2)?;
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.buf[self.rpos..self.rpos + 2]);
        self.rpos += 2;
        Ok(u16::from_be_bytes(bytes))
    }

    pub fn read_i16(&mut self) -> io::Result<i16> {
        Ok(self.read_u16()? as i16)
    }

    pub fn read_i32(&mut self) -> io::Result<i32> {
        self.ensure(4)?;
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.buf[self.rpos..self.rpos + 4]);
        self.rpos += 4;
        Ok(i32::from_be_bytes(bytes))
    }

    pub fn read_i64(&mut self) -> io::Result<i64> {
        self.ensure(8)?;
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.buf[self.rpos..self.rpos + 8]);
        self.rpos += 8;
        Ok(i64::from_be_bytes(bytes))
    }

    pub fn read_f32(&mut self) -> io::Result<f32> {
        self.ensure(4)?;
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.buf[self.rpos..self.rpos + 4]);
        self.rpos += 4;
        Ok(f32::from_be_bytes(bytes))
    }

    pub fn read_f64(&mut self) -> io::Result<f64> {
        self.ensure(8)?;
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.buf[self.rpos..self.rpos + 8]);
        self.rpos += 8;
        Ok(f64::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_u8()?;
            value |= ((current_byte & 0x7F) as i32) << position;
            if (current_byte & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
        }
        Ok(value)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            let mut temp = (value as u32 & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if (value as u32) != 0 {
                temp |= 0x80;
            }
            self.write_u8(temp);
            if (value as u32) == 0 {
                break;
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByteArray()
    pub fn read_byte_array(&mut self) -> io::Result<Vec<u8>> {
        let len = self.read_var_int()? as usize;
        self.ensure(len)?;
        let mut out = vec![0u8; len];
        out.copy_from_slice(&self.buf[self.rpos..self.rpos + len]);
        self.rpos += len;
        Ok(out)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByteArray(byte[])
    pub fn write_byte_array(&mut self, data: &[u8]) {
        self.write_var_int(data.len() as i32);
        self.buf.extend_from_slice(data);
    }

    // JAVA: net.minecraft.network.codec.ByteBufCodecs#optional(ByteBufCodecs.BYTE_ARRAY)
    pub fn read_optional_byte_array(&mut self) -> io::Result<Option<Vec<u8>>> {
        if self.read_bool()? {
            Ok(Some(self.read_byte_array()?))
        } else {
            Ok(None)
        }
    }

    // JAVA: net.minecraft.network.codec.ByteBufCodecs#optional(ByteBufCodecs.BYTE_ARRAY)
    pub fn write_optional_byte_array(&mut self, data: Option<&[u8]>) {
        match data {
            Some(bytes) => {
                self.write_bool(true);
                self.write_byte_array(bytes);
            }
            None => {
                self.write_bool(false);
            }
        }
    }
}

impl Default for FriendlyByteBuf {
    fn default() -> Self {
        Self::new()
    }
}

fn read_nbt_tag_payload(
    buf: &mut FriendlyByteBuf,
    tag_type: u8,
    out: &mut Vec<u8>,
) -> io::Result<()> {
    match tag_type {
        1 => {
            out.push(buf.read_u8()?);
        }
        2 => {
            let v = buf.read_i16()?;
            out.extend_from_slice(&v.to_be_bytes());
        }
        3 => {
            let v = buf.read_i32()?;
            out.extend_from_slice(&v.to_be_bytes());
        }
        4 => {
            let v = buf.read_i64()?;
            out.extend_from_slice(&v.to_be_bytes());
        }
        5 => {
            let v = buf.read_f32()?;
            out.extend_from_slice(&v.to_be_bytes());
        }
        6 => {
            let v = buf.read_f64()?;
            out.extend_from_slice(&v.to_be_bytes());
        }
        7 => {
            let len = buf.read_i32()?;
            out.extend_from_slice(&len.to_be_bytes());
            for _ in 0..len {
                out.push(buf.read_u8()?);
            }
        }
        8 => {
            let len = buf.read_u16()? as usize;
            out.extend_from_slice(&(len as u16).to_be_bytes());
            for _ in 0..len {
                out.push(buf.read_u8()?);
            }
        }
        9 => {
            let list_type = buf.read_u8()?;
            out.push(list_type);
            let len = buf.read_i32()?;
            out.extend_from_slice(&len.to_be_bytes());
            for _ in 0..len {
                if list_type != 0 {
                    read_nbt_tag_payload(buf, list_type, out)?;
                }
            }
        }
        10 => loop {
            let child_type = buf.read_u8()?;
            out.push(child_type);
            if child_type == 0 {
                break;
            }
            let name_len = buf.read_u16()? as usize;
            out.extend_from_slice(&(name_len as u16).to_be_bytes());
            for _ in 0..name_len {
                out.push(buf.read_u8()?);
            }
            read_nbt_tag_payload(buf, child_type, out)?;
        },
        11 => {
            let len = buf.read_i32()?;
            out.extend_from_slice(&len.to_be_bytes());
            for _ in 0..len {
                let v = buf.read_i32()?;
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
        12 => {
            let len = buf.read_i32()?;
            out.extend_from_slice(&len.to_be_bytes());
            for _ in 0..len {
                let v = buf.read_i64()?;
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown NBT tag type",
            ));
        }
    }
    Ok(())
}

/// Opaque network-NBT encoded Component payload.
/// The bytes include the root tag type byte and its payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub encoded_nbt: Vec<u8>,
}

impl Component {
    // JAVA: net.minecraft.network.chat.ComponentSerialization#TRUSTED_CONTEXT_FREE_STREAM_CODEC decode
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let tag_type = buf.read_u8()?;
        let mut encoded_nbt = vec![tag_type];
        if tag_type != 0 {
            read_nbt_tag_payload(buf, tag_type, &mut encoded_nbt)?;
        }
        Ok(Self { encoded_nbt })
    }

    // JAVA: net.minecraft.network.chat.ComponentSerialization#TRUSTED_CONTEXT_FREE_STREAM_CODEC encode
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.buf.extend_from_slice(&self.encoded_nbt);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundServerDataPacket {
    pub motd: Component,
    pub icon_bytes: Option<Vec<u8>>,
}

impl ClientboundServerDataPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundServerDataPacket#<init>(Component,Optional<byte[]>)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let motd = Component::read(buf)?;
        let icon_bytes = buf.read_optional_byte_array()?;
        Ok(Self { motd, icon_bytes })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundServerDataPacket#STREAM_CODEC encode
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        self.motd.write(buf);
        buf.write_optional_byte_array(self.icon_bytes.as_deref());
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundServerDataPacket#type()
    pub fn type_(&self) -> PacketType {
        CLIENTBOUND_SERVER_DATA_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundServerDataPacket#handle(ClientGamePacketListener)
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_server_data(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_compound() -> Component {
        Component {
            encoded_nbt: vec![10, 0],
        }
    }

    #[test]
    fn round_trip_no_icon() {
        let packet = ClientboundServerDataPacket {
            motd: empty_compound(),
            icon_bytes: None,
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundServerDataPacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_with_icon() {
        let packet = ClientboundServerDataPacket {
            motd: empty_compound(),
            icon_bytes: Some(vec![1, 2, 3, 4, 255]),
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundServerDataPacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn round_trip_complex_component_nbt() {
        // Compound { "text": String("hi") }
        let mut nbt = vec![10u8];
        nbt.push(8); // string tag
        nbt.extend_from_slice(&4u16.to_be_bytes());
        nbt.extend_from_slice(b"text");
        nbt.extend_from_slice(&2u16.to_be_bytes());
        nbt.extend_from_slice(b"hi");
        nbt.push(0); // end compound
        let packet = ClientboundServerDataPacket {
            motd: Component { encoded_nbt: nbt },
            icon_bytes: Some(vec![]),
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundServerDataPacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn optional_byte_array_encoding() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_optional_byte_array(None);
        buf.write_optional_byte_array(Some(&[0x7F, 0x00]));
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        assert_eq!(read_buf.read_optional_byte_array().unwrap(), None);
        assert_eq!(
            read_buf.read_optional_byte_array().unwrap(),
            Some(vec![0x7F, 0x00])
        );
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn handle_calls_listener() {
        struct Listener {
            called: bool,
            has_icon: bool,
        }
        impl ClientGamePacketListener for Listener {
            fn handle_server_data(&mut self, packet: &ClientboundServerDataPacket) {
                self.called = true;
                self.has_icon = packet.icon_bytes.is_some();
            }
        }

        let packet = ClientboundServerDataPacket {
            motd: empty_compound(),
            icon_bytes: None,
        };
        let mut listener = Listener {
            called: false,
            has_icon: true,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert!(!listener.has_icon);
        assert_eq!(packet.type_(), CLIENTBOUND_SERVER_DATA_TYPE);
    }
}
