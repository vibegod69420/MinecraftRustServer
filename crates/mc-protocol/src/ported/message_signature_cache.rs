use std::collections::{HashSet, VecDeque};

pub const NOT_FOUND: i32 = -1;
const DEFAULT_CAPACITY: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageSignature(pub Vec<u8>);

#[derive(Debug, Clone, Default)]
pub struct LastSeenMessages {
    pub entries: Vec<MessageSignature>,
}

impl LastSeenMessages {
    // JAVA: net.minecraft.network.chat.LastSeenMessages#entries
    pub fn entries(&self) -> &[MessageSignature] {
        &self.entries
    }
}

#[derive(Debug, Clone, Default)]
pub struct SignedMessageBody {
    pub last_seen: LastSeenMessages,
}

impl SignedMessageBody {
    // JAVA: net.minecraft.network.chat.SignedMessageBody#lastSeen
    pub fn last_seen(&self) -> &LastSeenMessages {
        &self.last_seen
    }
}

pub struct MessageSignatureCache {
    entries: Vec<Option<MessageSignature>>,
}

impl MessageSignatureCache {
    // JAVA: net.minecraft.network.chat.MessageSignatureCache#<init>
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: vec![None; capacity],
        }
    }

    // JAVA: net.minecraft.network.chat.MessageSignatureCache#createDefault
    pub fn create_default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }

    // JAVA: net.minecraft.network.chat.MessageSignatureCache#pack
    pub fn pack(&self, signature: &MessageSignature) -> i32 {
        for i in 0..self.entries.len() {
            if let Some(entry) = &self.entries[i] {
                if entry == signature {
                    return i as i32;
                }
            }
        }
        NOT_FOUND
    }

    // JAVA: net.minecraft.network.chat.MessageSignatureCache#unpack
    pub fn unpack(&self, index: i32) -> Option<&MessageSignature> {
        self.entries[index as usize].as_ref()
    }

    // JAVA: net.minecraft.network.chat.MessageSignatureCache#push
    pub fn push(&mut self, body: &SignedMessageBody, signature: Option<&MessageSignature>) {
        let last_seen = body.last_seen().entries();
        let mut deque: VecDeque<MessageSignature> = VecDeque::with_capacity(last_seen.len() + 1);
        for sig in last_seen {
            deque.push_back(sig.clone());
        }
        if let Some(sig) = signature {
            deque.push_back(sig.clone());
        }
        self.push_deque(deque);
    }

    // JAVA: net.minecraft.network.chat.MessageSignatureCache#push
    pub fn push_list(&mut self, signatures: Vec<MessageSignature>) {
        self.push_deque(VecDeque::from(signatures));
    }

    // JAVA: net.minecraft.network.chat.MessageSignatureCache#push
    fn push_deque(&mut self, mut deque: VecDeque<MessageSignature>) {
        let set: HashSet<MessageSignature> = deque.iter().cloned().collect();

        let mut i: usize = 0;
        while !deque.is_empty() && i < self.entries.len() {
            let old = self.entries[i].take();
            self.entries[i] = deque.pop_back();
            if let Some(old_sig) = old {
                if !set.contains(&old_sig) {
                    deque.push_front(old_sig);
                }
            }
            i = i.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(id: u8) -> MessageSignature {
        MessageSignature(vec![id])
    }

    #[test]
    fn test_create_default_and_not_found() {
        let cache = MessageSignatureCache::create_default();
        assert_eq!(cache.entries.len(), 128);
        assert_eq!(cache.pack(&sig(1)), NOT_FOUND);
    }

    #[test]
    fn test_pack_and_unpack() {
        let mut cache = MessageSignatureCache::new(4);
        cache.push_list(vec![sig(1), sig(2)]);
        // push_list pops from back into slots: slot 0 gets sig(2), slot 1 gets sig(1)
        assert_eq!(cache.pack(&sig(2)), 0);
        assert_eq!(cache.pack(&sig(1)), 1);
        assert_eq!(cache.unpack(0), Some(&sig(2)));
        assert_eq!(cache.unpack(1), Some(&sig(1)));
        assert_eq!(cache.unpack(2), None);
        assert_eq!(cache.pack(&sig(9)), NOT_FOUND);
    }

    #[test]
    fn test_push_body_includes_signature() {
        let body = SignedMessageBody {
            last_seen: LastSeenMessages {
                entries: vec![sig(10), sig(11)],
            },
        };
        let mut cache = MessageSignatureCache::new(4);
        cache.push(&body, Some(&sig(12)));
        // deque becomes [10,11,12], set has all three
        // i=0: slot0=None, slot0=pop_back=12
        // i=1: slot1=None, slot1=pop_back=11
        // i=2: slot2=None, slot2=pop_back=10
        assert_eq!(cache.unpack(0), Some(&sig(12)));
        assert_eq!(cache.unpack(1), Some(&sig(11)));
        assert_eq!(cache.unpack(2), Some(&sig(10)));
    }

    #[test]
    fn test_push_preserves_displaced_not_in_input() {
        let mut cache = MessageSignatureCache::new(2);
        cache.push_list(vec![sig(1), sig(2)]);
        // slot0=sig(2), slot1=sig(1)
        cache.push_list(vec![sig(3)]);
        // deque=[3], set={3}
        // i=0: old=Some(2), slot0=3, 2 not in {3} => push_front(2), deque=[2]
        // i=1: old=Some(1), slot1=2, 1 not in {3} => push_front(1), deque=[1]
        // loop ends (deque not empty but i == len)
        assert_eq!(cache.unpack(0), Some(&sig(3)));
        assert_eq!(cache.unpack(1), Some(&sig(2)));
    }

    #[test]
    fn test_push_with_negative_unpack_panics_like_java() {
        let cache = MessageSignatureCache::new(2);
        let result = std::panic::catch_unwind(|| {
            let _ = cache.unpack(-1);
        });
        assert!(result.is_err());
    }
}
