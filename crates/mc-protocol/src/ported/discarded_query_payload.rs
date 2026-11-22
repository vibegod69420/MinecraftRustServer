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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscardedQueryPayload {
    pub id: Identifier,
}

impl DiscardedQueryPayload {
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.network.protocol.login.custom.DiscardedQueryPayload#id
    pub fn id(&self) -> &Identifier {
        &self.id
    }
}

impl CustomQueryPayload for DiscardedQueryPayload {
    // JAVA: net.minecraft.network.protocol.login.custom.DiscardedQueryPayload#id
    fn id(&self) -> &Identifier {
        &self.id
    }

    // JAVA: net.minecraft.network.protocol.login.custom.DiscardedQueryPayload#write
    fn write(&self, _buf: &mut FriendlyByteBuf) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_returns_payload_identifier() {
        let id = Identifier::new("minecraft", "custom");
        let payload = DiscardedQueryPayload::new(id.clone());
        assert_eq!(payload.id(), &id);
        assert_eq!(CustomQueryPayload::id(&payload), &id);
    }

    #[test]
    fn write_is_noop() {
        let payload = DiscardedQueryPayload::new(Identifier::new("ns", "path"));
        let mut buf = FriendlyByteBuf::new();
        payload.write(&mut buf);
        assert!(buf.data.is_empty());
    }

    #[test]
    fn write_preserves_existing_buffer_contents() {
        let payload = DiscardedQueryPayload::new(Identifier::new("ns", "path"));
        let mut buf = FriendlyByteBuf {
            data: vec![1, 2, 3],
        };
        payload.write(&mut buf);
        assert_eq!(buf.data, vec![1, 2, 3]);
    }

    #[test]
    fn equality_and_clone() {
        let a = DiscardedQueryPayload::new(Identifier::new("a", "b"));
        let b = a.clone();
        let c = DiscardedQueryPayload::new(Identifier::new("a", "c"));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn empty_identifier_components() {
        let payload = DiscardedQueryPayload::new(Identifier::new("", ""));
        assert_eq!(payload.id().namespace, "");
        assert_eq!(payload.id().path, "");
    }
}
