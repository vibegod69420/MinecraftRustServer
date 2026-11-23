use std::io;

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn read_unsigned_byte(&mut self) -> Result<i32, io::Error> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "read_unsigned_byte",
            ));
        }
        let b = self.data[self.read_pos];
        self.read_pos = self.read_pos.wrapping_add(1);
        Ok(i32::from(b))
    }

    pub fn read_float(&mut self) -> Result<f32, io::Error> {
        let end = self.read_pos.wrapping_add(4);
        if end > self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_float"));
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.data[self.read_pos..end]);
        self.read_pos = end;
        Ok(f32::from_bits(u32::from_be_bytes(bytes)))
    }

    pub fn write_byte(&mut self, value: i32) {
        self.data.push(value as u8);
    }

    pub fn write_float(&mut self, value: f32) {
        self.data
            .extend_from_slice(&value.to_bits().to_be_bytes());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub trait ClientGamePacketListener {
    fn handle_game_event(&mut self, packet: &ClientboundGameEventPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameEventType {
    pub id: i32,
}

pub const NO_RESPAWN_BLOCK_AVAILABLE: GameEventType = GameEventType { id: 0 };
pub const START_RAINING: GameEventType = GameEventType { id: 1 };
pub const STOP_RAINING: GameEventType = GameEventType { id: 2 };
pub const CHANGE_GAME_MODE: GameEventType = GameEventType { id: 3 };
pub const WIN_GAME: GameEventType = GameEventType { id: 4 };
pub const DEMO_EVENT: GameEventType = GameEventType { id: 5 };
pub const PLAY_ARROW_HIT_SOUND: GameEventType = GameEventType { id: 6 };
pub const RAIN_LEVEL_CHANGE: GameEventType = GameEventType { id: 7 };
pub const THUNDER_LEVEL_CHANGE: GameEventType = GameEventType { id: 8 };
pub const PUFFER_FISH_STING: GameEventType = GameEventType { id: 9 };
pub const GUARDIAN_ELDER_EFFECT: GameEventType = GameEventType { id: 10 };
pub const IMMEDIATE_RESPAWN: GameEventType = GameEventType { id: 11 };
pub const LIMITED_CRAFTING: GameEventType = GameEventType { id: 12 };
pub const LEVEL_CHUNKS_LOAD_START: GameEventType = GameEventType { id: 13 };

impl GameEventType {
    // JAVA: net.minecraft.network.protocol.game.ClientboundGameEventPacket$Type#<init>
    pub const fn new(id: i32) -> Self {
        Self { id }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(NO_RESPAWN_BLOCK_AVAILABLE),
            1 => Some(START_RAINING),
            2 => Some(STOP_RAINING),
            3 => Some(CHANGE_GAME_MODE),
            4 => Some(WIN_GAME),
            5 => Some(DEMO_EVENT),
            6 => Some(PLAY_ARROW_HIT_SOUND),
            7 => Some(RAIN_LEVEL_CHANGE),
            8 => Some(THUNDER_LEVEL_CHANGE),
            9 => Some(PUFFER_FISH_STING),
            10 => Some(GUARDIAN_ELDER_EFFECT),
            11 => Some(IMMEDIATE_RESPAWN),
            12 => Some(LIMITED_CRAFTING),
            13 => Some(LEVEL_CHUNKS_LOAD_START),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundGameEventPacket {
    event: GameEventType,
    param: f32,
}

impl ClientboundGameEventPacket {
    pub const DEMO_PARAM_INTRO: i32 = 0;
    pub const DEMO_PARAM_HINT_1: i32 = 101;
    pub const DEMO_PARAM_HINT_2: i32 = 102;
    pub const DEMO_PARAM_HINT_3: i32 = 103;
    pub const DEMO_PARAM_HINT_4: i32 = 104;

    pub const NO_RESPAWN_BLOCK_AVAILABLE: GameEventType = NO_RESPAWN_BLOCK_AVAILABLE;
    pub const START_RAINING: GameEventType = START_RAINING;
    pub const STOP_RAINING: GameEventType = STOP_RAINING;
    pub const CHANGE_GAME_MODE: GameEventType = CHANGE_GAME_MODE;
    pub const WIN_GAME: GameEventType = WIN_GAME;
    pub const DEMO_EVENT: GameEventType = DEMO_EVENT;
    pub const PLAY_ARROW_HIT_SOUND: GameEventType = PLAY_ARROW_HIT_SOUND;
    pub const RAIN_LEVEL_CHANGE: GameEventType = RAIN_LEVEL_CHANGE;
    pub const THUNDER_LEVEL_CHANGE: GameEventType = THUNDER_LEVEL_CHANGE;
    pub const PUFFER_FISH_STING: GameEventType = PUFFER_FISH_STING;
    pub const GUARDIAN_ELDER_EFFECT: GameEventType = GUARDIAN_ELDER_EFFECT;
    pub const IMMEDIATE_RESPAWN: GameEventType = IMMEDIATE_RESPAWN;
    pub const LIMITED_CRAFTING: GameEventType = LIMITED_CRAFTING;
    pub const LEVEL_CHUNKS_LOAD_START: GameEventType = LEVEL_CHUNKS_LOAD_START;

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameEventPacket#<init>
    pub fn new(event: GameEventType, param: f32) -> Self {
        Self { event, param }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameEventPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, io::Error> {
        let id = buf.read_unsigned_byte()?;
        let event = GameEventType::from_id(id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown game event type: {id}"),
            )
        })?;
        let param = buf.read_float()?;
        Ok(Self { event, param })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameEventPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_byte(self.event.id);
        buf.write_float(self.param);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameEventPacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameEventPacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_game_event(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameEventPacket#getEvent
    pub fn get_event(&self) -> GameEventType {
        self.event
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameEventPacket#getParam
    pub fn get_param(&self) -> f32 {
        self.param
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(event: GameEventType, param: f32) -> ClientboundGameEventPacket {
        let pkt = ClientboundGameEventPacket::new(event, param);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        ClientboundGameEventPacket::read(&mut read_buf).expect("decode should succeed")
    }

    #[test]
    fn roundtrip_all_event_types() {
        let events = [
            NO_RESPAWN_BLOCK_AVAILABLE,
            START_RAINING,
            STOP_RAINING,
            CHANGE_GAME_MODE,
            WIN_GAME,
            DEMO_EVENT,
            PLAY_ARROW_HIT_SOUND,
            RAIN_LEVEL_CHANGE,
            THUNDER_LEVEL_CHANGE,
            PUFFER_FISH_STING,
            GUARDIAN_ELDER_EFFECT,
            IMMEDIATE_RESPAWN,
            LIMITED_CRAFTING,
            LEVEL_CHUNKS_LOAD_START,
        ];
        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.id, i as i32);
            let decoded = roundtrip(*event, 1.0);
            assert_eq!(decoded.get_event(), *event);
            assert_eq!(decoded.get_param().to_bits(), 1.0f32.to_bits());
        }
    }

    #[test]
    fn float_bits_preserved_including_nan_and_infinities() {
        let values = [
            0.0f32,
            -0.0f32,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            -123.456f32,
        ];
        for v in values {
            let pkt = ClientboundGameEventPacket::new(START_RAINING, v);
            let mut buf = FriendlyByteBuf::new();
            pkt.write(&mut buf);
            let bytes = buf.as_slice();
            let wire_bits = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
            assert_eq!(wire_bits, v.to_bits(), "wire bits mismatch for {v:?}");
            let decoded = roundtrip(START_RAINING, v);
            assert_eq!(decoded.get_param().to_bits(), v.to_bits());
        }
    }

    #[test]
    fn negative_param_and_demo_constants() {
        let decoded = roundtrip(RAIN_LEVEL_CHANGE, -1.0);
        assert_eq!(decoded.get_event(), RAIN_LEVEL_CHANGE);
        assert_eq!(decoded.get_param().to_bits(), (-1.0f32).to_bits());

        assert_eq!(ClientboundGameEventPacket::DEMO_PARAM_INTRO, 0);
        assert_eq!(ClientboundGameEventPacket::DEMO_PARAM_HINT_1, 101);
        assert_eq!(ClientboundGameEventPacket::DEMO_PARAM_HINT_2, 102);
        assert_eq!(ClientboundGameEventPacket::DEMO_PARAM_HINT_3, 103);
        assert_eq!(ClientboundGameEventPacket::DEMO_PARAM_HINT_4, 104);
    }

    #[test]
    fn unknown_event_type_and_underflow_errors() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_byte(255);
        buf.write_float(0.0);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let err = ClientboundGameEventPacket::read(&mut read_buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);

        let mut short_buf = FriendlyByteBuf::from_vec(vec![0]);
        let err = ClientboundGameEventPacket::read(&mut short_buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn write_byte_truncation_negative_and_overflow() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_byte(-1);
        buf.write_byte(256);
        buf.write_byte(i32::MIN);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        assert_eq!(read_buf.read_unsigned_byte().unwrap(), 255);
        assert_eq!(read_buf.read_unsigned_byte().unwrap(), 0);
        assert_eq!(read_buf.read_unsigned_byte().unwrap(), 0);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct MockListener {
            seen: Option<(GameEventType, u32)>,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_game_event(&mut self, packet: &ClientboundGameEventPacket) {
                self.seen = Some((packet.get_event(), packet.get_param().to_bits()));
            }
        }

        let pkt = ClientboundGameEventPacket::new(ClientboundGameEventPacket::WIN_GAME, 1.0);
        let mut listener = MockListener { seen: None };
        pkt.handle(&mut listener);
        let (ev, param_bits) = listener.seen.expect("listener should be called");
        assert_eq!(ev, WIN_GAME);
        assert_eq!(param_bits, 1.0f32.to_bits());
        assert_eq!(pkt.get_type(), PacketType);
    }
}
