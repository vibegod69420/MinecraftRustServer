#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier {
    pub value: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#<init>(String)
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

pub type RegistryFriendlyByteBuf = FriendlyByteBuf;

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>(ByteBuf)
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#array
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte
    fn read_byte(&mut self) -> u8 {
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        b
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte
    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
    pub fn read_boolean(&mut self) -> bool {
        self.read_byte() != 0
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_boolean(&mut self, value: bool) {
        self.write_byte(if value { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut value: u32 = 0;
        let mut position: u32 = 0;
        loop {
            let b = self.read_byte();
            value |= ((b & 0x7F) as u32).wrapping_shl(position);
            if (b & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                panic!("VarInt too big");
            }
        }
        value as i32
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            let uval = value as u32;
            if (uval & !0x7Fu32) == 0 {
                self.write_byte(uval as u8);
                return;
            }
            self.write_byte(((uval & 0x7F) | 0x80) as u8);
            value = (uval >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong
    pub fn read_long(&mut self) -> i64 {
        let mut bytes = [0u8; 8];
        for b in &mut bytes {
            *b = self.read_byte();
        }
        i64::from_be_bytes(bytes)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
    pub fn write_long(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf(int)
    pub fn read_utf(&mut self, max_length: usize) -> String {
        let max_bytes = max_length.saturating_mul(3);
        let byte_len = self.read_var_int();
        if byte_len < 0 {
            panic!("The received encoded string buffer length is less than zero! Weird string!");
        }
        let byte_len = byte_len as usize;
        if byte_len > max_bytes {
            panic!(
                "The received encoded string buffer length is longer than maximum allowed ({} > {})",
                byte_len, max_bytes
            );
        }
        let end = self.read_pos + byte_len;
        let s = std::str::from_utf8(&self.data[self.read_pos..end])
            .expect("Invalid UTF-8")
            .to_string();
        self.read_pos = end;
        let char_len = s.encode_utf16().count();
        if char_len > max_length {
            panic!(
                "The received string length is longer than maximum allowed ({} > {})",
                char_len, max_length
            );
        }
        s
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf(String,int)
    pub fn write_utf(&mut self, value: &str, max_length: usize) {
        let char_len = value.encode_utf16().count();
        if char_len > max_length {
            panic!(
                "String too big (was {} characters, max {})",
                char_len, max_length
            );
        }
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readIdentifier
    pub fn read_identifier(&mut self) -> Identifier {
        Identifier::new(self.read_utf(32767))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeIdentifier
    pub fn write_identifier(&mut self, id: &Identifier) {
        self.write_utf(&id.value, 32767);
    }
}

// JAVA: net.minecraft.advancements.CriterionProgress
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionProgress {
    obtained: Option<i64>,
}

impl CriterionProgress {
    // JAVA: net.minecraft.advancements.CriterionProgress#<init>(Instant)
    pub fn obtained(epoch_millis: i64) -> Self {
        Self {
            obtained: Some(epoch_millis),
        }
    }

    // JAVA: net.minecraft.advancements.CriterionProgress#<init>()
    pub fn not_obtained() -> Self {
        Self { obtained: None }
    }

    // JAVA: net.minecraft.advancements.CriterionProgress#fromNetwork
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        if buf.read_boolean() {
            Self {
                obtained: Some(buf.read_long()),
            }
        } else {
            Self { obtained: None }
        }
    }

    // JAVA: net.minecraft.advancements.CriterionProgress#serializeToNetwork
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        if let Some(ts) = self.obtained {
            buf.write_boolean(true);
            buf.write_long(ts);
        } else {
            buf.write_boolean(false);
        }
    }

    // JAVA: net.minecraft.advancements.CriterionProgress#isDone
    pub fn is_done(&self) -> bool {
        self.obtained.is_some()
    }

    // JAVA: net.minecraft.advancements.CriterionProgress#getObtained
    pub fn get_obtained(&self) -> Option<i64> {
        self.obtained
    }
}

// JAVA: net.minecraft.advancements.AdvancementProgress
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AdvancementProgress {
    criteria: Vec<(String, CriterionProgress)>,
}

impl AdvancementProgress {
    // JAVA: net.minecraft.advancements.AdvancementProgress#<init>
    pub fn new(criteria: Vec<(String, CriterionProgress)>) -> Self {
        Self { criteria }
    }

    // JAVA: net.minecraft.advancements.AdvancementProgress#fromNetwork
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let count = buf.read_var_int() as usize;
        let mut criteria = Vec::with_capacity(count);
        for _ in 0..count {
            let name = buf.read_utf(32767);
            let progress = CriterionProgress::read(buf);
            criteria.push((name, progress));
        }
        Self { criteria }
    }

    // JAVA: net.minecraft.advancements.AdvancementProgress#serializeToNetwork
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.criteria.len() as i32);
        for (name, progress) in &self.criteria {
            buf.write_utf(name, 32767);
            progress.write(buf);
        }
    }

    // JAVA: net.minecraft.advancements.AdvancementProgress#isDone
    pub fn is_done(&self) -> bool {
        !self.criteria.is_empty() && self.criteria.iter().all(|(_, c)| c.is_done())
    }

    // JAVA: net.minecraft.advancements.AdvancementProgress#criteria
    pub fn criteria(&self) -> &[(String, CriterionProgress)] {
        &self.criteria
    }
}

// JAVA: net.minecraft.advancements.Advancement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Advancement {
    pub parent: Option<Identifier>,
    pub sends_telemetry: bool,
}

impl Advancement {
    // JAVA: net.minecraft.advancements.Advancement#<init>
    pub fn new(parent: Option<Identifier>, sends_telemetry: bool) -> Self {
        Self {
            parent,
            sends_telemetry,
        }
    }

    // JAVA: net.minecraft.advancements.Advancement#STREAM_CODEC decode (minimal self-contained subset)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let parent = if buf.read_boolean() {
            Some(buf.read_identifier())
        } else {
            None
        };
        let has_display = buf.read_boolean();
        if has_display {
            panic!("DisplayInfo not supported in this self-contained port");
        }
        let _experience = buf.read_var_int();
        let loot_count = buf.read_var_int() as usize;
        for _ in 0..loot_count {
            let _ = buf.read_identifier();
        }
        let recipe_count = buf.read_var_int() as usize;
        for _ in 0..recipe_count {
            let _ = buf.read_identifier();
        }
        let has_function = buf.read_boolean();
        if has_function {
            let _ = buf.read_identifier();
        }
        let criteria_count = buf.read_var_int() as usize;
        if criteria_count > 0 {
            panic!("Criterion codec not supported in this self-contained port");
        }
        let req_count = buf.read_var_int() as usize;
        for _ in 0..req_count {
            let inner = buf.read_var_int() as usize;
            for _ in 0..inner {
                let _ = buf.read_utf(32767);
            }
        }
        let sends_telemetry = buf.read_boolean();
        Self {
            parent,
            sends_telemetry,
        }
    }

    // JAVA: net.minecraft.advancements.Advancement#STREAM_CODEC encode (minimal self-contained subset)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        match &self.parent {
            Some(p) => {
                buf.write_boolean(true);
                buf.write_identifier(p);
            }
            None => {
                buf.write_boolean(false);
            }
        }
        buf.write_boolean(false); // no display
        buf.write_var_int(0); // experience
        buf.write_var_int(0); // loot
        buf.write_var_int(0); // recipes
        buf.write_boolean(false); // no function
        buf.write_var_int(0); // criteria
        buf.write_var_int(0); // requirements
        buf.write_boolean(self.sends_telemetry);
    }
}

