use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    pub message: String,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecodeError {}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
fn read_boolean(buf: &mut &[u8]) -> Result<bool, DecodeError> {
    if buf.is_empty() {
        return Err(DecodeError {
            message: "Not enough bytes for boolean".to_string(),
        });
    }
    let b = buf[0];
    *buf = &buf[1..];
    Ok(b != 0)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
fn write_boolean(buf: &mut Vec<u8>, value: bool) {
    buf.push(if value { 1 } else { 0 });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPaddleBoatPacket {
    pub left: bool,
    pub right: bool,
}

impl ServerboundPaddleBoatPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPaddleBoatPacket#<init>
    pub fn new(left: bool, right: bool) -> Self {
        Self { left, right }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPaddleBoatPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let left = read_boolean(buf)?;
        let right = read_boolean(buf)?;
        Ok(Self { left, right })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPaddleBoatPacket#write
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_boolean(buf, self.left);
        write_boolean(buf, self.right);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPaddleBoatPacket#type
    pub fn packet_type() -> &'static str {
        "minecraft:paddle_boat"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPaddleBoatPacket#getLeft
    pub fn get_left(&self) -> bool {
        self.left
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPaddleBoatPacket#getRight
    pub fn get_right(&self) -> bool {
        self.right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_all_combinations() {
        for left in [false, true] {
            for right in [false, true] {
                let pkt = ServerboundPaddleBoatPacket::new(left, right);
                let mut buf = Vec::new();
                pkt.write(&mut buf);
                let mut slice: &[u8] = &buf;
                let decoded = ServerboundPaddleBoatPacket::read(&mut slice).unwrap();
                assert_eq!(decoded, pkt);
                assert_eq!(decoded.get_left(), left);
                assert_eq!(decoded.get_right(), right);
                assert!(slice.is_empty());
            }
        }
    }

    #[test]
    fn wire_encoding() {
        let pkt = ServerboundPaddleBoatPacket::new(true, false);
        let mut buf = Vec::new();
        pkt.write(&mut buf);
        assert_eq!(buf, vec![1, 0]);
    }

    #[test]
    fn read_nonzero_as_true() {
        let data = [2u8, 255u8];
        let mut slice: &[u8] = &data;
        let pkt = ServerboundPaddleBoatPacket::read(&mut slice).unwrap();
        assert!(pkt.left);
        assert!(pkt.right);
    }

    #[test]
    fn short_buffer_errors() {
        let mut empty: &[u8] = &[];
        assert!(ServerboundPaddleBoatPacket::read(&mut empty).is_err());
        let mut one: &[u8] = &[1u8];
        assert!(ServerboundPaddleBoatPacket::read(&mut one).is_err());
    }
}
