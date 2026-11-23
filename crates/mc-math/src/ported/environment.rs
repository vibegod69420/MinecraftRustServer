/// Minimal trait representing the MinecraftServer dependency needed by `Environment::from`.
/// In the full port this is provided by the server module.
pub trait MinecraftServer {
    fn is_dedicated_server(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Environment {
    Client,
    Server,
}

impl Environment {
    // JAVA: net.minecraft.util.profiling.jfr.Environment#from
    pub fn from_server(server: &impl MinecraftServer) -> Self {
        if server.is_dedicated_server() {
            Self::Server
        } else {
            Self::Client
        }
    }

    // JAVA: net.minecraft.util.profiling.jfr.Environment#getDescription
    pub fn get_description(&self) -> &str {
        match self {
            Self::Client => "client",
            Self::Server => "server",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockServer(bool);

    impl MinecraftServer for MockServer {
        fn is_dedicated_server(&self) -> bool {
            self.0
        }
    }

    #[test]
    fn descriptions_match_java() {
        assert_eq!(Environment::Client.get_description(), "client");
        assert_eq!(Environment::Server.get_description(), "server");
    }

    #[test]
    fn from_dedicated_server_returns_server() {
        let server = MockServer(true);
        assert_eq!(Environment::from_server(&server), Environment::Server);
    }

    #[test]
    fn from_integrated_server_returns_client() {
        let server = MockServer(false);
        assert_eq!(Environment::from_server(&server), Environment::Client);
    }

    #[test]
    fn enum_variants_are_distinct() {
        assert_ne!(Environment::Client, Environment::Server);
        let c = Environment::Client;
        assert_eq!(c, c.clone());
    }
}
