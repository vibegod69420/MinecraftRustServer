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

    pub fn read_utf(&mut self) -> String {
        self.read_utf_with_max_len(32767)
    }

    pub fn read_utf_with_max_len(&mut self, max_len: usize) -> String {
        let byte_len = self.read_var_int();
        assert!(byte_len >= 0, "negative string length");
        let byte_len = byte_len as usize;
        let max_bytes = max_len.saturating_mul(3);
        assert!(byte_len <= max_bytes, "string too long");
        let end = self
            .reader_index
            .checked_add(byte_len)
            .expect("length overflow");
        assert!(end <= self.data.len(), "buffer underflow");
        let s = std::str::from_utf8(&self.data[self.reader_index..end])
            .expect("invalid utf8")
            .to_string();
        assert!(s.chars().count() <= max_len, "string too long");
        self.reader_index = end;
        s
    }

    pub fn write_utf(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    pub fn read_by_id<T, F: FnOnce(i32) -> T>(&mut self, f: F) -> T {
        let id = self.read_var_int();
        f(id)
    }

    pub fn write_by_id<T, F: FnOnce(&T) -> i32>(&mut self, f: F, value: &T) {
        self.write_var_int(f(value));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplaySlot {
    List,
    Sidebar,
    BelowName,
    TeamBlack,
    TeamDarkBlue,
    TeamDarkGreen,
    TeamDarkAqua,
    TeamDarkRed,
    TeamDarkPurple,
    TeamGold,
    TeamGray,
    TeamDarkGray,
    TeamBlue,
    TeamGreen,
    TeamAqua,
    TeamRed,
    TeamLightPurple,
    TeamYellow,
    TeamWhite,
}

impl DisplaySlot {
    pub fn id(&self) -> i32 {
        match self {
            DisplaySlot::List => 0,
            DisplaySlot::Sidebar => 1,
            DisplaySlot::BelowName => 2,
            DisplaySlot::TeamBlack => 3,
            DisplaySlot::TeamDarkBlue => 4,
            DisplaySlot::TeamDarkGreen => 5,
            DisplaySlot::TeamDarkAqua => 6,
            DisplaySlot::TeamDarkRed => 7,
            DisplaySlot::TeamDarkPurple => 8,
            DisplaySlot::TeamGold => 9,
            DisplaySlot::TeamGray => 10,
            DisplaySlot::TeamDarkGray => 11,
            DisplaySlot::TeamBlue => 12,
            DisplaySlot::TeamGreen => 13,
            DisplaySlot::TeamAqua => 14,
            DisplaySlot::TeamRed => 15,
            DisplaySlot::TeamLightPurple => 16,
            DisplaySlot::TeamYellow => 17,
            DisplaySlot::TeamWhite => 18,
        }
    }

    pub fn by_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(DisplaySlot::List),
            1 => Some(DisplaySlot::Sidebar),
            2 => Some(DisplaySlot::BelowName),
            3 => Some(DisplaySlot::TeamBlack),
            4 => Some(DisplaySlot::TeamDarkBlue),
            5 => Some(DisplaySlot::TeamDarkGreen),
            6 => Some(DisplaySlot::TeamDarkAqua),
            7 => Some(DisplaySlot::TeamDarkRed),
            8 => Some(DisplaySlot::TeamDarkPurple),
            9 => Some(DisplaySlot::TeamGold),
            10 => Some(DisplaySlot::TeamGray),
            11 => Some(DisplaySlot::TeamDarkGray),
            12 => Some(DisplaySlot::TeamBlue),
            13 => Some(DisplaySlot::TeamGreen),
            14 => Some(DisplaySlot::TeamAqua),
            15 => Some(DisplaySlot::TeamRed),
            16 => Some(DisplaySlot::TeamLightPurple),
            17 => Some(DisplaySlot::TeamYellow),
            18 => Some(DisplaySlot::TeamWhite),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Objective {
    name: String,
}

impl Objective {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub trait ClientGamePacketListener {
    fn handle_set_display_objective(&mut self, packet: &ClientboundSetDisplayObjectivePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetDisplayObjectivePacket {
    slot: DisplaySlot,
    objective_name: String,
}

impl ClientboundSetDisplayObjectivePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDisplayObjectivePacket#<init>
    pub fn new(slot: DisplaySlot, objective: Option<&Objective>) -> Self {
        let objective_name = match objective {
            None => String::new(),
            Some(o) => o.get_name().to_string(),
        };
        Self {
            slot,
            objective_name,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDisplayObjectivePacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        let slot = buf.read_by_id(|id| DisplaySlot::by_id(id).expect("unknown DisplaySlot id"));
        let objective_name = buf.read_utf();
        Self {
            slot,
            objective_name,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDisplayObjectivePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_by_id(|s: &DisplaySlot| s.id(), &self.slot);
        buf.write_utf(&self.objective_name);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDisplayObjectivePacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDisplayObjectivePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_set_display_objective(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDisplayObjectivePacket#getSlot
    pub fn get_slot(&self) -> DisplaySlot {
        self.slot
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDisplayObjectivePacket#getObjectiveName
    pub fn get_objective_name(&self) -> Option<&str> {
        if self.objective_name.is_empty() {
            None
        } else {
            Some(&self.objective_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen: Option<ClientboundSetDisplayObjectivePacket>,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_set_display_objective(&mut self, packet: &ClientboundSetDisplayObjectivePacket) {
            self.seen = Some(packet.clone());
        }
    }

    #[test]
    fn round_trip_with_objective() {
        let obj = Objective::new("score".to_string());
        let packet = ClientboundSetDisplayObjectivePacket::new(DisplaySlot::Sidebar, Some(&obj));
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundSetDisplayObjectivePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_slot(), DisplaySlot::Sidebar);
        assert_eq!(decoded.get_objective_name(), Some("score"));
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn null_objective_becomes_empty_name() {
        let packet = ClientboundSetDisplayObjectivePacket::new(DisplaySlot::List, None);
        assert_eq!(packet.get_objective_name(), None);
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundSetDisplayObjectivePacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_objective_name(), None);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ClientboundSetDisplayObjectivePacket::new(DisplaySlot::BelowName, None);
        let mut listener = TestListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen.as_ref().map(|p| p.get_slot()), Some(DisplaySlot::BelowName));
    }

    #[test]
    fn display_slot_id_round_trip() {
        for id in -1..=20 {
            let slot = DisplaySlot::by_id(id);
            if (0..=18).contains(&id) {
                let s = slot.expect("expected slot");
                assert_eq!(s.id(), id);
            } else {
                assert!(slot.is_none());
            }
        }
    }

    #[test]
    fn var_int_negative_round_trip() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_var_int(-1);
        buf.write_var_int(i32::MIN);
        buf.write_var_int(i32::MAX);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        assert_eq!(read_buf.read_var_int(), -1);
        assert_eq!(read_buf.read_var_int(), i32::MIN);
        assert_eq!(read_buf.read_var_int(), i32::MAX);
        assert_eq!(read_buf.remaining(), 0);
    }
}
