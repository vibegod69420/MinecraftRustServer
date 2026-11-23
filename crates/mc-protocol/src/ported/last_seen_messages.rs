use std::fmt;

pub const LAST_SEEN_MESSAGES_MAX_LENGTH: i32 = 20;

#[derive(Debug)]
pub struct SignatureException;

impl fmt::Display for SignatureException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SignatureException")
    }
}

impl std::error::Error for SignatureException {}

pub trait SignatureUpdaterOutput {
    fn update(&mut self, data: &[u8]) -> Result<(), SignatureException>;
}

#[derive(Clone, Debug, Default)]
pub struct MessageSignatureCache;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitSet {
    bits: Vec<u8>,
    size: usize,
}

impl BitSet {
    pub fn new(size: usize) -> Self {
        Self {
            bits: vec![0u8; (size + 7) / 8],
            size,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, size: usize) -> Self {
        Self { bits: bytes, size }
    }

    pub fn to_bytes(&self, byte_len: usize) -> Vec<u8> {
        let mut out = vec![0u8; byte_len];
        let n = out.len().min(self.bits.len());
        out[..n].copy_from_slice(&self.bits[..n]);
        out
    }

    pub fn get(&self, index: usize) -> bool {
        if index >= self.size {
            return false;
        }
        (self.bits[index / 8] >> (index % 8)) & 1 == 1
    }

    pub fn set(&mut self, index: usize, value: bool) {
        if index >= self.size {
            return;
        }
        let byte = index / 8;
        let bit = index % 8;
        if value {
            self.bits[byte] |= 1 << bit;
        } else {
            self.bits[byte] &= !(1 << bit);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn read_byte(&mut self) -> i8 {
        let b = self.data[self.pos] as i8;
        self.pos += 1;
        b
    }

    pub fn write_byte(&mut self, v: i8) {
        self.data.push(v as u8);
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut val: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.data[self.pos];
            self.pos += 1;
            val |= ((b & 0x7F) as i32) << shift;
            if (b & 0x80) == 0 {
                break;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                break;
            }
        }
        val
    }

    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F) == 0 {
                self.data.push(value as u8);
                return;
            }
            self.data.push(((value & 0x7F) as u8) | 0x80);
            value = ((value as u32) >> 7) as i32;
        }
    }

    pub fn read_fixed_bit_set(&mut self, size: usize) -> BitSet {
        let byte_len = (size + 7) / 8;
        let mut bytes = vec![0u8; byte_len];
        bytes.copy_from_slice(&self.data[self.pos..self.pos + byte_len]);
        self.pos += byte_len;
        BitSet::from_bytes(bytes, size)
    }

    pub fn write_fixed_bit_set(&mut self, bit_set: &BitSet, size: usize) {
        let byte_len = (size + 7) / 8;
        let bytes = bit_set.to_bytes(byte_len);
        self.data.extend_from_slice(&bytes);
    }

    pub fn read_bool(&mut self) -> bool {
        let b = self.data[self.pos];
        self.pos += 1;
        b != 0
    }

    pub fn write_bool(&mut self, v: bool) {
        self.data.push(if v { 1 } else { 0 });
    }

    pub fn read_bytes_into(&mut self, dst: &mut [u8]) {
        dst.copy_from_slice(&self.data[self.pos..self.pos + dst.len()]);
        self.pos += dst.len();
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn read_collection_limited<T, F>(&mut self, limit: usize, mut reader: F) -> Vec<T>
    where
        F: FnMut(&mut FriendlyByteBuf) -> T,
    {
        let len = self.read_var_int() as usize;
        assert!(len <= limit, "Collection too large: {} > {}", len, limit);
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(reader(self));
        }
        out
    }

    pub fn write_collection<T, F>(&mut self, items: &[T], mut writer: F)
    where
        F: FnMut(&mut FriendlyByteBuf, &T),
    {
        self.write_var_int(items.len() as i32);
        for item in items {
            writer(self, item);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSignature {
    bytes: Vec<u8>,
}

impl MessageSignature {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn checksum(&self) -> i32 {
        let mut h: i32 = 1;
        for &b in &self.bytes {
            h = h.wrapping_mul(31).wrapping_add(i32::from(b as i8));
        }
        h
    }

    pub fn pack(&self, _cache: &MessageSignatureCache) -> MessageSignaturePacked {
        MessageSignaturePacked {
            bytes: Some(self.bytes.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSignaturePacked {
    bytes: Option<Vec<u8>>,
}

impl MessageSignaturePacked {
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let present = buf.read_bool();
        if present {
            let len = buf.read_var_int() as usize;
            let mut bytes = vec![0u8; len];
            buf.read_bytes_into(&mut bytes);
            Self { bytes: Some(bytes) }
        } else {
            Self { bytes: None }
        }
    }

    pub fn write(buf: &mut FriendlyByteBuf, packed: &MessageSignaturePacked) {
        match &packed.bytes {
            Some(bytes) => {
                buf.write_bool(true);
                buf.write_var_int(bytes.len() as i32);
                buf.write_bytes(bytes);
            }
            None => {
                buf.write_bool(false);
            }
        }
    }

    pub fn unpack(&self, _cache: &MessageSignatureCache) -> Option<MessageSignature> {
        self.bytes.clone().map(MessageSignature::new)
    }
}

// JAVA: net.minecraft.network.chat.LastSeenMessages
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastSeenMessages {
    pub entries: Vec<MessageSignature>,
}

impl LastSeenMessages {
    // JAVA: net.minecraft.network.chat.LastSeenMessages#<init>
    pub fn new(entries: Vec<MessageSignature>) -> Self {
        Self { entries }
    }

    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn entries(&self) -> &[MessageSignature] {
        &self.entries
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages#updateSignature
    pub fn update_signature<O: SignatureUpdaterOutput>(
        &self,
        output: &mut O,
    ) -> Result<(), SignatureException> {
        let len_bytes = (self.entries.len() as i32).to_be_bytes();
        output.update(&len_bytes)?;
        for entry in &self.entries {
            output.update(entry.bytes())?;
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages#pack
    pub fn pack(&self, cache: &MessageSignatureCache) -> Packed {
        Packed {
            entries: self.entries.iter().map(|e| e.pack(cache)).collect(),
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages#computeChecksum
    pub fn compute_checksum(&self) -> i8 {
        let mut hash: i32 = 1;
        for entry in &self.entries {
            hash = hash.wrapping_mul(31).wrapping_add(entry.checksum());
        }
        let b = hash as i8;
        if b == 0 {
            1
        } else {
            b
        }
    }
}

// JAVA: net.minecraft.network.chat.LastSeenMessages$Packed
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packed {
    pub entries: Vec<MessageSignaturePacked>,
}

impl Packed {
    // JAVA: net.minecraft.network.chat.LastSeenMessages$Packed#<init>
    pub fn new(entries: Vec<MessageSignaturePacked>) -> Self {
        Self { entries }
    }

    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Packed#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let entries = buf.read_collection_limited(20, MessageSignaturePacked::read);
        Self { entries }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Packed#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.entries, |b, e| MessageSignaturePacked::write(b, e));
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Packed#unpack
    pub fn unpack(&self, cache: &MessageSignatureCache) -> Option<LastSeenMessages> {
        let mut out = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            match entry.unpack(cache) {
                Some(sig) => out.push(sig),
                None => return None,
            }
        }
        Some(LastSeenMessages::new(out))
    }
}

pub const IGNORE_CHECKSUM: i8 = 0;

// JAVA: net.minecraft.network.chat.LastSeenMessages$Update
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Update {
    pub offset: i32,
    pub acknowledged: BitSet,
    pub checksum: i8,
}

impl Update {
    // JAVA: net.minecraft.network.chat.LastSeenMessages$Update#<init>
    pub fn new(offset: i32, acknowledged: BitSet, checksum: i8) -> Self {
        Self {
            offset,
            acknowledged,
            checksum,
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Update#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let offset = buf.read_var_int();
        let acknowledged = buf.read_fixed_bit_set(20);
        let checksum = buf.read_byte();
        Self {
            offset,
            acknowledged,
            checksum,
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Update#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.offset);
        buf.write_fixed_bit_set(&self.acknowledged, 20);
        buf.write_byte(self.checksum);
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessages$Update#verifyChecksum
    pub fn verify_checksum(&self, messages: &LastSeenMessages) -> bool {
        self.checksum == 0 || self.checksum == messages.compute_checksum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct VecUpdater(Vec<u8>);
    impl SignatureUpdaterOutput for VecUpdater {
        fn update(&mut self, data: &[u8]) -> Result<(), SignatureException> {
            self.0.extend_from_slice(data);
            Ok(())
        }
    }

    fn sig(bytes: &[u8]) -> MessageSignature {
        MessageSignature::new(bytes.to_vec())
    }

    #[test]
    fn empty_checksum_and_update_signature() {
        let msgs = LastSeenMessages::empty();
        assert_eq!(msgs.compute_checksum(), 1);
        let mut out = VecUpdater(Vec::new());
        msgs.update_signature(&mut out).unwrap();
        assert_eq!(out.0, 0i32.to_be_bytes());
    }

    #[test]
    fn compute_checksum_wrapping_matches_formula() {
        let msgs = LastSeenMessages::new(vec![sig(&[1, 2, 3]), sig(&[4, 5])]);
        let mut h: i32 = 1;
        for e in msgs.entries() {
            h = h.wrapping_mul(31).wrapping_add(e.checksum());
        }
        let mut b = h as i8;
        if b == 0 {
            b = 1;
        }
        assert_eq!(msgs.compute_checksum(), b);
    }

    #[test]
    fn update_verify_checksum_handles_ignore_and_mismatch() {
        let msgs = LastSeenMessages::new(vec![sig(&[10])]);
        let c = msgs.compute_checksum();
        let ack = BitSet::new(20);
        let u_ok = Update::new(0, ack.clone(), c);
        let u_ignore = Update::new(0, ack.clone(), IGNORE_CHECKSUM);
        let u_bad = Update::new(0, ack, c.wrapping_add(1) as i8);
        assert!(u_ok.verify_checksum(&msgs));
        assert!(u_ignore.verify_checksum(&msgs));
        assert!(!u_bad.verify_checksum(&msgs));
    }

    #[test]
    fn update_write_read_roundtrip() {
        let mut ack = BitSet::new(20);
        ack.set(1, true);
        ack.set(19, true);
        let u = Update::new(-7, ack, -5);
        let mut buf = FriendlyByteBuf::default();
        u.write(&mut buf);
        let mut r = FriendlyByteBuf::new(buf.into_inner());
        let u2 = Update::read(&mut r);
        assert_eq!(u, u2);
    }

    #[test]
    fn pack_unpack_roundtrip() {
        let cache = MessageSignatureCache;
        let msgs = LastSeenMessages::new(vec![sig(&[1]), sig(&[2, 3])]);
        let packed = msgs.pack(&cache);
        let unpacked = packed.unpack(&cache).unwrap();
        assert_eq!(unpacked, msgs);
    }

    #[test]
    fn packed_write_read_roundtrip() {
        let cache = MessageSignatureCache;
        let msgs = LastSeenMessages::new(vec![sig(&[9, 8, 7])]);
        let packed = msgs.pack(&cache);
        let mut buf = FriendlyByteBuf::default();
        packed.write(&mut buf);
        let mut r = FriendlyByteBuf::new(buf.into_inner());
        let packed2 = Packed::read(&mut r);
        assert_eq!(packed2, packed);
    }
}
