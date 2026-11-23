//! Faithful port of `net.minecraft.world.level.storage.loot.ContainerComponentManipulators`.
//!
//! This module is self-contained: it defines minimal local representations for
//! `ItemStack`, data component types, container component payloads, and the
//! container component manipulator dispatch used by loot container modification.

use std::collections::HashMap;

/// Minimal self-contained item stack representation used by container component manipulators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
}

/// Minimal self-contained data component type identifier for the supported container components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataComponentType {
    Container,
    BundleContents,
    ChargedProjectiles,
}

/// Minimal self-contained `ItemContainerContents` payload.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemContainerContents {
    pub items: Vec<ItemStack>,
}

/// Minimal self-contained `BundleContents` payload.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BundleContents {
    pub items: Vec<ItemStack>,
}

/// Minimal self-contained `ChargedProjectiles` payload.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChargedProjectiles {
    pub items: Vec<ItemStack>,
}

/// Type-erased container component value for self-contained manipulator dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerComponentValue {
    Container(ItemContainerContents),
    Bundle(BundleContents),
    Charged(ChargedProjectiles),
}

/// Port of `net.minecraft.world.level.storage.loot.ContainerComponentManipulator` dispatch targets
/// declared in `ContainerComponentManipulators`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerComponentManipulator {
    Container,
    BundleContents,
    ChargedProjectiles,
}

/// JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulators#CONTAINER
pub const CONTAINER: ContainerComponentManipulator = ContainerComponentManipulator::Container;
/// JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulators#BUNDLE_CONTENTS
pub const BUNDLE_CONTENTS: ContainerComponentManipulator =
    ContainerComponentManipulator::BundleContents;
/// JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulators#CHARGED_PROJECTILES
pub const CHARGED_PROJECTILES: ContainerComponentManipulator =
    ContainerComponentManipulator::ChargedProjectiles;