// JAVA: net.minecraft.advancements.AdvancementHolder
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementHolder {
    pub id: Identifier,
    pub value: Advancement,
}

impl AdvancementHolder {
    // JAVA: net.minecraft.advancements.AdvancementHolder#<init>
    pub fn new(id: Identifier, value: Advancement) -> Self {
        Self { id, value }
    }

    // JAVA: net.minecraft.advancements.AdvancementHolder#read
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let id = buf.read_identifier();
        let value = Advancement::read(buf);
        Self { id, value }
    }

    // JAVA: net.minecraft.advancements.AdvancementHolder#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_identifier(&self.id);
        self.value.write(buf);
    }
}

// JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateAdvancementsPacket {
    reset: bool,
    added: Vec<AdvancementHolder>,
    removed: Vec<Identifier>,
    progress: Vec<(Identifier, AdvancementProgress)>,
    show_advancements: bool,
}

impl ClientboundUpdateAdvancementsPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket#<init>(boolean,Collection,Set,Map,boolean)
    pub fn new(
        reset: bool,
        added: Vec<AdvancementHolder>,
        removed: Vec<Identifier>,
        progress: Vec<(Identifier, AdvancementProgress)>,
        show_advancements: bool,
    ) -> Self {
        Self {
            reset,
            added,
            removed,
            progress,
            show_advancements,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let reset = buf.read_boolean();
        let added_count = buf.read_var_int() as usize;
        let mut added = Vec::with_capacity(added_count);
        for _ in 0..added_count {
            added.push(AdvancementHolder::read(buf));
        }
        let removed_count = buf.read_var_int() as usize;
        let mut removed = Vec::with_capacity(removed_count);
        for _ in 0..removed_count {
            removed.push(buf.read_identifier());
        }
        let progress_count = buf.read_var_int() as usize;
        let mut progress = Vec::with_capacity(progress_count);
        for _ in 0..progress_count {
            let key = buf.read_identifier();
            let value = AdvancementProgress::read(buf);
            progress.push((key, value));
        }
        let show_advancements = buf.read_boolean();
        Self {
            reset,
            added,
            removed,
            progress,
            show_advancements,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_boolean(self.reset);
        buf.write_var_int(self.added.len() as i32);
        for holder in &self.added {
            holder.write(buf);
        }
        buf.write_var_int(self.removed.len() as i32);
        for id in &self.removed {
            buf.write_identifier(id);
        }
        buf.write_var_int(self.progress.len() as i32);
        for (key, value) in &self.progress {
            buf.write_identifier(key);
            value.write(buf);
        }
        buf.write_boolean(self.show_advancements);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket#getAdded
    pub fn get_added(&self) -> &[AdvancementHolder] {
        &self.added
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket#getRemoved
    pub fn get_removed(&self) -> &[Identifier] {
        &self.removed
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket#getProgress
    pub fn get_progress(&self) -> &[(Identifier, AdvancementProgress)] {
        &self.progress
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket#shouldReset
    pub fn should_reset(&self) -> bool {
        self.reset
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket#shouldShowAdvancements
    pub fn should_show_advancements(&self) -> bool {
        self.show_advancements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> Identifier {
        Identifier::new(s)
    }

    #[test]
    fn round_trip_empty() {
        let pkt = ClientboundUpdateAdvancementsPacket::new(
            false,
            vec![],
            vec![],
            vec![],
            false,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundUpdateAdvancementsPacket::read(&mut r);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn round_trip_with_progress() {
        let progress = AdvancementProgress::new(vec![
            ("criterion_a".to_string(), CriterionProgress::obtained(123_456_789i64)),
            ("criterion_b".to_string(), CriterionProgress::not_obtained()),
        ]);
        let pkt = ClientboundUpdateAdvancementsPacket::new(
            true,
            vec![],
            vec![id("minecraft:test")],
            vec![(id("minecraft:root"), progress)],
            true,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundUpdateAdvancementsPacket::read(&mut r);
        assert!(decoded.should_reset());
        assert!(decoded.should_show_advancements());
        assert_eq!(decoded.get_removed().len(), 1);
        assert_eq!(decoded.get_progress().len(), 1);
        assert!(decoded.get_progress()[0].1.criteria()[0].1.is_done());
        assert!(!decoded.get_progress()[0].1.criteria()[1].1.is_done());
    }

    #[test]
    fn round_trip_with_added_holders() {
        let holder = AdvancementHolder::new(
            id("minecraft:story/root"),
            Advancement::new(None, false),
        );
        let holder2 = AdvancementHolder::new(
            id("minecraft:story/mine"),
            Advancement::new(Some(id("minecraft:story/root")), true),
        );
        let pkt = ClientboundUpdateAdvancementsPacket::new(
            false,
            vec![holder, holder2],
            vec![],
            vec![],
            false,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundUpdateAdvancementsPacket::read(&mut r);
        assert_eq!(pkt, decoded);
        assert_eq!(
            decoded.get_added()[1]
                .value
                .parent
                .as_ref()
                .unwrap()
                .value,
            "minecraft:story/root"
        );
    }

    #[test]
    fn criterion_negative_timestamp_and_empty_progress() {
        let progress = AdvancementProgress::new(vec![
            ("c".to_string(), CriterionProgress::obtained(-1i64)),
            ("d".to_string(), CriterionProgress::obtained(i64::MIN)),
        ]);
        let mut buf = FriendlyByteBuf::new();
        progress.write(&mut buf);
        let mut r = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = AdvancementProgress::read(&mut r);
        assert_eq!(decoded, progress);
        assert!(decoded.is_done());
    }

    #[test]
    fn flags_and_large_collection_round_trip() {
        let removed: Vec<Identifier> = (0..10).map(|i| id(&format!("minecraft:r{i}"))).collect();
        let pkt = ClientboundUpdateAdvancementsPacket::new(
            true,
            vec![],
            removed.clone(),
            vec![],
            true,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundUpdateAdvancementsPacket::read(&mut r);
        assert_eq!(decoded.get_removed(), removed.as_slice());
        assert!(decoded.should_reset());
        assert!(decoded.should_show_advancements());
    }
}
