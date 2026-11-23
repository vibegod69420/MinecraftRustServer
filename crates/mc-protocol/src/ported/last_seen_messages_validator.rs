use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSignature(pub Vec<u8>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastSeenTrackedEntry {
    signature: MessageSignature,
    pending: bool,
}

impl LastSeenTrackedEntry {
    pub fn new(signature: MessageSignature, pending: bool) -> Self {
        Self { signature, pending }
    }

    pub fn acknowledge(&self) -> Self {
        Self {
            signature: self.signature.clone(),
            pending: false,
        }
    }

    pub fn pending(&self) -> bool {
        self.pending
    }

    pub fn signature(&self) -> &MessageSignature {
        &self.signature
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    pub fn new() -> Self {
        Self { words: Vec::new() }
    }

    pub fn from_bools(bits: &[bool]) -> Self {
        let mut bs = Self::new();
        let mut i: i32 = 0;
        for &b in bits {
            if b {
                bs.set(i);
            }
            i = i.wrapping_add(1);
        }
        bs
    }

    pub fn set(&mut self, index: i32) {
        if index < 0 {
            return;
        }
        let idx = index as usize;
        let word_idx = idx / 64;
        let bit = idx % 64;
        if word_idx >= self.words.len() {
            self.words.resize(word_idx.wrapping_add(1), 0u64);
        }
        self.words[word_idx] |= 1u64 << bit;
    }

    pub fn cardinality(&self) -> i32 {
        let mut count: i32 = 0;
        for &w in &self.words {
            count = count.wrapping_add(w.count_ones() as i32);
        }
        count
    }

    pub fn length(&self) -> i32 {
        let mut i = self.words.len();
        while i > 0 {
            i = i.wrapping_sub(1);
            let w = self.words[i];
            if w != 0 {
                let highest_bit = 63i32.wrapping_sub(w.leading_zeros() as i32);
                return (i as i32)
                    .wrapping_mul(64)
                    .wrapping_add(highest_bit)
                    .wrapping_add(1);
            }
        }
        0
    }

    pub fn get(&self, index: i32) -> bool {
        if index < 0 {
            return false;
        }
        let idx = index as usize;
        let word_idx = idx / 64;
        let bit = idx % 64;
        if word_idx >= self.words.len() {
            return false;
        }
        (self.words[word_idx] & (1u64 << bit)) != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastSeenMessages {
    pub entries: Vec<MessageSignature>,
}

impl LastSeenMessages {
    pub fn new(entries: Vec<MessageSignature>) -> Self {
        Self { entries }
    }

    /// Deterministic checksum for self-contained validation.
    pub fn calculate_checksum(&self) -> u8 {
        let mut h: i32 = 1i32;
        for sig in &self.entries {
            for &b in &sig.0 {
                h = 31i32.wrapping_mul(h).wrapping_add(i32::from(b as i8));
            }
        }
        h as u8
    }
}

#[derive(Clone, Debug)]
pub struct LastSeenMessagesUpdate {
    offset: i32,
    acknowledged: BitSet,
    checksum: u8,
}

impl LastSeenMessagesUpdate {
    pub fn new(offset: i32, acknowledged: BitSet, checksum: u8) -> Self {
        Self {
            offset,
            acknowledged,
            checksum,
        }
    }

    pub fn offset(&self) -> i32 {
        self.offset
    }

    pub fn acknowledged(&self) -> &BitSet {
        &self.acknowledged
    }

    pub fn verify_checksum(&self, messages: &LastSeenMessages) -> bool {
        self.checksum == messages.calculate_checksum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationException {
    pub message: String,
}

impl ValidationException {
    // JAVA: net.minecraft.network.chat.LastSeenMessagesValidator$ValidationException#<init>
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for ValidationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ValidationException {}

pub struct LastSeenMessagesValidator {
    last_seen_count: i32,
    tracked_messages: Vec<Option<LastSeenTrackedEntry>>,
    last_pending_message: Option<MessageSignature>,
}

impl LastSeenMessagesValidator {
    // JAVA: net.minecraft.network.chat.LastSeenMessagesValidator#<init>
    pub fn new(last_seen_count: i32) -> Self {
        let mut tracked_messages: Vec<Option<LastSeenTrackedEntry>> = Vec::new();
        let mut i: i32 = 0;
        while i < last_seen_count {
            tracked_messages.push(None);
            i = i.wrapping_add(1);
        }
        Self {
            last_seen_count,
            tracked_messages,
            last_pending_message: None,
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesValidator#addPending
    pub fn add_pending(&mut self, signature: MessageSignature) {
        let is_same = match &self.last_pending_message {
            Some(last) => *last == signature,
            None => false,
        };
        if !is_same {
            self.tracked_messages
                .push(Some(LastSeenTrackedEntry::new(signature.clone(), true)));
            self.last_pending_message = Some(signature);
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesValidator#trackedMessagesCount
    pub fn tracked_messages_count(&self) -> i32 {
        self.tracked_messages.len() as i32
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesValidator#applyOffset
    pub fn apply_offset(&mut self, offset: i32) -> Result<(), ValidationException> {
        let size = self.tracked_messages.len() as i32;
        let max_offset = size.wrapping_sub(self.last_seen_count);
        if offset >= 0 && offset <= max_offset {
            let from = 0usize;
            let to = offset as usize;
            self.tracked_messages.drain(from..to);
            Ok(())
        } else {
            Err(ValidationException::new(format!(
                "Advanced last seen window by {} messages, but expected at most {}",
                offset, max_offset
            )))
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesValidator#applyUpdate
    pub fn apply_update(
        &mut self,
        update: &LastSeenMessagesUpdate,
    ) -> Result<LastSeenMessages, ValidationException> {
        self.apply_offset(update.offset())?;

        let acknowledged = update.acknowledged();
        let mut signatures: Vec<MessageSignature> =
            Vec::with_capacity(acknowledged.cardinality().max(0) as usize);

        if acknowledged.length() > self.last_seen_count {
            return Err(ValidationException::new(format!(
                "Last seen update contained {} messages, but maximum window size is {}",
                acknowledged.length(),
                self.last_seen_count
            )));
        }

        let mut i: i32 = 0;
        while i < self.last_seen_count {
            let idx = i as usize;
            let is_acknowledged = acknowledged.get(i);
            let entry = self.tracked_messages.get(idx).cloned().unwrap_or(None);
            if is_acknowledged {
                match entry {
                    None => {
                        return Err(ValidationException::new(format!(
                            "Last seen update acknowledged unknown or previously ignored message at index {}",
                            i
                        )));
                    }
                    Some(ref tracked) => {
                        self.tracked_messages[idx] = Some(tracked.acknowledge());
                        signatures.push(tracked.signature().clone());
                    }
                }
            } else {
                if let Some(ref tracked) = entry {
                    if !tracked.pending() {
                        return Err(ValidationException::new(format!(
                            "Last seen update ignored previously acknowledged message at index {} and signature {:?}",
                            i,
                            tracked.signature()
                        )));
                    }
                }
                self.tracked_messages[idx] = None;
            }
            i = i.wrapping_add(1);
        }

        let messages = LastSeenMessages::new(signatures);
        if !update.verify_checksum(&messages) {
            return Err(ValidationException::new(
                "Checksum mismatch on last seen update: the client and server must have desynced"
                    .to_string(),
            ));
        }
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(bytes: &[u8]) -> MessageSignature {
        MessageSignature(bytes.to_vec())
    }

    #[test]
    fn basic_add_acknowledge_and_offset() {
        let mut v = LastSeenMessagesValidator::new(5);
        let s1 = sig(&[1, 2, 3]);
        let s2 = sig(&[4, 5, 6]);
        v.add_pending(s1.clone());
        v.add_pending(s2.clone());
        assert_eq!(v.tracked_messages_count(), 7);

        let mut ack = BitSet::new();
        ack.set(3);
        ack.set(4);
        let expected = LastSeenMessages::new(vec![s1.clone(), s2.clone()]);
        let update = LastSeenMessagesUpdate::new(2, ack, expected.calculate_checksum());
        let result = v.apply_update(&update).unwrap();
        assert_eq!(result.entries, vec![s1, s2]);
    }

    #[test]
    fn offset_validation_negative_and_overflow_edge() {
        let mut v = LastSeenMessagesValidator::new(2);
        let ack = BitSet::new();
        let empty = LastSeenMessages::new(vec![]);
        let checksum = empty.calculate_checksum();

        let err_neg = v
            .apply_update(&LastSeenMessagesUpdate::new(-1, ack.clone(), checksum))
            .unwrap_err();
        assert!(err_neg
            .message
            .contains("Advanced last seen window by -1"));

        let mut v2 = LastSeenMessagesValidator::new(2);
        let err_large = v2
            .apply_update(&LastSeenMessagesUpdate::new(1, ack, checksum))
            .unwrap_err();
        assert!(err_large.message.contains("expected at most 0"));

        // Negative lastSeenCount exercises wrapping subtraction in applyOffset:
        // max_offset = 0i32.wrapping_sub(-1) == 1
        let mut v3 = LastSeenMessagesValidator::new(-1);
        let err_window = v3
            .apply_update(&LastSeenMessagesUpdate::new(
                0,
                BitSet::new(),
                empty.calculate_checksum(),
            ))
            .unwrap_err();
        assert!(err_window.message.contains("maximum window size is -1"));
    }

    #[test]
    fn acknowledge_unknown_and_ignore_acknowledged_throw() {
        let mut v = LastSeenMessagesValidator::new(3);
        let mut ack = BitSet::new();
        ack.set(1);
        let msgs = LastSeenMessages::new(vec![sig(&[1])]);
        let err = v
            .apply_update(&LastSeenMessagesUpdate::new(
                0,
                ack,
                msgs.calculate_checksum(),
            ))
            .unwrap_err();
        assert!(err
            .message
            .contains("acknowledged unknown or previously ignored message at index 1"));

        let mut v2 = LastSeenMessagesValidator::new(3);
        let s = sig(&[42]);
        v2.add_pending(s.clone());
        let mut ack1 = BitSet::new();
        ack1.set(2);
        let msgs1 = LastSeenMessages::new(vec![s.clone()]);
        v2.apply_update(&LastSeenMessagesUpdate::new(1, ack1, msgs1.calculate_checksum()))
            .unwrap()
            ;

        let msgs2 = LastSeenMessages::new(vec![]);
        let err2 = v2
            .apply_update(&LastSeenMessagesUpdate::new(
                0,
                BitSet::new(),
                msgs2.calculate_checksum(),
            ))
            .unwrap_err();
        assert!(err2
            .message
            .contains("ignored previously acknowledged message at index 2"));
    }

    #[test]
    fn checksum_mismatch_and_ack_length_throw() {
        let mut v = LastSeenMessagesValidator::new(2);
        let ack = BitSet::new();
        let correct = LastSeenMessages::new(vec![]).calculate_checksum();
        let wrong = correct.wrapping_add(1);
        let err = v
            .apply_update(&LastSeenMessagesUpdate::new(0, ack, wrong))
            .unwrap_err();
        assert!(err.message.contains("Checksum mismatch"));

        let mut v2 = LastSeenMessagesValidator::new(2);
        let mut ack_long = BitSet::new();
        ack_long.set(2); // length 3 > 2
        let err2 = v2
            .apply_update(&LastSeenMessagesUpdate::new(
                0,
                ack_long,
                LastSeenMessages::new(vec![]).calculate_checksum(),
            ))
            .unwrap_err();
        assert!(err2.message.contains("maximum window size is 2"));
    }

    #[test]
    fn float_bits_in_signatures_and_duplicate_pending() {
        let mut v = LastSeenMessagesValidator::new(4);
        let f1_bits = f32::NEG_INFINITY.to_bits();
        let f2_bits = f32::NAN.to_bits();
        let s1 = MessageSignature(f1_bits.to_le_bytes().to_vec());
        let s2 = MessageSignature(f2_bits.to_le_bytes().to_vec());

        v.add_pending(s1.clone());
        v.add_pending(s1.clone()); // duplicate should be ignored
        v.add_pending(s2.clone());
        assert_eq!(v.tracked_messages_count(), 6);

        let mut ack = BitSet::new();
        ack.set(2);
        ack.set(3);
        let expected = LastSeenMessages::new(vec![s1.clone(), s2.clone()]);
        let update = LastSeenMessagesUpdate::new(2, ack, expected.calculate_checksum());
        let result = v.apply_update(&update).unwrap();

        assert_eq!(result.entries.len(), 2);
        assert_eq!(
            u32::from_le_bytes(result.entries[0].0[..4].try_into().unwrap()),
            f1_bits
        );
        assert_eq!(
            u32::from_le_bytes(result.entries[1].0[..4].try_into().unwrap()),
            f2_bits
        );
    }
}
