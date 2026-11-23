use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedMessageLink {
    index: i32,
    session_id: i64,
}

impl SignedMessageLink {
    pub fn new(index: i32, session_id: i64) -> Self {
        Self { index, session_id }
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn session_id(&self) -> i64 {
        self.session_id
    }

    pub fn is_descendant_of(&self, other: &SignedMessageLink) -> bool {
        self.session_id == other.session_id && self.index > other.index
    }
}

pub trait SignatureValidator {
    fn validate_player_message(&self, message: &PlayerChatMessage) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerChatMessage {
    sender_name: String,
    link: SignedMessageLink,
    signed: bool,
}

impl PlayerChatMessage {
    pub fn new(sender_name: impl Into<String>, link: SignedMessageLink, signed: bool) -> Self {
        Self {
            sender_name: sender_name.into(),
            link,
            signed,
        }
    }

    pub fn sender(&self) -> &str {
        &self.sender_name
    }

    pub fn link(&self) -> &SignedMessageLink {
        &self.link
    }

    pub fn has_signature(&self) -> bool {
        self.signed
    }

    // JAVA: net.minecraft.network.chat.PlayerChatMessage#removeSignature
    pub fn remove_signature(mut self) -> Self {
        self.signed = false;
        self
    }

    // JAVA: net.minecraft.network.chat.PlayerChatMessage#verify
    pub fn verify(&self, validator: &dyn SignatureValidator) -> bool {
        self.signed && validator.validate_player_message(self)
    }

    // JAVA: net.minecraft.network.chat.PlayerChatMessage#describeSigned
    pub fn describe_signed(message: &PlayerChatMessage) -> String {
        format!(
            "PlayerChatMessage{{sender={}, index={}, session={}, signed={}}}",
            message.sender_name,
            message.link.index(),
            message.link.session_id(),
            message.signed
        )
    }
}

impl fmt::Display for PlayerChatMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Self::describe_signed(self))
    }
}

// JAVA: net.minecraft.network.chat.SignedMessageValidator
pub trait SignedMessageValidator {
    // JAVA: net.minecraft.network.chat.SignedMessageValidator#updateAndValidate
    fn update_and_validate(&mut self, message: PlayerChatMessage) -> Option<PlayerChatMessage>;
}

// JAVA: net.minecraft.network.chat.SignedMessageValidator#ACCEPT_UNSIGNED
pub struct AcceptUnsigned;

impl SignedMessageValidator for AcceptUnsigned {
    // JAVA: net.minecraft.network.chat.SignedMessageValidator#updateAndValidate
    fn update_and_validate(&mut self, message: PlayerChatMessage) -> Option<PlayerChatMessage> {
        Some(message.remove_signature())
    }
}

// JAVA: net.minecraft.network.chat.SignedMessageValidator#REJECT_ALL
pub struct RejectAll;

impl SignedMessageValidator for RejectAll {
    // JAVA: net.minecraft.network.chat.SignedMessageValidator#updateAndValidate
    fn update_and_validate(&mut self, message: PlayerChatMessage) -> Option<PlayerChatMessage> {
        eprintln!(
            "Received chat message from {}, but they have no chat session initialized and secure chat is enforced",
            message.sender()
        );
        None
    }
}

// JAVA: net.minecraft.network.chat.SignedMessageValidator$KeyBased
pub struct KeyBased {
    validator: Box<dyn SignatureValidator>,
    expired: Box<dyn Fn() -> bool>,
    last_message: Option<PlayerChatMessage>,
    is_chain_valid: bool,
}

impl KeyBased {
    // JAVA: net.minecraft.network.chat.SignedMessageValidator$KeyBased#<init>
    pub fn new(validator: Box<dyn SignatureValidator>, expired: Box<dyn Fn() -> bool>) -> Self {
        Self {
            validator,
            expired,
            last_message: None,
            is_chain_valid: true,
        }
    }

