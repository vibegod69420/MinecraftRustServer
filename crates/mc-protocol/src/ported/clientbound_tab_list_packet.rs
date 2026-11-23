use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketType {
    ClientboundTabList,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            read_pos: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_varint(&mut self) -> io::Result<i32> {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_u8()? as i32;
            let shift = j.wrapping_mul(7) & 31;
            i |= (b0 & 127) << shift;
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if (b0 & 128) == 0 {
                break;
            }
        }
        Ok(i)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_varint(&mut self, mut value: i32) -> io::Result<()> {
        loop {
            if (value & !0x7F) == 0 {
                self.data.push(value as u8);
                return Ok(());
            }
            self.data.push(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByteArray
    pub fn read_byte_array(&mut self) -> io::Result<Vec<u8>> {
        let len = self.read_varint()?;
        if len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative byte array length",
            ));
        }
        let len = len as usize;
        let remaining = self.data.len().saturating_sub(self.read_pos);
        if remaining < len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "byte array past end",
            ));
        }
        let out = self.data[self.read_pos..self.read_pos + len].to_vec();
        self.read_pos += len;
        Ok(out)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByteArray
    pub fn write_byte_array(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.write_varint(bytes.len() as i32)?;
        self.data.extend_from_slice(bytes);
        Ok(())
    }
}

pub type RegistryFriendlyByteBuf = FriendlyByteBuf;

/// Minimal self-contained component placeholder.
/// In Minecraft this is serialized via ComponentSerialization.TRUSTED_STREAM_CODEC;
/// here we preserve the packet structure with an opaque payload for faithful round-trips.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub payload: Vec<u8>,
}

impl Component {
    // JAVA: net.minecraft.network.chat.ComponentSerialization#TRUSTED_STREAM_CODEC
    pub fn write_trusted(&self, buf: &mut RegistryFriendlyByteBuf) -> io::Result<()> {
        buf.write_byte_array(&self.payload)
    }

    // JAVA: net.minecraft.network.chat.ComponentSerialization#TRUSTED_STREAM_CODEC
    pub fn read_trusted(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        Ok(Self {
            payload: buf.read_byte_array()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundTabListPacket {
    pub header: Component,
    pub footer: Component,
}

pub trait ClientGamePacketListener {
    fn handle_tab_list_customisation(&mut self, packet: &ClientboundTabListPacket);
}

impl ClientboundTabListPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundTabListPacket#<init>
    pub fn new(header: Component, footer: Component) -> Self {
        Self { header, footer }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTabListPacket#header
    pub fn header(&self) -> &Component {
        &self.header
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTabListPacket#footer
    pub fn footer(&self) -> &Component {
        &self.footer
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTabListPacket#STREAM_CODEC
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let header = Component::read_trusted(buf)?;
        let footer = Component::read_trusted(buf)?;
        Ok(Self::new(header, footer))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTabListPacket#STREAM_CODEC
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) -> io::Result<()> {
        self.header.write_trusted(buf)?;
        self.footer.write_trusted(buf)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTabListPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ClientboundTabList
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTabListPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_tab_list_customisation(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let pkt = ClientboundTabListPacket::new(
            Component {
                payload: vec![1, 2, 3],
            },
            Component { payload: vec![4, 5] },
        );
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundTabListPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.header().payload, vec![1, 2, 3]);
        assert_eq!(decoded.footer().payload, vec![4, 5]);
    }

    #[test]
    fn empty_components_round_trip() {
        let pkt = ClientboundTabListPacket::new(
            Component { payload: vec![] },
            Component { payload: vec![] },
        );
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundTabListPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn type_and_handle() {
        let pkt = ClientboundTabListPacket::new(
            Component { payload: vec![9] },
            Component { payload: vec![8] },
        );
        assert_eq!(pkt.packet_type(), PacketType::ClientboundTabList);

        struct Listener(Option<ClientboundTabListPacket>);
        impl ClientGamePacketListener for Listener {
            fn handle_tab_list_customisation(&mut self, packet: &ClientboundTabListPacket) {
                self.0 = Some(packet.clone());
            }
        }

        let mut listener = Listener(None);
        pkt.handle(&mut listener);
        assert_eq!(listener.0, Some(pkt));
    }

    #[test]
    fn varint_negative_round_trip() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_varint(-1).unwrap();
        buf.write_varint(i32::MIN).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        assert_eq!(read_buf.read_varint().unwrap(), -1);
        assert_eq!(read_buf.read_varint().unwrap(), i32::MIN);
    }
}