impl ContainerComponentManipulator {
    // JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulators#type
    pub fn component_type(&self) -> DataComponentType {
        match self {
            Self::Container => DataComponentType::Container,
            Self::BundleContents => DataComponentType::BundleContents,
            Self::ChargedProjectiles => DataComponentType::ChargedProjectiles,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulators#getContents
    pub fn get_contents(&self, value: &ContainerComponentValue) -> Vec<ItemStack> {
        match (self, value) {
            (Self::Container, ContainerComponentValue::Container(contents)) => {
                contents.items.clone()
            }
            (Self::BundleContents, ContainerComponentValue::Bundle(contents)) => {
                contents.items.clone()
            }
            (Self::ChargedProjectiles, ContainerComponentValue::Charged(contents)) => {
                contents.items.clone()
            }
            _ => Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulators#empty
    pub fn empty(&self) -> ContainerComponentValue {
        match self {
            Self::Container => {
                ContainerComponentValue::Container(ItemContainerContents::default())
            }
            Self::BundleContents => ContainerComponentValue::Bundle(BundleContents::default()),
            Self::ChargedProjectiles => {
                ContainerComponentValue::Charged(ChargedProjectiles::default())
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulators#setContents
    pub fn set_contents(
        &self,
        value: &ContainerComponentValue,
        contents: impl IntoIterator<Item = ItemStack>,
    ) -> ContainerComponentValue {
        let items: Vec<ItemStack> = contents.into_iter().collect();
        match self {
            Self::Container => {
                // ItemContainerContents.fromItems($$1.toList())
                ContainerComponentValue::Container(ItemContainerContents { items })
            }
            Self::BundleContents => {
                // Mutable $$2 = new Mutable($$0).clearItems();
                // $$1.forEach($$2::tryInsert);
                // return $$2.toImmutable();
                let mut out = match value {
                    ContainerComponentValue::Bundle(bundle) => bundle.clone(),
                    _ => BundleContents::default(),
                };
                out.items.clear();
                for item in items {
                    // Minimal self-contained tryInsert: append the item.
                    out.items.push(item);
                }
                ContainerComponentValue::Bundle(out)
            }
            Self::ChargedProjectiles => {
                // ChargedProjectiles.of($$1.toList())
                ContainerComponentValue::Charged(ChargedProjectiles { items })
            }
        }
    }
}

// JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulators#ALL_MANIPULATORS
pub fn all_manipulators() -> HashMap<DataComponentType, ContainerComponentManipulator> {
    let mut map = HashMap::new();
    for manipulator in [CONTAINER, BUNDLE_CONTENTS, CHARGED_PROJECTILES] {
        map.insert(manipulator.component_type(), manipulator);
    }
    map
}

// JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulators#CODEC
pub fn codec_lookup(
    component_type: &DataComponentType,
) -> Result<ContainerComponentManipulator, &'static str> {
    match all_manipulators().get(component_type).copied() {
        Some(manipulator) => Ok(manipulator),
        None => Err("No items in component"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: &str, count: i32) -> ItemStack {
        ItemStack {
            id: id.to_string(),
            count,
        }
    }

    #[test]
    fn container_roundtrip_and_empty() {
        let empty = CONTAINER.empty();
        assert_eq!(CONTAINER.get_contents(&empty), Vec::new());

        let items = vec![stack("minecraft:stone", 1), stack("minecraft:dirt", 2)];
        let value = CONTAINER.set_contents(&empty, items.clone());
        assert_eq!(CONTAINER.get_contents(&value), items);
        assert_eq!(CONTAINER.component_type(), DataComponentType::Container);
    }

    #[test]
    fn bundle_set_contents_clears_existing_items() {
        let original = ContainerComponentValue::Bundle(BundleContents {
            items: vec![stack("minecraft:apple", 3)],
        });
        let replacement = vec![stack("minecraft:bread", 1), stack("minecraft:stick", 64)];
        let updated = BUNDLE_CONTENTS.set_contents(&original, replacement.clone());

        assert_eq!(BUNDLE_CONTENTS.get_contents(&updated), replacement);
        assert_eq!(
            BUNDLE_CONTENTS.component_type(),
            DataComponentType::BundleContents
        );
    }

    #[test]
    fn charged_projectiles_roundtrip() {
        let empty = CHARGED_PROJECTILES.empty();
        let items = vec![stack("minecraft:arrow", 1), stack("minecraft:firework_rocket", 1)];
        let value = CHARGED_PROJECTILES.set_contents(&empty, items.clone());

        assert_eq!(CHARGED_PROJECTILES.get_contents(&value), items);
        assert_eq!(
            CHARGED_PROJECTILES.component_type(),
            DataComponentType::ChargedProjectiles
        );
    }

    #[test]
    fn negative_and_boundary_counts_preserved() {
        let items = vec![
            stack("minecraft:stone", -1),
            stack("minecraft:stone", i32::MIN),
            stack("minecraft:stone", i32::MAX),
            stack("minecraft:stone", 0),
        ];
        let value = CONTAINER.set_contents(&CONTAINER.empty(), items.clone());
        let out = CONTAINER.get_contents(&value);
        assert_eq!(out[0].count, -1);
        assert_eq!(out[1].count, i32::MIN);
        assert_eq!(out[2].count, i32::MAX);
        assert_eq!(out[3].count, 0);
    }

    #[test]
    fn float_bits_preserved_through_manipulation() {
        let nan_bits = f32::NAN.to_bits() as i32;
        let neg_zero_bits = (-0.0f32).to_bits() as i32;
        let inf_bits = f32::INFINITY.to_bits() as i32;
        let items = vec![
            stack("a", nan_bits),
            stack("b", neg_zero_bits),
            stack("c", inf_bits),
        ];

        let value = CHARGED_PROJECTILES.set_contents(&CHARGED_PROJECTILES.empty(), items);
        let out = CHARGED_PROJECTILES.get_contents(&value);

        assert_eq!(f32::from_bits(out[0].count as u32).to_bits(), f32::NAN.to_bits());
        assert_eq!(
            f32::from_bits(out[1].count as u32).to_bits(),
            (-0.0f32).to_bits()
        );
        assert_eq!(
            f32::from_bits(out[2].count as u32).to_bits(),
            f32::INFINITY.to_bits()
        );
    }

    #[test]
    fn all_manipulators_and_codec_lookup() {
        let all = all_manipulators();
        assert_eq!(all.len(), 3);
        assert_eq!(
            all.get(&DataComponentType::Container).copied(),
            Some(CONTAINER)
        );
        assert_eq!(
            all.get(&DataComponentType::BundleContents).copied(),
            Some(BUNDLE_CONTENTS)
        );
        assert_eq!(
            all.get(&DataComponentType::ChargedProjectiles).copied(),
            Some(CHARGED_PROJECTILES)
        );

        assert_eq!(
            codec_lookup(&DataComponentType::Container),
            Ok(CONTAINER)
        );
        assert_eq!(
            codec_lookup(&DataComponentType::BundleContents),
            Ok(BUNDLE_CONTENTS)
        );
        assert_eq!(
            codec_lookup(&DataComponentType::ChargedProjectiles),
            Ok(CHARGED_PROJECTILES)
        );
    }
}
