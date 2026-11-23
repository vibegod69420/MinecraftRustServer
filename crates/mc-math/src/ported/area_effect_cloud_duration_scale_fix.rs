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
    // JAVA: com.mojang.serialization.Dynamic#createFloat
    pub fn create_float(&self, value: f32) -> Self {
        Dynamic::Float(value)
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
pub struct AreaEffectCloudDurationScaleFix {
    pub schema: Schema,
    pub always_rewrite: bool,
    pub name: String,
    pub entity_name: String,
}

impl AreaEffectCloudDurationScaleFix {
    // JAVA: net.minecraft.util.datafix.fixes.AreaEffectCloudDurationScaleFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            always_rewrite: false,
            name: "AreaEffectCloudDurationScaleFix".to_owned(),
            entity_name: "minecraft:area_effect_cloud".to_owned(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.AreaEffectCloudDurationScaleFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(Dsl::remainder_finder(), |dynamic| {
            let value = dynamic.create_float(0.25_f32);
            dynamic.set("potion_duration_scale", value)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_sets_fields() {
        let fix = AreaEffectCloudDurationScaleFix::new(Schema);
        assert!(!fix.always_rewrite);
        assert_eq!(fix.name, "AreaEffectCloudDurationScaleFix");
        assert_eq!(fix.entity_name, "minecraft:area_effect_cloud");
    }

    #[test]
    fn fix_sets_potion_duration_scale() {
        let fix = AreaEffectCloudDurationScaleFix::new(Schema);
        let typed = Typed {
            remainder: Dynamic::Map(vec![]),
        };
        let out = fix.fix(typed);
        assert_eq!(
            out.remainder,
            Dynamic::Map(vec![(
                "potion_duration_scale".to_owned(),
                Dynamic::Float(0.25_f32),
            )])
        );
    }

    #[test]
    fn fix_preserves_float_bits() {
        let fix = AreaEffectCloudDurationScaleFix::new(Schema);
        let out = fix.fix(Typed {
            remainder: Dynamic::Map(vec![]),
        });
        match out.remainder {
            Dynamic::Map(entries) => match entries[0].1 {
                Dynamic::Float(v) => assert_eq!(v.to_bits(), 0.25_f32.to_bits()),
                _ => panic!("expected float"),
            },
            _ => panic!("expected map"),
        }
    }

    #[test]
    fn fix_preserves_existing_fields_and_overwrites_scale() {
        let fix = AreaEffectCloudDurationScaleFix::new(Schema);
        let typed = Typed {
            remainder: Dynamic::Map(vec![
                ("Radius".to_owned(), Dynamic::Float(3.0_f32)),
                ("potion_duration_scale".to_owned(), Dynamic::Float(1.0_f32)),
            ]),
        };
        let out = fix.fix(typed);
        assert_eq!(
            out.remainder,
            Dynamic::Map(vec![
                ("Radius".to_owned(), Dynamic::Float(3.0_f32)),
                ("potion_duration_scale".to_owned(), Dynamic::Float(0.25_f32)),
            ])
        );
    }
}
