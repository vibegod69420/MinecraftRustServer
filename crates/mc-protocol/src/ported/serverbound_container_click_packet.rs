use bytes::{Buf, BufMut, BytesMut};
use std::collections::BTreeMap;

const MAX_SLOT_COUNT: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError(pub String);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DecodeError {}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_varint(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut value: i32 = 0;
    let mut position: i32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(DecodeError("VarInt too short".into()));
        }
        let read = buf.get_u8();
        value |= ((read & 0x7F) as i32) << position;
        if (read & 0x80) == 0 {
            return Ok(value);
        }
        position += 7;
        if position >= 32 {
            return Err(DecodeError("VarInt too big".into()));
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_varint(buf: &mut BytesMut, mut value: i32) {
    loop {
        let mut temp = (value as u32 & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            temp |= 0x80;
        }
        buf.put_u8(temp);
        if value == 0 {
            return;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readShort
fn read_short(buf: &mut &[u8]) -> Result<i16, DecodeError> {
    if buf.remaining() < 2 {
        return Err(DecodeError("Short underflow".into()));
    }
    Ok(buf.get_i16())
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readByte
fn read_byte(buf: &mut &[u8]) -> Result<i8, DecodeError> {
    if !buf.has_remaining() {
        return Err(DecodeError("Byte underflow".into()));
    }
    Ok(buf.get_i8())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickType {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

impl ClickType {
    // JAVA: net.minecraft.world.inventory.ClickType#ordinal
    pub fn id(self) -> i32 {
        match self {
            Self::Pickup => 0,
            Self::QuickMove => 1,
            Self::Swap => 2,
            Self::Clone => 3,
            Self::Throw => 4,
            Self::QuickCraft => 5,
            Self::PickupAll => 6,
        }
    }

    // JAVA: net.minecraft.world.inventory.ClickType#values
    pub fn from_id(id: i32) -> Result<Self, DecodeError> {
        match id {
            0 => Ok(Self::Pickup),
            1 => Ok(Self::QuickMove),
            2 => Ok(Self::Swap),
            3 => Ok(Self::Clone),
            4 => Ok(Self::Throw),
            5 => Ok(Self::QuickCraft),
            6 => Ok(Self::PickupAll),
            _ => Err(DecodeError(format!("Unknown ClickType id: {id}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedActualItem {
    pub item_id: i32,
    pub count: i32,
    pub added_components: BTreeMap<i32, i32>,
    pub removed_components: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HashedStack {
    #[default]
    Empty,
    Actual(HashedActualItem),
}

// JAVA: net.minecraft.network.HashedStack#read
fn read_hashed_stack(buf: &mut &[u8]) -> Result<HashedStack, DecodeError> {
    if !buf.has_remaining() {
        return Err(DecodeError("HashedStack underflow".into()));
    }
    let present = buf.get_u8();
    if present == 0 {
        return Ok(HashedStack::Empty);
    }
    let item_id = read_varint(buf)?;
    let count = read_varint(buf)?;

    let added_len = read_varint(buf)? as usize;
    if added_len > 256 {
        return Err(DecodeError("Too many added components".into()));
    }
    let mut added_components = BTreeMap::new();
    for _ in 0..added_len {
        let key = read_varint(buf)?;
        if buf.remaining() < 4 {
            return Err(DecodeError("Component hash underflow".into()));
        }
        let hash = buf.get_i32();
        added_components.insert(key, hash);
    }

    let removed_len = read_varint(buf)? as usize;
    if removed_len > 256 {
        return Err(DecodeError("Too many removed components".into()));
    }
    let mut removed_components = Vec::with_capacity(removed_len);
    for _ in 0..removed_len {
        removed_components.push(read_varint(buf)?);
    }

    Ok(HashedStack::Actual(HashedActualItem {
        item_id,
        count,
        added_components,
        removed_components,
    }))
}

// JAVA: net.minecraft.network.HashedStack#write
fn write_hashed_stack(buf: &mut BytesMut, stack: &HashedStack) {
    match stack {
        HashedStack::Empty => {
            buf.put_u8(0);
        }
        HashedStack::Actual(item) => {
            buf.put_u8(1);
            write_varint(buf, item.item_id);
            write_varint(buf, item.count);
            write_varint(buf, item.added_components.len() as i32);
            for (&k, &v) in &item.added_components {
                write_varint(buf, k);
                buf.put_i32(v);
            }
            write_varint(buf, item.removed_components.len() as i32);
            for &k in &item.removed_components {
                write_varint(buf, k);
            }
        }
    }
}

// JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#SLOTS_STREAM_CODEC
fn read_changed_slots(buf: &mut &[u8]) -> Result<BTreeMap<i32, HashedStack>, DecodeError> {
    let count = read_varint(buf)? as usize;
    if count > MAX_SLOT_COUNT {
        return Err(DecodeError(format!("Too many slots: {count}")));
    }
    let mut map = BTreeMap::new();
    for _ in 0..count {
        let key = read_short(buf)? as i32;
        let value = read_hashed_stack(buf)?;
        map.insert(key, value);
    }
    Ok(map)
}

// JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#SLOTS_STREAM_CODEC
fn write_changed_slots(buf: &mut BytesMut, slots: &BTreeMap<i32, HashedStack>) {
    write_varint(buf, slots.len() as i32);
    for (&key, value) in slots {
        buf.put_i16(key as i16);
        write_hashed_stack(buf, value);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundContainerClickPacket {
    container_id: i32,
    state_id: i32,
    slot_num: i16,
    button_num: i8,
    click_type: ClickType,
    changed_slots: BTreeMap<i32, HashedStack>,
    carried_item: HashedStack,
}

impl ServerboundContainerClickPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#<init>
    pub fn new(
        container_id: i32,
        state_id: i32,
        slot_num: i16,
        button_num: i8,
        click_type: ClickType,
        changed_slots: BTreeMap<i32, HashedStack>,
        carried_item: HashedStack,
    ) -> Self {
        Self {
            container_id,
            state_id,
            slot_num,
            button_num,
            click_type,
            changed_slots,
            carried_item,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let container_id = read_varint(buf)?;
        let state_id = read_varint(buf)?;
        let slot_num = read_short(buf)?;
        let button_num = read_byte(buf)?;
        let click_type = ClickType::from_id(read_varint(buf)?)?;
        let changed_slots = read_changed_slots(buf)?;
        let carried_item = read_hashed_stack(buf)?;
        Ok(Self::new(
            container_id,
            state_id,
            slot_num,
            button_num,
            click_type,
            changed_slots,
            carried_item,
        ))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#write
    pub fn write(&self, buf: &mut BytesMut) {
        write_varint(buf, self.container_id);
        write_varint(buf, self.state_id);
        buf.put_i16(self.slot_num);
        buf.put_i8(self.button_num);
        write_varint(buf, self.click_type.id());
        write_changed_slots(buf, &self.changed_slots);
        write_hashed_stack(buf, &self.carried_item);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#containerId
    pub fn container_id(&self) -> i32 {
        self.container_id
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#stateId
    pub fn state_id(&self) -> i32 {
        self.state_id
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#slotNum
    pub fn slot_num(&self) -> i16 {
        self.slot_num
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#buttonNum
    pub fn button_num(&self) -> i8 {
        self.button_num
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#clickType
    pub fn click_type(&self) -> ClickType {
        self.click_type
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#changedSlots
    pub fn changed_slots(&self) -> &BTreeMap<i32, HashedStack> {
        &self.changed_slots
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClickPacket#carriedItem
    pub fn carried_item(&self) -> &HashedStack {
        &self.carried_item
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(pkt: &ServerboundContainerClickPacket) -> ServerboundContainerClickPacket {
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundContainerClickPacket::read(&mut slice).unwrap();
        assert!(!slice.has_remaining());
        decoded
    }

    #[test]
    fn round_trip_basic_empty_slots() {
        let pkt = ServerboundContainerClickPacket::new(
            1,
            2,
            5,
            0,
            ClickType::Pickup,
            BTreeMap::new(),
            HashedStack::Empty,
        );
        assert_eq!(round_trip(&pkt), pkt);
    }

    #[test]
    fn round_trip_with_slots_and_item() {
        let mut slots = BTreeMap::new();
        slots.insert(0, HashedStack::Empty);
        slots.insert(
            5,
            HashedStack::Actual(HashedActualItem {
                item_id: 42,
                count: 64,
                added_components: BTreeMap::from([(1, 10), (2, -5)]),
                removed_components: vec![3, 4],
            }),
        );
        let pkt = ServerboundContainerClickPacket::new(
            7,
            100,
            -999,
            -1,
            ClickType::QuickMove,
            slots,
            HashedStack::Actual(HashedActualItem {
                item_id: 1,
                count: 1,
                added_components: BTreeMap::new(),
                removed_components: vec![],
            }),
        );
        assert_eq!(round_trip(&pkt), pkt);
    }

    #[test]
    fn round_trip_negative_ids_and_extreme_values() {
        let pkt = ServerboundContainerClickPacket::new(
            -1,
            i32::MIN,
            i16::MIN,
            i8::MIN,
            ClickType::Throw,
            BTreeMap::new(),
            HashedStack::Empty,
        );
        let decoded = round_trip(&pkt);
        assert_eq!(decoded.container_id(), -1);
        assert_eq!(decoded.state_id(), i32::MIN);
        assert_eq!(decoded.slot_num(), i16::MIN);
        assert_eq!(decoded.button_num(), i8::MIN);
    }

    #[test]
    fn rejects_too_many_slots() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, 0); // container_id
        write_varint(&mut buf, 0); // state_id
        buf.put_i16(0); // slot_num
        buf.put_i8(0); // button_num
        write_varint(&mut buf, 0); // click_type
        write_varint(&mut buf, (MAX_SLOT_COUNT as i32) + 1); // too many slots
        let mut slice: &[u8] = &buf;
        let err = ServerboundContainerClickPacket::read(&mut slice).unwrap_err();
        assert!(err.to_string().contains("Too many slots"));
    }

    #[test]
    fn invalid_click_type_errors() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, 0);
        write_varint(&mut buf, 0);
        buf.put_i16(0);
        buf.put_i8(0);
        write_varint(&mut buf, 99); // invalid click type
        let mut slice: &[u8] = &buf;
        assert!(ServerboundContainerClickPacket::read(&mut slice).is_err());
    }
}
