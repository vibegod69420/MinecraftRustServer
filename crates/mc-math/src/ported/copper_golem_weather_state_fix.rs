use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix transformations.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Null,
}

impl Dynamic {
    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v as i32,
            _ => default,
        }
    }

    pub fn create_string(&self, s: &str) -> Self {
        Dynamic::String(s.to_string())
    }

    pub fn update(mut self, key: &str, f: impl FnOnce(Dynamic) -> Dynamic) -> Self {
        if let Dynamic::Map(ref mut m) = self {
            if let Some(v) = m.remove(key) {
                m.insert(key.to_string(), f(v));
            }
        }
        self
    }
}

pub struct CopperGolemWeatherStateFix;

impl CopperGolemWeatherStateFix {
    // JAVA: net.minecraft.util.datafix.fixes.CopperGolemWeatherStateFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.CopperGolemWeatherStateFix#fix
    pub fn fix(&self, typed: Dynamic) -> Dynamic {
        typed.update("weather_state", Self::fix_weather_state)
    }

    // JAVA: net.minecraft.util.datafix.fixes.CopperGolemWeatherStateFix#fixWeatherState
    fn fix_weather_state(d: Dynamic) -> Dynamic {
        match d.as_int(0) {
            1 => d.create_string("exposed"),
            2 => d.create_string("weathered"),
            3 => d.create_string("oxidized"),
            _ => d.create_string("unaffected"),
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
        Dynamic::Map(m)
    }

    #[test]
    fn maps_all_weather_states() {
        let fix = CopperGolemWeatherStateFix::new(());
        let cases = vec![
            (0, "unaffected"),
            (1, "exposed"),
            (2, "weathered"),
            (3, "oxidized"),
        ];
        for (input_val, expected_str) in cases {
            let input = map(vec![("weather_state", Dynamic::Int(input_val))]);
            let out = fix.fix(input);
            assert_eq!(
                out,
                map(vec![("weather_state", Dynamic::String(expected_str.to_string()))])
            );
        }
    }

    #[test]
    fn unknown_and_negative_values_default_to_unaffected() {
        let fix = CopperGolemWeatherStateFix::new(());
        for val in [-1i64, 4, 100, i32::MIN as i64, i32::MAX as i64] {
            let input = map(vec![("weather_state", Dynamic::Int(val))]);
            let out = fix.fix(input);
            assert_eq!(
                out,
                map(vec![("weather_state", Dynamic::String("unaffected".to_string()))])
            );
        }
    }

    #[test]
    fn missing_weather_state_is_unchanged() {
        let fix = CopperGolemWeatherStateFix::new(());
        let input = map(vec![("id", Dynamic::String("minecraft:copper_golem".to_string()))]);
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn non_int_weather_state_uses_default() {
        let fix = CopperGolemWeatherStateFix::new(());
        let input = map(vec![("weather_state", Dynamic::String("something".to_string()))]);
        let out = fix.fix(input);
        assert_eq!(
            out,
            map(vec![("weather_state", Dynamic::String("unaffected".to_string()))])
        );
    }

    #[test]
    fn preserves_other_fields() {
        let fix = CopperGolemWeatherStateFix::new(());
        let input = map(vec![
            ("id", Dynamic::String("minecraft:copper_golem".to_string())),
            ("weather_state", Dynamic::Int(2)),
            ("Age", Dynamic::Int(-10)),
        ]);
        let out = fix.fix(input);
        assert_eq!(
            out,
            map(vec![
                ("id", Dynamic::String("minecraft:copper_golem".to_string())),
                ("weather_state", Dynamic::String("weathered".to_string())),
                ("Age", Dynamic::Int(-10)),
            ])
        );
    }
}
