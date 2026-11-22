use bytes::{BufMut, BytesMut};
use std::io;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    buf: BytesMut,
    read_pos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self {
            buf: BytesMut::from(&data[..]),
            read_pos: 0,
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf.to_vec()
    }

    fn read_byte(&mut self) -> io::Result<i8> {
        if self.read_pos >= self.buf.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.buf[self.read_pos] as i8;
        self.read_pos += 1;
        Ok(b)
    }

    fn write_byte(&mut self, value: i8) {
        self.buf.put_i8(value);
    }

    fn read_bytes_into(&mut self, dst: &mut [u8]) -> io::Result<()> {
        let end = self
            .read_pos
            .checked_add(dst.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"))?;
        if end > self.buf.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        dst.copy_from_slice(&self.buf[self.read_pos..end]);
        self.read_pos = end;
        Ok(())
    }

    fn write_bytes(&mut self, src: &[u8]) {
        self.buf.extend_from_slice(src);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_byte()?;
            i |= ((b0 as i32) & 127) << j.wrapping_mul(7);
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if ((b0 as i32) & 128) != 128 {
                break;
            }
        }
        Ok(i)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & -128) != 0 {
                self.write_byte(((value & 127) | 128) as i8);
                value = ((value as u32) >> 7) as i32;
            } else {
                self.write_byte(value as i8);
                break;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignature {
    pub bytes: [u8; 256],
}

impl MessageSignature {
    // JAVA: net.minecraft.network.chat.MessageSignature#read
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let mut bytes = [0u8; 256];
        buf.read_bytes_into(&mut bytes)?;
        Ok(Self { bytes })
    }

    // JAVA: net.minecraft.network.chat.MessageSignature#write
    pub fn write(buf: &mut FriendlyByteBuf, signature: &MessageSignature) {
        buf.write_bytes(&signature.bytes);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignaturePacked {
    pub id: i32,
    pub full_signature: Option<MessageSignature>,
}

impl MessageSignaturePacked {
    // JAVA: net.minecraft.network.chat.MessageSignature$Packed#read
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let id = buf.read_var_int()?.wrapping_sub(1);
        let full_signature = if id == -1 {
            Some(MessageSignature::read(buf)?)
        } else {
            None
        };
        Ok(Self { id, full_signature })
    }

    // JAVA: net.minecraft.network.chat.MessageSignature$Packed#write
    pub fn write(buf: &mut FriendlyByteBuf, packed: &MessageSignaturePacked) {
        buf.write_var_int(packed.id.wrapping_add(1));
        if let Some(sig) = &packed.full_signature {
            MessageSignature::write(buf, sig);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType<T>(PhantomData<T>);

pub trait ClientGamePacketListener {
    fn handle_delete_chat(&mut self, packet: &ClientboundDeleteChatPacket);
}

pub const CLIENTBOUND_DELETE_CHAT: PacketType<ClientboundDeleteChatPacket> =
    PacketType(PhantomData);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDeleteChatPacket {
    pub message_signature: MessageSignaturePacked,
}

impl ClientboundDeleteChatPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundDeleteChatPacket#<init>
    pub fn new(message_signature: MessageSignaturePacked) -> Self {
        Self { message_signature }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDeleteChatPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        Ok(Self::new(MessageSignaturePacked::read(buf)?))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDeleteChatPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        MessageSignaturePacked::write(buf, &self.message_signature);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDeleteChatPacket#type
    pub fn get_type(&self) -> PacketType<Self> {
        CLIENTBOUND_DELETE_CHAT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDeleteChatPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_delete_chat(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_full_signature() {
        let sig = MessageSignature { bytes: [0xAB; 256] };
        let packed = MessageSignaturePacked {
            id: -1,
            full_signature: Some(sig),
        };
        let packet = ClientboundDeleteChatPacket::new(packed.clone());
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundDeleteChatPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.message_signature, packed);
    }

    #[test]
    fn roundtrip_reference_id() {
        let packed = MessageSignaturePacked {
            id: 42,
            full_signature: None,
        };
        let packet = ClientboundDeleteChatPacket::new(packed.clone());
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundDeleteChatPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.message_signature, packed);
    }

    #[test]
    fn roundtrip_negative_and_extreme_ids() {
        for id in [0, -2, i32::MIN, i32::MAX - 1] {
            let packed = MessageSignaturePacked {
                id,
                full_signature: None,
            };
            let packet = ClientboundDeleteChatPacket::new(packed.clone());
            let mut buf = FriendlyByteBuf::new();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
            let decoded = ClientboundDeleteChatPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.message_signature.id, id);
            assert!(decoded.message_signature.full_signature.is_none());
        }
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener {
            seen_id: Option<i32>,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_delete_chat(&mut self, packet: &ClientboundDeleteChatPacket) {
                self.seen_id = Some(packet.message_signature.id);
            }
        }
        let packet = ClientboundDeleteChatPacket::new(MessageSignaturePacked {
            id: 5,
            full_signature: None,
        });
        let mut listener = MockListener { seen_id: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen_id, Some(5));
    }
}
