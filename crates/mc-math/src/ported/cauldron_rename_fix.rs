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

    pub fn empty_map() -> Self {
        Self {
            value: DynamicValue::Map(BTreeMap::new()),
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

    pub fn as_string_or(&self, default: &str) -> String {
        self.as_string_result()
            .unwrap_or_else(|| default.to_owned())
    }

    pub fn or_else_empty_map(&self) -> Dynamic {
        match &self.value {
            DynamicValue::Map(_) => self.clone(),
            _ => Dynamic::empty_map(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match &self.value {
            DynamicValue::Map(m) => {
                let mut new_m = m.clone();
                new_m.remove(key);
                Dynamic::of_map(new_m)
            }
            _ => self.clone(),
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

pub struct CauldronRenameFix;

impl CauldronRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.CauldronRenameFix#<init>
    pub fn new(_schema: (), _changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.CauldronRenameFix#fix
    fn fix(data: &Dynamic) -> Dynamic {
        let name = data.get("Name").as_string_result();
        if name.as_deref() == Some("minecraft:cauldron") {
            let properties = data.get("Properties").or_else_empty_map();
            if properties.get("level").as_string_or("0") == "0" {
                data.remove("Properties")
            } else {
                data.set("Name", data.create_string("minecraft:water_cauldron"))
            }
        } else {
            data.clone()
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.CauldronRenameFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic {
        |d| Self::fix(&d)
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
    fn cauldron_level_0_removes_properties() {
        let input = map(vec![
            ("Name", Dynamic::of_string("minecraft:cauldron")),
            (
                "Properties",
                map(vec![("level", Dynamic::of_string("0"))]),
            ),
        ]);
        let out = CauldronRenameFix::fix(&input);
        let expected = map(vec![("Name", Dynamic::of_string("minecraft:cauldron"))]);
        assert_eq!(out, expected);
    }

    #[test]
    fn cauldron_level_1_becomes_water_cauldron() {
        let props = map(vec![("level", Dynamic::of_string("1"))]);
        let input = map(vec![
            ("Name", Dynamic::of_string("minecraft:cauldron")),
            ("Properties", props.clone()),
        ]);
        let out = CauldronRenameFix::fix(&input);
        assert_eq!(
            out.get("Name").as_string_result().as_deref(),
            Some("minecraft:water_cauldron")
        );
        assert_eq!(out.get("Properties"), props);
    }

    #[test]
    fn cauldron_missing_properties_treated_as_level_0() {
        let input = map(vec![("Name", Dynamic::of_string("minecraft:cauldron"))]);
        let out = CauldronRenameFix::fix(&input);
        assert_eq!(
            out.get("Name").as_string_result().as_deref(),
            Some("minecraft:cauldron")
        );
        assert_eq!(out.get("Properties"), Dynamic::null());
    }

    #[test]
    fn non_cauldron_unchanged() {
        let input = map(vec![
            ("Name", Dynamic::of_string("minecraft:stone")),
            (
                "Properties",
                map(vec![("level", Dynamic::of_string("1"))]),
            ),
        ]);
        let out = CauldronRenameFix::fix(&input);
        assert_eq!(out, input);
    }

    #[test]
    fn missing_name_unchanged() {
        let input = map(vec![(
            "Properties",
            map(vec![("level", Dynamic::of_string("1"))]),
        )]);
        let out = CauldronRenameFix::fix(&input);
        assert_eq!(out, input);
    }
}
