#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: String) -> Self {
        Self { text }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringRange {
    start: i32,
    length: i32,
}

impl StringRange {
    pub fn between(start: i32, end: i32) -> Self {
        Self {
            start,
            length: end.wrapping_sub(start),
        }
    }

    pub fn get_start(self) -> i32 {
        self.start
    }

    pub fn get_length(self) -> i32 {
        self.length
    }

    pub fn get_end(self) -> i32 {
        self.start.wrapping_add(self.length)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    range: StringRange,
    text: String,
    tooltip: Option<Component>,
}

impl Suggestion {
    pub fn new(range: StringRange, text: String, tooltip: Option<Component>) -> Self {
        Self {
            range,
            text,
            tooltip,
        }
    }

    pub fn get_range(&self) -> StringRange {
        self.range
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn get_tooltip(&self) -> Option<&Component> {
        self.tooltip.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestions {
    range: StringRange,
    suggestions: Vec<Suggestion>,
}

impl Suggestions {
    pub fn new(range: StringRange, suggestions: Vec<Suggestion>) -> Self {
        Self { range, suggestions }
    }

    pub fn get_range(&self) -> StringRange {
        self.range
    }

    pub fn get_list(&self) -> &[Suggestion] {
        &self.suggestions
    }
}

#[derive(Debug, Clone, Default)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.data[self.read_pos];
            self.read_pos += 1;
            value |= ((current_byte & 0x7F) as i32) << position;
            if (current_byte & 0x80) == 0 {
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
        let mut val = value as u32;
        loop {
            if (val & !0x7F) == 0 {
                self.data.push(val as u8);
                return;
            }
            self.data.push(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
    }

    pub fn read_utf(&mut self) -> String {
        let len = self.read_var_int() as usize;
        let bytes = self.data[self.read_pos..self.read_pos + len].to_vec();
        self.read_pos += len;
        String::from_utf8(bytes).expect("invalid utf8")
    }

    pub fn write_utf(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    pub fn read_bool(&mut self) -> bool {
        let v = self.data[self.read_pos];
        self.read_pos += 1;
        v != 0
    }

    pub fn write_bool(&mut self, v: bool) {
        self.data.push(if v { 1 } else { 0 });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ClientGamePacketListener {
    fn handle_command_suggestions(&mut self, packet: &ClientboundCommandSuggestionsPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    text: String,
    tooltip: Option<Component>,
}

impl Entry {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket$Entry#<init>
    pub fn new(text: String, tooltip: Option<Component>) -> Self {
        Self { text, tooltip }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket$Entry#text
    pub fn text(&self) -> &str {
        &self.text
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket$Entry#tooltip
    pub fn tooltip(&self) -> Option<&Component> {
        self.tooltip.as_ref()
    }

    fn encode(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_utf(&self.text);
        let has_tooltip = self.tooltip.is_some();
        buf.write_bool(has_tooltip);
        if let Some(component) = &self.tooltip {
            buf.write_utf(&component.text);
        }
    }

    fn decode(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let text = buf.read_utf();
        let has_tooltip = buf.read_bool();
        let tooltip = if has_tooltip {
            Some(Component {
                text: buf.read_utf(),
            })
        } else {
            None
        };
        Self { text, tooltip }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCommandSuggestionsPacket {
    id: i32,
    start: i32,
    length: i32,
    suggestions: Vec<Entry>,
}

impl ClientboundCommandSuggestionsPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket#<init>
    pub fn new(id: i32, start: i32, length: i32, suggestions: Vec<Entry>) -> Self {
        Self {
            id,
            start,
            length,
            suggestions,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket#<init>
    pub fn from_suggestions(id: i32, suggestions: &Suggestions) -> Self {
        let range = suggestions.get_range();
        let entries = suggestions
            .get_list()
            .iter()
            .map(|s| Entry {
                text: s.get_text().to_string(),
                tooltip: s.get_tooltip().cloned(),
            })
            .collect();
        Self {
            id,
            start: range.get_start(),
            length: range.get_length(),
            suggestions: entries,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket#id
    pub fn id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket#start
    pub fn start(&self) -> i32 {
        self.start
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket#length
    pub fn length(&self) -> i32 {
        self.length
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket#suggestions
    pub fn suggestions(&self) -> &[Entry] {
        &self.suggestions
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:command_suggestions")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_command_suggestions(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket#toSuggestions
    pub fn to_suggestions(&self) -> Suggestions {
        let range = StringRange::between(self.start, self.start.wrapping_add(self.length));
        let list = self
            .suggestions
            .iter()
            .map(|e| Suggestion::new(range, e.text.clone(), e.tooltip.clone()))
            .collect();
        Suggestions::new(range, list)
    }

    pub fn encode(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_var_int(self.start);
        buf.write_var_int(self.length);
        buf.write_var_int(self.suggestions.len() as i32);
        for entry in &self.suggestions {
            entry.encode(buf);
        }
    }

    pub fn decode(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let id = buf.read_var_int();
        let start = buf.read_var_int();
        let length = buf.read_var_int();
        let count = buf.read_var_int() as usize;
        let mut suggestions = Vec::with_capacity(count);
        for _ in 0..count {
            suggestions.push(Entry::decode(buf));
        }
        Self {
            id,
            start,
            length,
            suggestions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_suggestions_maps_fields() {
        let pkt = ClientboundCommandSuggestionsPacket::new(
            1,
            5,
            3,
            vec![
                Entry::new("alpha".to_string(), None),
                Entry::new(
                    "beta".to_string(),
                    Some(Component::literal("tooltip".to_string())),
                ),
            ],
        );
        let sug = pkt.to_suggestions();
        assert_eq!(sug.get_range().get_start(), 5);
        assert_eq!(sug.get_range().get_length(), 3);
        let list = sug.get_list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].get_text(), "alpha");
        assert_eq!(list[0].get_tooltip(), None);
        assert_eq!(
            list[1].get_tooltip(),
            Some(&Component::literal("tooltip".to_string()))
        );
    }

    #[test]
    fn from_suggestions_extracts_range() {
        let range = StringRange::between(2, 7);
        let suggestions = Suggestions::new(
            range,
            vec![
                Suggestion::new(range, "foo".to_string(), None),
                Suggestion::new(range, "bar".to_string(), Some(Component::literal("b".to_string()))),
            ],
        );
        let pkt = ClientboundCommandSuggestionsPacket::from_suggestions(42, &suggestions);
        assert_eq!(pkt.id(), 42);
        assert_eq!(pkt.start(), 2);
        assert_eq!(pkt.length(), 5);
        assert_eq!(pkt.suggestions().len(), 2);
        assert_eq!(pkt.suggestions()[0].text(), "foo");
    }

    #[test]
    fn encode_decode_roundtrip_negative_inputs() {
        let pkt = ClientboundCommandSuggestionsPacket::new(
            -1,
            -10,
            0,
            vec![
                Entry::new(String::new(), None),
                Entry::new("x".to_string(), Some(Component::literal("y".to_string()))),
            ],
        );
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.encode(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundCommandSuggestionsPacket::decode(&mut read_buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn to_suggestions_wrapping_end() {
        let pkt = ClientboundCommandSuggestionsPacket::new(i32::MIN, i32::MAX, 2, vec![]);
        let sug = pkt.to_suggestions();
        let range = sug.get_range();
        assert_eq!(range.get_start(), i32::MAX);
        assert_eq!(range.get_length(), 2);
        assert_eq!(range.get_end(), i32::MAX.wrapping_add(2));
    }
}
