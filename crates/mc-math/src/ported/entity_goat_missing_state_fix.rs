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
    // JAVA: com.mojang.serialization.Dynamic#createBoolean
    pub fn create_boolean(&self, value: bool) -> Self {
        Dynamic::Bool(value)
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

pub struct RemainderFinder;

pub struct Dsl;

impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#remainderFinder
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

impl Typed {
    // JAVA: com.mojang.datafixers.Typed#update
    pub fn update<F>(self, _finder: RemainderFinder, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Self {
            remainder: f(self.remainder),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntityGoatMissingStateFix {
    pub schema: Schema,
    pub always_rewrite: bool,
    pub name: String,
    pub entity_name: String,
}

impl EntityGoatMissingStateFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityGoatMissingStateFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            always_rewrite: false,
            name: "EntityGoatMissingStateFix".to_owned(),
            entity_name: "minecraft:goat".to_owned(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityGoatMissingStateFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(Dsl::remainder_finder(), |dynamic| {
            let left = dynamic.create_boolean(true);
            let right = dynamic.create_boolean(true);
            dynamic
                .set("HasLeftHorn", left)
                .set("HasRightHorn", right)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_sets_fields() {
        let fix = EntityGoatMissingStateFix::new(Schema);
        assert!(!fix.always_rewrite);
        assert_eq!(fix.name, "EntityGoatMissingStateFix");
        assert_eq!(fix.entity_name, "minecraft:goat");
    }

    #[test]
    fn fix_sets_both_horn_flags() {
        let fix = EntityGoatMissingStateFix::new(Schema);
        let out = fix.fix(Typed {
            remainder: Dynamic::Map(vec![]),
        });
        assert_eq!(
            out.remainder,
            Dynamic::Map(vec![
                ("HasLeftHorn".to_owned(), Dynamic::Bool(true)),
                ("HasRightHorn".to_owned(), Dynamic::Bool(true)),
            ])
        );
    }

    #[test]
    fn fix_overwrites_existing_horn_flags() {
        let fix = EntityGoatMissingStateFix::new(Schema);
        let out = fix.fix(Typed {
            remainder: Dynamic::Map(vec![
                ("HasLeftHorn".to_owned(), Dynamic::Bool(false)),
                ("HasRightHorn".to_owned(), Dynamic::Bool(false)),
                ("IsScreamingGoat".to_owned(), Dynamic::Bool(true)),
            ]),
        });
        assert_eq!(
            out.remainder,
            Dynamic::Map(vec![
                ("HasLeftHorn".to_owned(), Dynamic::Bool(true)),
                ("HasRightHorn".to_owned(), Dynamic::Bool(true)),
                ("IsScreamingGoat".to_owned(), Dynamic::Bool(true)),
            ])
        );
    }

    #[test]
    fn fix_works_on_non_map_remainder() {
        let fix = EntityGoatMissingStateFix::new(Schema);
        let out = fix.fix(Typed {
            remainder: Dynamic::Nil,
        });
        assert_eq!(
            out.remainder,
            Dynamic::Map(vec![
                ("HasLeftHorn".to_owned(), Dynamic::Bool(true)),
                ("HasRightHorn".to_owned(), Dynamic::Bool(true)),
            ])
        );
    }
}
