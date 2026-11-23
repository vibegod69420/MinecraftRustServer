use bytes::{Buf, BufMut};

/// Minimal world border trait for self-contained compilation.
pub trait WorldBorderLike {
    fn get_center_x(&self) -> f64;
    fn get_center_z(&self) -> f64;
}

/// Placeholder packet type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_SET_BORDER_CENTER_TYPE: PacketType =
    PacketType("CLIENTBOUND_SET_BORDER_CENTER");

/// Minimal listener trait for self-contained compilation.
pub trait ClientGamePacketListener {
    fn handle_set_border_center(&mut self, packet: &ClientboundSetBorderCenterPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetBorderCenterPacket {
    new_center_x: f64,
    new_center_z: f64,
}

impl ClientboundSetBorderCenterPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket#<init>
    pub fn new(border: &dyn WorldBorderLike) -> Self {
        Self {
            new_center_x: border.get_center_x(),
            new_center_z: border.get_center_z(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket#<init>
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let new_center_x = buf.get_f64();
        let new_center_z = buf.get_f64();
        Self {
            new_center_x,
            new_center_z,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_f64(self.new_center_x);
        buf.put_f64(self.new_center_z);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_SET_BORDER_CENTER_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_set_border_center(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket#getNewCenterZ
    pub fn get_new_center_z(&self) -> f64 {
        self.new_center_z
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket#getNewCenterX
    pub fn get_new_center_x(&self) -> f64 {
        self.new_center_x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockBorder {
        x: f64,
        z: f64,
    }
    impl WorldBorderLike for MockBorder {
        fn get_center_x(&self) -> f64 {
            self.x
        }
        fn get_center_z(&self) -> f64 {
            self.z
        }
    }

    #[derive(Default)]
    struct MockListener {
        x: f64,
        z: f64,
    }
    impl ClientGamePacketListener for MockListener {
        fn handle_set_border_center(&mut self, packet: &ClientboundSetBorderCenterPacket) {
            self.x = packet.get_new_center_x();
            self.z = packet.get_new_center_z();
        }
    }

    #[test]
    fn test_new_from_border() {
        let pkt = ClientboundSetBorderCenterPacket::new(&MockBorder { x: 100.5, z: -200.25 });
        assert_eq!(pkt.get_new_center_x().to_bits(), 100.5f64.to_bits());
        assert_eq!(pkt.get_new_center_z().to_bits(), (-200.25f64).to_bits());
    }

    #[test]
    fn test_write_read_roundtrip() {
        let pkt = ClientboundSetBorderCenterPacket::new(&MockBorder { x: 1.0, z: 2.0 });
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSetBorderCenterPacket::read(&mut reader);
        assert_eq!(decoded.get_new_center_x().to_bits(), 1.0f64.to_bits());
        assert_eq!(decoded.get_new_center_z().to_bits(), 2.0f64.to_bits());
    }

    #[test]
    fn test_negative_and_extreme_values() {
        let x = -f64::MAX;
        let z = f64::MIN_POSITIVE;
        let pkt = ClientboundSetBorderCenterPacket::new(&MockBorder { x, z });
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSetBorderCenterPacket::read(&mut reader);
        assert_eq!(decoded.get_new_center_x().to_bits(), x.to_bits());
        assert_eq!(decoded.get_new_center_z().to_bits(), z.to_bits());
    }

    #[test]
    fn test_nan_and_infinity_bits_preserved() {
        let x = f64::NAN;
        let z = f64::NEG_INFINITY;
        let pkt = ClientboundSetBorderCenterPacket::new(&MockBorder { x, z });
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSetBorderCenterPacket::read(&mut reader);
        assert_eq!(decoded.get_new_center_x().to_bits(), x.to_bits());
        assert_eq!(decoded.get_new_center_z().to_bits(), z.to_bits());
    }

    #[test]
    fn test_handle_and_type() {
        let pkt = ClientboundSetBorderCenterPacket::new(&MockBorder { x: 3.0, z: 4.0 });
        assert_eq!(pkt.get_type(), CLIENTBOUND_SET_BORDER_CENTER_TYPE);
        let mut listener = MockListener::default();
        pkt.handle(&mut listener);
        assert_eq!(listener.x.to_bits(), 3.0f64.to_bits());
        assert_eq!(listener.z.to_bits(), 4.0f64.to_bits());
    }
}
