#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub &'static str);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_RECIPE_BOOK_SETTINGS: PacketType =
        PacketType("CLIENTBOUND_RECIPE_BOOK_SETTINGS");
}

/// Placeholder for `net.minecraft.stats.RecipeBookSettings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeBookSettings {
    pub flags: u32,
}

/// Placeholder for `net.minecraft.network.protocol.game.ClientGamePacketListener`.
pub trait ClientGamePacketListener {
    fn handle_recipe_book_settings(&mut self, packet: &ClientboundRecipeBookSettingsPacket);
}

/// Port of `net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookSettingsPacket {
    book_settings: RecipeBookSettings,
}

impl ClientboundRecipeBookSettingsPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket#<init>
    pub fn new(book_settings: RecipeBookSettings) -> Self {
        Self { book_settings }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket#bookSettings
    pub fn book_settings(&self) -> &RecipeBookSettings {
        &self.book_settings
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket#type
    pub fn type_(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_SETTINGS
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_recipe_book_settings(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        handled: bool,
        flags: u32,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_recipe_book_settings(
            &mut self,
            packet: &ClientboundRecipeBookSettingsPacket,
        ) {
            self.handled = true;
            self.flags = packet.book_settings().flags;
        }
    }

    #[test]
    fn new_and_book_settings_accessor() {
        let settings = RecipeBookSettings { flags: 0xAA55 };
        let packet = ClientboundRecipeBookSettingsPacket::new(settings.clone());
        assert_eq!(packet.book_settings(), &settings);
    }

    #[test]
    fn packet_type_is_recipe_book_settings() {
        let packet =
            ClientboundRecipeBookSettingsPacket::new(RecipeBookSettings { flags: 0 });
        assert_eq!(
            packet.type_(),
            GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_SETTINGS
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ClientboundRecipeBookSettingsPacket::new(RecipeBookSettings {
            flags: u32::MAX,
        });
        let mut listener = MockListener {
            handled: false,
            flags: 0,
        };
        packet.handle(&mut listener);
        assert!(listener.handled);
        assert_eq!(listener.flags, u32::MAX);
    }

    #[test]
    fn clone_eq_and_negative_bit_pattern() {
        let packet = ClientboundRecipeBookSettingsPacket::new(RecipeBookSettings {
            flags: i32::MIN as u32,
        });
        let cloned = packet.clone();
        assert_eq!(packet, cloned);
        assert_eq!(packet.book_settings().flags, 0x8000_0000);
    }
}
