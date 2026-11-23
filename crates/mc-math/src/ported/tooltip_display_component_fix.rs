// JAVA: net.minecraft.util.datafix.fixes.TooltipDisplayComponentFix
use std::collections::{BTreeMap, BTreeSet};

const CONVERTED_ADDITIONAL_TOOLTIP_TYPES: &[&str] = &[
    "minecraft:banner_patterns",
    "minecraft:bees",
    "minecraft:block_entity_data",
    "minecraft:block_state",
    "minecraft:bundle_contents",
    "minecraft:charged_projectiles",
    "minecraft:container",
    "minecraft:container_loot",
    "minecraft:firework_explosion",
    "minecraft:fireworks",
    "minecraft:instrument",
    "minecraft:map_id",
    "minecraft:painting/variant",
    "minecraft:pot_decorations",
    "minecraft:potion_contents",
    "minecraft:tropical_fish/pattern",
    "minecraft:written_book_content",
];

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Str(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Dynamic> {
        match self {
            Dynamic::Map(m) => m.get_mut(key),
            _ => None,
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<Dynamic> {
        match self {
            Dynamic::Map(m) => m.remove(key),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: Dynamic) {
        if let Dynamic::Map(m) = self {
            m.insert(key.to_string(), value);
        }
    }

    pub fn as_bool(&self, default: bool) -> bool {
        match self {
            Dynamic::Bool(v) => *v,
            _ => default,
        }
    }

    pub fn create_map(entries: BTreeMap<String, Dynamic>) -> Self {
        Dynamic::Map(entries)
    }

    pub fn create_list(values: Vec<Dynamic>) -> Self {
        Dynamic::List(values)
    }

    pub fn create_string(value: &str) -> Self {
        Dynamic::Str(value.to_string())
    }

    pub fn create_boolean(value: bool) -> Self {
        Dynamic::Bool(value)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.TooltipDisplayComponentFix#fix
pub fn fix(components: Dynamic) -> Dynamic {
    let mut components = components;
    fix_in_place(&mut components);
    components
}

// JAVA: net.minecraft.util.datafix.fixes.TooltipDisplayComponentFix#fix
pub fn fix_in_place(components: &mut Dynamic) {
    let mut hidden: BTreeSet<String> = BTreeSet::new();

    fix_adventure_mode_predicate(components, "minecraft:can_place_on", &mut hidden);
    fix_adventure_mode_predicate(components, "minecraft:can_break", &mut hidden);

    if let Dynamic::Map(_) = components {
        fix_remainder_component(components, "minecraft:trim", &mut hidden, |x| x.clone());
        fix_remainder_component(components, "minecraft:unbreakable", &mut hidden, |x| x.clone());
        fix_remainder_component(components, "minecraft:dyed_color", &mut hidden, |x| {
            x.get("rgb").cloned().unwrap_or_else(|| x.clone())
        });
        fix_remainder_component(components, "minecraft:attribute_modifiers", &mut hidden, |x| {
            x.get("modifiers").cloned().unwrap_or_else(|| x.clone())
        });
        fix_remainder_component(components, "minecraft:enchantments", &mut hidden, |x| {
            x.get("levels").cloned().unwrap_or_else(|| x.clone())
        });
        fix_remainder_component(components, "minecraft:stored_enchantments", &mut hidden, |x| {
            x.get("levels").cloned().unwrap_or_else(|| x.clone())
        });
        fix_remainder_component(components, "minecraft:jukebox_playable", &mut hidden, |x| {
            x.get("song").cloned().unwrap_or_else(|| x.clone())
        });

        let hide_tooltip = components.get("minecraft:hide_tooltip").is_some();
        components.remove("minecraft:hide_tooltip");

        let hide_additional_tooltip = components.get("minecraft:hide_additional_tooltip").is_some();
        components.remove("minecraft:hide_additional_tooltip");

        if hide_additional_tooltip {
            for component in CONVERTED_ADDITIONAL_TOOLTIP_TYPES {
                if components.get(*component).is_some() {
                    hidden.insert((*component).to_string());
                }
            }
        }

        if hidden.is_empty() && !hide_tooltip {
            return;
        }

        let mut map = BTreeMap::new();
        map.insert("hide_tooltip".to_string(), Dynamic::Bool(hide_tooltip));
        map.insert(
            "hidden_components".to_string(),
            Dynamic::List(hidden.into_iter().map(Dynamic::Str).collect()),
        );
        components.set("minecraft:tooltip_display", Dynamic::Map(map));
    }
}

// JAVA: net.minecraft.util.datafix.fixes.TooltipDisplayComponentFix#fixAdventureModePredicate
fn fix_adventure_mode_predicate(components: &mut Dynamic, field: &str, hidden: &mut BTreeSet<String>) {
    if let Some(value) = components.get_mut(field).cloned() {
        if let Some(predicates) = value.get("predicates") {
            let _ = predicates;
            let show = value.get("show_in_tooltip").map(|v| v.as_bool(true)).unwrap_or(true);
            if !show {
                hidden.insert(field.to_string());
            }
            if let Some(predicates) = value.get("predicates").cloned() {
                components.set(field, predicates);
            }
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.TooltipDisplayComponentFix#fixRemainderComponent
fn fix_remainder_component<F>(components: &mut Dynamic, field: &str, hidden: &mut BTreeSet<String>, unwrap: F)
where
    F: Fn(&Dynamic) -> Dynamic,
{
    if let Some(value) = components.get_mut(field).cloned() {
        let show = value.get("show_in_tooltip").map(|v| v.as_bool(true)).unwrap_or(true);
        if !show {
            hidden.insert(field.to_string());
        }
        let mut value = value;
        value.remove("show_in_tooltip");
        let unwrapped = unwrap(&value);
        components.set(field, unwrapped);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_simple_component() {
        let mut m = BTreeMap::new();
        m.insert("minecraft:trim".to_string(), Dynamic::Map({
            let mut x = BTreeMap::new();
            x.insert("show_in_tooltip".to_string(), Dynamic::Bool(false));
            x
        }));
        let out = fix(Dynamic::Map(m));
        assert!(out.get("minecraft:tooltip_display").is_some());
    }

    #[test]
    fn unwraps_dyed_color() {
        let mut inner = BTreeMap::new();
        inner.insert("rgb".to_string(), Dynamic::Str("red".to_string()));
        inner.insert("show_in_tooltip".to_string(), Dynamic::Bool(true));
        let mut m = BTreeMap::new();
        m.insert("minecraft:dyed_color".to_string(), Dynamic::Map(inner));
        let out = fix(Dynamic::Map(m));
        match out.get("minecraft:dyed_color") {
            Some(Dynamic::Str(s)) => assert_eq!(s, "red"),
            _ => panic!("not unwrapped"),
        }
    }

    #[test]
    fn hide_tooltip_flag() {
        let mut m = BTreeMap::new();
        m.insert("minecraft:hide_tooltip".to_string(), Dynamic::Bool(true));
        let out = fix(Dynamic::Map(m));
        let td = out.get("minecraft:tooltip_display").unwrap();
        assert_eq!(td.get("hide_tooltip").unwrap(), &Dynamic::Bool(true));
    }

    #[test]
    fn additional_tooltip_conversion() {
        let mut m = BTreeMap::new();
        m.insert("minecraft:hide_additional_tooltip".to_string(), Dynamic::Bool(true));
        m.insert("minecraft:bees".to_string(), Dynamic::Null);
        let out = fix(Dynamic::Map(m));
        let td = out.get("minecraft:tooltip_display").unwrap();
        let list = td.get("hidden_components").unwrap();
        match list {
            Dynamic::List(v) => assert!(v.contains(&Dynamic::Str("minecraft:bees".to_string()))),
            _ => panic!("not list"),
        }
    }

    #[test]
    fn no_change_when_nothing_hidden() {
        let m = BTreeMap::new();
        let out = fix(Dynamic::Map(m));
        assert!(out.get("minecraft:tooltip_display").is_none());
    }

    #[test]
    fn adventure_predicate_unwrap() {
        let mut pred = BTreeMap::new();
        pred.insert("predicates".to_string(), Dynamic::Str("p".to_string()));
        pred.insert("show_in_tooltip".to_string(), Dynamic::Bool(false));
        let mut m = BTreeMap::new();
        m.insert("minecraft:can_place_on".to_string(), Dynamic::Map(pred));
        let out = fix(Dynamic::Map(m));
        assert_eq!(out.get("minecraft:can_place_on"), Some(&Dynamic::Str("p".to_string())));
    }
}
