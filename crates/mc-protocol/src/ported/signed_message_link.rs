use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

pub const NIL_UUID: Uuid = Uuid {
    most_significant_bits: 0,
    least_significant_bits: 0,
};

pub fn uuid_to_byte_array(uuid: Uuid) -> [u8; 16] {
    let mut out = [0u8; 16];
    out[0..8].copy_from_slice(&uuid.most_significant_bits.to_be_bytes());
    out[8..16].copy_from_slice(&uuid.least_significant_bits.to_be_bytes());
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureException;

impl fmt::Display for SignatureException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignatureException")
    }
}

impl std::error::Error for SignatureException {}

pub trait SignatureUpdaterOutput {
    fn update(&mut self, data: &[u8]) -> Result<(), SignatureException>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignedMessageLink {
    pub index: i32,
    pub sender: Uuid,
    pub session_id: Uuid,
}

impl SignedMessageLink {
    // JAVA: net.minecraft.network.chat.SignedMessageLink#unsigned
    pub fn unsigned(sender: Uuid) -> Self {
        Self::root(sender, NIL_UUID)
    }

    // JAVA: net.minecraft.network.chat.SignedMessageLink#root
    pub fn root(sender: Uuid, session_id: Uuid) -> Self {
        Self {
            index: 0,
            sender,
            session_id,
        }
    }

    // JAVA: net.minecraft.network.chat.SignedMessageLink#updateSignature
    pub fn update_signature<O: SignatureUpdaterOutput>(
        &self,
        output: &mut O,
    ) -> Result<(), SignatureException> {
        output.update(&uuid_to_byte_array(self.sender))?;
        output.update(&uuid_to_byte_array(self.session_id))?;
        output.update(&self.index.to_be_bytes())?;
        Ok(())
    }

    // JAVA: net.minecraft.network.chat.SignedMessageLink#isDescendantOf
    pub fn is_descendant_of(&self, other: &SignedMessageLink) -> bool {
        self.index > other.index
            && self.sender == other.sender
            && self.session_id == other.session_id
    }

    // JAVA: net.minecraft.network.chat.SignedMessageLink#advance
    pub fn advance(&self) -> Option<SignedMessageLink> {
        if self.index == i32::MAX {
            None
        } else {
            Some(SignedMessageLink {
                index: self.index.wrapping_add(1),
                sender: self.sender,
                session_id: self.session_id,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CollectOutput(Vec<u8>);
    impl SignatureUpdaterOutput for CollectOutput {
        fn update(&mut self, data: &[u8]) -> Result<(), SignatureException> {
            self.0.extend_from_slice(data);
            Ok(())
        }
    }

    fn uuid(msb: i64, lsb: i64) -> Uuid {
        Uuid {
            most_significant_bits: msb,
            least_significant_bits: lsb,
        }
    }

    #[test]
    fn test_unsigned_and_root() {
        let sender = uuid(1, 2);
        let link = SignedMessageLink::unsigned(sender);
        assert_eq!(link.index, 0);
        assert_eq!(link.sender, sender);
        assert_eq!(link.session_id, NIL_UUID);

        let session = uuid(3, 4);
        let root = SignedMessageLink::root(sender, session);
        assert_eq!(root.index, 0);
        assert_eq!(root.sender, sender);
        assert_eq!(root.session_id, session);
    }

    #[test]
    fn test_update_signature_byte_order() {
        let link = SignedMessageLink {
            index: 0x0102_0304,
            sender: uuid(0x1122_3344_5566_7788_u64 as i64, 0x99AA_BBCC_DDEE_FF00_u64 as i64),
            session_id: uuid(0x0102_0304_0506_0708, 0x090A_0B0C_0D0E_0F10),
        };
        let mut out = CollectOutput(Vec::new());
        link.update_signature(&mut out).unwrap();
        assert_eq!(out.0.len(), 36);
        assert_eq!(
            &out.0[0..8],
            &0x1122_3344_5566_7788_u64.to_be_bytes()
        );
        assert_eq!(
            &out.0[8..16],
            &0x99AA_BBCC_DDEE_FF00_u64.to_be_bytes()
        );
        assert_eq!(&out.0[16..24], &0x0102_0304_0506_0708_i64.to_be_bytes());
        assert_eq!(&out.0[24..32], &0x090A_0B0C_0D0E_0F10_i64.to_be_bytes());
        assert_eq!(&out.0[32..36], &0x0102_0304_i32.to_be_bytes());
    }

    #[test]
    fn test_is_descendant_of() {
        let sender = uuid(1, 2);
        let session = uuid(3, 4);
        let a = SignedMessageLink::root(sender, session);
        let b = SignedMessageLink {
            index: 1,
            sender,
            session_id: session,
        };
        let c = SignedMessageLink {
            index: 1,
            sender: uuid(9, 9),
            session_id: session,
        };

        assert!(b.is_descendant_of(&a));
        assert!(!a.is_descendant_of(&b));
        assert!(!a.is_descendant_of(&a));
        assert!(!c.is_descendant_of(&a));
    }

    #[test]
    fn test_advance_normal_and_max() {
        let link = SignedMessageLink::root(uuid(1, 0), uuid(2, 0));
        let next = link.advance().unwrap();
        assert_eq!(next.index, 1);
        assert_eq!(next.sender, link.sender);
        assert_eq!(next.session_id, link.session_id);

        let max_link = SignedMessageLink {
            index: i32::MAX,
            sender: uuid(-1, -2),
            session_id: uuid(i64::MIN, i64::MAX),
        };
        assert!(max_link.advance().is_none());
    }

    #[test]
    fn test_negative_index_descendant_and_advance() {
        let sender = uuid(5, 6);
        let session = uuid(7, 8);
        let neg = SignedMessageLink {
            index: -10,
            sender,
            session_id: session,
        };
        let neg2 = SignedMessageLink {
            index: -9,
            sender,
            session_id: session,
        };
        assert!(neg2.is_descendant_of(&neg));
        assert_eq!(neg.advance().unwrap().index, -9);
    }
}
