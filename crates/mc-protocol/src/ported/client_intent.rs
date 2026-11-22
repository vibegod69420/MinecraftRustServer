//! Port of `net.minecraft.network.protocol.handshake.ClientIntent`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientIntent {
    Status,
    Login,
    Transfer,
}

impl ClientIntent {
    const STATUS_ID: i32 = 1;
    const LOGIN_ID: i32 = 2;
    const TRANSFER_ID: i32 = 3;

    // JAVA: net.minecraft.network.protocol.handshake.ClientIntent#byId
    pub fn by_id(id: i32) -> Self {
        match id {
            Self::STATUS_ID => Self::Status,
            Self::LOGIN_ID => Self::Login,
            Self::TRANSFER_ID => Self::Transfer,
            _ => panic!("Unknown connection intent: {}", id),
        }
    }

    // JAVA: net.minecraft.network.protocol.handshake.ClientIntent#id
    pub fn id(self) -> i32 {
        match self {
            Self::Status => Self::STATUS_ID,
            Self::Login => Self::LOGIN_ID,
            Self::Transfer => Self::TRANSFER_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_values_match_java() {
        assert_eq!(ClientIntent::Status.id(), 1);
        assert_eq!(ClientIntent::Login.id(), 2);
        assert_eq!(ClientIntent::Transfer.id(), 3);
    }

    #[test]
    fn by_id_roundtrip() {
        for intent in [ClientIntent::Status, ClientIntent::Login, ClientIntent::Transfer] {
            assert_eq!(ClientIntent::by_id(intent.id()), intent);
        }
    }

    #[test]
    #[should_panic(expected = "Unknown connection intent: 0")]
    fn by_id_zero_panics() {
        let _ = ClientIntent::by_id(0);
    }

    #[test]
    #[should_panic(expected = "Unknown connection intent: -1")]
    fn by_id_negative_panics() {
        let _ = ClientIntent::by_id(-1);
    }

    #[test]
    #[should_panic(expected = "Unknown connection intent: 4")]
    fn by_id_out_of_range_panics() {
        let _ = ClientIntent::by_id(4);
    }

    #[test]
    #[should_panic(expected = "Unknown connection intent: 2147483647")]
    fn by_id_i32_max_panics() {
        let _ = ClientIntent::by_id(i32::MAX);
    }
}
