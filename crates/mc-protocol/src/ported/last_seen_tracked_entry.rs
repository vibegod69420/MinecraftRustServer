/// Minimal stub for `net.minecraft.network.chat.MessageSignature`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSignature {
    pub bytes: Vec<u8>,
}

/// Port of `net.minecraft.network.chat.LastSeenTrackedEntry`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastSeenTrackedEntry {
    pub signature: MessageSignature,
    pub pending: bool,
}

impl LastSeenTrackedEntry {
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

#[cfg(test)]
mod tests {
    use super::{LastSeenTrackedEntry, MessageSignature};

    fn sig(bytes: &[u8]) -> MessageSignature {
        MessageSignature {
            bytes: bytes.to_vec(),
        }
    }

    #[test]
    fn acknowledge_pending_becomes_not_pending() {
        let entry = LastSeenTrackedEntry {
            signature: sig(&[1, 2, 3]),
            pending: true,
        };
        let acked = entry.acknowledge();
        assert!(!acked.pending);
        assert_eq!(acked.signature, sig(&[1, 2, 3]));
    }

    #[test]
    fn acknowledge_not_pending_returns_equal_entry() {
        let entry = LastSeenTrackedEntry {
            signature: sig(&[9, 8, 7]),
            pending: false,
        };
        let acked = entry.acknowledge();
        assert_eq!(acked, entry);
        assert!(!acked.pending);
    }

    #[test]
    fn acknowledge_preserves_signature_bytes() {
        let entry = LastSeenTrackedEntry {
            signature: sig(&[0, 255, 128]),
            pending: true,
        };
        let acked = entry.acknowledge();
        assert_eq!(acked.signature.bytes, vec![0, 255, 128]);
    }

    #[test]
    fn acknowledge_does_not_mutate_original() {
        let entry = LastSeenTrackedEntry {
            signature: sig(&[42]),
            pending: true,
        };
        let _ = entry.acknowledge();
        assert!(entry.pending);
    }
}
