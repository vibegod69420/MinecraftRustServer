#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Null,
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Str(String),
    Map(Vec<(String, DynamicValue)>),
    List(Vec<DynamicValue>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynamicValue,
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match &self.value {
            DynamicValue::Map(entries) => {
                for (k, v) in entries {
                    if k == key {
                        return Dynamic { value: v.clone() };
                    }
                }
                Dynamic {
                    value: DynamicValue::Null,
                }
            }
            _ => Dynamic {
                value: DynamicValue::Null,
            },
        }
    }

    pub fn result(&self) -> Option<Dynamic> {
        if self.value == DynamicValue::Null {
            None
        } else {
            Some(self.clone())
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match &self.value {
            DynamicValue::Map(entries) => {
                let new_entries: Vec<(String, DynamicValue)> = entries
                    .iter()
                    .filter(|(k, _)| k != key)
                    .cloned()
                    .collect();
                Dynamic {
                    value: DynamicValue::Map(new_entries),
                }
            }
            _ => self.clone(),
        }
    }

    pub fn empty_map(&self) -> Dynamic {
        Dynamic {
            value: DynamicValue::Map(Vec::new()),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match &self.value {
            DynamicValue::Map(entries) => {
                let mut new_entries: Vec<(String, DynamicValue)> =
                    Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k == key {
                        new_entries.push((k.clone(), value.value.clone()));
                        replaced = true;
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                if !replaced {
                    new_entries.push((key.to_string(), value.value));
                }
                Dynamic {
                    value: DynamicValue::Map(new_entries),
                }
            }
            _ => Dynamic {
                value: DynamicValue::Map(vec![(key.to_string(), value.value)]),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Schema;

#[derive(Clone, Debug)]
pub struct Typed {
    pub value: Dynamic,
}

impl Typed {
    pub fn update<F>(self, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Typed { value: f(self.value) }
    }
}

pub struct AreaEffectCloudPotionFix {
    _schema: Schema,
}

impl AreaEffectCloudPotionFix {
    // JAVA: net.minecraft.util.datafix.fixes.AreaEffectCloudPotionFix#<init>
    pub fn new(schema: Schema) -> Self {
        // super(schema, false, "AreaEffectCloudPotionFix", References.ENTITY, "minecraft:area_effect_cloud");
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.AreaEffectCloudPotionFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(|d| self.fix_dynamic(d))
    }

    // JAVA: net.minecraft.util.datafix.fixes.AreaEffectCloudPotionFix#fix
    fn fix_dynamic(&self, dynamic: Dynamic) -> Dynamic {
        let color = dynamic.get("Color").result();
        let effects = dynamic.get("effects").result();
        let potion = dynamic.get("Potion").result();
        let dynamic = dynamic.remove("Color").remove("effects").remove("Potion");
        if color.is_none() && effects.is_none() && potion.is_none() {
            return dynamic;
        }

        let mut potion_contents = dynamic.empty_map();
        if let Some(c) = color {
            potion_contents = potion_contents.set("custom_color", c);
        }
        if let Some(e) = effects {
            potion_contents = potion_contents.set("custom_effects", e);
        }
        if let Some(p) = potion {
            potion_contents = potion_contents.set("potion", p);
        }

        dynamic.set("potion_contents", potion_contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, DynamicValue)>) -> Dynamic {
        Dynamic {
            value: DynamicValue::Map(
                entries
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
            ),
        }
    }

    #[test]
    fn no_potion_fields_unchanged() {
        let fix = AreaEffectCloudPotionFix::new(Schema);
        let input = map(vec![("Radius", DynamicValue::Float(3.0))]);
        let out = fix.fix_dynamic(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn all_potion_fields_moved_into_potion_contents() {
        let fix = AreaEffectCloudPotionFix::new(Schema);
        let input = map(vec![
            ("Color", DynamicValue::Int(0x00FF_0000)),
            ("effects", DynamicValue::List(vec![DynamicValue::Int(1)])),
            ("Potion", DynamicValue::Str("minecraft:healing".to_string())),
            ("Radius", DynamicValue::Float(1.5)),
        ]);
        let out = fix.fix_dynamic(input);
        let pc = out.get("potion_contents");
        assert!(pc.result().is_some());
        assert_eq!(
            pc.get("custom_color").result().unwrap().value,
            DynamicValue::Int(0x00FF_0000)
        );
        assert_eq!(
            pc.get("custom_effects").result().unwrap().value,
            DynamicValue::List(vec![DynamicValue::Int(1)])
        );
        assert_eq!(
            pc.get("potion").result().unwrap().value,
            DynamicValue::Str("minecraft:healing".to_string())
        );
        assert!(out.get("Color").result().is_none());
        assert!(out.get("effects").result().is_none());
        assert!(out.get("Potion").result().is_none());
        assert_eq!(
            out.get("Radius").result().unwrap().value,
            DynamicValue::Float(1.5)
        );
    }

    #[test]
    fn partial_fields_only_present_ones_moved() {
        let fix = AreaEffectCloudPotionFix::new(Schema);
        let input = map(vec![
            ("Color", DynamicValue::Int(-1)),
            ("DurationOnUse", DynamicValue::Int(i32::MIN)),
        ]);
        let out = fix.fix_dynamic(input);
        let pc = out.get("potion_contents");
        assert_eq!(
            pc.get("custom_color").result().unwrap().value,
            DynamicValue::Int(-1)
        );
        assert!(pc.get("custom_effects").result().is_none());
        assert!(pc.get("potion").result().is_none());
        assert_eq!(
            out.get("DurationOnUse").result().unwrap().value,
            DynamicValue::Int(i32::MIN)
        );
    }

    #[test]
    fn typed_fix_wraps_dynamic_and_preserves_float_bits() {
        let fix = AreaEffectCloudPotionFix::new(Schema);
        let nan_bits = f64::NAN.to_bits();
        let input = Typed {
            value: map(vec![
                ("Potion", DynamicValue::Str("x".to_string())),
                ("custom", DynamicValue::Long(nan_bits as i64)),
            ]),
        };
        let out = fix.fix(input);
        assert!(out.value.get("potion_contents").result().is_some());
        let got_bits = match out.value.get("custom").result().unwrap().value {
            DynamicValue::Long(v) => v as u64,
            _ => panic!("expected long"),
        };
        assert_eq!(got_bits, nan_bits);
    }
}
