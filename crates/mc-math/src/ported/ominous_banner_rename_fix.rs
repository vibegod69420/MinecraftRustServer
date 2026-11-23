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

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(&self, key: &str, f: F) -> Dynamic {
        match &self.value {
            DynamicValue::Map(m) => {
                if let Some(child) = m.get(key) {
                    let updated = f(child.clone());
                    self.set(key, updated)
                } else {
                    self.clone()
                }
            }
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

pub struct OminousBannerRenameFix {
    pub target_item: String,
}

impl OminousBannerRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.OminousBannerRenameFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self {
            target_item: "minecraft:white_banner".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OminousBannerRenameFix#fixItemStackTag
    fn fix_item_stack_tag_dynamic(data: &Dynamic) -> Dynamic {
        data.update("display", |display| {
            display.update("Name", |name_dyn| {
                let name = name_dyn.as_string_result();
                if let Some(s) = name {
                    name_dyn.create_string(&s.replace(
                        "\"translate\":\"block.minecraft.illager_banner\"",
                        "\"translate\":\"block.minecraft.ominous_banner\"",
                    ))
                } else {
                    name_dyn
                }
            })
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.OminousBannerRenameFix#fixItemStackTag
    pub fn fix_item_stack_tag(&self, typed: Typed) -> Typed {
        Typed {
            value: Self::fix_item_stack_tag_dynamic(&typed.value),
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
    fn renames_illager_banner_to_ominous_banner() {
        let input = map(vec![(
            "display",
            map(vec![(
                "Name",
                Dynamic::of_string("{\"translate\":\"block.minecraft.illager_banner\"}"),
            )]),
        )]);
        let out = OminousBannerRenameFix::fix_item_stack_tag_dynamic(&input);
        let name = out.get("display").get("Name").as_string_result().unwrap();
        assert_eq!(
            name,
            "{\"translate\":\"block.minecraft.ominous_banner\"}"
        );
    }

    #[test]
    fn no_display_unchanged() {
        let input = map(vec![("Count", Dynamic::of_string("1"))]);
        let out = OminousBannerRenameFix::fix_item_stack_tag_dynamic(&input);
        assert_eq!(out, input);
    }

    #[test]
    fn name_without_translation_unchanged() {
        let input = map(vec![(
            "display",
            map(vec![(
                "Name",
                Dynamic::of_string("{\"text\":\"Hello\"}"),
            )]),
        )]);
        let out = OminousBannerRenameFix::fix_item_stack_tag_dynamic(&input);
        assert_eq!(out, input);
    }

    #[test]
    fn missing_name_in_display_unchanged() {
        let input = map(vec![(
            "display",
            map(vec![("Lore", Dynamic::of_string("[]"))]),
        )]);
        let out = OminousBannerRenameFix::fix_item_stack_tag_dynamic(&input);
        assert_eq!(out, input);
    }

    #[test]
    fn typed_wrapper_applies_fix() {
        let input = Typed {
            value: map(vec![(
                "display",
                map(vec![(
                    "Name",
                    Dynamic::of_string("{\"translate\":\"block.minecraft.illager_banner\"}"),
                )]),
            )]),
        };
        let fix = OminousBannerRenameFix::new(());
        let out = fix.fix_item_stack_tag(input);
        let name = out
            .value
            .get("display")
            .get("Name")
            .as_string_result()
            .unwrap();
        assert_eq!(name, "{\"translate\":\"block.minecraft.ominous_banner\"}");
    }
}
