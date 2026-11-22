use std::marker::PhantomData;

/// Minimal self-contained placeholder for Mojang's MapCodec.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapCodec<T>(PhantomData<T>);

impl<T> MapCodec<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

/// Minimal self-contained representation of `ResolvableProfile`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvableProfile {
    pub name: Option<String>,
}

impl ResolvableProfile {
    // JAVA: net.minecraft.world.item.component.ResolvableProfile#<init>
    pub fn new(name: Option<String>) -> Self {
        Self { name }
    }

    // JAVA: net.minecraft.world.item.component.ResolvableProfile#name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Minimal self-contained representation of `FontDescription`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontDescription {
    PlayerSprite {
        player: ResolvableProfile,
        hat: bool,
    },
}

/// Port of `net.minecraft.network.chat.contents.objects.PlayerSprite`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerSprite {
    pub player: ResolvableProfile,
    pub hat: bool,
}

impl PlayerSprite {
    pub const MAP_CODEC: MapCodec<PlayerSprite> = MapCodec::new();

    // JAVA: net.minecraft.network.chat.contents.objects.PlayerSprite#<init>
    pub fn new(player: ResolvableProfile, hat: bool) -> Self {
        Self { player, hat }
    }

    // JAVA: net.minecraft.network.chat.contents.objects.PlayerSprite#player
    pub fn player(&self) -> &ResolvableProfile {
        &self.player
    }

    // JAVA: net.minecraft.network.chat.contents.objects.PlayerSprite#hat
    pub fn hat(&self) -> bool {
        self.hat
    }

    // JAVA: net.minecraft.network.chat.contents.objects.PlayerSprite#fontDescription
    pub fn font_description(&self) -> FontDescription {
        FontDescription::PlayerSprite {
            player: self.player.clone(),
            hat: self.hat,
        }
    }

    // JAVA: net.minecraft.network.chat.contents.objects.PlayerSprite#description
    pub fn description(&self) -> String {
        match self.player.name() {
            Some(name) => format!("[{} head]", name),
            None => "[unknown player head]".to_string(),
        }
    }

    // JAVA: net.minecraft.network.chat.contents.objects.PlayerSprite#codec
    pub fn codec(&self) -> MapCodec<PlayerSprite> {
        Self::MAP_CODEC
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn description_with_name() {
        let sprite = PlayerSprite::new(ResolvableProfile::new(Some("Steve".to_string())), true);
        assert_eq!(sprite.description(), "[Steve head]");
    }

    #[test]
    fn description_unknown_player() {
        let sprite = PlayerSprite::new(ResolvableProfile::new(None), true);
        assert_eq!(sprite.description(), "[unknown player head]");
    }

    #[test]
    fn description_empty_name_is_preserved() {
        let sprite = PlayerSprite::new(ResolvableProfile::new(Some(String::new())), false);
        assert_eq!(sprite.description(), "[ head]");
    }

    #[test]
    fn font_description_contains_player_and_hat() {
        let profile = ResolvableProfile::new(Some("Alex".to_string()));
        let sprite = PlayerSprite::new(profile.clone(), false);
        assert_eq!(
            sprite.font_description(),
            FontDescription::PlayerSprite {
                player: profile,
                hat: false,
            }
        );
    }

    #[test]
    fn accessors_and_codec_match_fields() {
        let profile = ResolvableProfile::new(Some("Player".to_string()));
        let sprite = PlayerSprite::new(profile.clone(), true);
        assert_eq!(sprite.player(), &profile);
        assert!(sprite.hat());
        assert_eq!(sprite.codec(), PlayerSprite::MAP_CODEC);
    }
}
