use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    pub data: BytesMut,
    pub reader_index: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: BytesMut) -> Self {
        Self {
            data,
            reader_index: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
    pub fn read_boolean(&mut self) -> bool {
        let b = self.data[self.reader_index];
        self.reader_index += 1;
        b != 0
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_boolean(&mut self, value: bool) {
        self.data.put_u8(if value { 1 } else { 0 });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub const SERVERBOUND_LOCK_DIFFICULTY_TYPE: PacketType = PacketType {
    id: "minecraft:lock_difficulty",
};

pub trait ServerGamePacketListener {
    fn handle_lock_difficulty(&mut self, packet: &ServerboundLockDifficultyPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundLockDifficultyPacket {
    locked: bool,
}

impl ServerboundLockDifficultyPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundLockDifficultyPacket#<init>
    pub fn new(locked: bool) -> Self {
        Self { locked }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundLockDifficultyPacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        Self {
            locked: buf.read_boolean(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundLockDifficultyPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_boolean(self.locked);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundLockDifficultyPacket#type
    pub fn get_type(&self) -> PacketType {
        SERVERBOUND_LOCK_DIFFICULTY_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundLockDifficultyPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_lock_difficulty(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundLockDifficultyPacket#isLocked
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_true() {
        let pkt = ServerboundLockDifficultyPacket::new(true);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf);
        buf.reader_index = 0;
        let decoded = ServerboundLockDifficultyPacket::from_buf(&mut buf);
        assert_eq!(pkt, decoded);
        assert!(decoded.is_locked());
    }

    #[test]
    fn round_trip_false() {
        let pkt = ServerboundLockDifficultyPacket::new(false);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf);
        buf.reader_index = 0;
        let decoded = ServerboundLockDifficultyPacket::from_buf(&mut buf);
        assert_eq!(pkt, decoded);
        assert!(!decoded.is_locked());
    }

    #[test]
    fn read_boolean_treats_nonzero_as_true() {
        let mut buf = FriendlyByteBuf::default();
        buf.data.put_u8(2);
        buf.reader_index = 0;
        assert!(buf.read_boolean());

        let mut buf2 = FriendlyByteBuf::default();
        buf2.data.put_u8(0xFF);
        buf2.reader_index = 0;
        assert!(buf2.read_boolean());
    }

    #[test]
    fn write_boolean_encodes_zero_or_one() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_boolean(true);
        buf.write_boolean(false);
        assert_eq!(buf.data.as_ref(), &[1, 0]);
    }

    struct MockListener {
        called: bool,
        locked: bool,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_lock_difficulty(&mut self, packet: &ServerboundLockDifficultyPacket) {
            self.called = true;
            self.locked = packet.is_locked();
        }
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let pkt = ServerboundLockDifficultyPacket::new(true);
        let mut listener = MockListener {
            called: false,
            locked: false,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert!(listener.locked);
        assert_eq!(pkt.get_type(), SERVERBOUND_LOCK_DIFFICULTY_TYPE);
    }
}
