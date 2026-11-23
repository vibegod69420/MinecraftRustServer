use bytes::{Buf, BufMut};
use std::collections::HashMap;

/// Placeholder packet type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_UPDATE_RECIPES_TYPE: PacketType =
    PacketType("CLIENTBOUND_UPDATE_RECIPES");

/// Placeholder for `net.minecraft.resources.ResourceKey<RecipePropertySet>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub namespace: String,
    pub path: String,
}

/// Placeholder for `net.minecraft.world.item.crafting.RecipePropertySet`.
/// The real codec is external; this keeps the module self-contained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipePropertySet {
    pub raw_data: Vec<u8>,
}

/// Placeholder for `net.minecraft.world.item.crafting.SelectableRecipe.SingleInputSet<StonecutterRecipe>`.
/// The real codec is external; this keeps the module self-contained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleInputSetStonecutterRecipe {
    pub raw_data: Vec<u8>,
}

/// Minimal listener trait for self-contained compilation.
pub trait ClientGamePacketListener {
    fn handle_update_recipes(&mut self, packet: &ClientboundUpdateRecipesPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateRecipesPacket {
    item_sets: HashMap<ResourceKey, RecipePropertySet>,
    stonecutter_recipes: SingleInputSetStonecutterRecipe,
}

fn read_varint<B: Buf>(buf: &mut B) -> i32 {
    let mut val: i32 = 0;
    let mut pos: u32 = 0;
    loop {
        let b = buf.get_u8();
        val |= ((b & 0x7F) as i32).wrapping_shl(pos);
        if (b & 0x80) == 0 {
            return val;
        }
        pos = pos.wrapping_add(7);
        if pos >= 32 {
            return val;
        }
    }
}

fn write_varint<B: BufMut>(buf: &mut B, value: i32) {
    let mut val = value as u32;
    loop {
        let mut b = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            b |= 0x80;
        }
        buf.put_u8(b);
        if val == 0 {
            return;
        }
    }
}

fn read_string<B: Buf>(buf: &mut B) -> String {
    let len = read_varint(buf).max(0) as usize;
    let mut bytes = vec![0u8; len];
    buf.copy_to_slice(&mut bytes);
    String::from_utf8(bytes).unwrap_or_default()
}

fn write_string<B: BufMut>(buf: &mut B, s: &str) {
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

impl ResourceKey {
    fn read<B: Buf>(buf: &mut B) -> Self {
        let s = read_string(buf);
        match s.split_once(':') {
            Some((ns, path)) => Self {
                namespace: ns.to_string(),
                path: path.to_string(),
            },
            None => Self {
                namespace: "minecraft".to_string(),
                path: s,
            },
        }
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        let s = format!("{}:{}", self.namespace, self.path);
        write_string(buf, &s);
    }
}

impl RecipePropertySet {
    fn read<B: Buf>(buf: &mut B) -> Self {
        let len = read_varint(buf).max(0) as usize;
        let mut raw_data = vec![0u8; len];
        buf.copy_to_slice(&mut raw_data);
        Self { raw_data }
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        write_varint(buf, self.raw_data.len() as i32);
        buf.put_slice(&self.raw_data);
    }
}

impl SingleInputSetStonecutterRecipe {
    fn read<B: Buf>(buf: &mut B) -> Self {
        let len = read_varint(buf).max(0) as usize;
        let mut raw_data = vec![0u8; len];
        buf.copy_to_slice(&mut raw_data);
        Self { raw_data }
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        write_varint(buf, self.raw_data.len() as i32);
        buf.put_slice(&self.raw_data);
    }
}

impl ClientboundUpdateRecipesPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket#<init>
    pub fn new(
        item_sets: HashMap<ResourceKey, RecipePropertySet>,
        stonecutter_recipes: SingleInputSetStonecutterRecipe,
    ) -> Self {
        Self {
            item_sets,
            stonecutter_recipes,
        }
    }

    /// Simplified placeholder codec read; real wire format is delegated to external StreamCodecs.
    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket#STREAM_CODEC
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let item_set_count = read_varint(buf).max(0) as usize;
        let mut item_sets = HashMap::with_capacity(item_set_count);
        for _ in 0..item_set_count {
            let key = ResourceKey::read(buf);
            let value = RecipePropertySet::read(buf);
            item_sets.insert(key, value);
        }
        let stonecutter_recipes = SingleInputSetStonecutterRecipe::read(buf);
        Self {
            item_sets,
            stonecutter_recipes,
        }
    }

