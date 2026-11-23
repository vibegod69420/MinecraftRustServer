use std::io;

#[derive(Clone, Debug, Default)]
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

    fn read_u8(&mut self) -> io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn write_u8(&mut self, b: u8) -> io::Result<()> {
        self.data.push(b);
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.read_u8()? as i32;
            result |= (byte & 0x7F) << shift;
            if (byte & 0x80) == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 32 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> io::Result<()> {
        loop {
            if (value & !0x7F) == 0 {
                self.write_u8(value as u8)?;
                return Ok(());
            }
            self.write_u8(((value & 0x7F) | 0x80) as u8)?;
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.write_u8(if value { 1 } else { 0 })
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
    pub fn read_utf(&mut self) -> io::Result<String> {
        let len = self.read_var_int()? as usize;
        if self.pos + len > self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "UTF too short"));
        }
        let bytes = &self.data[self.pos..self.pos + len];
        let s = String::from_utf8(bytes.to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.pos += len;
        Ok(s)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
    pub fn write_utf(&mut self, value: &str) -> io::Result<()> {
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32)?;
        self.data.extend_from_slice(bytes);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    // JAVA: net.minecraft.network.chat.ComponentSerialization#TRUSTED_STREAM_CODEC
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        Ok(Self {
            text: buf.read_utf()?,
        })
    }

    // JAVA: net.minecraft.network.chat.ComponentSerialization#TRUSTED_STREAM_CODEC
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_utf(&self.text)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatTypeBound {
    pub chat_type_id: i32,
    pub name: Component,
    pub target_name: Option<Component>,
}

impl ChatTypeBound {
    // JAVA: net.minecraft.network.chat.ChatType$Bound#STREAM_CODEC
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let chat_type_id = buf.read_var_int()?;
        let name = Component::read(buf)?;
        let has_target = buf.read_bool()?;
        let target_name = if has_target {
            Some(Component::read(buf)?)
        } else {
            None
        };
        Ok(Self {
            chat_type_id,
            name,
            target_name,
        })
    }

    // JAVA: net.minecraft.network.chat.ChatType$Bound#STREAM_CODEC
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_var_int(self.chat_type_id)?;
        self.name.write(buf)?;
        buf.write_bool(self.target_name.is_some())?;
        if let Some(target) = &self.target_name {
            target.write(buf)?;
        }
        Ok(())
    }
}

pub trait ClientGamePacketListener {
    fn handle_disguised_chat(&mut self, packet: &ClientboundDisguisedChatPacket);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundDisguisedChatPacket {
    pub message: Component,
    pub chat_type: ChatTypeBound,
}

impl ClientboundDisguisedChatPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundDisguisedChatPacket#<init>
    pub fn new(message: Component, chat_type: ChatTypeBound) -> Self {
        Self { message, chat_type }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDisguisedChatPacket#message
    pub fn message(&self) -> &Component {
        &self.message
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDisguisedChatPacket#chatType
    pub fn chat_type(&self) -> &ChatTypeBound {
        &self.chat_type
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDisguisedChatPacket#STREAM_CODEC
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let message = Component::read(buf)?;
        let chat_type = ChatTypeBound::read(buf)?;
        Ok(Self { message, chat_type })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDisguisedChatPacket#STREAM_CODEC
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        self.message.write(buf)?;
        self.chat_type.write(buf)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDisguisedChatPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:disguised_chat"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDisguisedChatPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_disguised_chat(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDisguisedChatPacket#isSkippable
    pub fn is_skippable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_packet(with_target: bool) -> ClientboundDisguisedChatPacket {
        ClientboundDisguisedChatPacket::new(
            Component {
                text: "hello".to_string(),
            },
            ChatTypeBound {
                chat_type_id: 7,
                name: Component {
                    text: "sender".to_string(),
                },
                target_name: if with_target {
                    Some(Component {
                        text: "target".to_string(),
                    })
                } else {
                    None
                },
            },
        )
    }

    #[test]
    fn roundtrip_without_target() {
        let pkt = sample_packet(false);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundDisguisedChatPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_with_target() {
        let pkt = sample_packet(true);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundDisguisedChatPacket::read(&mut read_buf).unwrap();
        assert_eq!(
            decoded.chat_type().target_name.as_ref().unwrap().text,
            "target"
        );
    }

    #[test]
    fn is_skippable_true() {
        let pkt = sample_packet(false);
        assert!(pkt.is_skippable());
    }

    #[test]
    fn packet_type_name() {
        let pkt = sample_packet(false);
        assert_eq!(pkt.packet_type(), "minecraft:disguised_chat");
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener {
            called: bool,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_disguised_chat(&mut self, _packet: &ClientboundDisguisedChatPacket) {
                self.called = true;
            }
        }
        let pkt = sample_packet(false);
        let mut listener = MockListener { called: false };
        pkt.handle(&mut listener);
        assert!(listener.called);
    }

    #[test]
    fn negative_chat_type_id_roundtrip() {
        let pkt = ClientboundDisguisedChatPacket::new(
            Component {
                text: String::new(),
            },
            ChatTypeBound {
                chat_type_id: -1,
                name: Component {
                    text: "n".to_string(),
                },
                target_name: None,
            },
        );
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundDisguisedChatPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.chat_type().chat_type_id, -1);
    }
}
