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

#[derive(Clone, Debug, PartialEq)]
pub struct OptionalDynamic {
    inner: Dynamic,
}

impl OptionalDynamic {
    // JAVA: com.mojang.serialization.OptionalDynamic#asInt
    pub fn as_int(&self, default: i32) -> i32 {
        match &self.inner {
            Dynamic::Int(v) => *v,
            Dynamic::Byte(v) => i32::from(*v),
            _ => default,
        }
    }
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Map(entries) => {
                for (k, v) in entries {
                    if k == key {
                        return OptionalDynamic { inner: v.clone() };
                    }
                }
                OptionalDynamic {
                    inner: Dynamic::Nil,
                }
            }
            _ => OptionalDynamic {
                inner: Dynamic::Nil,
            },
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createByte
    pub fn create_byte(&self, value: i8) -> Self {
        Dynamic::Byte(value)
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
pub struct ColorlessShulkerEntityFix {
    pub schema: Schema,
    pub always_rewrite: bool,
    pub name: String,
    pub entity_name: String,
}

impl ColorlessShulkerEntityFix {
    // JAVA: net.minecraft.util.datafix.fixes.ColorlessShulkerEntityFix#<init>
    pub fn new(schema: Schema, always_rewrite: bool) -> Self {
        Self {
            schema,
            always_rewrite,
            name: "Colorless shulker entity fix".to_owned(),
            entity_name: "minecraft:shulker".to_owned(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ColorlessShulkerEntityFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(Dsl::remainder_finder(), |dynamic| {
            if dynamic.get("Color").as_int(0) == 10 {
                let new_color = dynamic.create_byte(16_i8);
                dynamic.set("Color", new_color)
            } else {
                dynamic
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_sets_fields() {
        let fix = ColorlessShulkerEntityFix::new(Schema, true);
        assert!(fix.always_rewrite);
        assert_eq!(fix.name, "Colorless shulker entity fix");
        assert_eq!(fix.entity_name, "minecraft:shulker");
    }

    #[test]
    fn fix_rewrites_color_10_to_16() {
        let fix = ColorlessShulkerEntityFix::new(Schema, false);
        let out = fix.fix(Typed {
            remainder: Dynamic::Map(vec![("Color".to_owned(), Dynamic::Int(10))]),
        });
        assert_eq!(
            out.remainder,
            Dynamic::Map(vec![("Color".to_owned(), Dynamic::Byte(16_i8))])
        );
    }

    #[test]
    fn fix_rewrites_byte_color_10() {
        let fix = ColorlessShulkerEntityFix::new(Schema, false);
        let out = fix.fix(Typed {
            remainder: Dynamic::Map(vec![("Color".to_owned(), Dynamic::Byte(10_i8))]),
        });
        assert_eq!(
            out.remainder,
            Dynamic::Map(vec![("Color".to_owned(), Dynamic::Byte(16_i8))])
        );
    }

    #[test]
    fn fix_leaves_other_colors_unchanged() {
        let fix = ColorlessShulkerEntityFix::new(Schema, false);
        let original = Typed {
            remainder: Dynamic::Map(vec![("Color".to_owned(), Dynamic::Int(0))]),
        };
        let out = fix.fix(original.clone());
        assert_eq!(out, original);
    }

    #[test]
    fn fix_missing_color_defaults_to_zero() {
        let fix = ColorlessShulkerEntityFix::new(Schema, false);
        let original = Typed {
            remainder: Dynamic::Map(vec![("id".to_owned(), Dynamic::String("minecraft:shulker".to_owned()))]),
        };
        let out = fix.fix(original.clone());
        assert_eq!(out, original);
    }

    #[test]
    fn fix_negative_color_unchanged() {
        let fix = ColorlessShulkerEntityFix::new(Schema, false);
        let original = Typed {
            remainder: Dynamic::Map(vec![("Color".to_owned(), Dynamic::Byte(-1_i8))]),
        };
        let out = fix.fix(original.clone());
        assert_eq!(out, original);
    }
}
