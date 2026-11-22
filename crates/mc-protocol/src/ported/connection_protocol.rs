//! Port of `net.minecraft.network.ConnectionProtocol`.

/// Port of `net.minecraft.network.ConnectionProtocol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    Handshaking,
    Play,
    Status,
    Login,
    Configuration,
}

impl ConnectionProtocol {
    // JAVA: net.minecraft.network.ConnectionProtocol#id
    pub fn id(self) -> &'static str {
        match self {
            Self::Handshaking => "handshake",
            Self::Play => "play",
            Self::Status => "status",
            Self::Login => "login",
            Self::Configuration => "configuration",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_match_java() {
        assert_eq!(ConnectionProtocol::Handshaking.id(), "handshake");
        assert_eq!(ConnectionProtocol::Play.id(), "play");
        assert_eq!(ConnectionProtocol::Status.id(), "status");
        assert_eq!(ConnectionProtocol::Login.id(), "login");
        assert_eq!(ConnectionProtocol::Configuration.id(), "configuration");
    }

    #[test]
    fn all_variants_have_distinct_ids() {
        let ids = [
            ConnectionProtocol::Handshaking.id(),
            ConnectionProtocol::Play.id(),
            ConnectionProtocol::Status.id(),
            ConnectionProtocol::Login.id(),
            ConnectionProtocol::Configuration.id(),
        ];
        for (i, a) in ids.iter().enumerate() {
            for (j, b) in ids.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn enum_equality_and_copy() {
        let a = ConnectionProtocol::Play;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, ConnectionProtocol::Login);
    }

    #[test]
    fn id_is_stable_across_calls() {
        let proto = ConnectionProtocol::Configuration;
        assert_eq!(proto.id(), proto.id());
        assert_eq!(proto.id(), "configuration");
    }

    #[test]
    fn negative_and_overflow_like_integer_patterns_do_not_affect_enum() {
        // Edge-case coverage: enum behavior is independent of integer arithmetic pitfalls.
        let idx: i32 = -1_i32;
        let wrapped = idx.wrapping_add(2); // 1
        let proto = match wrapped {
            0 => ConnectionProtocol::Handshaking,
            1 => ConnectionProtocol::Play,
            _ => ConnectionProtocol::Status,
        };
        assert_eq!(proto, ConnectionProtocol::Play);
        assert_eq!(proto.id(), "play");
    }
}
