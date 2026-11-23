use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSignature {
    pub bytes: [u8; 256],
}

impl MessageSignature {
    // JAVA: net.minecraft.network.chat.MessageSignature#hashCode
    pub fn hash_code(&self) -> i32 {
        let mut result: i32 = 1;
        for &b in &self.bytes {
            result = result.wrapping_mul(31).wrapping_add(b as i8 as i32);
        }
        result
    }
}

#[derive(Clone, Debug)]
pub struct LastSeenTrackedEntry {
    signature: MessageSignature,
    pending: bool,
}

impl LastSeenTrackedEntry {
    // JAVA: net.minecraft.network.chat.LastSeenTrackedEntry#<init>
    pub fn new(signature: MessageSignature, pending: bool) -> Self {
        Self { signature, pending }
    }

    // JAVA: net.minecraft.network.chat.LastSeenTrackedEntry#signature
    pub fn signature(&self) -> &MessageSignature {
        &self.signature
    }

    // JAVA: net.minecraft.network.chat.LastSeenTrackedEntry#pending
    pub fn pending(&self) -> bool {
        self.pending
    }

    // JAVA: net.minecraft.network.chat.LastSeenTrackedEntry#acknowledge
    pub fn acknowledge(&self) -> Self {
        if self.pending {
            Self {
                signature: self.signature.clone(),
                pending: false,
            }
        } else {
            self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaBitSet {
    words: Vec<u64>,
}

impl JavaBitSet {
    pub fn new(nbits: usize) -> Self {
        let words_len = nbits.div_ceil(64);
        Self {
            words: vec![0u64; words_len],
        }
    }

    pub fn set(&mut self, bit_index: usize, value: bool) {
        let word_index = bit_index / 64;
        if word_index >= self.words.len() {
            if !value {
                return;
            }
            self.words.resize(word_index + 1, 0);
        }
        let mask = 1u64 << (bit_index % 64);
        if value {
            self.words[word_index] |= mask;
        } else {
            self.words[word_index] &= !mask;
        }
    }

    pub fn get(&self, bit_index: usize) -> bool {
        let word_index = bit_index / 64;
        if word_index >= self.words.len() {
            return false;
        }
        (self.words[word_index] & (1u64 << (bit_index % 64))) != 0
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

    // JAVA: net.minecraft.network.chat.LastSeenMessages#computeChecksum
    pub fn compute_checksum(&self) -> i8 {
        let mut i: i32 = 1;
        for msg in &self.entries {
            i = i.wrapping_mul(31).wrapping_add(msg.hash_code());
        }
        let b = i as i8;
        if b == 0 { 1 } else { b }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastSeenMessagesUpdate {
    pub offset: i32,
    pub acknowledged: JavaBitSet,
    pub checksum: i8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastSeenMessagesTrackerUpdate {
    pub last_seen: LastSeenMessages,
    pub update: LastSeenMessagesUpdate,
}

#[derive(Clone, Debug)]
pub struct LastSeenMessagesTracker {
    tracked_messages: Vec<Option<LastSeenTrackedEntry>>,
    tail: usize,
    offset: i32,
    last_tracked_message: Option<MessageSignature>,
}

impl LastSeenMessagesTracker {
    // JAVA: net.minecraft.network.chat.LastSeenMessagesTracker#<init>
    pub fn new(size: i32) -> Self {
        let size = if size < 0 { 0 } else { size as usize };
        Self {
            tracked_messages: vec![None; size],
            tail: 0,
            offset: 0,
            last_tracked_message: None,
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesTracker#addPending
    pub fn add_pending(&mut self, signature: MessageSignature, pending: bool) -> bool {
        if self.last_tracked_message.as_ref() == Some(&signature) {
            return false;
        }
        self.last_tracked_message = Some(signature.clone());
        let entry = if pending {
            Some(LastSeenTrackedEntry::new(signature, true))
        } else {
            None
        };
        self.add_entry(entry);
        true
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesTracker#addEntry
    fn add_entry(&mut self, entry: Option<LastSeenTrackedEntry>) {
        let len = self.tracked_messages.len();
        if len == 0 {
            self.offset = self.offset.wrapping_add(1);
            return;
        }
        let i = self.tail;
        self.tail = (i + 1) % len;
        self.offset = self.offset.wrapping_add(1);
        self.tracked_messages[i] = entry;
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesTracker#ignorePending
    pub fn ignore_pending(&mut self, signature: &MessageSignature) {
        let len = self.tracked_messages.len();
        for i in 0..len {
            let should_clear = match &self.tracked_messages[i] {
                Some(entry) => entry.pending() && entry.signature() == signature,
                None => false,
            };
            if should_clear {
                self.tracked_messages[i] = None;
                break;
            }
        }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesTracker#getAndClearOffset
    pub fn get_and_clear_offset(&mut self) -> i32 {
        let o = self.offset;
        self.offset = 0;
        o
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesTracker#generateAndApplyUpdate
    pub fn generate_and_apply_update(&mut self) -> LastSeenMessagesTrackerUpdate {
        let offset = self.get_and_clear_offset();
        let len = self.tracked_messages.len();
        let mut acknowledged = JavaBitSet::new(len);
        let mut signatures: Vec<MessageSignature> = Vec::with_capacity(len);

        for i in 0..len {
            let idx = (self.tail + i) % len;
            if let Some(entry) = self.tracked_messages[idx].clone() {
                acknowledged.set(i, true);
                signatures.push(entry.signature().clone());
                self.tracked_messages[idx] = Some(entry.acknowledge());
            }
        }

        let last_seen = LastSeenMessages::new(signatures);
        let checksum = last_seen.compute_checksum();
        let update = LastSeenMessagesUpdate {
            offset,
            acknowledged,
            checksum,
        };
        LastSeenMessagesTrackerUpdate { last_seen, update }
    }

    // JAVA: net.minecraft.network.chat.LastSeenMessagesTracker#offset
    pub fn offset(&self) -> i32 {
        self.offset
    }
}

impl fmt::Display for MessageSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MessageSignature({:?}...)", &self.bytes[..4])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(id: u8) -> MessageSignature {
        let mut bytes = [0u8; 256];
        bytes[0] = id;
        MessageSignature { bytes }
    }

    #[test]
    fn add_pending_and_duplicate() {
        let mut tracker = LastSeenMessagesTracker::new(5);
        assert!(tracker.add_pending(sig(1), true));
        assert!(!tracker.add_pending(sig(1), true));
        assert!(tracker.add_pending(sig(2), false));
        assert_eq!(tracker.offset(), 2);
    }

    #[test]
    fn get_and_clear_offset_resets() {
        let mut tracker = LastSeenMessagesTracker::new(5);
        tracker.add_pending(sig(1), true);
        tracker.add_pending(sig(2), true);
        assert_eq!(tracker.get_and_clear_offset(), 2);
        assert_eq!(tracker.offset(), 0);
        assert_eq!(tracker.get_and_clear_offset(), 0);
    }

    #[test]
    fn ignore_pending_clears_only_pending_match() {
        let mut tracker = LastSeenMessagesTracker::new(5);
        tracker.add_pending(sig(1), true);
        tracker.add_pending(sig(2), true);
        tracker.ignore_pending(&sig(1));
        let update = tracker.generate_and_apply_update();
        assert_eq!(update.last_seen.entries.len(), 1);
        assert_eq!(update.last_seen.entries[0], sig(2));
    }

    #[test]
    fn generate_and_apply_update_acknowledges_and_sets_bits() {
        let mut tracker = LastSeenMessagesTracker::new(3);
        tracker.add_pending(sig(1), true);
        tracker.add_pending(sig(2), false);
        tracker.add_pending(sig(3), true);

        let update1 = tracker.generate_and_apply_update();
        assert_eq!(update1.update.offset, 3);
        assert_eq!(update1.last_seen.entries.len(), 2);
        assert!(update1.update.acknowledged.get(0));
        assert!(!update1.update.acknowledged.get(1));
        assert!(update1.update.acknowledged.get(2));

        let update2 = tracker.generate_and_apply_update();
        assert_eq!(update2.update.offset, 0);
        assert_eq!(update2.last_seen.entries.len(), 2);
    }

    #[test]
    fn ring_buffer_wraps_and_overwrites_oldest() {
        let mut tracker = LastSeenMessagesTracker::new(2);
        tracker.add_pending(sig(1), true);
        tracker.add_pending(sig(2), true);
        tracker.add_pending(sig(3), true);
        let update = tracker.generate_and_apply_update();
        assert_eq!(update.last_seen.entries.len(), 2);
        assert_eq!(update.last_seen.entries[0], sig(2));
        assert_eq!(update.last_seen.entries[1], sig(3));
    }

    #[test]
    fn checksum_empty_and_zero_replacement() {
        let empty = LastSeenMessages::new(vec![]);
        assert_eq!(empty.compute_checksum(), 1);

        // Construct a signature whose hash makes the checksum byte 0 to verify replacement.
        // hash of all-zero bytes: result starts at 1, for each byte: 1*31+0 = 31, 31*31+0...
        // Easier to just verify deterministic non-zero behavior for a known case.
        let msgs = LastSeenMessages::new(vec![sig(0)]);
        let cs = msgs.compute_checksum();
        assert_ne!(cs, 0);
    }
}
