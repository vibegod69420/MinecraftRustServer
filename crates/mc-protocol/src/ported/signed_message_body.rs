use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instant {
    pub seconds: i64,
    pub nanos: i32,
}

impl Instant {
    // JAVA: java.time.Instant#now
    pub fn now() -> Self {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => Self {
                seconds: d.as_secs() as i64,
                nanos: d.subsec_nanos() as i32,
            },
            Err(e) => {
                let d = e.duration();
                Self {
                    seconds: -(d.as_secs() as i64),
                    nanos: -(d.subsec_nanos() as i32),
                }
            }
        }
    }

    // JAVA: java.time.Instant#getEpochSecond
    pub fn get_epoch_second(&self) -> i64 {
        self.seconds
    }

    // JAVA: java.time.Instant#toEpochMilli
    pub fn to_epoch_milli(&self) -> i64 {
        self.seconds
            .wrapping_mul(1000)
            .wrapping_add(i64::from(self.nanos / 1_000_000))
    }

    // JAVA: java.time.Instant#ofEpochMilli
    pub fn from_epoch_milli(millis: i64) -> Self {
        let seconds = millis.div_euclid(1000);
        let rem = millis.rem_euclid(1000) as i32;
        Self {
            seconds,
            nanos: rem.wrapping_mul(1_000_000),
        }
    }
}

#[derive(Debug)]
pub struct SignatureException;

