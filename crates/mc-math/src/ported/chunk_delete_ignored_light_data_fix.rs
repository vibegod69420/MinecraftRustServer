use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Missing,
    Bool(bool),
    Byte(i8),
    Int(i32),
    Long(i64),
    String(String),
    List(Vec<Dynamic>),
    Compound(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Compound(m) => m.get(key).cloned().unwrap_or(Dynamic::Missing),
            _ => Dynamic::Missing,
        }
    }

    pub fn as_bool(&self, default: bool) -> bool {
        match self {
            Dynamic::Bool(b) => *b,
            Dynamic::Byte(b) => *b != 0,
            _ => default,
        }
    }

    pub fn as_byte(&self, default: i8) -> i8 {
        match self {
            Dynamic::Byte(b) => *b,
            Dynamic::Int(i) => *i as i8,
            Dynamic::Long(l) => *l as i8,
            _ => default,
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn create_byte(&self, v: i8) -> Dynamic {
        Dynamic::Byte(v)
    }

    pub fn create_string(&self, v: &str) -> Dynamic {
        Dynamic::String(v.to_string())
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Compound(m) => {
                let mut m2 = m.clone();
                m2.insert(key.to_string(), value);
                Dynamic::Compound(m2)
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Compound(m) => {
                let mut m2 = m.clone();
                m2.remove(key);
                Dynamic::Compound(m2)
            }
            _ => self.clone(),
        }
    }
}

pub struct ChunkDeleteIgnoredLightDataFix;

impl ChunkDeleteIgnoredLightDataFix {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkDeleteIgnoredLightDataFix#makeRule
    pub fn fix_chunk(chunk: &Dynamic) -> Dynamic {
        let is_light_on = chunk.get("isLightOn").as_bool(false);
        if !is_light_on {
            match chunk.get("sections") {
                Dynamic::List(list) => {
                    let new_sections = list
                        .iter()
                        .map(|s| s.remove("BlockLight").remove("SkyLight"))
                        .collect();
                    chunk.set("sections", Dynamic::List(new_sections))
                }
                _ => chunk.clone(),
            }
        } else {
            chunk.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn compound(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Compound(m)
    }

    #[test]
    fn light_on_returns_chunk_unchanged() {
        let section = compound(vec![
            ("BlockLight", Dynamic::List(vec![])),
            ("SkyLight", Dynamic::List(vec![])),
            ("Y", Dynamic::Byte(0)),
        ]);
        let chunk = compound(vec![
            ("isLightOn", Dynamic::Bool(true)),
            ("sections", Dynamic::List(vec![section])),
        ]);
        let fixed = ChunkDeleteIgnoredLightDataFix::fix_chunk(&chunk);
        assert_eq!(fixed, chunk);
    }

    #[test]
    fn light_off_removes_section_light_data() {
        let section = compound(vec![
            ("BlockLight", Dynamic::List(vec![])),
            ("SkyLight", Dynamic::List(vec![])),
            ("Y", Dynamic::Byte(-1)),
        ]);
        let chunk = compound(vec![
            ("isLightOn", Dynamic::Bool(false)),
            ("sections", Dynamic::List(vec![section])),
        ]);
        let fixed = ChunkDeleteIgnoredLightDataFix::fix_chunk(&chunk);
        assert_eq!(fixed.get("isLightOn"), Dynamic::Bool(false));
        let sec = match fixed.get("sections") {
            Dynamic::List(v) => v[0].clone(),
            _ => panic!("expected sections list"),
        };
        assert_eq!(sec.get("BlockLight"), Dynamic::Missing);
        assert_eq!(sec.get("SkyLight"), Dynamic::Missing);
        assert_eq!(sec.get("Y"), Dynamic::Byte(-1));
    }

    #[test]
    fn missing_is_light_on_defaults_to_false() {
        let section = compound(vec![("BlockLight", Dynamic::List(vec![]))]);
        let chunk = compound(vec![("sections", Dynamic::List(vec![section]))]);
        let fixed = ChunkDeleteIgnoredLightDataFix::fix_chunk(&chunk);
        let sec = match fixed.get("sections") {
            Dynamic::List(v) => v[0].clone(),
            _ => panic!("expected sections list"),
        };
        assert_eq!(sec.get("BlockLight"), Dynamic::Missing);
    }

    #[test]
    fn byte_is_light_on_and_float_bits_preserved() {
        let bits = f32::to_bits(f32::NEG_INFINITY) as i32;
        let chunk_on = compound(vec![
            ("isLightOn", Dynamic::Byte(1)),
            ("marker", Dynamic::Int(bits)),
            ("sections", Dynamic::List(vec![compound(vec![("BlockLight", Dynamic::List(vec![]))])])),
        ]);
        let fixed_on = ChunkDeleteIgnoredLightDataFix::fix_chunk(&chunk_on);
        assert_eq!(fixed_on.get("marker"), Dynamic::Int(bits));
        let sec_on = match fixed_on.get("sections") {
            Dynamic::List(v) => v[0].clone(),
            _ => panic!("expected sections list"),
        };
        assert_eq!(sec_on.get("BlockLight"), Dynamic::List(vec![]));

        let chunk_off = compound(vec![
            ("isLightOn", Dynamic::Byte(0)),
            ("marker", Dynamic::Int(bits)),
            ("sections", Dynamic::List(vec![compound(vec![("BlockLight", Dynamic::List(vec![]))])])),
        ]);
        let fixed_off = ChunkDeleteIgnoredLightDataFix::fix_chunk(&chunk_off);
        assert_eq!(fixed_off.get("marker"), Dynamic::Int(bits));
        let sec_off = match fixed_off.get("sections") {
            Dynamic::List(v) => v[0].clone(),
            _ => panic!("expected sections list"),
        };
        assert_eq!(sec_off.get("BlockLight"), Dynamic::Missing);
    }

    #[test]
    fn preserves_negative_and_extreme_integer_fields() {
        let chunk = compound(vec![
            ("isLightOn", Dynamic::Bool(false)),
            ("xPos", Dynamic::Int(i32::MIN)),
            ("zPos", Dynamic::Int(-128)),
            ("seed", Dynamic::Long(i64::MIN)),
            ("sections", Dynamic::List(vec![])),
        ]);
        let fixed = ChunkDeleteIgnoredLightDataFix::fix_chunk(&chunk);
        assert_eq!(fixed.get("xPos"), Dynamic::Int(i32::MIN));
        assert_eq!(fixed.get("zPos"), Dynamic::Int(-128));
        assert_eq!(fixed.get("seed"), Dynamic::Long(i64::MIN));
    }
}
