use std::sync::{Mutex, OnceLock};

/// Placeholder for `com.mojang.serialization.MapCodec`.
/// The full codec framework is not yet ported; this stub preserves
/// the type identity used by object info registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapCodec {
    pub type_key: String,
}

impl MapCodec {
    pub fn new(type_key: impl Into<String>) -> Self {
        Self {
            type_key: type_key.into(),
        }
    }
}

/// Minimal placeholder for `net.minecraft.util.ExtraCodecs.LateBoundIdMapper`.
/// Supports the registration and lookup behavior used by `ObjectInfos`.
#[derive(Debug)]
pub struct LateBoundIdMapper {
    entries: Mutex<Vec<(String, MapCodec)>>,
}

impl LateBoundIdMapper {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }

    pub fn put(&self, key: impl Into<String>, codec: MapCodec) {
        self.entries
            .lock()
            .expect("object info id mapper lock poisoned")
            .push((key.into(), codec));
    }

    pub fn by_id(&self, key: &str) -> Option<MapCodec> {
        self.entries
            .lock()
            .expect("object info id mapper lock poisoned")
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    pub fn keys(&self) -> Vec<String> {
        self.entries
            .lock()
            .expect("object info id mapper lock poisoned")
            .iter()
            .map(|(k, _)| k.clone())
            .collect()
    }
}

impl Default for LateBoundIdMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder for the dispatch codec produced by
/// `ComponentSerialization.createLegacyComponentMatcher(...)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfoDispatchCodec {
    pub discriminator: String,
}

static ID_MAPPER: OnceLock<LateBoundIdMapper> = OnceLock::new();
static CODEC: OnceLock<ObjectInfoDispatchCodec> = OnceLock::new();

// JAVA: net.minecraft.network.chat.contents.objects.ObjectInfos#<clinit>
fn init_id_mapper() -> LateBoundIdMapper {
    let mapper = LateBoundIdMapper::new();
    // AtlasSprite.MAP_CODEC
    mapper.put("atlas", MapCodec::new("atlas"));
    // PlayerSprite.MAP_CODEC
    mapper.put("player", MapCodec::new("player"));
    mapper
}

// JAVA: net.minecraft.network.chat.contents.objects.ObjectInfos#ID_MAPPER
pub fn id_mapper() -> &'static LateBoundIdMapper {
    ID_MAPPER.get_or_init(init_id_mapper)
}

// JAVA: net.minecraft.network.chat.contents.objects.ObjectInfos#CODEC
pub fn codec() -> &'static ObjectInfoDispatchCodec {
    // Ensure mapper initialization order matches Java static initialization.
    let _ = id_mapper();
    CODEC.get_or_init(|| ObjectInfoDispatchCodec {
        discriminator: "object".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_mapper_contains_atlas() {
        let found = id_mapper().by_id("atlas");
        assert_eq!(found, Some(MapCodec::new("atlas")));
    }

    #[test]
    fn id_mapper_contains_player() {
        let found = id_mapper().by_id("player");
        assert_eq!(found, Some(MapCodec::new("player")));
    }

    #[test]
    fn id_mapper_missing_key_returns_none() {
        assert_eq!(id_mapper().by_id("unknown"), None);
        assert_eq!(id_mapper().by_id(""), None);
        assert_eq!(id_mapper().by_id("Atlas"), None);
    }

    #[test]
    fn id_mapper_has_expected_registration_order() {
        assert_eq!(
            id_mapper().keys(),
            vec!["atlas".to_string(), "player".to_string()]
        );
    }

    #[test]
    fn codec_discriminator_is_object() {
        assert_eq!(codec().discriminator, "object");
    }

    #[test]
    fn codec_and_mapper_init_are_idempotent() {
        let keys_a = id_mapper().keys();
        let keys_b = id_mapper().keys();
        let disc_a = codec().discriminator.clone();
        let disc_b = codec().discriminator.clone();
        assert_eq!(keys_a, keys_b);
        assert_eq!(disc_a, disc_b);
    }
}
