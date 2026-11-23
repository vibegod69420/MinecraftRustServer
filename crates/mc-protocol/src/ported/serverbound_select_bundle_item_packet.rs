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

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut i: i32 = 0;
    let mut j: u32 = 0;
    loop {
        if buf.is_empty() {
            return Err(DecodeError {
                message: "VarInt too short".to_string(),
            });
        }
        let b0 = buf[0];
        *buf = &buf[1..];
        i |= ((b0 & 0x7F) as i32).wrapping_shl(j.wrapping_mul(7));
        j = j.wrapping_add(1);
        if j > 5 {
            return Err(DecodeError {
                message: "VarInt too big".to_string(),
            });
        }
        if (b0 & 0x80) == 0 {
            break;
        }
    }
    Ok(i)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int(buf: &mut Vec<u8>, mut value: i32) {
    loop {
        if (value & !0x7F) == 0 {
            buf.push(value as u8);
            return;
        }
        buf.push(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSelectBundleItemPacket {
    pub slot_id: i32,
    pub selected_item_index: i32,
}

impl ServerboundSelectBundleItemPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectBundleItemPacket#<init>
    pub fn new(slot_id: i32, selected_item_index: i32) -> Self {
        Self {
            slot_id,
            selected_item_index,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectBundleItemPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let slot_id = read_var_int(buf)?;
        let selected_item_index = read_var_int(buf)?;
        if selected_item_index < 0 && selected_item_index != -1 {
            return Err(DecodeError {
                message: format!("Invalid selectedItemIndex: {}", selected_item_index),
            });
        }
        Ok(Self {
            slot_id,
            selected_item_index,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectBundleItemPacket#write
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_var_int(buf, self.slot_id);
        write_var_int(buf, self.selected_item_index);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectBundleItemPacket#type
    pub fn packet_type() -> &'static str {
        "minecraft:bundle_item_selected"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let pkt = ServerboundSelectBundleItemPacket::new(5, 3);
        let mut buf = Vec::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundSelectBundleItemPacket::read(&mut slice).unwrap();
        assert_eq!(decoded, pkt);
        assert!(slice.is_empty());
    }

    #[test]
    fn negative_one_selected_is_allowed() {
        let pkt = ServerboundSelectBundleItemPacket::new(0, -1);
        let mut buf = Vec::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundSelectBundleItemPacket::read(&mut slice).unwrap();
        assert_eq!(decoded.selected_item_index, -1);
    }

    #[test]
    fn invalid_negative_selected_rejected() {
        let mut buf = Vec::new();
        write_var_int(&mut buf, 1);
        write_var_int(&mut buf, -2);
        let mut slice: &[u8] = &buf;
        let err = ServerboundSelectBundleItemPacket::read(&mut slice).unwrap_err();
        assert!(err.message.contains("Invalid selectedItemIndex: -2"));
    }

    #[test]
    fn varint_negative_encoding() {
        let mut buf = Vec::new();
        write_var_int(&mut buf, -1);
        assert_eq!(buf, vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        let mut slice: &[u8] = &buf;
        assert_eq!(read_var_int(&mut slice).unwrap(), -1);
        assert!(slice.is_empty());
    }

    #[test]
    fn extreme_slot_id_values() {
        for slot_id in [i32::MIN, -1, 0, i32::MAX] {
            let pkt = ServerboundSelectBundleItemPacket::new(slot_id, -1);
            let mut buf = Vec::new();
            pkt.write(&mut buf);
            let mut slice: &[u8] = &buf;
            let decoded = ServerboundSelectBundleItemPacket::read(&mut slice).unwrap();
            assert_eq!(decoded.slot_id, slot_id);
        }
    }
}
