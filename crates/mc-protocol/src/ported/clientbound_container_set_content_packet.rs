use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PacketDecodeError {
    #[error("unexpected end of buffer")]
    Eof,
    #[error("varint too large")]
    VarIntTooLarge,
    #[error("invalid item stack: {0}")]
    InvalidItemStack(String),
}

pub type DecodeResult<T> = Result<T, PacketDecodeError>;

/// Minimal byte buffer matching the subset of RegistryFriendlyByteBuf used by this packet.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl fmt::Debug for RegistryFriendlyByteBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistryFriendlyByteBuf")
            .field("data", &self.data)
            .field("pos", &self.pos)
            .finish()
    }
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    fn read_u8(&mut self) -> DecodeResult<u8> {
        let b = *self.data.get(self.pos).ok_or(PacketDecodeError::Eof)?;
        self.pos = self.pos.wrapping_add(1);
        Ok(b)
    }

    /// Writes a Java-style VarInt (variable-length i32, logical right shifts).
    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7F_u32) == 0 {
                self.write_u8(val as u8);
                return;
            }
            self.write_u8(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
    }

    /// Reads a Java-style VarInt.
    pub fn read_var_int(&mut self) -> DecodeResult<i32> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_u8()?;
            result |= ((b & 0x7F) as u32).wrapping_shl(shift);
            if (b & 0x80) == 0 {
                return Ok(result as i32);
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                return Err(PacketDecodeError::VarIntTooLarge);
            }
        }
    }

    /// ByteBufCodecs.CONTAINER_ID: unsigned byte on the wire.
    pub fn write_container_id(&mut self, container_id: i32) {
        self.write_u8((container_id & 0xFF) as u8);
    }

    /// ByteBufCodecs.CONTAINER_ID: unsigned byte on the wire.
    pub fn read_container_id(&mut self) -> DecodeResult<i32> {
        Ok(i32::from(self.read_u8()?))
    }
}

/// Minimal self-contained ItemStack for ItemStack.OPTIONAL_STREAM_CODEC.
/// Empty when `count <= 0`. Non-empty stacks encode count, item id, and an
/// empty DataComponentPatch (add=0, remove=0) in this minimal wire format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub count: i32,
    pub item_id: i32,
}

