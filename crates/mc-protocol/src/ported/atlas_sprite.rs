use std::fmt;
use std::marker::PhantomData;

/// Minimal self-contained placeholder for Mojang's MapCodec.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapCodec<T>(PhantomData<T>);

impl<T> MapCodec<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

/// Minimal self-contained representation of `Identifier` / `ResourceLocation`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#<init>
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: impl Into<String>) -> Self {
        Self::new("minecraft", path)
    }

    // JAVA: net.minecraft.resources.Identifier#getNamespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    // JAVA: net.minecraft.resources.Identifier#getPath
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Minimal self-contained representation of `FontDescription`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontDescription {
    AtlasSprite {
        atlas: Identifier,
        sprite: Identifier,
    },
}

/// Port of `net.minecraft.network.chat.contents.objects.AtlasSprite`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasSprite {
    pub atlas: Identifier,
    pub sprite: Identifier,
}

impl AtlasSprite {
    pub const MAP_CODEC: MapCodec<AtlasSprite> = MapCodec::new();

    // JAVA: net.minecraft.network.chat.contents.objects.AtlasSprite#DEFAULT_ATLAS
    pub fn default_atlas() -> Identifier {
        Identifier::with_default_namespace("blocks")
    }

    // JAVA: net.minecraft.network.chat.contents.objects.AtlasSprite#<init>
    pub fn new(atlas: Identifier, sprite: Identifier) -> Self {
        Self { atlas, sprite }
    }

    // JAVA: net.minecraft.network.chat.contents.objects.AtlasSprite#atlas
    pub fn atlas(&self) -> &Identifier {
        &self.atlas
    }

    // JAVA: net.minecraft.network.chat.contents.objects.AtlasSprite#sprite
    pub fn sprite(&self) -> &Identifier {
        &self.sprite
    }

    // JAVA: net.minecraft.network.chat.contents.objects.AtlasSprite#codec
    pub fn codec(&self) -> MapCodec<AtlasSprite> {
        Self::MAP_CODEC
    }

    // JAVA: net.minecraft.network.chat.contents.objects.AtlasSprite#fontDescription
    pub fn font_description(&self) -> FontDescription {
        FontDescription::AtlasSprite {
            atlas: self.atlas.clone(),
            sprite: self.sprite.clone(),
        }
    }

    // JAVA: net.minecraft.network.chat.contents.objects.AtlasSprite#toShortName
    fn to_short_name(id: &Identifier) -> String {
        if id.namespace() == "minecraft" {
            id.path().to_string()
        } else {
            id.to_string()
        }
    }

    // JAVA: net.minecraft.network.chat.contents.objects.AtlasSprite#description
    pub fn description(&self) -> String {
        let short_sprite = Self::to_short_name(&self.sprite);
        if self.atlas == Self::default_atlas() {
            format!("[{}]", short_sprite)
        } else {
            format!(
                "[{}@{}]",
                short_sprite,
                Self::to_short_name(&self.atlas)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn description_default_atlas_minecraft_sprite() {
        let sprite = AtlasSprite::new(
            AtlasSprite::default_atlas(),
            Identifier::with_default_namespace("stone"),
        );
        assert_eq!(sprite.description(), "[stone]");
    }

    #[test]
    fn description_default_atlas_non_minecraft_sprite() {
        let sprite = AtlasSprite::new(
            AtlasSprite::default_atlas(),
            Identifier::new("mymod", "custom_block"),
        );
        assert_eq!(sprite.description(), "[mymod:custom_block]");
    }

    #[test]
    fn description_custom_atlas_minecraft_sprite() {
        let sprite = AtlasSprite::new(
            Identifier::with_default_namespace("gui"),
            Identifier::with_default_namespace("heart"),
        );
        assert_eq!(sprite.description(), "[heart@gui]");
    }

    #[test]
    fn description_custom_atlas_non_minecraft_names() {
        let sprite = AtlasSprite::new(
            Identifier::new("other", "atlas"),
            Identifier::new("mod", "icon"),
        );
        assert_eq!(sprite.description(), "[mod:icon@other:atlas]");
    }

    #[test]
    fn font_description_and_accessors() {
        let atlas = Identifier::with_default_namespace("blocks");
        let spr = Identifier::with_default_namespace("dirt");
        let sprite = AtlasSprite::new(atlas.clone(), spr.clone());
        assert_eq!(sprite.atlas(), &atlas);
        assert_eq!(sprite.sprite(), &spr);
        assert_eq!(
            sprite.font_description(),
            FontDescription::AtlasSprite { atlas, sprite: spr }
        );
        assert_eq!(sprite.codec(), AtlasSprite::MAP_CODEC);
    }

    #[test]
    fn to_short_name_uses_path_for_minecraft_namespace() {
        let id = Identifier::with_default_namespace("stone");
        assert_eq!(AtlasSprite::to_short_name(&id), "stone");
        let mod_id = Identifier::new("mod", "stone");
        assert_eq!(AtlasSprite::to_short_name(&mod_id), "mod:stone");
    }
}
