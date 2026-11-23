use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    List(Vec<Dynamic>),
}

#[derive(Clone, Debug)]
pub struct DynamicOpt(Option<Dynamic>);

impl DynamicOpt {
    pub fn as_string(&self, default: &str) -> String {
        self.0
            .as_ref()
            .map(|d| d.as_string(default))
            .unwrap_or_else(|| default.to_string())
    }

    pub fn as_float(&self, default: f32) -> f32 {
        self.0
            .as_ref()
            .map(|d| d.as_float(default))
            .unwrap_or(default)
    }
}

impl Dynamic {
    pub fn get(&self, key: &str) -> DynamicOpt {
        match self {
            Dynamic::Map(m) => DynamicOpt(m.get(key).cloned()),
            _ => DynamicOpt(None),
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn as_float(&self, default: f32) -> f32 {
        match self {
            Dynamic::Float(v) => *v,
            Dynamic::Double(v) => *v as f32,
            Dynamic::Int(v) => *v as f32,
            Dynamic::Long(v) => *v as f32,
            _ => default,
        }
    }

    pub fn create_string(&self, s: String) -> Dynamic {
        Dynamic::String(s)
    }

    pub fn create_float(&self, v: f32) -> Dynamic {
        Dynamic::Float(v)
    }

    pub fn set(self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn remove(self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.remove(key);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, key: &str, f: F) -> Dynamic {
        match &self {
            Dynamic::Map(m) if m.contains_key(key) => {
                let val = m.get(key).cloned().unwrap();
                self.set(key, f(val))
            }
            _ => self,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct PlayerRespawnDataFix {
    _schema: Schema,
}

impl PlayerRespawnDataFix {
    // JAVA: net.minecraft.util.datafix.fixes.PlayerRespawnDataFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.PlayerRespawnDataFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic {
        |player: Dynamic| {
            player.update("respawn", |respawn| {
                let dimension = respawn.get("dimension").as_string("minecraft:overworld");
                let angle = respawn.get("angle").as_float(0.0_f32);
                let dimension_val = respawn.create_string(dimension);
                let yaw_val = respawn.create_float(angle);
                let pitch_val = respawn.create_float(0.0_f32);
                respawn
                    .set("dimension", dimension_val)
                    .set("yaw", yaw_val)
                    .set("pitch", pitch_val)
                    .remove("angle")
            })
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
    fn converts_respawn_angle_to_yaw_pitch() {
        let fix = PlayerRespawnDataFix::new(Schema);
        let input = map(vec![(
            "respawn",
            map(vec![
                ("dimension", Dynamic::String("minecraft:the_nether".to_string())),
                ("angle", Dynamic::Float(90.0_f32)),
            ]),
        )]);
        let out = (fix.make_rule())(input);
        let respawn = match out {
            Dynamic::Map(ref m) => m.get("respawn").cloned().unwrap(),
            _ => panic!("expected map"),
        };
        if let Dynamic::Map(m) = respawn {
            assert_eq!(m.get("dimension"), Some(&Dynamic::String("minecraft:the_nether".to_string())));
            assert_eq!(m.get("yaw"), Some(&Dynamic::Float(90.0_f32)));
            assert_eq!(m.get("pitch"), Some(&Dynamic::Float(0.0_f32)));
            assert!(!m.contains_key("angle"));
        } else {
            panic!("expected respawn map");
        }
    }

    #[test]
    fn defaults_missing_dimension_and_angle() {
        let fix = PlayerRespawnDataFix::new(Schema);
        let input = map(vec![("respawn", map(vec![]))]);
        let out = (fix.make_rule())(input);
        let respawn = match out {
            Dynamic::Map(ref m) => m.get("respawn").cloned().unwrap(),
            _ => panic!("expected map"),
        };
        if let Dynamic::Map(m) = respawn {
            assert_eq!(m.get("dimension"), Some(&Dynamic::String("minecraft:overworld".to_string())));
            assert_eq!(m.get("yaw"), Some(&Dynamic::Float(0.0_f32)));
            assert_eq!(m.get("pitch"), Some(&Dynamic::Float(0.0_f32)));
        } else {
            panic!("expected respawn map");
        }
    }

    #[test]
    fn preserves_float_bits_for_nan_and_negative() {
        let fix = PlayerRespawnDataFix::new(Schema);
        let nan_bits = f32::NAN.to_bits();
        let neg = -123.5_f32;
        let input = map(vec![(
            "respawn",
            map(vec![("angle", Dynamic::Float(f32::from_bits(nan_bits)))]),
        )]);
        let out = (fix.make_rule())(input);
        let yaw = match out {
            Dynamic::Map(m) => match m.get("respawn").unwrap() {
                Dynamic::Map(r) => match r.get("yaw").unwrap() {
                    Dynamic::Float(v) => *v,
                    _ => panic!("yaw not float"),
                },
                _ => panic!("respawn not map"),
            },
            _ => panic!("not map"),
        };
        assert_eq!(yaw.to_bits(), nan_bits);

        let input2 = map(vec![("respawn", map(vec![("angle", Dynamic::Float(neg))]))]);
        let out2 = (fix.make_rule())(input2);
        if let Dynamic::Map(m) = out2 {
            if let Dynamic::Map(r) = m.get("respawn").unwrap() {
                assert_eq!(r.get("yaw"), Some(&Dynamic::Float(neg)));
            }
        }
    }

    #[test]
    fn no_respawn_field_is_unchanged() {
        let fix = PlayerRespawnDataFix::new(Schema);
        let input = map(vec![("Name", Dynamic::String("A".to_string()))]);
        let out = (fix.make_rule())(input.clone());
        assert_eq!(out, input);
    }
}
