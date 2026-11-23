#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDisplayEntry {
    pub data: Vec<u8>,
}

pub trait ClientGamePacketListener {
    fn handle_recipe_book_add(&mut self, packet: &ClientboundRecipeBookAddPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookAddPacket {
    entries: Vec<Entry>,
    replace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    contents: RecipeDisplayEntry,
    flags: i8,
}

impl Entry {
    pub const FLAG_NOTIFICATION: i8 = 1;
    pub const FLAG_HIGHLIGHT: i8 = 2;

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket$Entry#<init>
    pub fn new(contents: RecipeDisplayEntry, notification: bool, highlight: bool) -> Self {
        let flags = (if notification { 1i8 } else { 0i8 }) | (if highlight { 2i8 } else { 0i8 });
        Self { contents, flags }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket$Entry#<init>
    pub fn from_parts(contents: RecipeDisplayEntry, flags: i8) -> Self {
        Self { contents, flags }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket$Entry#contents
    pub fn contents(&self) -> &RecipeDisplayEntry {
        &self.contents
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket$Entry#flags
    pub fn flags(&self) -> i8 {
        self.flags
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket$Entry#notification
    pub fn notification(&self) -> bool {
        (self.flags & 1) != 0
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket$Entry#highlight
    pub fn highlight(&self) -> bool {
        (self.flags & 2) != 0
    }
}

impl ClientboundRecipeBookAddPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket#<init>
    pub fn new(entries: Vec<Entry>, replace: bool) -> Self {
        Self { entries, replace }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket#entries
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket#replace
    pub fn replace(&self) -> bool {
        self.replace
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_recipe_book_add(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_contents(id: u8) -> RecipeDisplayEntry {
        RecipeDisplayEntry { data: vec![id] }
    }

    #[test]
    fn entry_flag_packing() {
        let e00 = Entry::new(dummy_contents(1), false, false);
        assert_eq!(e00.flags(), 0);
        assert!(!e00.notification());
        assert!(!e00.highlight());

        let e10 = Entry::new(dummy_contents(2), true, false);
        assert_eq!(e10.flags(), Entry::FLAG_NOTIFICATION);
        assert!(e10.notification());
        assert!(!e10.highlight());

        let e01 = Entry::new(dummy_contents(3), false, true);
        assert_eq!(e01.flags(), Entry::FLAG_HIGHLIGHT);
        assert!(!e01.notification());
        assert!(e01.highlight());

        let e11 = Entry::new(dummy_contents(4), true, true);
        assert_eq!(e11.flags(), 3);
        assert!(e11.notification());
        assert!(e11.highlight());
    }

    #[test]
    fn entry_from_parts_preserves_raw_flags() {
        let e = Entry::from_parts(dummy_contents(5), -1i8);
        assert_eq!(e.flags(), -1i8);
        assert!(e.notification());
        assert!(e.highlight());

        let e2 = Entry::from_parts(dummy_contents(6), 0x7Ci8);
        assert!(!e2.notification());
        assert!(!e2.highlight());
    }

    #[test]
    fn packet_accessors_and_replace() {
        let entries = vec![
            Entry::new(dummy_contents(1), true, false),
            Entry::new(dummy_contents(2), false, true),
        ];
        let pkt = ClientboundRecipeBookAddPacket::new(entries.clone(), true);
        assert_eq!(pkt.entries(), entries.as_slice());
        assert!(pkt.replace());

        let empty = ClientboundRecipeBookAddPacket::new(Vec::new(), false);
        assert!(empty.entries().is_empty());
        assert!(!empty.replace());
    }

    #[test]
    fn handle_delegates_to_listener() {
        struct MockListener {
            called: bool,
            replace: bool,
            count: usize,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_recipe_book_add(&mut self, packet: &ClientboundRecipeBookAddPacket) {
                self.called = true;
                self.replace = packet.replace();
                self.count = packet.entries().len();
            }
        }

        let pkt = ClientboundRecipeBookAddPacket::new(vec![Entry::new(dummy_contents(1), true, true)], false);
        let mut listener = MockListener { called: false, replace: true, count: 0 };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert!(!listener.replace);
        assert_eq!(listener.count, 1);
    }

    #[test]
    fn entry_contents_accessor() {
        let contents = RecipeDisplayEntry { data: vec![10, 20, 30] };
        let e = Entry::from_parts(contents.clone(), 0);
        assert_eq!(e.contents(), &contents);
    }
}
