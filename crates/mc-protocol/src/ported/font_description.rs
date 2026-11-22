use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvableProfile {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataResult<T> {
    Success(T),
    Error(String),
}

impl<T> DataResult<T> {
    pub fn success(value: T) -> Self {
        Self::Success(value)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtlasSprite {
    pub atlas_id: Identifier,
    pub sprite_id: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSprite {
    pub profile: ResolvableProfile,
    pub hat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub id: Identifier,
}

impl Resource {
    // JAVA: net.minecraft.network.chat.FontDescription$Resource#<init>
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.network.chat.FontDescription#DEFAULT
    pub fn default_font_description() -> Self {
        Self::new(Identifier::with_default_namespace("default"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontDescription {
    AtlasSprite(AtlasSprite),
    PlayerSprite(PlayerSprite),
    Resource(Resource),
}

impl FontDescription {
    fn type_name(&self) -> &'static str {
        match self {
            Self::AtlasSprite(_) => "AtlasSprite",
            Self::PlayerSprite(_) => "PlayerSprite",
            Self::Resource(_) => "Resource",
        }
    }

    // JAVA: net.minecraft.network.chat.FontDescription#CODEC
    pub fn codec_decode(id: Identifier) -> Self {
        Self::Resource(Resource::new(id))
    }

    // JAVA: net.minecraft.network.chat.FontDescription#CODEC
    pub fn codec_encode(&self) -> DataResult<Identifier> {
        match self {
            Self::Resource(r) => DataResult::success(r.id.clone()),
            _ => DataResult::error(format!(
                "Unsupported font description type: {}",
                self.type_name()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_font_description_uses_minecraft_default() {
        let d = Resource::default_font_description();
        assert_eq!(d.id.namespace, "minecraft");
        assert_eq!(d.id.path, "default");
        assert_eq!(d.id, Identifier::with_default_namespace("default"));
    }

    #[test]
    fn codec_decode_creates_resource() {
        let id = Identifier::with_default_namespace("custom_font");
        let desc = FontDescription::codec_decode(id.clone());
        assert_eq!(desc, FontDescription::Resource(Resource::new(id)));
    }

    #[test]
    fn codec_encode_resource_succeeds() {
        let id = Identifier {
            namespace: "ns".to_string(),
            path: "path".to_string(),
        };
        let desc = FontDescription::Resource(Resource::new(id.clone()));
        assert_eq!(desc.codec_encode(), DataResult::Success(id));
    }

    #[test]
    fn codec_encode_atlas_sprite_fails() {
        let desc = FontDescription::AtlasSprite(AtlasSprite {
            atlas_id: Identifier::with_default_namespace("atlas"),
            sprite_id: Identifier::with_default_namespace("sprite"),
        });
        match desc.codec_encode() {
            DataResult::Error(msg) => {
                assert!(msg.contains("Unsupported font description type: AtlasSprite"))
            }
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[test]
    fn codec_encode_player_sprite_fails_and_preserves_fields() {
        let profile = ResolvableProfile {
            id: "player-1".to_string(),
        };
        let desc = FontDescription::PlayerSprite(PlayerSprite {
            profile: profile.clone(),
            hat: false,
        });
        match &desc {
            FontDescription::PlayerSprite(ps) => {
                assert_eq!(ps.profile, profile);
                assert!(!ps.hat);
            }
            _ => panic!("wrong variant"),
        }
        match desc.codec_encode() {
            DataResult::Error(msg) => {
                assert!(msg.contains("Unsupported font description type: PlayerSprite"))
            }
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[test]
    fn resource_with_empty_namespace_and_path_roundtrips_through_encode() {
        let id = Identifier {
            namespace: String::new(),
            path: String::new(),
        };
        let desc = FontDescription::Resource(Resource::new(id.clone()));
        assert_eq!(desc.codec_encode(), DataResult::Success(id));
    }
}