    // JAVA: net.minecraft.network.chat.SignedMessageValidator$KeyBased#validateChain
    fn validate_chain(&self, message: &PlayerChatMessage) -> bool {
        if Some(message) == self.last_message.as_ref() {
            return true;
        }
        if let Some(last) = &self.last_message {
            if !message.link().is_descendant_of(last.link()) {
                eprintln!(
                    "Received out-of-order chat message from {}: expected index > {} for session {}, but was {} for session {}",
                    message.sender(),
                    last.link().index(),
                    last.link().session_id(),
                    message.link().index(),
                    message.link().session_id()
                );
                return false;
            }
        }
        true
    }

    // JAVA: net.minecraft.network.chat.SignedMessageValidator$KeyBased#validate
    fn validate(&self, message: &PlayerChatMessage) -> bool {
        if (self.expired)() {
            eprintln!(
                "Received message with expired profile public key from {} with session {}",
                message.sender(),
                message.link().session_id()
            );
            return false;
        }
        if !message.verify(self.validator.as_ref()) {
            eprintln!(
                "Received message with invalid signature (is the session wrong, or signature cache out of sync?): {}",
                PlayerChatMessage::describe_signed(message)
            );
            return false;
        }
        self.validate_chain(message)
    }
}

impl SignedMessageValidator for KeyBased {
    // JAVA: net.minecraft.network.chat.SignedMessageValidator$KeyBased#updateAndValidate
    fn update_and_validate(&mut self, message: PlayerChatMessage) -> Option<PlayerChatMessage> {
        self.is_chain_valid = self.is_chain_valid && self.validate(&message);
        if !self.is_chain_valid {
            return None;
        }
        self.last_message = Some(message.clone());
        Some(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysValid;
    impl SignatureValidator for AlwaysValid {
        fn validate_player_message(&self, _message: &PlayerChatMessage) -> bool {
            true
        }
    }

    struct AlwaysInvalid;
    impl SignatureValidator for AlwaysInvalid {
        fn validate_player_message(&self, _message: &PlayerChatMessage) -> bool {
            false
        }
    }

    fn msg(sender: &str, index: i32, session: i64, signed: bool) -> PlayerChatMessage {
        PlayerChatMessage::new(sender, SignedMessageLink::new(index, session), signed)
    }

    #[test]
    fn accept_unsigned_removes_signature() {
        let mut v = AcceptUnsigned;
        let m = msg("alice", 0, 1, true);
        let out = v.update_and_validate(m).unwrap();
        assert!(!out.has_signature());
        assert_eq!(out.sender(), "alice");
    }

    #[test]
    fn reject_all_returns_none() {
        let mut v = RejectAll;
        let m = msg("alice", 0, 1, true);
        assert!(v.update_and_validate(m).is_none());
    }

    #[test]
    fn key_based_accepts_valid_chain() {
        let mut v = KeyBased::new(Box::new(AlwaysValid), Box::new(|| false));
        let m1 = msg("alice", 0, 1, true);
        let m2 = msg("alice", 1, 1, true);
        assert!(v.update_and_validate(m1).is_some());
        let out2 = v.update_and_validate(m2).unwrap();
        assert_eq!(out2.link().index(), 1);
    }

    #[test]
    fn key_based_rejects_out_of_order_and_stays_invalid() {
        let mut v = KeyBased::new(Box::new(AlwaysValid), Box::new(|| false));
        let m1 = msg("alice", 5, 1, true);
        let m2 = msg("alice", 3, 1, true);
        assert!(v.update_and_validate(m1).is_some());
        assert!(v.update_and_validate(m2).is_none());
        let m3 = msg("alice", 6, 1, true);
        assert!(v.update_and_validate(m3).is_none());
    }

    #[test]
    fn key_based_rejects_expired_or_invalid_signature() {
        let mut expired = KeyBased::new(Box::new(AlwaysValid), Box::new(|| true));
        assert!(expired.update_and_validate(msg("a", 0, 1, true)).is_none());

        let mut invalid = KeyBased::new(Box::new(AlwaysInvalid), Box::new(|| false));
        assert!(invalid.update_and_validate(msg("a", 0, 1, true)).is_none());
    }

    #[test]
    fn key_based_accepts_same_message_again() {
        let mut v = KeyBased::new(Box::new(AlwaysValid), Box::new(|| false));
        let m = msg("alice", 0, 1, true);
        assert!(v.update_and_validate(m.clone()).is_some());
        assert!(v.update_and_validate(m).is_some());
    }
}
