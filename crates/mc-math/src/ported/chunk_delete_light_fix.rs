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

pub struct ChunkDeleteLightFix;

impl ChunkDeleteLightFix {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkDeleteLightFix#makeRule
    pub fn fix_chunk(chunk: &Dynamic) -> Dynamic {
        let chunk = chunk.remove("isLightOn");
        match chunk.get("sections") {
            Dynamic::List(list) => {
                let new_sections = list
                    .iter()
                    .map(|s| s.remove("BlockLight").remove("SkyLight"))
                    .collect();
                chunk.set("sections", Dynamic::List(new_sections))
            }
            _ => chunk,
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
    fn removes_is_light_on_and_section_light() {
        let section = compound(vec![
            ("BlockLight", Dynamic::List(vec![Dynamic::Byte(0)])),
            ("SkyLight", Dynamic::List(vec![Dynamic::Byte(0)])),
            ("Y", Dynamic::Byte(0)),
        ]);
        let chunk = compound(vec![
            ("isLightOn", Dynamic::Bool(true)),
            ("sections", Dynamic::List(vec![section])),
            ("xPos", Dynamic::Int(1)),
        ]);
        let fixed = ChunkDeleteLightFix::fix_chunk(&chunk);
        assert_eq!(fixed.get("isLightOn"), Dynamic::Missing);
        assert_eq!(fixed.get("xPos"), Dynamic::Int(1));
        let sec = match fixed.get("sections") {
            Dynamic::List(v) => v[0].clone(),
            _ => panic!("expected sections list"),
        };
        assert_eq!(sec.get("BlockLight"), Dynamic::Missing);
        assert_eq!(sec.get("SkyLight"), Dynamic::Missing);
        assert_eq!(sec.get("Y"), Dynamic::Byte(0));
    }

    #[test]
    fn handles_missing_sections_and_missing_is_light_on() {
        let chunk = compound(vec![("xPos", Dynamic::Int(-3))]);
        let fixed = ChunkDeleteLightFix::fix_chunk(&chunk);
        assert_eq!(fixed.get("isLightOn"), Dynamic::Missing);
        assert_eq!(fixed.get("xPos"), Dynamic::Int(-3));
        assert_eq!(fixed.get("sections"), Dynamic::Missing);
    }

    #[test]
    fn preserves_float_bits_and_negative_values() {
        let fbits = f32::to_bits(f32::NAN) as i32;
        let dbits = f64::to_bits(-0.0_f64) as i64;
        let chunk = compound(vec![
            ("isLightOn", Dynamic::Byte(1)),
            ("xPos", Dynamic::Int(i32::MIN)),
            ("zPos", Dynamic::Int(-1)),
            ("floatBits", Dynamic::Int(fbits)),
            ("doubleBits", Dynamic::Long(dbits)),
            ("sections", Dynamic::List(vec![])),
        ]);
        let fixed = ChunkDeleteLightFix::fix_chunk(&chunk);
        assert_eq!(fixed.get("isLightOn"), Dynamic::Missing);
        assert_eq!(fixed.get("xPos"), Dynamic::Int(i32::MIN));
        assert_eq!(fixed.get("zPos"), Dynamic::Int(-1));
        assert_eq!(fixed.get("floatBits"), Dynamic::Int(fbits));
        assert_eq!(fixed.get("doubleBits"), Dynamic::Long(dbits));
    }

    #[test]
    fn updates_multiple_sections() {
        let s1 = compound(vec![("BlockLight", Dynamic::List(vec![])), ("Y", Dynamic::Byte(-4))]);
        let s2 = compound(vec![("SkyLight", Dynamic::List(vec![])), ("Y", Dynamic::Byte(20))]);
        let chunk = compound(vec![("sections", Dynamic::List(vec![s1, s2]))]);
        let fixed = ChunkDeleteLightFix::fix_chunk(&chunk);
        let sections = match fixed.get("sections") {
            Dynamic::List(v) => v,
            _ => panic!("expected sections list"),
        };
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].get("BlockLight"), Dynamic::Missing);
        assert_eq!(sections[0].get("Y"), Dynamic::Byte(-4));
        assert_eq!(sections[1].get("SkyLight"), Dynamic::Missing);
        assert_eq!(sections[1].get("Y"), Dynamic::Byte(20));
    }
}
