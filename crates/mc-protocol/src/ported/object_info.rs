use std::fmt::Debug;

/// Placeholder for `net.minecraft.network.chat.FontDescription`.
/// The full font description type is not yet ported; this stub preserves
/// the interface contract for `ObjectInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDescription {
    pub id: String,
}

impl FontDescription {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// Placeholder for `com.mojang.serialization.MapCodec`.
/// The full codec framework is not yet ported; this stub preserves
/// the type identity used by object info registration and dispatch.
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

/// Port of `net.minecraft.network.chat.contents.objects.ObjectInfo`.
pub trait ObjectInfo: Debug {
    // JAVA: net.minecraft.network.chat.contents.objects.ObjectInfo#fontDescription
    fn font_description(&self) -> FontDescription;

    // JAVA: net.minecraft.network.chat.contents.objects.ObjectInfo#description
    fn description(&self) -> String;

    // JAVA: net.minecraft.network.chat.contents.objects.ObjectInfo#codec
    fn codec(&self) -> MapCodec;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct DummyObject {
        font: FontDescription,
        desc: String,
        codec_key: String,
    }

    impl ObjectInfo for DummyObject {
        fn font_description(&self) -> FontDescription {
            self.font.clone()
        }

        fn description(&self) -> String {
            self.desc.clone()
        }

        fn codec(&self) -> MapCodec {
            MapCodec::new(self.codec_key.clone())
        }
    }

    #[test]
    fn font_description_round_trip() {
        let obj = DummyObject {
            font: FontDescription::new("minecraft:default"),
            desc: "test".to_string(),
            codec_key: "atlas".to_string(),
        };
        assert_eq!(obj.font_description(), FontDescription::new("minecraft:default"));
    }

    #[test]
    fn description_returns_value() {
        let obj = DummyObject {
            font: FontDescription::new("f"),
            desc: "sprite description".to_string(),
            codec_key: "k".to_string(),
        };
        assert_eq!(obj.description(), "sprite description");
    }

    #[test]
    fn codec_returns_type_key() {
        let obj = DummyObject {
            font: FontDescription::new("f"),
            desc: "d".to_string(),
            codec_key: "player".to_string(),
        };
        assert_eq!(obj.codec(), MapCodec::new("player"));
    }

    #[test]
    fn empty_and_special_strings_preserved() {
        let obj = DummyObject {
            font: FontDescription::new(""),
            desc: "".to_string(),
            codec_key: "atlas:sprite/with-special_chars.1".to_string(),
        };
        assert_eq!(obj.font_description().id, "");
        assert_eq!(obj.description(), "");
        assert_eq!(obj.codec().type_key, "atlas:sprite/with-special_chars.1");
    }

    #[test]
    fn map_codec_equality_and_clone() {
        let a = MapCodec::new("atlas");
        let b = a.clone();
        let c = MapCodec::new("player");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