impl ItemStack {
    pub fn empty() -> Self {
        Self {
            count: 0,
            item_id: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0
    }

    pub fn new(count: i32, item_id: i32) -> Self {
        if count <= 0 {
            Self::empty()
        } else {
            Self { count, item_id }
        }
    }
}

fn write_optional_item_stack(buf: &mut RegistryFriendlyByteBuf, stack: &ItemStack) {
    if stack.is_empty() {
        buf.write_var_int(0);
    } else {
        buf.write_var_int(stack.count);
        buf.write_var_int(stack.item_id);
        // DataComponentPatch.EMPTY: add count, remove count
        buf.write_var_int(0);
        buf.write_var_int(0);
    }
}

fn read_optional_item_stack(buf: &mut RegistryFriendlyByteBuf) -> DecodeResult<ItemStack> {
    let count = buf.read_var_int()?;
    if count <= 0 {
        return Ok(ItemStack::empty());
    }
    let item_id = buf.read_var_int()?;
    let add_count = buf.read_var_int()?;
    let remove_count = buf.read_var_int()?;
    if add_count != 0 || remove_count != 0 {
        return Err(PacketDecodeError::InvalidItemStack(
            "non-empty component patch not supported in minimal port".into(),
        ));
    }
    Ok(ItemStack { count, item_id })
}

fn write_optional_item_stack_list(buf: &mut RegistryFriendlyByteBuf, items: &[ItemStack]) {
    buf.write_var_int(items.len() as i32);
    for item in items {
        write_optional_item_stack(buf, item);
    }
}

fn read_optional_item_stack_list(buf: &mut RegistryFriendlyByteBuf) -> DecodeResult<Vec<ItemStack>> {
    let len = buf.read_var_int()?;
    if len < 0 {
        return Err(PacketDecodeError::InvalidItemStack(
            "negative item list length".into(),
        ));
    }
    let mut items = Vec::with_capacity(len as usize);
    let mut i = 0;
    while i < len {
        items.push(read_optional_item_stack(buf)?);
        i = i.wrapping_add(1);
    }
    Ok(items)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_CONTAINER_SET_CONTENT: PacketType = PacketType {
        id: "minecraft:container_set_content",
    };
}

pub trait ClientGamePacketListener {
    fn handle_container_content(&mut self, packet: &ClientboundContainerSetContentPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundContainerSetContentPacket {
    pub container_id: i32,
    pub state_id: i32,
    pub items: Vec<ItemStack>,
    pub carried_item: ItemStack,
}

impl ClientboundContainerSetContentPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket#<init>
    pub fn new(
        container_id: i32,
        state_id: i32,
        items: Vec<ItemStack>,
        carried_item: ItemStack,
    ) -> Self {
        Self {
            container_id,
            state_id,
            items,
            carried_item,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket#containerId
    pub fn container_id(&self) -> i32 {
        self.container_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket#stateId
    pub fn state_id(&self) -> i32 {
        self.state_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket#items
    pub fn items(&self) -> &[ItemStack] {
        &self.items
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket#carriedItem
    pub fn carried_item(&self) -> &ItemStack {
        &self.carried_item
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket#type
    pub fn get_type(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_CONTAINER_SET_CONTENT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_container_content(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket#STREAM_CODEC
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        buf.write_var_int(self.state_id);
        write_optional_item_stack_list(buf, &self.items);
        write_optional_item_stack(buf, &self.carried_item);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket#STREAM_CODEC
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> DecodeResult<Self> {
        let container_id = buf.read_container_id()?;
        let state_id = buf.read_var_int()?;
        let items = read_optional_item_stack_list(buf)?;
        let carried_item = read_optional_item_stack(buf)?;
        Ok(Self::new(container_id, state_id, items, carried_item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_basic_packet() {
        let packet = ClientboundContainerSetContentPacket::new(
            42,
            7,
            vec![
                ItemStack::new(1, 10),
                ItemStack::empty(),
                ItemStack::new(64, 20),
            ],
            ItemStack::new(3, 99),
        );
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundContainerSetContentPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(decoded.container_id(), 42);
        assert_eq!(decoded.state_id(), 7);
        assert_eq!(decoded.items().len(), 3);
        assert_eq!(decoded.carried_item(), &ItemStack::new(3, 99));
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn empty_content_and_container_id_bit_masking() {
        let packet = ClientboundContainerSetContentPacket::new(255, 0, vec![], ItemStack::empty());
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let bytes = buf.into_inner();
        // container_id 255 -> 0xFF, state_id 0 -> 0x00, list len 0 -> 0x00, empty carried -> 0x00
        assert_eq!(bytes, vec![0xFF, 0x00, 0x00, 0x00]);
        let decoded =
            ClientboundContainerSetContentPacket::read(&mut RegistryFriendlyByteBuf::from_vec(
                bytes,
            ))
            .unwrap();
        assert_eq!(decoded, packet);

        let masked = ClientboundContainerSetContentPacket::new(
            -1,
            i32::MIN,
            vec![ItemStack::empty()],
            ItemStack::empty(),
        );
        let mut buf2 = RegistryFriendlyByteBuf::new();
        masked.write(&mut buf2);
        let decoded2 = ClientboundContainerSetContentPacket::read(
            &mut RegistryFriendlyByteBuf::from_vec(buf2.into_inner()),
        )
        .unwrap();
        assert_eq!(decoded2.container_id(), 255); // -1 & 0xFF
        assert_eq!(decoded2.state_id(), i32::MIN);
    }

    #[test]
    fn negative_inputs_and_varint_bit_patterns() {
        // VarInt -1 encodes as 5 bytes: ff ff ff ff 0f
        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_var_int(-1);
        assert_eq!(buf.as_slice(), &[0xff, 0xff, 0xff, 0xff, 0x0f]);
        let mut r = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        assert_eq!(r.read_var_int().unwrap(), -1);

        let packet = ClientboundContainerSetContentPacket::new(
            -256,
            -12345,
            vec![ItemStack::new(-5, 7)], // normalized to empty
            ItemStack {
                count: 1,
                item_id: -42,
            },
        );
        assert!(packet.items()[0].is_empty());
        let mut b = RegistryFriendlyByteBuf::new();
        packet.write(&mut b);
        let decoded = ClientboundContainerSetContentPacket::read(
            &mut RegistryFriendlyByteBuf::from_vec(b.into_inner()),
        )
        .unwrap();
        assert_eq!(decoded.container_id(), 0); // -256 & 0xFF
        assert_eq!(decoded.state_id(), -12345);
        assert_eq!(decoded.carried_item().item_id, -42);
    }

    #[test]
    fn integer_overflow_edge_values_round_trip() {
        let packet = ClientboundContainerSetContentPacket::new(
            i32::MAX,
            i32::MAX,
            vec![ItemStack {
                count: i32::MAX,
                item_id: i32::MIN,
            }],
            ItemStack {
                count: 1,
                item_id: i32::MAX,
            },
        );
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded = ClientboundContainerSetContentPacket::read(
            &mut RegistryFriendlyByteBuf::from_vec(buf.into_inner()),
        )
        .unwrap();
        assert_eq!(decoded.container_id(), i32::MAX & 0xFF);
        assert_eq!(decoded.state_id(), i32::MAX);
        assert_eq!(decoded.items()[0].count, i32::MAX);
        assert_eq!(decoded.items()[0].item_id, i32::MIN);
        assert_eq!(decoded.carried_item().item_id, i32::MAX);
    }

    #[test]
    fn float_bit_patterns_preserved_through_varint_transport() {
        let nan_bits = f32::NAN.to_bits() as i32;
        let neg_zero_bits = (-0.0f32).to_bits() as i32;
        let inf_bits = f32::INFINITY.to_bits() as i32;
        let neg_inf_bits = f32::NEG_INFINITY.to_bits() as i32;

        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_var_int(nan_bits);
        buf.write_var_int(neg_zero_bits);
        buf.write_var_int(inf_bits);
        buf.write_var_int(neg_inf_bits);

        let mut r = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        assert_eq!(f32::from_bits(r.read_var_int().unwrap() as u32).to_bits(), nan_bits as u32);
        assert_eq!(
            f32::from_bits(r.read_var_int().unwrap() as u32).to_bits(),
            neg_zero_bits as u32
        );
        assert_eq!(f32::from_bits(r.read_var_int().unwrap() as u32).to_bits(), inf_bits as u32);
        assert_eq!(
            f32::from_bits(r.read_var_int().unwrap() as u32).to_bits(),
            neg_inf_bits as u32
        );
    }

    #[test]
    fn handle_type_and_truncated_decode_error() {
        struct MockListener {
            called: bool,
            seen_container_id: i32,
        }

        impl ClientGamePacketListener for MockListener {
            fn handle_container_content(&mut self, packet: &ClientboundContainerSetContentPacket) {
                self.called = true;
                self.seen_container_id = packet.container_id();
            }
        }

        let packet =
            ClientboundContainerSetContentPacket::new(5, 1, vec![], ItemStack::empty());
        assert_eq!(
            packet.get_type(),
            GamePacketTypes::CLIENTBOUND_CONTAINER_SET_CONTENT
        );

        let mut listener = MockListener {
            called: false,
            seen_container_id: -1,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.seen_container_id, 5);

        let mut truncated = RegistryFriendlyByteBuf::from_vec(vec![0x01]);
        let err = ClientboundContainerSetContentPacket::read(&mut truncated).unwrap_err();
        assert!(matches!(err, PacketDecodeError::Eof));
    }
}