pub trait SignatureUpdaterOutput {
    fn update(&mut self, data: &[u8]) -> Result<(), SignatureException>;
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MessageSignature(pub Vec<u8>);

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MessageSignatureCache;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LastSeenMessages {
    pub entries: Vec<MessageSignature>,
}

impl LastSeenMessages {
    // JAVA: net.minecraft.network.chat.LastSeenMessages#<init>
    pub fn new(entries: Vec<MessageSignature>) -> Self {
        Self { entries }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages#empty
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages#updateSignature
    pub fn update_signature(
        &self,
        _output: &mut dyn SignatureUpdaterOutput,
    ) -> Result<(), SignatureException> {
        Ok(())
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages#pack
    pub fn pack(&self, _cache: &MessageSignatureCache) -> LastSeenMessagesPacked {
        LastSeenMessagesPacked {
            entries: self.entries.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LastSeenMessagesPacked {
    pub entries: Vec<MessageSignature>,
}

impl LastSeenMessagesPacked {
    // JAVA: net.minecraft.network.chat.LastSeenMessages$Packed#<init>
    pub fn read(_buf: &mut FriendlyByteBuf) -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Packed#write
    pub fn write(&self, _buf: &mut FriendlyByteBuf) {}

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Packed#unpack
    pub fn unpack(&self, _cache: &MessageSignatureCache) -> Option<LastSeenMessages> {
        Some(LastSeenMessages {
            entries: self.entries.clone(),
        })
    }
}

pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writerIndex/readableBytes equivalent
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong
    pub fn read_long(&mut self) -> i64 {
        let mut bytes = [0u8; 8];
        if self.pos + 8 <= self.data.len() {
            bytes.copy_from_slice(&self.data[self.pos..self.pos + 8]);
            self.pos += 8;
        }
        i64::from_be_bytes(bytes)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
    pub fn write_long(&mut self, v: i64) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_varint(&mut self) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            if self.pos >= self.data.len() {
                break;
            }
            let b = self.data[self.pos];
            self.pos += 1;
            result |= i32::from(b & 0x7F) << (shift & 31);
            if (b & 0x80) == 0 {
                break;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                break;
            }
        }
        result
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_varint(&mut self, mut value: i32) {
        let mut v = value as u32;
        loop {
            if (v & !0x7Fu32) == 0 {
                self.data.push(v as u8);
                return;
            }
            self.data.push(((v & 0x7F) as u8) | 0x80);
            v >>= 7;
            let _ = &mut value;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf(int)
    pub fn read_utf(&mut self, _max_length: usize) -> String {
        let len = self.read_varint() as usize;
        if self.pos + len > self.data.len() {
            return String::new();
        }
        let bytes = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        String::from_utf8(bytes).unwrap_or_default()
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf(String,int)
    pub fn write_utf(&mut self, s: &str, _max_length: usize) {
        let bytes = s.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readInstant
    pub fn read_instant(&mut self) -> Instant {
        Instant::from_epoch_milli(self.read_long())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeInstant
    pub fn write_instant(&mut self, instant: &Instant) {
        self.write_long(instant.to_epoch_milli());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedMessageBody {
    pub content: String,
    pub time_stamp: Instant,
    pub salt: i64,
    pub last_seen: LastSeenMessages,
}

impl SignedMessageBody {
    // JAVA: net.minecraft.network.chat.SignedMessageBody#unsigned
    pub fn unsigned(content: String) -> Self {
        Self {
            content,
            time_stamp: Instant::now(),
            salt: 0i64,
            last_seen: LastSeenMessages::empty(),
        }
    }

    // JAVA: net.minecraft.network.chat.SignedMessageBody#updateSignature
    pub fn update_signature(
        &self,
        output: &mut dyn SignatureUpdaterOutput,
    ) -> Result<(), SignatureException> {
        output.update(&self.salt.to_be_bytes())?;
        output.update(&self.time_stamp.get_epoch_second().to_be_bytes())?;
        let content_bytes = self.content.as_bytes();
        let len = content_bytes.len() as i32;
        output.update(&len.to_be_bytes())?;
        output.update(content_bytes)?;
        self.last_seen.update_signature(output)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.chat.SignedMessageBody#pack
    pub fn pack(&self, cache: &MessageSignatureCache) -> Packed {
        Packed {
            content: self.content.clone(),
            time_stamp: self.time_stamp,
            salt: self.salt,
            last_seen: self.last_seen.pack(cache),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packed {
    pub content: String,
    pub time_stamp: Instant,
    pub salt: i64,
    pub last_seen: LastSeenMessagesPacked,
}

impl Packed {
    // JAVA: net.minecraft.network.chat.SignedMessageBody$Packed#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let content = buf.read_utf(256);
        let time_stamp = buf.read_instant();
        let salt = buf.read_long();
        let last_seen = LastSeenMessagesPacked::read(buf);
        Self {
            content,
            time_stamp,
            salt,
            last_seen,
        }
    }

    // JAVA: net.minecraft.network.chat.SignedMessageBody$Packed#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.content, 256);
        buf.write_instant(&self.time_stamp);
        buf.write_long(self.salt);
        self.last_seen.write(buf);
    }

    // JAVA: net.minecraft.network.chat.SignedMessageBody$Packed#unpack
    pub fn unpack(&self, cache: &MessageSignatureCache) -> Option<SignedMessageBody> {
        self.last_seen
            .unpack(cache)
            .map(|last_seen| SignedMessageBody {
                content: self.content.clone(),
                time_stamp: self.time_stamp,
                salt: self.salt,
                last_seen,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockOutput {
        data: Vec<u8>,
    }

    impl SignatureUpdaterOutput for MockOutput {
        fn update(&mut self, data: &[u8]) -> Result<(), SignatureException> {
            self.data.extend_from_slice(data);
            Ok(())
        }
    }

    #[test]
    fn unsigned_creates_empty_body() {
        let body = SignedMessageBody::unsigned("hello".to_string());
        assert_eq!(body.content, "hello");
        assert_eq!(body.salt, 0i64);
        assert!(body.last_seen.entries.is_empty());
    }

    #[test]
    fn update_signature_writes_expected_bytes() {
        let body = SignedMessageBody {
            content: "hi".to_string(),
            time_stamp: Instant {
                seconds: 100,
                nanos: 0,
            },
            salt: 42i64,
            last_seen: LastSeenMessages::empty(),
        };
        let mut out = MockOutput { data: Vec::new() };
        body.update_signature(&mut out).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&42i64.to_be_bytes());
        expected.extend_from_slice(&100i64.to_be_bytes());
        expected.extend_from_slice(&2i32.to_be_bytes());
        expected.extend_from_slice(b"hi");
        assert_eq!(out.data, expected);
    }

    #[test]
    fn update_signature_negative_values() {
        let body = SignedMessageBody {
            content: String::new(),
            time_stamp: Instant {
                seconds: -1,
                nanos: 0,
            },
            salt: -1i64,
            last_seen: LastSeenMessages::empty(),
        };
        let mut out = MockOutput { data: Vec::new() };
        body.update_signature(&mut out).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&(-1i64).to_be_bytes());
        expected.extend_from_slice(&(-1i64).to_be_bytes());
        expected.extend_from_slice(&0i32.to_be_bytes());
        assert_eq!(out.data, expected);
    }

    #[test]
    fn pack_and_unpack_round_trip() {
        let body = SignedMessageBody {
            content: "msg".to_string(),
            time_stamp: Instant {
                seconds: 123,
                nanos: 0,
            },
            salt: 999i64,
            last_seen: LastSeenMessages::empty(),
        };
        let cache = MessageSignatureCache;
        let packed = body.pack(&cache);
        let unpacked = packed.unpack(&cache).unwrap();
        assert_eq!(unpacked.content, body.content);
        assert_eq!(unpacked.salt, body.salt);
        assert_eq!(unpacked.time_stamp, body.time_stamp);
    }

    #[test]
    fn packed_write_read_round_trip() {
        let packed = Packed {
            content: "test".to_string(),
            time_stamp: Instant::from_epoch_milli(5000),
            salt: 7i64,
            last_seen: LastSeenMessagesPacked::default(),
        };
        let mut buf = FriendlyByteBuf::new(Vec::new());
        packed.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let read_back = Packed::read(&mut read_buf);
        assert_eq!(read_back.content, packed.content);
        assert_eq!(read_back.salt, packed.salt);
        assert_eq!(read_back.time_stamp, packed.time_stamp);
    }
}