    /// Simplified placeholder codec write; real wire format is delegated to external StreamCodecs.
    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket#STREAM_CODEC
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_varint(buf, self.item_sets.len() as i32);
        for (key, value) in &self.item_sets {
            key.write(buf);
            value.write(buf);
        }
        self.stonecutter_recipes.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_UPDATE_RECIPES_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_update_recipes(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket#itemSets
    pub fn item_sets(&self) -> &HashMap<ResourceKey, RecipePropertySet> {
        &self.item_sets
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket#stonecutterRecipes
    pub fn stonecutter_recipes(&self) -> &SingleInputSetStonecutterRecipe {
        &self.stonecutter_recipes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        called: bool,
        item_set_count: usize,
    }
    impl ClientGamePacketListener for MockListener {
        fn handle_update_recipes(&mut self, packet: &ClientboundUpdateRecipesPacket) {
            self.called = true;
            self.item_set_count = packet.item_sets().len();
        }
    }

    fn key(ns: &str, path: &str) -> ResourceKey {
        ResourceKey {
            namespace: ns.to_string(),
            path: path.to_string(),
        }
    }

    #[test]
    fn test_new_and_accessors() {
        let mut item_sets = HashMap::new();
        item_sets.insert(key("minecraft", "test"), RecipePropertySet { raw_data: vec![1, 2, 3] });
        let sc = SingleInputSetStonecutterRecipe { raw_data: vec![4, 5] };
        let pkt = ClientboundUpdateRecipesPacket::new(item_sets.clone(), sc.clone());
        assert_eq!(pkt.item_sets(), &item_sets);
        assert_eq!(pkt.stonecutter_recipes(), &sc);
    }

    #[test]
    fn test_empty_roundtrip() {
        let pkt = ClientboundUpdateRecipesPacket::new(
            HashMap::new(),
            SingleInputSetStonecutterRecipe { raw_data: vec![] },
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundUpdateRecipesPacket::read(&mut reader);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_write_read_roundtrip() {
        let mut item_sets = HashMap::new();
        item_sets.insert(key("minecraft", "planks"), RecipePropertySet { raw_data: vec![10, 20, 30] });
        item_sets.insert(key("custom", "block"), RecipePropertySet { raw_data: vec![0xFF, 0x00, 0x80] });
        let pkt = ClientboundUpdateRecipesPacket::new(
            item_sets,
            SingleInputSetStonecutterRecipe { raw_data: vec![7, 8, 9] },
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundUpdateRecipesPacket::read(&mut reader);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_handle_and_type() {
        let pkt = ClientboundUpdateRecipesPacket::new(
            HashMap::new(),
            SingleInputSetStonecutterRecipe { raw_data: vec![] },
        );
        assert_eq!(pkt.get_type(), CLIENTBOUND_UPDATE_RECIPES_TYPE);
        let mut listener = MockListener { called: false, item_set_count: 99 };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.item_set_count, 0);
    }

    #[test]
    fn test_large_raw_data_roundtrip() {
        let raw: Vec<u8> = (0u16..256).map(|i| i as u8).collect();
        let mut item_sets = HashMap::new();
        item_sets.insert(key("ns", "path"), RecipePropertySet { raw_data: raw.clone() });
        let pkt = ClientboundUpdateRecipesPacket::new(
            item_sets,
            SingleInputSetStonecutterRecipe { raw_data: raw },
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundUpdateRecipesPacket::read(&mut reader);
        assert_eq!(pkt, decoded);
    }
}
