#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    reader_index: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            reader_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.reader_index)
    }

    fn read_byte(&mut self) -> u8 {
        assert!(self.reader_index < self.data.len(), "buffer underflow");
        let b = self.data[self.reader_index];
        self.reader_index += 1;
        b
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let b = self.read_byte();
            value |= ((b & 0x7F) as i32) << position;
            if (b & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                panic!("VarInt too big");
            }
        }
        value
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut v = value as u32;
        loop {
            if (v & !0x7F_u32) == 0 {
                self.data.push(v as u8);
                return;
            }
            self.data.push(((v as u8) & 0x7F) | 0x80);
            v >>= 7;
        }
    }

    pub fn read_boolean(&mut self) -> bool {
        self.read_byte() != 0
    }

    pub fn write_boolean(&mut self, value: bool) {
        self.data.push(if value { 1 } else { 0 });
    }

    pub fn read_enum<T, F: FnOnce(i32) -> T>(&mut self, f: F) -> T {
        let ordinal = self.read_var_int();
        f(ordinal)
    }

    pub fn write_enum<T, F: FnOnce(&T) -> i32>(&mut self, f: F, value: &T) {
        self.write_var_int(f(value));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

impl RecipeBookType {
    pub fn ordinal(&self) -> i32 {
        match self {
            RecipeBookType::Crafting => 0,
            RecipeBookType::Furnace => 1,
            RecipeBookType::BlastFurnace => 2,
            RecipeBookType::Smoker => 3,
        }
    }

    pub fn from_ordinal(n: i32) -> Option<Self> {
        match n {
            0 => Some(RecipeBookType::Crafting),
            1 => Some(RecipeBookType::Furnace),
            2 => Some(RecipeBookType::BlastFurnace),
            3 => Some(RecipeBookType::Smoker),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub trait ServerGamePacketListener {
    fn handle_recipe_book_change_settings_packet(
        &mut self,
        packet: &ServerboundRecipeBookChangeSettingsPacket,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundRecipeBookChangeSettingsPacket {
    book_type: RecipeBookType,
    is_open: bool,
    is_filtering: bool,
}

impl ServerboundRecipeBookChangeSettingsPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookChangeSettingsPacket#<init>
    pub fn new(book_type: RecipeBookType, is_open: bool, is_filtering: bool) -> Self {
        Self {
            book_type,
            is_open,
            is_filtering,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookChangeSettingsPacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        let book_type = buf.read_enum(|n| {
            RecipeBookType::from_ordinal(n).expect("unknown RecipeBookType ordinal")
        });
        let is_open = buf.read_boolean();
        let is_filtering = buf.read_boolean();
        Self {
            book_type,
            is_open,
            is_filtering,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookChangeSettingsPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_enum(|v: &RecipeBookType| v.ordinal(), &self.book_type);
        buf.write_boolean(self.is_open);
        buf.write_boolean(self.is_filtering);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookChangeSettingsPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookChangeSettingsPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_recipe_book_change_settings_packet(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookChangeSettingsPacket#getBookType
    pub fn get_book_type(&self) -> RecipeBookType {
        self.book_type
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookChangeSettingsPacket#isOpen
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRecipeBookChangeSettingsPacket#isFiltering
    pub fn is_filtering(&self) -> bool {
        self.is_filtering
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen: Option<ServerboundRecipeBookChangeSettingsPacket>,
    }

    impl ServerGamePacketListener for TestListener {
        fn handle_recipe_book_change_settings_packet(
            &mut self,
            packet: &ServerboundRecipeBookChangeSettingsPacket,
        ) {
            self.seen = Some(packet.clone());
        }
    }

    #[test]
    fn round_trip_all_fields() {
        let packet = ServerboundRecipeBookChangeSettingsPacket::new(
            RecipeBookType::BlastFurnace,
            true,
            false,
        );
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundRecipeBookChangeSettingsPacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_book_type(), RecipeBookType::BlastFurnace);
        assert!(decoded.is_open());
        assert!(!decoded.is_filtering());
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_all_enum_variants() {
        let variants = [
            RecipeBookType::Crafting,
            RecipeBookType::Furnace,
            RecipeBookType::BlastFurnace,
            RecipeBookType::Smoker,
        ];
        for (i, v) in variants.iter().copied().enumerate() {
            let packet = ServerboundRecipeBookChangeSettingsPacket::new(v, i % 2 == 0, i % 2 == 1);
            let mut buf = FriendlyByteBuf::default();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ServerboundRecipeBookChangeSettingsPacket::from_buf(&mut read_buf);
            assert_eq!(decoded.get_book_type(), v);
            assert_eq!(decoded.is_open(), i % 2 == 0);
            assert_eq!(decoded.is_filtering(), i % 2 == 1);
        }
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet =
            ServerboundRecipeBookChangeSettingsPacket::new(RecipeBookType::Smoker, false, true);
        let mut listener = TestListener { seen: None };
        packet.handle(&mut listener);
        let seen = listener.seen.expect("packet not handled");
        assert_eq!(seen.get_book_type(), RecipeBookType::Smoker);
        assert!(!seen.is_open());
        assert!(seen.is_filtering());
    }

    #[test]
    fn enum_ordinal_mapping() {
        assert_eq!(RecipeBookType::Crafting.ordinal(), 0);
        assert_eq!(RecipeBookType::Furnace.ordinal(), 1);
        assert_eq!(RecipeBookType::BlastFurnace.ordinal(), 2);
        assert_eq!(RecipeBookType::Smoker.ordinal(), 3);
        assert_eq!(RecipeBookType::from_ordinal(-1), None);
        assert_eq!(RecipeBookType::from_ordinal(4), None);
    }
}
