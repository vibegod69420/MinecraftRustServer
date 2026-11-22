use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryFriendlyByteBuf {
    buf: BytesMut,
}

impl RegistryFriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            buf: BytesMut::from(bytes),
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#asByteBuf
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writerIndex
    pub fn remaining(&self) -> usize {
        self.buf.remaining()
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, value: i32) {
        let mut v = value as u32;
        loop {
            if (v & !0x7F_u32) == 0 {
                self.buf.put_u8(v as u8);
                return;
            }
            self.buf.put_u8(((v & 0x7F_u32) as u8) | 0x80);
            v >>= 7;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.buf.get_u8();
            result |= ((b & 0x7F) as i32).wrapping_shl(shift);
            if (b & 0x80) == 0 {
                return result;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                panic!("VarInt too big");
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByteArray
    pub fn write_raw_bytes(&mut self, bytes: &[u8]) {
        self.buf.put_slice(bytes);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByteArray
    pub fn read_remaining_bytes(&mut self) -> Vec<u8> {
        let len = self.buf.remaining();
        let bytes = self.buf.split_to(len);
        bytes.to_vec()
    }
}

/// Minimal self-contained stub for `RecipeDisplay`.
/// The real type uses `RecipeDisplay.STREAM_CODEC`; this stub preserves the
/// remaining encoded payload for faithful packet round-trips in isolation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDisplay {
    pub raw: Vec<u8>,
}

impl RecipeDisplay {
    // JAVA: net.minecraft.world.item.crafting.display.RecipeDisplay#<stream codec decode>
    pub fn decode(buf: &mut RegistryFriendlyByteBuf) -> Self {
        Self {
            raw: buf.read_remaining_bytes(),
        }
    }

    // JAVA: net.minecraft.world.item.crafting.display.RecipeDisplay#<stream codec encode>
    pub fn encode(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_raw_bytes(&self.raw);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_PLACE_GHOST_RECIPE: PacketType =
    PacketType("minecraft:clientbound_place_ghost_recipe");

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handlePlaceRecipe
    fn handle_place_recipe(&mut self, packet: &ClientboundPlaceGhostRecipePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlaceGhostRecipePacket {
    pub container_id: i32,
    pub recipe_display: RecipeDisplay,
}

impl ClientboundPlaceGhostRecipePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlaceGhostRecipePacket#<init>
    pub fn new(container_id: i32, recipe_display: RecipeDisplay) -> Self {
        Self {
            container_id,
            recipe_display,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlaceGhostRecipePacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_PLACE_GHOST_RECIPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlaceGhostRecipePacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_place_recipe(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlaceGhostRecipePacket#containerId
    pub fn container_id(&self) -> i32 {
        self.container_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlaceGhostRecipePacket#recipeDisplay
    pub fn recipe_display(&self) -> &RecipeDisplay {
        &self.recipe_display
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlaceGhostRecipePacket#STREAM_CODEC
    pub fn encode(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_var_int(self.container_id);
        self.recipe_display.encode(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlaceGhostRecipePacket#STREAM_CODEC
    pub fn decode(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let container_id = buf.read_var_int();
        let recipe_display = RecipeDisplay::decode(buf);
        Self::new(container_id, recipe_display)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_packet() {
        let pkt = ClientboundPlaceGhostRecipePacket::new(
            7,
            RecipeDisplay {
                raw: vec![1, 2, 3],
            },
        );
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.encode(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ClientboundPlaceGhostRecipePacket::decode(&mut read_buf);
        assert_eq!(decoded.container_id(), 7);
        assert_eq!(decoded.recipe_display().raw, vec![1, 2, 3]);
        assert_eq!(decoded.packet_type(), CLIENTBOUND_PLACE_GHOST_RECIPE);
    }

    #[test]
    fn round_trip_negative_container_id() {
        let pkt = ClientboundPlaceGhostRecipePacket::new(
            -1,
            RecipeDisplay { raw: vec![] },
        );
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.encode(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ClientboundPlaceGhostRecipePacket::decode(&mut read_buf);
        assert_eq!(decoded.container_id(), -1);
        assert_eq!(decoded.recipe_display().raw, Vec::<u8>::new());
    }

    #[test]
    fn round_trip_large_varint_container_id() {
        let pkt = ClientboundPlaceGhostRecipePacket::new(
            i32::MAX,
            RecipeDisplay { raw: vec![0xFF, 0x00] },
        );
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.encode(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ClientboundPlaceGhostRecipePacket::decode(&mut read_buf);
        assert_eq!(decoded.container_id(), i32::MAX);
        assert_eq!(decoded.recipe_display().raw, vec![0xFF, 0x00]);
    }

    #[test]
    fn varint_negative_one_encoding_round_trip() {
        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_var_int(-1);
        assert_eq!(buf.as_bytes(), &[0xff, 0xff, 0xff, 0xff, 0x0f]);
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(buf.as_bytes());
        assert_eq!(read_buf.read_var_int(), -1);
    }

    #[test]
    fn handle_calls_listener() {
        struct L {
            container_id: i32,
        }
        impl ClientGamePacketListener for L {
            fn handle_place_recipe(&mut self, packet: &ClientboundPlaceGhostRecipePacket) {
                self.container_id = packet.container_id();
            }
        }
        let pkt = ClientboundPlaceGhostRecipePacket::new(
            42,
            RecipeDisplay { raw: vec![9] },
        );
        let mut l = L { container_id: 0 };
        pkt.handle(&mut l);
        assert_eq!(l.container_id, 42);
    }
}
