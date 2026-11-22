/// Minimal stand-in for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

/// Minimal stand-in for `net.minecraft.network.FriendlyByteBuf`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    pub data: Vec<u8>,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

pub trait CustomQueryPayload {
    // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryPayload#id
    fn id(&self) -> &Identifier;

    // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryPayload#write
    fn write(&self, buf: &mut FriendlyByteBuf);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyPayload {
        identifier: Identifier,
    }

    impl CustomQueryPayload for DummyPayload {
        fn id(&self) -> &Identifier {
            &self.identifier
        }

        fn write(&self, buf: &mut FriendlyByteBuf) {
            buf.data.extend_from_slice(self.identifier.path.as_bytes());
        }
    }

    #[test]
    fn trait_id_returns_identifier() {
        let payload = DummyPayload {
            identifier: Identifier::new("minecraft", "test"),
        };
        assert_eq!(payload.id(), &Identifier::new("minecraft", "test"));
    }

    #[test]
    fn trait_write_mutates_buffer() {
        let payload = DummyPayload {
            identifier: Identifier::new("minecraft", "abc"),
        };
        let mut buf = FriendlyByteBuf::new();
        payload.write(&mut buf);
        assert_eq!(buf.data, b"abc");
    }

    #[test]
    fn identifier_equality_and_clone() {
        let id1 = Identifier::new("ns", "path");
        let id2 = id1.clone();
        assert_eq!(id1, id2);
        assert_ne!(id1, Identifier::new("ns", "other"));
    }

    #[test]
    fn identifier_empty_components() {
        let id = Identifier::new("", "");
        assert_eq!(id.namespace, "");
        assert_eq!(id.path, "");
    }

    #[test]
    fn friendly_byte_buf_default_is_empty() {
        let buf = FriendlyByteBuf::default();
        assert!(buf.data.is_empty());
    }
}
