use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i32),
    String(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Null),
            _ => Dynamic::Null,
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn as_stream(&self) -> Vec<Dynamic> {
        match self {
            Dynamic::List(v) => v.clone(),
            _ => Vec::new(),
        }
    }

    pub fn update<F>(&self, key: &str, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                let old = out.get(key).cloned().unwrap_or(Dynamic::Null);
                out.insert(key.to_string(), f(old));
                Dynamic::Map(out)
            }
            other => other.clone(),
        }
    }

    pub fn map(entries: Vec<(String, Dynamic)>) -> Self {
        Dynamic::Map(entries.into_iter().collect())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum References {
    Entity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NamedEntityFix {
    pub schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub reference: References,
    pub entity_name: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RemainderFinder;

pub struct Dsl;

impl Dsl {
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

impl Typed {
    pub fn new(remainder: Dynamic) -> Self {
        Self { remainder }
    }

    pub fn update<F>(self, _finder: RemainderFinder, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Typed {
            remainder: f(self.remainder),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemoveGolemGossipFix {
    pub base: NamedEntityFix,
}

impl RemoveGolemGossipFix {
    // JAVA: net.minecraft.util.datafix.fixes.RemoveGolemGossipFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            base: NamedEntityFix {
                schema,
                changes_type,
                name: "Remove Golem Gossip Fix".to_string(),
                reference: References::Entity,
                entity_name: "minecraft:villager".to_string(),
            },
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.RemoveGolemGossipFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(Dsl::remainder_finder(), Self::fix_value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.RemoveGolemGossipFix#fixValue
    fn fix_value(d: Dynamic) -> Dynamic {
        d.update("Gossips", |gossips| {
            let filtered: Vec<Dynamic> = gossips
                .as_stream()
                .into_iter()
                .filter(|entry| entry.get("Type").as_string("") != "golem")
                .collect();
            Dynamic::List(filtered)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gossip(gossip_type: &str) -> Dynamic {
        Dynamic::map(vec![(
            "Type".to_string(),
            Dynamic::String(gossip_type.to_string()),
        )])
    }

    fn villager_with_gossips(gossips: Vec<Dynamic>) -> Dynamic {
        Dynamic::map(vec![("Gossips".to_string(), Dynamic::List(gossips))])
    }

    #[test]
    fn removes_golem_gossip() {
        let fix = RemoveGolemGossipFix::new(Schema, true);
        let input = Typed::new(villager_with_gossips(vec![
            gossip("golem"),
            gossip("major_positive"),
        ]));
        let out = fix.fix(input);
        let gossips = out.remainder.get("Gossips").as_stream();
        assert_eq!(gossips.len(), 1);
        assert_eq!(gossips[0].get("Type").as_string(""), "major_positive");
    }

    #[test]
    fn keeps_non_golem_gossips() {
        let fix = RemoveGolemGossipFix::new(Schema, false);
        let input = Typed::new(villager_with_gossips(vec![
            gossip("trading"),
            gossip("minor_negative"),
        ]));
        let out = fix.fix(input);
        assert_eq!(out.remainder.get("Gossips").as_stream().len(), 2);
    }

    #[test]
    fn empty_gossips_remains_empty() {
        let fix = RemoveGolemGossipFix::new(Schema, true);
        let input = Typed::new(villager_with_gossips(vec![]));
        let out = fix.fix(input);
        assert!(out.remainder.get("Gossips").as_stream().is_empty());
    }

    #[test]
    fn missing_gossips_becomes_empty_list() {
        let fix = RemoveGolemGossipFix::new(Schema, true);
        let input = Typed::new(Dynamic::map(vec![]));
        let out = fix.fix(input);
        assert_eq!(out.remainder.get("Gossips"), Dynamic::List(vec![]));
    }

    #[test]
    fn removes_all_golem_gossips() {
        let fix = RemoveGolemGossipFix::new(Schema, true);
        let input = Typed::new(villager_with_gossips(vec![gossip("golem"), gossip("golem")]));
        let out = fix.fix(input);
        assert!(out.remainder.get("Gossips").as_stream().is_empty());
    }
}
