use std::marker::PhantomData;

/// Minimal self-contained stub for `net.minecraft.resources.ResourceKey<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey<T> {
    pub location: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T> ResourceKey<T> {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            _marker: PhantomData,
        }
    }
}

/// Minimal self-contained stub for `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    key: Option<ResourceKey<T>>,
}

impl<T> Holder<T> {
    pub fn from_key(key: ResourceKey<T>) -> Self {
        Self { key: Some(key) }
    }

    pub fn direct() -> Self {
        Self { key: None }
    }

    pub fn unwrap_key(&self) -> Option<&ResourceKey<T>> {
        self.key.as_ref()
    }

    /// Convenience accessor matching Java `Holder#key()` style usage.
    pub fn key(&self) -> Option<&ResourceKey<T>> {
        self.key.as_ref()
    }
}

/// Minimal self-contained stub for `net.minecraft.core.HolderSet<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T> {
    holders: Vec<Holder<T>>,
}

impl<T> HolderSet<T> {
    pub fn from_holders(holders: Vec<Holder<T>>) -> Self {
        Self { holders }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Holder<T>> {
        self.holders.iter()
    }
}

/// Marker type for `net.minecraft.world.item.JukeboxSong`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JukeboxSong;

/// Minimal self-contained stub for `net.minecraft.world.item.JukeboxPlayable`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JukeboxPlayable {
    song: Holder<JukeboxSong>,
}

impl JukeboxPlayable {
    pub fn new(song: Holder<JukeboxSong>) -> Self {
        Self { song }
    }

    pub fn song(&self) -> &Holder<JukeboxSong> {
        &self.song
    }
}

/// Minimal self-contained stub for `net.minecraft.core.component.DataComponentType<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataComponentType<T> {
    id: u32,
    _marker: PhantomData<fn() -> T>,
}

/// Minimal self-contained stub for `net.minecraft.core.component.DataComponents`.
pub struct DataComponents;

impl DataComponents {
    pub fn jukebox_playable() -> DataComponentType<JukeboxPlayable> {
        DataComponentType {
            id: 0,
            _marker: PhantomData,
        }
    }
}

/// Port of `net.minecraft.core.component.predicates.JukeboxPlayablePredicate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JukeboxPlayablePredicate {
    pub song: Option<HolderSet<JukeboxSong>>,
}

impl JukeboxPlayablePredicate {
    // JAVA: net.minecraft.core.component.predicates.JukeboxPlayablePredicate#song
    pub fn song(&self) -> Option<&HolderSet<JukeboxSong>> {
        self.song.as_ref()
    }

    // JAVA: net.minecraft.core.component.predicates.JukeboxPlayablePredicate#componentType
    pub fn component_type(&self) -> DataComponentType<JukeboxPlayable> {
        DataComponents::jukebox_playable()
    }

    // JAVA: net.minecraft.core.component.predicates.JukeboxPlayablePredicate#matches
    pub fn matches(&self, playable: &JukeboxPlayable) -> bool {
        let Some(song_set) = &self.song else {
            return true;
        };

        let mut found = false;
        for holder in song_set.iter() {
            let key = holder.unwrap_key();
            if key.is_some() && key == playable.song().key() {
                found = true;
                break;
            }
        }
        found
    }

    // JAVA: net.minecraft.core.component.predicates.JukeboxPlayablePredicate#any
    pub fn any() -> Self {
        Self { song: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &str) -> ResourceKey<JukeboxSong> {
        ResourceKey::new(name)
    }

    #[test]
    fn any_matches_everything() {
        let pred = JukeboxPlayablePredicate::any();
        let playable = JukeboxPlayable::new(Holder::from_key(key("minecraft:13")));
        assert!(pred.matches(&playable));
        let direct = JukeboxPlayable::new(Holder::direct());
        assert!(pred.matches(&direct));
    }

    #[test]
    fn matches_matching_key() {
        let pred = JukeboxPlayablePredicate {
            song: Some(HolderSet::from_holders(vec![
                Holder::from_key(key("minecraft:cat")),
                Holder::from_key(key("minecraft:13")),
            ])),
        };
        let playable = JukeboxPlayable::new(Holder::from_key(key("minecraft:13")));
        assert!(pred.matches(&playable));
    }

    #[test]
    fn rejects_non_matching_key() {
        let pred = JukeboxPlayablePredicate {
            song: Some(HolderSet::from_holders(vec![Holder::from_key(key(
                "minecraft:cat",
            ))])),
        };
        let playable = JukeboxPlayable::new(Holder::from_key(key("minecraft:13")));
        assert!(!pred.matches(&playable));
    }

    #[test]
    fn direct_holder_in_set_does_not_match() {
        let pred = JukeboxPlayablePredicate {
            song: Some(HolderSet::from_holders(vec![Holder::direct()])),
        };
        let playable = JukeboxPlayable::new(Holder::direct());
        assert!(!pred.matches(&playable));
    }

    #[test]
    fn empty_set_matches_nothing() {
        let pred = JukeboxPlayablePredicate {
            song: Some(HolderSet::from_holders(vec![])),
        };
        let playable = JukeboxPlayable::new(Holder::from_key(key("minecraft:13")));
        assert!(!pred.matches(&playable));
    }

    #[test]
    fn direct_playable_does_not_match_keyed_predicate() {
        let pred = JukeboxPlayablePredicate {
            song: Some(HolderSet::from_holders(vec![Holder::from_key(key(
                "minecraft:13",
            ))])),
        };
        let playable = JukeboxPlayable::new(Holder::direct());
        assert!(!pred.matches(&playable));
    }
}
