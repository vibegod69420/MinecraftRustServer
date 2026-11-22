use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ServerCommonPacketListener {
    fn handle_client_information(&mut self, packet: &ServerboundClientInformationPacket);
}

fn read_varint<B: Buf>(buf: &mut B) -> i32 {
    let mut val: i32 = 0;
    let mut pos: u32 = 0;
    loop {
        let b = buf.get_u8();
        val |= ((b & 0x7F) as i32).wrapping_shl(pos);
        if (b & 0x80) == 0 {
            break;
        }
        pos = pos.wrapping_add(7);
        if pos >= 32 {
            break;
        }
    }
    val
}

fn write_varint<B: BufMut>(buf: &mut B, mut val: i32) {
    loop {
        let mut b = (val as u32 & 0x7F) as u8;
        val = ((val as u32) >> 7) as i32;
        if val != 0 {
            b |= 0x80;
        }
        buf.put_u8(b);
        if val == 0 {
            break;
        }
    }
}

fn read_string<B: Buf>(buf: &mut B, max_chars: usize) -> String {
    let len = read_varint(buf).max(0) as usize;
    let max_bytes = max_chars.saturating_mul(4);
    let read_len = len.min(max_bytes);
    let mut bytes = vec![0u8; read_len];
    if read_len > 0 {
        buf.copy_to_slice(&mut bytes);
    }
    if len > read_len {
        buf.advance(len - read_len);
    }
    String::from_utf8(bytes).unwrap_or_default()
}

fn write_string<B: BufMut>(buf: &mut B, s: &str) {
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInformation {
    pub language: String,
    pub view_distance: i8,
    pub chat_visibility: i32,
    pub chat_colors: bool,
    pub model_customisation: u8,
    pub main_hand: i32,
    pub text_filtering_enabled: bool,
    pub allows_listing: bool,
    pub particle_status: i32,
}

impl ClientInformation {
    // JAVA: net.minecraft.server.level.ClientInformation#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        Self {
            language: read_string(buf, 16),
            view_distance: buf.get_u8() as i8,
            chat_visibility: read_varint(buf),
            chat_colors: buf.get_u8() != 0,
            model_customisation: buf.get_u8(),
            main_hand: read_varint(buf),
            text_filtering_enabled: buf.get_u8() != 0,
            allows_listing: buf.get_u8() != 0,
            particle_status: read_varint(buf),
        }
    }

    // JAVA: net.minecraft.server.level.ClientInformation#write(FriendlyByteBuf)
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_string(buf, &self.language);
        buf.put_u8(self.view_distance as u8);
        write_varint(buf, self.chat_visibility);
        buf.put_u8(u8::from(self.chat_colors));
        buf.put_u8(self.model_customisation);
        write_varint(buf, self.main_hand);
        buf.put_u8(u8::from(self.text_filtering_enabled));
        buf.put_u8(u8::from(self.allows_listing));
        write_varint(buf, self.particle_status);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundClientInformationPacket {
    information: ClientInformation,
}

impl ServerboundClientInformationPacket {
    pub const TYPE: PacketType = PacketType("common/serverbound_client_information");

    // JAVA: net.minecraft.network.protocol.common.ServerboundClientInformationPacket#<init>(ClientInformation)
    pub fn new(information: ClientInformation) -> Self {
        Self { information }
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundClientInformationPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        Self {
            information: ClientInformation::read(buf),
        }
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundClientInformationPacket#write(FriendlyByteBuf)
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        self.information.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundClientInformationPacket#type()
    pub fn packet_type(&self) -> PacketType {
        Self::TYPE
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundClientInformationPacket#handle(ServerCommonPacketListener)
    pub fn handle<L: ServerCommonPacketListener>(&self, listener: &mut L) {
        listener.handle_client_information(self);
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundClientInformationPacket#information()
    pub fn information(&self) -> &ClientInformation {
        &self.information
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        received: Option<ServerboundClientInformationPacket>,
    }

    impl ServerCommonPacketListener for MockListener {
        fn handle_client_information(&mut self, packet: &ServerboundClientInformationPacket) {
            self.received = Some(packet.clone());
        }
    }

    fn sample_info() -> ClientInformation {
        ClientInformation {
            language: "en_us".to_string(),
            view_distance: 12,
            chat_visibility: 0,
            chat_colors: true,
            model_customisation: 127,
            main_hand: 1,
            text_filtering_enabled: false,
            allows_listing: true,
            particle_status: 0,
        }
    }

    #[test]
    fn roundtrip_packet() {
        let pkt = ServerboundClientInformationPacket::new(sample_info());
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundClientInformationPacket::read(&mut buf.freeze());
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.information().language, "en_us");
    }

    #[test]
    fn negative_and_extreme_values() {
        let info = ClientInformation {
            language: String::new(),
            view_distance: -1,
            chat_visibility: -1,
            chat_colors: false,
            model_customisation: u8::MAX,
            main_hand: -1,
            text_filtering_enabled: true,
            allows_listing: false,
            particle_status: i32::MIN,
        };
        let pkt = ServerboundClientInformationPacket::new(info.clone());
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundClientInformationPacket::read(&mut buf.freeze());
        assert_eq!(decoded.information(), &info);
    }

    #[test]
    fn varint_max_and_overflowish_values() {
        let info = ClientInformation {
            language: "ja_jp".to_string(),
            view_distance: i8::MAX,
            chat_visibility: i32::MAX,
            chat_colors: true,
            model_customisation: 0,
            main_hand: i32::MAX,
            text_filtering_enabled: false,
            allows_listing: true,
            particle_status: i32::MAX,
        };
        let pkt = ServerboundClientInformationPacket::new(info.clone());
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundClientInformationPacket::read(&mut buf.freeze());
        assert_eq!(decoded.information(), &info);
    }

    #[test]
    fn handle_dispatches_and_type() {
        let pkt = ServerboundClientInformationPacket::new(sample_info());
        let mut listener = MockListener { received: None };
        pkt.handle(&mut listener);
        let received = listener.received.expect("handler should be called");
        assert_eq!(received.information(), pkt.information());
        assert_eq!(
            pkt.packet_type(),
            PacketType("common/serverbound_client_information")
        );
    }
}
