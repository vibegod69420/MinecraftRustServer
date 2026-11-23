#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Float(f32),
    Int(i32),
    String(String),
    List(Vec<Dynamic>),
    Map(Vec<(Dynamic, Dynamic)>),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#asNumber
    pub fn as_number_f32(&self, default: f32) -> f32 {
        match self {
            Dynamic::Float(v) => *v,
            Dynamic::Int(v) => *v as f32,
            _ => default,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createString
    pub fn create_string(&self, value: &str) -> Dynamic {
        Dynamic::String(value.to_owned())
    }

    // JAVA: com.mojang.serialization.Dynamic#createFloat
    pub fn create_float(&self, value: f32) -> Dynamic {
        Dynamic::Float(value)
    }

    // JAVA: com.mojang.serialization.Dynamic#createList
    pub fn create_list(&self, values: Vec<Dynamic>) -> Dynamic {
        Dynamic::List(values)
    }

    // JAVA: com.mojang.serialization.Dynamic#createMap
    pub fn create_map(&self, entries: Vec<(Dynamic, Dynamic)>) -> Dynamic {
        Dynamic::Map(entries)
    }

    // JAVA: com.mojang.serialization.Dynamic#update(String, Function)
    pub fn update<F>(self, key: &str, mut fixer: F) -> Dynamic
    where
        F: FnMut(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(entries) => {
                let mut out = Vec::with_capacity(entries.len());
                for (k, v) in entries {
                    let is_match = matches!(&k, Dynamic::String(s) if s == key);
                    if is_match {
                        out.push((k, fixer(v)));
                    } else {
                        out.push((k, v));
                    }
                }
                Dynamic::Map(out)
            }
            other => other,
        }
    }
}

pub struct CustomModelDataExpandFix;

impl CustomModelDataExpandFix {
    // JAVA: net.minecraft.util.datafix.fixes.CustomModelDataExpandFix#<init>
    pub fn new(_schema: Schema) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.CustomModelDataExpandFix#makeRule
    pub fn make_rule(&self, dynamic: Dynamic) -> Dynamic {
        dynamic.update("minecraft:custom_model_data", |v| {
            let f = v.as_number_f32(0.0_f32);
            let floats_key = v.create_string("floats");
            let float_val = v.create_float(f);
            let floats_list = v.create_list(vec![float_val]);
            v.create_map(vec![(floats_key, floats_list)])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(value: &str) -> Dynamic {
        Dynamic::String(value.to_owned())
    }

    #[test]
    fn expands_float_custom_model_data() {
        let fix = CustomModelDataExpandFix::new(Schema);
        let input = Dynamic::Map(vec![(s("minecraft:custom_model_data"), Dynamic::Float(1.5))]);
        let out = fix.make_rule(input);
        let expected = Dynamic::Map(vec![(
            s("minecraft:custom_model_data"),
            Dynamic::Map(vec![(
                s("floats"),
                Dynamic::List(vec![Dynamic::Float(1.5)]),
            )]),
        )]);
        assert_eq!(out, expected);
    }

    #[test]
    fn defaults_to_zero_for_non_number() {
        let fix = CustomModelDataExpandFix::new(Schema);
        let input = Dynamic::Map(vec![(s("minecraft:custom_model_data"), s("bad"))]);
        let out = fix.make_rule(input);
        let expected = Dynamic::Map(vec![(
            s("minecraft:custom_model_data"),
            Dynamic::Map(vec![(s("floats"), Dynamic::List(vec![Dynamic::Float(0.0)]))]),
        )]);
        assert_eq!(out, expected);
    }

    #[test]
    fn preserves_other_keys() {
        let fix = CustomModelDataExpandFix::new(Schema);
        let input = Dynamic::Map(vec![
            (s("minecraft:damage"), Dynamic::Int(3)),
            (s("minecraft:custom_model_data"), Dynamic::Float(2.0)),
        ]);
        let out = fix.make_rule(input);
        assert_eq!(
            out,
            Dynamic::Map(vec![
                (s("minecraft:damage"), Dynamic::Int(3)),
                (
                    s("minecraft:custom_model_data"),
                    Dynamic::Map(vec![(s("floats"), Dynamic::List(vec![Dynamic::Float(2.0)]))]),
                ),
            ])
        );
    }

    #[test]
    fn missing_key_unchanged() {
        let fix = CustomModelDataExpandFix::new(Schema);
        let input = Dynamic::Map(vec![(s("minecraft:damage"), Dynamic::Int(1))]);
        let out = fix.make_rule(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn int_value_converted_to_float() {
        let fix = CustomModelDataExpandFix::new(Schema);
        let input = Dynamic::Map(vec![(s("minecraft:custom_model_data"), Dynamic::Int(7))]);
        let out = fix.make_rule(input);
        let expected = Dynamic::Map(vec![(
            s("minecraft:custom_model_data"),
            Dynamic::Map(vec![(s("floats"), Dynamic::List(vec![Dynamic::Float(7.0)]))]),
        )]);
        assert_eq!(out, expected);
    }
}
