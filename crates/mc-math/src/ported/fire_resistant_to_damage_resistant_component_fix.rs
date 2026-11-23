#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Nil,
    Bool(bool),
    Byte(i8),
    Int(i32),
    Float(f32),
    String(String),
    Map(Vec<(String, Dynamic)>),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#emptyMap
    pub fn empty_map(&self) -> Self {
        Dynamic::Map(Vec::new())
    }

    // JAVA: com.mojang.serialization.Dynamic#createString
    pub fn create_string(&self, value: &str) -> Self {
        Dynamic::String(value.to_owned())
    }

    // JAVA: com.mojang.serialization.Dynamic#set
    pub fn set(&self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(entries) => {
                let mut new_entries = Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k == key {
                        new_entries.push((k.clone(), value.clone()));
                        replaced = true;
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                if !replaced {
                    new_entries.push((key.to_owned(), value));
                }
                Dynamic::Map(new_entries)
            }
            _ => Dynamic::Map(vec![(key.to_owned(), value)]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FireResistantToDamageResistantComponentFix {
    pub schema: Schema,
    pub name: String,
    pub old_component: String,
    pub new_component: String,
}

impl FireResistantToDamageResistantComponentFix {
    // JAVA: net.minecraft.util.datafix.fixes.FireResistantToDamageResistantComponentFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            name: "FireResistantToDamageResistantComponentFix".to_owned(),
            old_component: "minecraft:fire_resistant".to_owned(),
            new_component: "minecraft:damage_resistant".to_owned(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.FireResistantToDamageResistantComponentFix#fixComponent
    pub fn fix_component(&self, dynamic: &Dynamic) -> Dynamic {
        dynamic
            .empty_map()
            .set("types", dynamic.create_string("#minecraft:is_fire"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_sets_fields() {
        let fix = FireResistantToDamageResistantComponentFix::new(Schema);
        assert_eq!(fix.name, "FireResistantToDamageResistantComponentFix");
        assert_eq!(fix.old_component, "minecraft:fire_resistant");
        assert_eq!(fix.new_component, "minecraft:damage_resistant");
    }

    #[test]
    fn fix_component_creates_damage_resistant_map() {
        let fix = FireResistantToDamageResistantComponentFix::new(Schema);
        let input = Dynamic::String("ignored".to_owned());
        let out = fix.fix_component(&input);
        assert_eq!(
            out,
            Dynamic::Map(vec![(
                "types".to_owned(),
                Dynamic::String("#minecraft:is_fire".to_owned()),
            )])
        );
    }

    #[test]
    fn fix_component_replaces_existing_input() {
        let fix = FireResistantToDamageResistantComponentFix::new(Schema);
        let input = Dynamic::Map(vec![
            ("foo".to_owned(), Dynamic::Int(1)),
            ("bar".to_owned(), Dynamic::Bool(true)),
        ]);
        let out = fix.fix_component(&input);
        match out {
            Dynamic::Map(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "types");
                assert_eq!(entries[0].1, Dynamic::String("#minecraft:is_fire".to_owned()));
            }
            _ => panic!("expected map"),
        }
    }

    #[test]
    fn set_replaces_existing_key() {
        let base = Dynamic::Map(vec![("types".to_owned(), Dynamic::String("old".to_owned()))]);
        let out = base.set("types", Dynamic::String("new".to_owned()));
        assert_eq!(
            out,
            Dynamic::Map(vec![("types".to_owned(), Dynamic::String("new".to_owned()))])
        );
    }
}
