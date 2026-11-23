//! Port of `net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket`.

use std::fmt;
use std::io;

/// JAVA: net.minecraft.world.entity.EquipmentSlot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    Mainhand,
    Offhand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}

impl EquipmentSlot {
    /// JAVA: net.minecraft.world.entity.EquipmentSlot#VALUES
    pub const VALUES: [EquipmentSlot; 8] = [
        EquipmentSlot::Mainhand,
        EquipmentSlot::Offhand,
        EquipmentSlot::Feet,
        EquipmentSlot::Legs,
        EquipmentSlot::Chest,
        EquipmentSlot::Head,
        EquipmentSlot::Body,
        EquipmentSlot::Saddle,
    ];

    /// JAVA: net.minecraft.world.entity.EquipmentSlot#ordinal()
    pub fn ordinal(self) -> i32 {
        match self {
            EquipmentSlot::Mainhand => 0,
            EquipmentSlot::Offhand => 1,
            EquipmentSlot::Feet => 2,
            EquipmentSlot::Legs => 3,
            EquipmentSlot::Chest => 4,
            EquipmentSlot::Head => 5,
            EquipmentSlot::Body => 6,
            EquipmentSlot::Saddle => 7,
        }
    }

    /// JAVA: net.minecraft.world.entity.EquipmentSlot#VALUES.get(int)
    pub fn from_id(id: i32) -> Option<Self> {
        if id < 0 {
            return None;
        }
        Self::VALUES.get(id as usize).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    pub message: String,
}

impl DecodeError {
    fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecodeError {}

impl From<io::Error> for DecodeError {
    fn from(e: io::Error) -> Self {
        Self::new(e.to_string())
    }
}

fn read_byte(buf: &mut &[u8]) -> Result<i8, DecodeError> {
    if buf.is_empty() {
        return Err(DecodeError::new("EOF reading byte"));
    }
    let b = buf[0] as i8;
    *buf = &buf[1..];
    Ok(b)
}

fn read_unsigned_byte(buf: &mut &[u8]) -> Result<u8, DecodeError> {
    if buf.is_empty() {
        return Err(DecodeError::new("EOF reading unsigned byte"));
    }
    let b = buf[0];
    *buf = &buf[1..];
    Ok(b)
}

/// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
fn read_varint(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    loop {
        let b = read_byte(buf)?;
        i |= ((b & 127) as i32) << (j.wrapping_mul(7));
        j = j.wrapping_add(1);
        if j > 5 {
            return Err(DecodeError::new("VarInt too big"));
        }
        if (b & -128) != -128 {
            break;
        }
    }
    Ok(i)
}

/// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
fn write_varint(buf: &mut Vec<u8>, mut value: i32) {
    loop {
        let mut b = (value & 0x7F) as u8;
        let u = value as u32;
        value = (u >> 7) as i32;
        if value != 0 {
            b |= 0x80;
        }
        buf.push(b);
        if value == 0 {
            break;
        }
    }
}

/// Minimal self-contained ItemStack representation for packet codec.
/// JAVA: net.minecraft.world.item.ItemStack
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    /// Item count; <= 0 represents empty stack in OPTIONAL_STREAM_CODEC.
    pub count: i32,
    /// Item registry id when present.
    pub item_id: i32,
    /// Number of added/set components in patch (faithful field for empty-patch case).
    pub added_components_count: i32,
    /// Number of removed components in patch (faithful field for empty-patch case).
    pub removed_components_count: i32,
}

impl ItemStack {
    /// JAVA: net.minecraft.world.item.ItemStack#EMPTY
    pub fn empty() -> Self {
        Self {
            count: 0,
            item_id: 0,
            added_components_count: 0,
            removed_components_count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0
    }

    /// JAVA: net.minecraft.world.item.ItemStack#OPTIONAL_STREAM_CODEC decode
    /// Faithful for empty stacks and stacks with empty component patches.
    pub fn decode_optional(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let count = read_varint(buf)?;
        if count <= 0 {
            return Ok(Self::empty());
        }
        let item_id = read_varint(buf)?;
        let added_components_count = read_varint(buf)?;
        let removed_components_count = read_varint(buf)?;
        if added_components_count != 0 || removed_components_count != 0 {
            return Err(DecodeError::new(
                "ItemStack component patches with entries are not supported in this self-contained port",
            ));
        }
        Ok(Self {
            count,
            item_id,
            added_components_count,
            removed_components_count,
        })
    }

    /// JAVA: net.minecraft.world.item.ItemStack#OPTIONAL_STREAM_CODEC encode
    pub fn encode_optional(&self, buf: &mut Vec<u8>) -> Result<(), DecodeError> {
        if self.is_empty() {
            write_varint(buf, 0);
            return Ok(());
        }
        write_varint(buf, self.count);
        write_varint(buf, self.item_id);
        write_varint(buf, self.added_components_count);
        write_varint(buf, self.removed_components_count);
        if self.added_components_count != 0 || self.removed_components_count != 0 {
            return Err(DecodeError::new(
                "ItemStack component patches with entries are not supported in this self-contained port",
            ));
        }
        Ok(())
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetEquipmentPacket {
    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#entity
    pub entity: i32,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#slots
    pub slots: Vec<(EquipmentSlot, ItemStack)>,
}

impl ClientboundSetEquipmentPacket {
    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#CONTINUE_MASK
    pub const CONTINUE_MASK: i8 = -128;

    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#<init>(int, java.util.List)
    pub fn new(entity: i32, slots: Vec<(EquipmentSlot, ItemStack)>) -> Self {
        Self { entity, slots }
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#<init>(net.minecraft.network.RegistryFriendlyByteBuf)
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let entity = read_varint(buf)?;
        let mut slots: Vec<(EquipmentSlot, ItemStack)> = Vec::new();
        loop {
            let b = read_byte(buf)?;
            let slot_index = (b & 127) as i32;
            let slot = EquipmentSlot::from_id(slot_index).ok_or_else(|| {
                DecodeError::new(format!("Invalid equipment slot id: {slot_index}"))
            })?;
            let item = ItemStack::decode_optional(buf)?;
            slots.push((slot, item));
            if (b & Self::CONTINUE_MASK) != Self::CONTINUE_MASK {
                break;
            }
        }
        Ok(Self { entity, slots })
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#write(net.minecraft.network.RegistryFriendlyByteBuf)
    pub fn write(&self, buf: &mut Vec<u8>) -> Result<(), DecodeError> {
        write_varint(buf, self.entity);
        let len = self.slots.len();
        for i in 0..len {
            let (slot, item) = &self.slots[i];
            let has_next = i != len.wrapping_sub(1);
            let ordinal = slot.ordinal();
            let b = if has_next {
                (ordinal | (Self::CONTINUE_MASK as i32)) as i8
            } else {
                ordinal as i8
            };
            buf.push(b as u8);
            item.encode_optional(buf)?;
        }
        Ok(())
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#getEntity()
    pub fn get_entity(&self) -> i32 {
        self.entity
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#getSlots()
    pub fn get_slots(&self) -> &[(EquipmentSlot, ItemStack)] {
        &self.slots
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#type()
    pub fn type_id(&self) -> &'static str {
        "minecraft:set_equipment"
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket#handle(net.minecraft.network.protocol.game.ClientGamePacketListener)
    pub fn handle<H: FnOnce(&Self)>(&self, handler: H) {
        handler(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(packet: &ClientboundSetEquipmentPacket) -> ClientboundSetEquipmentPacket {
        let mut buf = Vec::new();
        packet.write(&mut buf).expect("write");
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSetEquipmentPacket::read(&mut slice).expect("read");
        assert!(slice.is_empty(), "all bytes should be consumed");
        decoded
    }

    #[test]
    fn roundtrip_single_empty_slot() {
        let packet = ClientboundSetEquipmentPacket::new(
            42,
            vec![(EquipmentSlot::Mainhand, ItemStack::empty())],
        );
        let decoded = roundtrip(&packet);
        assert_eq!(decoded, packet);
        assert_eq!(decoded.get_entity(), 42);
        assert_eq!(decoded.get_slots().len(), 1);
        assert_eq!(decoded.get_slots()[0].0, EquipmentSlot::Mainhand);
        assert!(decoded.get_slots()[0].1.is_empty());
    }

    #[test]
    fn roundtrip_multiple_slots_continue_mask() {
        let packet = ClientboundSetEquipmentPacket::new(
            -7,
            vec![
                (EquipmentSlot::Head, ItemStack::empty()),
                (
                    EquipmentSlot::Chest,
                    ItemStack {
                        count: 1,
                        item_id: 5,
                        added_components_count: 0,
                        removed_components_count: 0,
                    },
                ),
                (EquipmentSlot::Offhand, ItemStack::empty()),
            ],
        );
        let decoded = roundtrip(&packet);
        assert_eq!(decoded, packet);
        assert_eq!(decoded.get_slots().len(), 3);
    }

    #[test]
    fn wire_continue_mask_matches_java() {
        let packet = ClientboundSetEquipmentPacket::new(
            1,
            vec![
                (EquipmentSlot::Feet, ItemStack::empty()),
                (EquipmentSlot::Legs, ItemStack::empty()),
            ],
        );
        let mut buf = Vec::new();
        packet.write(&mut buf).expect("write");
        // entity varint = 1 -> [0x01]
        assert_eq!(buf[0], 0x01);
        // first slot: Feet ordinal 2, has next => 2 | 0x80 = 0x82
        assert_eq!(buf[1], 0x82);
        // empty itemstack -> varint 0
        assert_eq!(buf[2], 0x00);
        // second slot: Legs ordinal 3, last => 0x03
        assert_eq!(buf[3], 0x03);
        assert_eq!(buf[4], 0x00);
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn decode_reads_until_continue_mask_cleared() {
        // entity=2, two entries: Mainhand (0) with continue, Offhand (1) last, both empty
        let bytes: &[u8] = &[0x02, 0x80, 0x00, 0x01, 0x00];
        let mut slice = bytes;
        let packet = ClientboundSetEquipmentPacket::read(&mut slice).expect("read");
        assert!(slice.is_empty());
        assert_eq!(packet.get_entity(), 2);
        assert_eq!(
            packet.get_slots(),
            &[
                (EquipmentSlot::Mainhand, ItemStack::empty()),
                (EquipmentSlot::Offhand, ItemStack::empty()),
            ]
        );
    }

    #[test]
    fn equipment_slot_from_id_negative_and_out_of_range() {
        assert_eq!(EquipmentSlot::from_id(-1), None);
        assert_eq!(EquipmentSlot::from_id(8), None);
        assert_eq!(EquipmentSlot::from_id(i32::MIN), None);
        assert_eq!(EquipmentSlot::from_id(7), Some(EquipmentSlot::Saddle));
    }

    #[test]
    fn varint_roundtrip_negative_and_extremes() {
        for value in [0, -1, i32::MIN, i32::MAX, 127, 128, -128] {
            let mut buf = Vec::new();
            write_varint(&mut buf, value);
            let mut slice: &[u8] = &buf;
            let decoded = read_varint(&mut slice).expect("decode");
            assert!(slice.is_empty());
            assert_eq!(decoded, value, "varint roundtrip failed for {value}");
        }
    }
}
