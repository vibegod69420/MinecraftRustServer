use bytes::{Buf, BufMut, BytesMut};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

impl Uuid {
    // JAVA: java.util.UUID#<init>
    pub fn new(most_significant_bits: i64, least_significant_bits: i64) -> Self {
        Self {
            most_significant_bits,
            least_significant_bits,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ServerCommonPacketListener {
    fn handle_resource_pack_response(&mut self, packet: &ServerboundResourcePackPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    SuccessfullyLoaded,
    Declined,
    FailedDownload,
    Accepted,
    Downloaded,
    InvalidUrl,
    FailedReload,
    Discarded,
}

impl Action {
    // JAVA: net.minecraft.network.protocol.common.ServerboundResourcePackPacket$Action#isTerminal
    pub fn is_terminal(&self) -> bool {
        !matches!(self, Action::Accepted | Action::Downloaded)
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundResourcePackPacket$Action#ordinal
    fn ordinal(self) -> i32 {
        match self {
            Action::SuccessfullyLoaded => 0,
            Action::Declined => 1,
            Action::FailedDownload => 2,
            Action::Accepted => 3,
            Action::Downloaded => 4,
            Action::InvalidUrl => 5,
            Action::FailedReload => 6,
            Action::Discarded => 7,
        }
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundResourcePackPacket$Action#values
    fn from_ordinal(v: i32) -> Result<Self, io::Error> {
        match v {
            0 => Ok(Action::SuccessfullyLoaded),
            1 => Ok(Action::Declined),
            2 => Ok(Action::FailedDownload),
            3 => Ok(Action::Accepted),
            4 => Ok(Action::Downloaded),
            5 => Ok(Action::InvalidUrl),
            6 => Ok(Action::FailedReload),
            7 => Ok(Action::Discarded),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid action ordinal: {v}"),
            )),
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int(buf: &mut &[u8]) -> Result<i32, io::Error> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "varint"));
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32).wrapping_shl(shift);
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "varint too large",
            ));
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int(buf: &mut BytesMut, value: i32) {
    let mut v = value as u32;
    loop {
        if (v & !0x7F_u32) == 0 {
            buf.put_u8(v as u8);
            return;
        }
        buf.put_u8(((v & 0x7F_u32) as u8) | 0x80);
        v >>= 7;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readUUID
fn read_uuid(buf: &mut &[u8]) -> Result<Uuid, io::Error> {
    if buf.remaining() < 16 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "uuid"));
    }
    let most = buf.get_i64();
    let least = buf.get_i64();
    Ok(Uuid::new(most, least))
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeUUID
fn write_uuid(buf: &mut BytesMut, id: Uuid) {
    buf.put_i64(id.most_significant_bits);
    buf.put_i64(id.least_significant_bits);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundResourcePackPacket {
    pub id: Uuid,
    pub action: Action,
}

impl ServerboundResourcePackPacket {
    // JAVA: net.minecraft.network.protocol.common.ServerboundResourcePackPacket#<init>(java.util.UUID,net.minecraft.network.protocol.common.ServerboundResourcePackPacket$Action)
    pub fn new(id: Uuid, action: Action) -> Self {
        Self { id, action }
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundResourcePackPacket#<init>(net.minecraft.network.FriendlyByteBuf)
    pub fn read(buf: &mut &[u8]) -> Result<Self, io::Error> {
        let id = read_uuid(buf)?;
        let action = Action::from_ordinal(read_var_int(buf)?)?;
        Ok(Self { id, action })
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundResourcePackPacket#write
    pub fn write(&self, buf: &mut BytesMut) {
        write_uuid(buf, self.id);
        write_var_int(buf, self.action.ordinal());
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundResourcePackPacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:resource_pack")
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundResourcePackPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerCommonPacketListener) {
        listener.handle_resource_pack_response(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_terminal_states() {
        assert!(Action::SuccessfullyLoaded.is_terminal());
        assert!(Action::Declined.is_terminal());
        assert!(Action::FailedDownload.is_terminal());
        assert!(!Action::Accepted.is_terminal());
        assert!(!Action::Downloaded.is_terminal());
        assert!(Action::InvalidUrl.is_terminal());
        assert!(Action::FailedReload.is_terminal());
        assert!(Action::Discarded.is_terminal());
    }

    #[test]
    fn round_trip_with_negative_uuid_bits() {
        let pkt = ServerboundResourcePackPacket::new(
            Uuid::new(-1_i64, i64::MIN),
            Action::FailedReload,
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundResourcePackPacket::read(&mut slice).unwrap();
        assert_eq!(pkt, decoded);
        assert!(!slice.has_remaining());
    }

    #[test]
    fn varint_negative_and_extreme_values() {
        let mut buf = BytesMut::new();
        write_var_int(&mut buf, -1);
        write_var_int(&mut buf, i32::MIN);
        write_var_int(&mut buf, i32::MAX);
        write_var_int(&mut buf, 0);
        let mut slice: &[u8] = &buf;
        assert_eq!(read_var_int(&mut slice).unwrap(), -1);
        assert_eq!(read_var_int(&mut slice).unwrap(), i32::MIN);
        assert_eq!(read_var_int(&mut slice).unwrap(), i32::MAX);
        assert_eq!(read_var_int(&mut slice).unwrap(), 0);
    }

    #[test]
    fn read_invalid_action_ordinal() {
        let mut buf = BytesMut::new();
        write_uuid(&mut buf, Uuid::new(0, 0));
        write_var_int(&mut buf, 99);
        let mut slice: &[u8] = &buf;
        assert!(ServerboundResourcePackPacket::read(&mut slice).is_err());
    }

    #[test]
    fn packet_type_and_handle() {
        struct MockListener {
            seen: Option<Action>,
        }
        impl ServerCommonPacketListener for MockListener {
            fn handle_resource_pack_response(&mut self, packet: &ServerboundResourcePackPacket) {
                self.seen = Some(packet.action);
            }
        }

        let pkt = ServerboundResourcePackPacket::new(Uuid::new(1, 2), Action::Accepted);
        assert_eq!(pkt.get_type(), PacketType("minecraft:resource_pack"));
        let mut listener = MockListener { seen: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen, Some(Action::Accepted));
    }
}
