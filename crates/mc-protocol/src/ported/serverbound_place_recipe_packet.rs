use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const SERVERBOUND_PLACE_RECIPE_TYPE: PacketType =
    PacketType("minecraft:serverbound_place_recipe");

pub trait ServerGamePacketListener {
    fn handle_place_recipe(&mut self, packet: &ServerboundPlaceRecipePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    buf: Vec<u8>,
    rpos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            rpos: 0,
        }
    }

    pub fn from_vec(buf: Vec<u8>) -> Self {
        Self { buf, rpos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.rpos)
    }

    fn ensure(&self, n: usize) -> io::Result<()> {
        if self.remaining() < n {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "buffer underflow"))
        } else {
            Ok(())
        }
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.ensure(1)?;
        let v = self.buf[self.rpos];
        self.rpos += 1;
        Ok(v)
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn write_bool(&mut self, v: bool) {
        self.write_u8(if v { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_u8()?;
            value |= ((current_byte & 0x7F) as i32) << position;
            if (current_byte & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
        }
        Ok(value)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            let mut temp = (value as u32 & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if (value as u32) != 0 {
                temp |= 0x80;
            }
            self.write_u8(temp);
            if (value as u32) == 0 {
                break;
            }
        }
    }

    // JAVA: net.minecraft.network.codec.ByteBufCodecs#CONTAINER_ID decode
    pub fn read_container_id(&mut self) -> io::Result<i32> {
        self.read_var_int()
    }

    // JAVA: net.minecraft.network.codec.ByteBufCodecs#CONTAINER_ID encode
    pub fn write_container_id(&mut self, value: i32) {
        self.write_var_int(value);
    }
}

impl Default for FriendlyByteBuf {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecipeDisplayId(pub i32);

impl RecipeDisplayId {
    // JAVA: net.minecraft.world.item.crafting.display.RecipeDisplayId#STREAM_CODEC decode
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        Ok(Self(buf.read_var_int()?))
    }

    // JAVA: net.minecraft.world.item.crafting.display.RecipeDisplayId#STREAM_CODEC encode
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlaceRecipePacket {
    pub container_id: i32,
    pub recipe: RecipeDisplayId,
    pub use_max_items: bool,
}

impl ServerboundPlaceRecipePacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPlaceRecipePacket#STREAM_CODEC decode
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let container_id = buf.read_container_id()?;
        let recipe = RecipeDisplayId::read(buf)?;
        let use_max_items = buf.read_bool()?;
        Ok(Self {
            container_id,
            recipe,
            use_max_items,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlaceRecipePacket#STREAM_CODEC encode
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        self.recipe.write(buf);
        buf.write_bool(self.use_max_items);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlaceRecipePacket#type()
    pub fn type_(&self) -> PacketType {
        SERVERBOUND_PLACE_RECIPE_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlaceRecipePacket#handle(ServerGamePacketListener)
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_place_recipe(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_basic() {
        let packet = ServerboundPlaceRecipePacket {
            container_id: 5,
            recipe: RecipeDisplayId(42),
            use_max_items: true,
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ServerboundPlaceRecipePacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_negative_and_false() {
        let packet = ServerboundPlaceRecipePacket {
            container_id: -1,
            recipe: RecipeDisplayId(-100),
            use_max_items: false,
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ServerboundPlaceRecipePacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn round_trip_extreme_ints() {
        let packet = ServerboundPlaceRecipePacket {
            container_id: i32::MAX,
            recipe: RecipeDisplayId(i32::MIN),
            use_max_items: true,
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ServerboundPlaceRecipePacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn bool_wire_encoding() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_bool(false);
        buf.write_bool(true);
        let bytes = buf.into_inner();
        assert_eq!(bytes, vec![0, 1]);
    }

    #[test]
    fn handle_calls_listener() {
        struct Listener {
            called: bool,
            container_id: i32,
        }
        impl ServerGamePacketListener for Listener {
            fn handle_place_recipe(&mut self, packet: &ServerboundPlaceRecipePacket) {
                self.called = true;
                self.container_id = packet.container_id;
            }
        }

        let packet = ServerboundPlaceRecipePacket {
            container_id: 7,
            recipe: RecipeDisplayId(3),
            use_max_items: false,
        };
        let mut listener = Listener {
            called: false,
            container_id: -1,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.container_id, 7);
        assert_eq!(packet.type_(), SERVERBOUND_PLACE_RECIPE_TYPE);
    }
}
