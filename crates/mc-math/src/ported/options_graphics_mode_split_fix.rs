#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Null,
    Str(String),
    Map(Vec<(String, Dynamic)>),
}

static NULL_DYNAMIC: Dynamic = Dynamic::Null;

impl Dynamic {
    pub fn get(&self, key: &str) -> &Dynamic {
        match self {
            Dynamic::Map(entries) => entries
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .unwrap_or(&NULL_DYNAMIC),
            _ => &NULL_DYNAMIC,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(entries) => {
                let mut new_entries = Vec::with_capacity(entries.len() + 1);
                let mut found = false;
                for (k, v) in entries {
                    if k == key {
                        new_entries.push((k.clone(), value.clone()));
                        found = true;
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                if !found {
                    new_entries.push((key.to_string(), value));
                }
                Dynamic::Map(new_entries)
            }
            _ => self.clone(),
        }
    }

    pub fn create_string(s: &str) -> Dynamic {
        Dynamic::Str(s.to_string())
    }

    pub fn empty_map() -> Dynamic {
        Dynamic::Map(Vec::new())
    }
}

pub struct OptionsGraphicsModeSplitFix {
    new_field_name: String,
    value_if_fast: String,
    value_if_fancy: String,
    value_if_fabulous: String,
}

impl OptionsGraphicsModeSplitFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsGraphicsModeSplitFix#<init>
    pub fn new(
        new_field_name: &str,
        value_if_fast: &str,
        value_if_fancy: &str,
        value_if_fabulous: &str,
    ) -> Self {
        Self {
            new_field_name: new_field_name.to_string(),
            value_if_fast: value_if_fast.to_string(),
            value_if_fancy: value_if_fancy.to_string(),
            value_if_fabulous: value_if_fabulous.to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsGraphicsModeSplitFix#getValue
    fn get_value(&self, value: &str) -> String {
        match value {
            "2" => self.value_if_fabulous.clone(),
            "0" => self.value_if_fast.clone(),
            _ => self.value_if_fancy.clone(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsGraphicsModeSplitFix#makeRule
    pub fn fix(&self, input: &Dynamic) -> Dynamic {
        let new_value = match input.get("graphicsMode").as_string() {
            Some(mode) => self.get_value(mode),
            None => self.value_if_fancy.clone(),
        };
        input.set(&self.new_field_name, Dynamic::create_string(&new_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fix() -> OptionsGraphicsModeSplitFix {
        OptionsGraphicsModeSplitFix::new("graphicsMode", "fast", "fancy", "fabulous")
    }

    #[test]
    fn test_fabulous() {
        let fix = make_fix();
        let input = Dynamic::empty_map().set("graphicsMode", Dynamic::create_string("2"));
        let out = fix.fix(&input);
        assert_eq!(out.get("graphicsMode").as_string(), Some("fabulous"));
    }

    #[test]
    fn test_fast() {
        let fix = make_fix();
        let input = Dynamic::empty_map().set("graphicsMode", Dynamic::create_string("0"));
        let out = fix.fix(&input);
        assert_eq!(out.get("graphicsMode").as_string(), Some("fast"));
    }

    #[test]
    fn test_fancy_default_for_one() {
        let fix = make_fix();
        let input = Dynamic::empty_map().set("graphicsMode", Dynamic::create_string("1"));
        let out = fix.fix(&input);
        assert_eq!(out.get("graphicsMode").as_string(), Some("fancy"));
    }

    #[test]
    fn test_missing_graphics_mode_defaults_fancy() {
        let fix = make_fix();
        let input = Dynamic::empty_map().set("other", Dynamic::create_string("x"));
        let out = fix.fix(&input);
        assert_eq!(out.get("graphicsMode").as_string(), Some("fancy"));
        assert_eq!(out.get("other").as_string(), Some("x"));
    }

    #[test]
    fn test_unknown_value_defaults_fancy() {
        let fix = make_fix();
        let input = Dynamic::empty_map().set("graphicsMode", Dynamic::create_string("99"));
        let out = fix.fix(&input);
        assert_eq!(out.get("graphicsMode").as_string(), Some("fancy"));
    }

    #[test]
    fn test_custom_field_name() {
        let fix = OptionsGraphicsModeSplitFix::new("renderMode", "low", "med", "high");
        let input = Dynamic::empty_map().set("graphicsMode", Dynamic::create_string("2"));
        let out = fix.fix(&input);
        assert_eq!(out.get("renderMode").as_string(), Some("high"));
    }
}
