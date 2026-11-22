use bytes::{Buf, BufMut, BytesMut};

/// Minimal self-contained buffer for packet encoding/decoding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: BytesMut,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: BytesMut::new(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: BytesMut::from(bytes),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data.to_vec()
    }

    pub fn remaining(&self) -> usize {
        self.data.remaining()
    }

    pub fn read_varint(&mut self) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.data.get_u8() as i32;
            result |= (b & 0x7f).wrapping_shl(shift & 31);
            if (b & 0x80) == 0 {
                return result;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                return result;
            }
        }
    }

    pub fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut b = ((value as u32) & 0x7f) as u8;
            value = ((value as u32) >> 7) as i32;
            if value != 0 {
                b |= 0x80;
            }
            self.data.put_u8(b);
            if value == 0 {
                break;
            }
        }
    }
}

/// Minimal self-contained RecipeDisplayId representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecipeDisplayId {
    index: i32,
}

impl RecipeDisplayId {
    // JAVA: net.minecraft.world.item.crafting.display.RecipeDisplayId#<init>
    pub fn new(index: i32) -> Self {
        Self { index }
    }

    // JAVA: net.minecraft.world.item.crafting.display.RecipeDisplayId#index
    pub fn index(&self) -> i32 {
        self.index
    }

    fn read(buf: &mut FriendlyByteBuf) -> Self {
        Self::new(buf.read_varint())
    }

    fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_varint(self.index);
    }
}

pub const PACKET_TYPE_SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE: &str =
    "minecraft:serverbound_recipe_book_seen_recipe";

pub trait ServerGamePacketListener {
    fn handle_recipe_book_seen_recipe_packet(
        &mut self,
        packet: &ServerboundRecipeBookSeenRecipePacket,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundRecipeBookSeenRecipePacket {
    recipe: RecipeDisplayId,
}

impl ServerboundRecipeBookSeenRecipePacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookSeenRecipePacket#<init>
    pub fn new(recipe: RecipeDisplayId) -> Self {
        Self { recipe }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookSeenRecipePacket#recipe
    pub fn recipe(&self) -> RecipeDisplayId {
        self.recipe
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookSeenRecipePacket#type
    pub fn type_name(&self) -> &'static str {
        PACKET_TYPE_SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookSeenRecipePacket#handle
    pub fn handle<L: ServerGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_recipe_book_seen_recipe_packet(self);
    }

    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let recipe = RecipeDisplayId::read(buf);
        Self::new(recipe)
    }

    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        self.recipe.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        seen: Option<i32>,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_recipe_book_seen_recipe_packet(
            &mut self,
            packet: &ServerboundRecipeBookSeenRecipePacket,
        ) {
            self.seen = Some(packet.recipe().index());
        }
    }

    #[test]
    fn roundtrip_recipe_id() {
        let packet =
            ServerboundRecipeBookSeenRecipePacket::new(RecipeDisplayId::new(42));
        let mut buf = FriendlyByteBuf::new();
        packet.encode(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(&buf.into_bytes());
        let decoded = ServerboundRecipeBookSeenRecipePacket::decode(&mut read_buf);
        assert_eq!(decoded, packet);
        assert_eq!(decoded.recipe().index(), 42);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn negative_and_extreme_indices() {
        for index in [0, -1, i32::MIN, i32::MAX] {
            let packet =
                ServerboundRecipeBookSeenRecipePacket::new(RecipeDisplayId::new(index));
            let mut buf = FriendlyByteBuf::new();
            packet.encode(&mut buf);
            let decoded = ServerboundRecipeBookSeenRecipePacket::decode(
                &mut FriendlyByteBuf::from_bytes(&buf.into_bytes()),
            );
            assert_eq!(decoded.recipe().index(), index);
        }
    }

    #[test]
    fn type_and_handle() {
        let packet =
            ServerboundRecipeBookSeenRecipePacket::new(RecipeDisplayId::new(7));
        assert_eq!(
            packet.type_name(),
            "minecraft:serverbound_recipe_book_seen_recipe"
        );
        let mut listener = MockListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen, Some(7));
    }

    #[test]
    fn float_bits_preserved_through_varint_index() {
        let bits = f32::NAN.to_bits() as i32;
        let packet =
            ServerboundRecipeBookSeenRecipePacket::new(RecipeDisplayId::new(bits));
        let mut buf = FriendlyByteBuf::new();
        packet.encode(&mut buf);
        let decoded = ServerboundRecipeBookSeenRecipePacket::decode(
            &mut FriendlyByteBuf::from_bytes(&buf.into_bytes()),
        );
        assert_eq!(decoded.recipe().index() as u32, f32::NAN.to_bits());
    }
}
