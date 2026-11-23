#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Map(Vec<(Dynamic, Dynamic)>),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#asString(String)
    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_owned(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createString
    pub fn create_string(&self, value: &str) -> Dynamic {
        Dynamic::String(value.to_owned())
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

pub struct OptionsMenuBlurrinessFix;

impl OptionsMenuBlurrinessFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsMenuBlurrinessFix#<init>
    pub fn new(_schema: Schema) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsMenuBlurrinessFix#convertToIntRange
    fn convert_to_int_range(&self, value: &str) -> i32 {
        // Java Float.parseFloat does not accept leading/trailing whitespace.
        if value.trim().len() != value.len() {
            return 5;
        }
        match value.parse::<f32>() {
            Ok(v) => {
                let scaled = v * 10.0_f32;
                // Java Math.round(float): (int) floor(a + 0.5f), with saturating int cast semantics.
                let rounded = (scaled + 0.5_f32).floor();
                if rounded.is_nan() {
                    0
                } else if rounded >= i32::MAX as f32 {
                    i32::MAX
                } else if rounded <= i32::MIN as f32 {
                    i32::MIN
                } else {
                    rounded as i32
                }
            }
            Err(_) => 5,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsMenuBlurrinessFix#makeRule
    pub fn make_rule(&self, dynamic: Dynamic) -> Dynamic {
        dynamic.update("menuBackgroundBlurriness", |v| {
            let s = v.as_string("0.5");
            let i = self.convert_to_int_range(&s);
            v.create_string(&i.to_string())
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
    fn converts_half_to_five() {
        let fix = OptionsMenuBlurrinessFix::new(Schema);
        let input = Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("0.5"))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("5"))]));
    }

    #[test]
    fn converts_one_to_ten() {
        let fix = OptionsMenuBlurrinessFix::new(Schema);
        let input = Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("1.0"))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("10"))]));
    }

    #[test]
    fn converts_zero_to_zero() {
        let fix = OptionsMenuBlurrinessFix::new(Schema);
        let input = Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("0.0"))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("0"))]));
    }

    #[test]
    fn invalid_string_defaults_to_five() {
        let fix = OptionsMenuBlurrinessFix::new(Schema);
        let input = Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("abc"))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("5"))]));
    }

    #[test]
    fn rounds_half_up_like_java() {
        let fix = OptionsMenuBlurrinessFix::new(Schema);
        let input = Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("0.75"))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("menuBackgroundBlurriness"), s("8"))]));
    }

    #[test]
    fn missing_key_unchanged() {
        let fix = OptionsMenuBlurrinessFix::new(Schema);
        let input = Dynamic::Map(vec![(s("other"), s("x"))]);
        let out = fix.make_rule(input.clone());
        assert_eq!(out, input);
    }
}
