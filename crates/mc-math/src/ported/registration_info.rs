//! Faithful port of `net.minecraft.core.RegistrationInfo`.

/// Minimal self-contained stub for `net.minecraft.server.packs.repository.KnownPack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPack;

/// Minimal self-contained stub for `com.mojang.serialization.Lifecycle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
    Experimental,
}

impl Lifecycle {
    pub fn stable() -> Self {
        Self::Stable
    }
}

// JAVA: net.minecraft.core.RegistrationInfo
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationInfo {
    pub known_pack_info: Option<KnownPack>,
    pub lifecycle: Lifecycle,
}

impl RegistrationInfo {
    // JAVA: net.minecraft.core.RegistrationInfo#<init>
    pub fn new(known_pack_info: Option<KnownPack>, lifecycle: Lifecycle) -> Self {
        Self {
            known_pack_info,
            lifecycle,
        }
    }

    // JAVA: net.minecraft.core.RegistrationInfo#BUILT_IN
    pub fn built_in() -> Self {
        Self {
            known_pack_info: None,
            lifecycle: Lifecycle::stable(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_has_no_known_pack_and_stable_lifecycle() {
        let info = RegistrationInfo::built_in();
        assert_eq!(info.known_pack_info, None);
        assert_eq!(info.lifecycle, Lifecycle::Stable);
    }

    #[test]
    fn new_stores_fields() {
        let info = RegistrationInfo::new(Some(KnownPack), Lifecycle::Experimental);
        assert_eq!(info.known_pack_info, Some(KnownPack));
        assert_eq!(info.lifecycle, Lifecycle::Experimental);
    }

    #[test]
    fn record_equality() {
        let a = RegistrationInfo::new(None, Lifecycle::Stable);
        let b = RegistrationInfo::built_in();
        assert_eq!(a, b);
    }

    #[test]
    fn clone_preserves_value() {
        let info = RegistrationInfo::new(Some(KnownPack), Lifecycle::Stable);
        let cloned = info.clone();
        assert_eq!(info, cloned);
    }
}
