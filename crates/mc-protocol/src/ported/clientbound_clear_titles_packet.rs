use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundClearTitlesPacket {
    reset_times: bool,
}

pub trait ClientGamePacketListener {
    fn handle_titles_clear(&mut self, packet: &ClientboundClearTitlesPacket);
}

impl ClientboundClearTitlesPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundClearTitlesPacket#<init>
    pub fn new(reset_times: bool) -> Self {
        Self { reset_times }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundClearTitlesPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Self {
        let reset_times = read_boolean(buf);
        Self { reset_times }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundClearTitlesPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_boolean(buf, self.reset_times);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundClearTitlesPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:clear_titles"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundClearTitlesPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_titles_clear(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundClearTitlesPacket#shouldResetTimes
    pub fn should_reset_times(&self) -> bool {
        self.reset_times
    }
}

fn read_boolean(buf: &mut &[u8]) -> bool {
    buf.get_u8() != 0
}

fn write_boolean(buf: &mut impl BufMut, value: bool) {
    buf.put_u8(if value { 1 } else { 0 });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        called: bool,
        reset: bool,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_titles_clear(&mut self, packet: &ClientboundClearTitlesPacket) {
            self.called = true;
            self.reset = packet.should_reset_times();
        }
    }

    #[test]
    fn roundtrip_true() {
        let pkt = ClientboundClearTitlesPacket::new(true);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundClearTitlesPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
        assert!(decoded.should_reset_times());
        assert!(slice.is_empty());
    }

    #[test]
    fn roundtrip_false() {
        let pkt = ClientboundClearTitlesPacket::new(false);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundClearTitlesPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
        assert!(!decoded.should_reset_times());
    }

    #[test]
    fn read_nonzero_byte_as_true() {
        let data = [2u8];
        let mut slice: &[u8] = &data;
        let pkt = ClientboundClearTitlesPacket::read(&mut slice);
        assert!(pkt.should_reset_times());
    }

    #[test]
    fn handle_dispatches_and_type() {
        let pkt = ClientboundClearTitlesPacket::new(true);
        assert_eq!(pkt.packet_type(), "minecraft:clear_titles");
        let mut listener = MockListener {
            called: false,
            reset: false,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert!(listener.reset);
    }
}
