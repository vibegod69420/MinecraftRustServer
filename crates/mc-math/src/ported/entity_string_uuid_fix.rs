use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Missing,
    String(String),
    Long(i64),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Missing),
            _ => Dynamic::Missing,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Dynamic::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn create_long(&self, value: i64) -> Dynamic {
        Dynamic::Long(value)
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                out.remove(key);
                Dynamic::Map(out)
            }
            _ => self.clone(),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                out.insert(key.to_string(), value);
                Dynamic::Map(out)
            }
            _ => self.clone(),
        }
    }
}

fn parse_uuid_most_least(s: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5
        || parts[0].len() != 8
        || parts[1].len() != 4
        || parts[2].len() != 4
        || parts[3].len() != 4
        || parts[4].len() != 12
    {
        return None;
    }
    let hex = parts.concat();
    if hex.len() != 32 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let most = u64::from_str_radix(&hex[0..16], 16).ok()? as i64;
    let least = u64::from_str_radix(&hex[16..32], 16).ok()? as i64;
    Some((most, least))
}

pub struct EntityStringUuidFix {
    _changes_type: bool,
}

impl EntityStringUuidFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityStringUuidFix#EntityStringUuidFix
    pub fn new(_output_schema: (), changes_type: bool) -> Self {
        Self {
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityStringUuidFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic {
        |entity| {
            if let Some(uuid_str) = entity.get("UUID").as_string() {
                let (most, least) =
                    parse_uuid_most_least(&uuid_str).expect("invalid UUID string in EntityStringUuidFix");
                return entity
                    .remove("UUID")
                    .set("UUIDMost", entity.create_long(most))
                    .set("UUIDLeast", entity.create_long(least));
            }
            entity
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: &[(&str, Dynamic)]) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        Dynamic::Map(m)
    }

    #[test]
    fn converts_uuid_string_to_most_least() {
        let fix = EntityStringUuidFix::new((), true);
        let input = map(&[(
            "UUID",
            Dynamic::String("123e4567-e89b-12d3-a456-426614174000".to_string()),
        )]);
        let out = fix.make_rule()(input);
        assert_eq!(out.get("UUID"), Dynamic::Missing);
        assert_eq!(
            out.get("UUIDMost"),
            Dynamic::Long(0x123e4567_e89b_12d3_u64 as i64)
        );
        assert_eq!(
            out.get("UUIDLeast"),
            Dynamic::Long(0xa456_4266_1417_4000_u64 as i64)
        );
    }

    #[test]
    fn missing_uuid_returns_unchanged() {
        let fix = EntityStringUuidFix::new((), true);
        let input = map(&[("id", Dynamic::String("minecraft:pig".to_string()))]);
        let out = fix.make_rule()(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn non_string_uuid_returns_unchanged() {
        let fix = EntityStringUuidFix::new((), true);
        let input = map(&[("UUID", Dynamic::Long(123))]);
        let out = fix.make_rule()(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn negative_uuid_bits_overflow_representation() {
        let fix = EntityStringUuidFix::new((), false);
        let input = map(&[(
            "UUID",
            Dynamic::String("ffffffff-ffff-ffff-ffff-ffffffffffff".to_string()),
        )]);
        let out = fix.make_rule()(input);
        assert_eq!(out.get("UUIDMost"), Dynamic::Long(-1_i64));
        assert_eq!(out.get("UUIDLeast"), Dynamic::Long(-1_i64));
    }

    #[test]
    fn zero_uuid() {
        let fix = EntityStringUuidFix::new((), true);
        let input = map(&[(
            "UUID",
            Dynamic::String("00000000-0000-0000-0000-000000000000".to_string()),
        )]);
        let out = fix.make_rule()(input);
        assert_eq!(out.get("UUIDMost"), Dynamic::Long(0));
        assert_eq!(out.get("UUIDLeast"), Dynamic::Long(0));
    }
}
