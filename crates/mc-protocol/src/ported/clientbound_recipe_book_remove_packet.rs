use std::io::{self, Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecipeDisplayId(pub i32);

pub trait ClientGamePacketListener {
    fn handle_recipe_book_remove(&mut self, packet: &ClientboundRecipeBookRemovePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookRemovePacket {
    recipes: Vec<RecipeDisplayId>,
}

fn write_var_int<W: Write>(writer: &mut W, value: i32) -> io::Result<()> {
    let mut val = value as u32;
    loop {
        let mut b = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            b |= 0x80;
        }
        writer.write_all(&[b])?;
        if val == 0 {
            return Ok(());
        }
    }
}

fn read_var_int<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut val: i32 = 0;
    let mut pos: u32 = 0;
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let b = buf[0];
        val |= ((b & 0x7F) as i32) << pos;
        if (b & 0x80) == 0 {
            return Ok(val);
        }
        pos = pos.wrapping_add(7);
        if pos >= 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt too big",
            ));
        }
    }
}

impl ClientboundRecipeBookRemovePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookRemovePacket#<init>
    pub fn new(recipes: Vec<RecipeDisplayId>) -> Self {
        Self { recipes }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookRemovePacket#recipes
    pub fn recipes(&self) -> &[RecipeDisplayId] {
        &self.recipes
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookRemovePacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:recipe_book_remove")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookRemovePacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_recipe_book_remove(self);
    }

    /// Encodes as VarInt list length followed by VarInt RecipeDisplayId values.
    pub fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let len = i32::try_from(self.recipes.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "recipe list too large"))?;
        write_var_int(writer, len)?;
        for recipe in &self.recipes {
            write_var_int(writer, recipe.0)?;
        }
        Ok(())
    }

    /// Decodes a VarInt-length-prefixed list of VarInt RecipeDisplayId values.
    pub fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let len = read_var_int(reader)?;
        if len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative list length",
            ));
        }
        let mut recipes = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let id = read_var_int(reader)?;
            recipes.push(RecipeDisplayId(id));
        }
        Ok(Self { recipes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct MockListener {
        removed: Vec<RecipeDisplayId>,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_recipe_book_remove(&mut self, packet: &ClientboundRecipeBookRemovePacket) {
            self.removed = packet.recipes().to_vec();
        }
    }

    #[test]
    fn accessors_and_type() {
        let packet = ClientboundRecipeBookRemovePacket::new(vec![
            RecipeDisplayId(1),
            RecipeDisplayId(-2),
        ]);
        assert_eq!(
            packet.get_type(),
            PacketType("minecraft:recipe_book_remove")
        );
        assert_eq!(packet.recipes(), &[RecipeDisplayId(1), RecipeDisplayId(-2)]);
    }

    #[test]
    fn handle_dispatches() {
        let packet = ClientboundRecipeBookRemovePacket::new(vec![RecipeDisplayId(42)]);
        let mut listener = MockListener { removed: vec![] };
        packet.handle(&mut listener);
        assert_eq!(listener.removed, vec![RecipeDisplayId(42)]);
    }

    #[test]
    fn encode_decode_roundtrip_empty() {
        let packet = ClientboundRecipeBookRemovePacket::new(vec![]);
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        let decoded =
            ClientboundRecipeBookRemovePacket::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn encode_decode_roundtrip_edge_ids() {
        let packet = ClientboundRecipeBookRemovePacket::new(vec![
            RecipeDisplayId(i32::MIN),
            RecipeDisplayId(-1),
            RecipeDisplayId(0),
            RecipeDisplayId(i32::MAX),
        ]);
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        let decoded =
            ClientboundRecipeBookRemovePacket::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn decode_rejects_negative_length() {
        let mut buf = Vec::new();
        write_var_int(&mut buf, -1).unwrap();
        let err = ClientboundRecipeBookRemovePacket::decode(&mut Cursor::new(&buf)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
