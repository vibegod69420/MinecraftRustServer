use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Null,
    String(String),
    Map(BTreeMap<String, Dynamic>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynamicValue,
}

impl Dynamic {
    pub fn of_string(s: impl Into<String>) -> Self {
        Self {
            value: DynamicValue::String(s.into()),
        }
    }

    pub fn of_map(map: BTreeMap<String, Dynamic>) -> Self {
        Self {
            value: DynamicValue::Map(map),
        }
    }

    pub fn null() -> Self {
        Self {
            value: DynamicValue::Null,
        }
    }

    pub fn get(&self, key: &str) -> Dynamic {
        match &self.value {
            DynamicValue::Map(m) => m.get(key).cloned().unwrap_or_else(Dynamic::null),
            _ => Dynamic::null(),
        }
    }

    pub fn as_string_result(&self) -> Option<String> {
        match &self.value {
            DynamicValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match &self.value {
            DynamicValue::Map(m) => {
                let mut new_m = m.clone();
                new_m.insert(key.to_owned(), value);
                Dynamic::of_map(new_m)
            }
            _ => self.clone(),
        }
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::of_string(s)
    }
}

fn java_double_to_string(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    if v == 0.0 {
        return if v.is_sign_negative() {
            "-0.0".to_string()
        } else {
            "0.0".to_string()
        };
    }
    let s = format!("{}", v);
    if s.find(['.', 'e', 'E']).is_none() {
        format!("{}.0", s)
    } else {
        s
    }
}

fn java_parse_double(s: &str) -> Result<f64, ()> {
    let t = s.trim();
    match t {
        "NaN" => Ok(f64::NAN),
        "Infinity" | "+Infinity" => Ok(f64::INFINITY),
        "-Infinity" => Ok(f64::NEG_INFINITY),
        _ => t.parse::<f64>().map_err(|_| ()),
    }
}

pub struct OptionsAddTextBackgroundFix;

impl OptionsAddTextBackgroundFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsAddTextBackgroundFix#<init>
    pub fn new(_schema: (), _changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsAddTextBackgroundFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic + '_ {
        move |d| {
            let result = d.get("chatOpacity").as_string_result().map(|s| {
                let bg = self.calculate_background(&s);
                d.set(
                    "textBackgroundOpacity",
                    d.create_string(&java_double_to_string(bg)),
                )
            });
            result.unwrap_or(d)
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsAddTextBackgroundFix#calculateBackground
    fn calculate_background(&self, value: &str) -> f64 {
        match java_parse_double(value) {
            Ok(v) => {
                let v1 = 0.9_f64 * v + 0.1_f64;
                v1 / 2.0_f64
            }
            Err(_) => 0.5_f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::of_map(m)
    }

    #[test]
    fn chat_opacity_1_produces_0_5() {
        let fix = OptionsAddTextBackgroundFix::new((), false);
        let rule = fix.make_rule();
        let input = map(vec![("chatOpacity", Dynamic::of_string("1.0"))]);
        let out = rule(input);
        assert_eq!(
            out.get("textBackgroundOpacity").as_string_result().as_deref(),
            Some("0.5")
        );
    }

    #[test]
    fn chat_opacity_0_produces_0_05() {
        let fix = OptionsAddTextBackgroundFix::new((), false);
        let rule = fix.make_rule();
        let input = map(vec![("chatOpacity", Dynamic::of_string("0.0"))]);
        let out = rule(input);
        assert_eq!(
            out.get("textBackgroundOpacity").as_string_result().as_deref(),
            Some("0.05")
        );
    }

    #[test]
    fn missing_chat_opacity_unchanged() {
        let fix = OptionsAddTextBackgroundFix::new((), false);
        let rule = fix.make_rule();
        let input = map(vec![("someOtherOption", Dynamic::of_string("true"))]);
        let out = rule(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn invalid_number_defaults_to_0_5() {
        let fix = OptionsAddTextBackgroundFix::new((), false);
        let rule = fix.make_rule();
        let input = map(vec![("chatOpacity", Dynamic::of_string("not_a_number"))]);
        let out = rule(input);
        assert_eq!(
            out.get("textBackgroundOpacity").as_string_result().as_deref(),
            Some("0.5")
        );
    }

    #[test]
    fn chat_opacity_half_produces_0_275() {
        let fix = OptionsAddTextBackgroundFix::new((), false);
        let rule = fix.make_rule();
        let input = map(vec![("chatOpacity", Dynamic::of_string("0.5"))]);
        let out = rule(input);
        assert_eq!(
            out.get("textBackgroundOpacity").as_string_result().as_deref(),
            Some("0.275")
        );
    }

    #[test]
    fn calculate_background_nan_input_returns_nan() {
        let fix = OptionsAddTextBackgroundFix::new((), false);
        let bg = fix.calculate_background("NaN");
        assert!(bg.is_nan());
    }
}
