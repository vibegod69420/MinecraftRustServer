use bytes::BytesMut;
use std::ops::{Deref, DerefMut};

/// Minimal stand-in for `net.minecraft.core.RegistryAccess`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryAccess;

/// Minimal stand-in for `net.minecraft.network.FriendlyByteBuf`.
#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    source: BytesMut,
}

impl FriendlyByteBuf {
    pub fn new(source: BytesMut) -> Self {
        Self { source }
    }

    pub fn source(&self) -> &BytesMut {
        &self.source
    }

    pub fn source_mut(&mut self) -> &mut BytesMut {
        &mut self.source
    }
}

pub struct RegistryFriendlyByteBuf {
    inner: FriendlyByteBuf,
    registry_access: RegistryAccess,
}

impl RegistryFriendlyByteBuf {
    // JAVA: net.minecraft.network.RegistryFriendlyByteBuf#<init>
    pub fn new(source: BytesMut, registry_access: RegistryAccess) -> Self {
        Self {
            inner: FriendlyByteBuf::new(source),
            registry_access,
        }
    }

    // JAVA: net.minecraft.network.RegistryFriendlyByteBuf#registryAccess
    pub fn registry_access(&self) -> &RegistryAccess {
        &self.registry_access
    }

    // JAVA: net.minecraft.network.RegistryFriendlyByteBuf#decorator
    pub fn decorator(
        registry_access: RegistryAccess,
    ) -> impl Fn(BytesMut) -> RegistryFriendlyByteBuf {
        move |buf| RegistryFriendlyByteBuf::new(buf, registry_access.clone())
    }
}

impl Deref for RegistryFriendlyByteBuf {
    type Target = FriendlyByteBuf;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RegistryFriendlyByteBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_source_and_registry_access() {
        let ra = RegistryAccess;
        let buf = RegistryFriendlyByteBuf::new(BytesMut::from(&[1u8, 2, 3][..]), ra.clone());
        assert_eq!(buf.registry_access(), &ra);
        assert_eq!(&buf.source()[..], &[1, 2, 3]);
    }

    #[test]
    fn decorator_creates_buf_with_same_registry() {
        let ra = RegistryAccess;
        let make = RegistryFriendlyByteBuf::decorator(ra.clone());
        let buf = make(BytesMut::from(&[9u8, 8][..]));
        assert_eq!(buf.registry_access(), &ra);
        assert_eq!(&buf.source()[..], &[9, 8]);
    }

    #[test]
    fn deref_allows_friendly_byte_buf_access() {
        let mut buf =
            RegistryFriendlyByteBuf::new(BytesMut::from(&[7u8][..]), RegistryAccess);
        assert_eq!(buf.source().len(), 1);
        buf.source_mut().extend_from_slice(&[8]);
        assert_eq!(&buf.source()[..], &[7, 8]);
    }

    #[test]
    fn decorator_is_reusable() {
        let make = RegistryFriendlyByteBuf::decorator(RegistryAccess);
        let a = make(BytesMut::from(&[1u8][..]));
        let b = make(BytesMut::from(&[2u8][..]));
        assert_eq!(&a.source()[..], &[1]);
        assert_eq!(&b.source()[..], &[2]);
    }
}
