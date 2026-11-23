use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub value: String,
}

impl ResourceLocation {
    // JAVA: net.minecraft.resources.Identifier#<init>(String)
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSelectAdvancementsTabPacket {
    tab: Option<ResourceLocation>,
}

impl ClientboundSelectAdvancementsTabPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSelectAdvancementsTabPacket#<init>(Identifier)
    pub fn new(tab: Option<ResourceLocation>) -> Self {
        Self { tab }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSelectAdvancementsTabPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let tab = read_nullable(buf, |b| read_identifier(b));
        Self { tab }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSelectAdvancementsTabPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_nullable(buf, self.tab.as_ref(), |b, id| write_identifier(b, id));
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSelectAdvancementsTabPacket#getTab
    pub fn get_tab(&self) -> Option<&ResourceLocation> {
        self.tab.as_ref()
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_varint(buf: &mut impl Buf) -> i32 {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32) << shift;
        if (b & 0x80) == 0 {
            break;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            panic!("VarInt too big");
        }
    }
    result
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_varint(buf: &mut impl BufMut, mut value: i32) {
    loop {
        let mut b = (value as u32 & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            b |= 0x80;
        }
        buf.put_u8(b);
        if value == 0 {
            break;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readUtf(int)
fn read_utf(buf: &mut impl Buf, max_len: usize) -> String {
    let byte_len = read_varint(buf) as usize;
    let max_bytes = max_len.saturating_mul(3);
    if byte_len > max_bytes {
        panic!("String too long (bytes)");
    }
    let mut bytes = vec![0u8; byte_len];
    buf.copy_to_slice(&mut bytes);
    let s = String::from_utf8(bytes).expect("invalid utf8");
    if s.chars().count() > max_len {
        panic!("String too long (chars)");
    }
    s
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf(String,int)
fn write_utf(buf: &mut impl BufMut, s: &str, max_len: usize) {
    if s.chars().count() > max_len {
        panic!("String too long");
    }
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readIdentifier
fn read_identifier(buf: &mut impl Buf) -> ResourceLocation {
    ResourceLocation::new(read_utf(buf, 32767))
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeIdentifier
fn write_identifier(buf: &mut impl BufMut, id: &ResourceLocation) {
    write_utf(buf, &id.value, 32767);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readNullable
fn read_nullable<T, B: Buf>(buf: &mut B, reader: impl FnOnce(&mut B) -> T) -> Option<T> {
    if buf.get_u8() != 0 {
        Some(reader(buf))
    } else {
        None
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeNullable
fn write_nullable<T, B: BufMut>(buf: &mut B, val: Option<&T>, writer: impl FnOnce(&mut B, &T)) {
    match val {
        Some(v) => {
            buf.put_u8(1);
            writer(buf, v);
        }
        None => {
            buf.put_u8(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_with_tab() {
        let pkt = ClientboundSelectAdvancementsTabPacket::new(Some(ResourceLocation::new(
            "minecraft:story/root",
        )));
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSelectAdvancementsTabPacket::read(&mut reader);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_tab().unwrap().value, "minecraft:story/root");
    }

    #[test]
    fn roundtrip_without_tab() {
        let pkt = ClientboundSelectAdvancementsTabPacket::new(None);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSelectAdvancementsTabPacket::read(&mut reader);
        assert_eq!(pkt, decoded);
        assert!(decoded.get_tab().is_none());
    }

    #[test]
    fn write_none_encodes_single_zero_byte() {
        let pkt = ClientboundSelectAdvancementsTabPacket::new(None);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        assert_eq!(&buf[..], &[0x00]);
    }

    #[test]
    fn write_some_encodes_present_flag_and_identifier() {
        let pkt = ClientboundSelectAdvancementsTabPacket::new(Some(ResourceLocation::new("a:b")));
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let bytes = buf.freeze();
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[1], 3); // varint length of "a:b"
        assert_eq!(&bytes[2..5], b"a:b");
    }

    #[test]
    fn varint_negative_roundtrip() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, -1);
        write_varint(&mut buf, i32::MIN);
        let mut reader = buf.freeze();
        assert_eq!(read_varint(&mut reader), -1);
        assert_eq!(read_varint(&mut reader), i32::MIN);
    }
}
