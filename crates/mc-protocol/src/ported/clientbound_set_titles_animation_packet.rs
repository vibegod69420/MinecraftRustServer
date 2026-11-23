use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_SET_TITLES_ANIMATION: PacketType =
    PacketType("minecraft:clientbound_set_titles_animation");

pub trait ClientGamePacketListener {
    fn set_titles_animation(&mut self, packet: &ClientboundSetTitlesAnimationPacket);
}

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readableBytes
    pub fn readable_bytes(&self) -> usize {
        self.data.len().saturating_sub(self.read_pos)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writerIndex/getBytes
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readInt
    pub fn read_int(&mut self) -> io::Result<i32> {
        if self.readable_bytes() < 4 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_int"));
        }
        let p = self.read_pos;
        let bytes = [
            self.data[p],
            self.data[p + 1],
            self.data[p + 2],
            self.data[p + 3],
        ];
        self.read_pos += 4;
        Ok(i32::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeInt
    pub fn write_int(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetTitlesAnimationPacket {
    fade_in: i32,
    stay: i32,
    fade_out: i32,
}

impl ClientboundSetTitlesAnimationPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket#<init>
    pub fn new(fade_in: i32, stay: i32, fade_out: i32) -> Self {
        Self {
            fade_in,
            stay,
            fade_out,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let fade_in = buf.read_int()?;
        let stay = buf.read_int()?;
        let fade_out = buf.read_int()?;
        Ok(Self {
            fade_in,
            stay,
            fade_out,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.fade_in);
        buf.write_int(self.stay);
        buf.write_int(self.fade_out);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_SET_TITLES_ANIMATION
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.set_titles_animation(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket#getFadeIn
    pub fn get_fade_in(&self) -> i32 {
        self.fade_in
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket#getStay
    pub fn get_stay(&self) -> i32 {
        self.stay
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket#getFadeOut
    pub fn get_fade_out(&self) -> i32 {
        self.fade_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    }

    impl MockListener {
        fn new() -> Self {
            Self {
                called: false,
                fade_in: 0,
                stay: 0,
                fade_out: 0,
            }
        }
    }

    impl ClientGamePacketListener for MockListener {
        fn set_titles_animation(&mut self, packet: &ClientboundSetTitlesAnimationPacket) {
            self.called = true;
            self.fade_in = packet.get_fade_in();
            self.stay = packet.get_stay();
            self.fade_out = packet.get_fade_out();
        }
    }

    #[test]
    fn roundtrip_basic() {
        let pkt = ClientboundSetTitlesAnimationPacket::new(10, 70, 20);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundSetTitlesAnimationPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn negative_and_extreme_values() {
        let pkt = ClientboundSetTitlesAnimationPacket::new(-1, i32::MIN, i32::MAX);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundSetTitlesAnimationPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.get_fade_in(), -1);
        assert_eq!(decoded.get_stay(), i32::MIN);
        assert_eq!(decoded.get_fade_out(), i32::MAX);
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ClientboundSetTitlesAnimationPacket::new(1, 2, 3);
        let mut listener = MockListener::new();
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!((listener.fade_in, listener.stay, listener.fade_out), (1, 2, 3));
    }

    #[test]
    fn packet_type_matches() {
        let pkt = ClientboundSetTitlesAnimationPacket::new(0, 0, 0);
        assert_eq!(pkt.packet_type(), CLIENTBOUND_SET_TITLES_ANIMATION);
    }

    #[test]
    fn wire_format_is_three_big_endian_ints() {
        let pkt = ClientboundSetTitlesAnimationPacket::new(1, 2, 3);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let bytes = buf.into_vec();
        assert_eq!(bytes.len(), 12);
        assert_eq!(&bytes[0..4], &1i32.to_be_bytes());
        assert_eq!(&bytes[4..8], &2i32.to_be_bytes());
        assert_eq!(&bytes[8..12], &3i32.to_be_bytes());
    }
}
