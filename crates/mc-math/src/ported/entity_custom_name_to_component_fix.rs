//! Port of `net.minecraft.util.datafix.fixes.EntityCustomNameToComponentFix`.

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Null,
    String(String),
    Map(Vec<(String, Dynamic)>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynValue,
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<Dynamic> {
        match &self.value {
            DynValue::Map(entries) => entries
                .iter()
                .find(|(k, _)| k.as_str() == key)
                .map(|(_, v)| v.clone()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match &self.value {
            DynValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn remove(&self, key: &str) -> Self {
        match &self.value {
            DynValue::Map(entries) => Self {
                value: DynValue::Map(
                    entries
                        .iter()
                        .filter(|(k, _)| k.as_str() != key)
                        .cloned()
                        .collect(),
                ),
            },
            _ => self.clone(),
        }
    }

    pub fn set(&self, key: &str, val: Dynamic) -> Self {
        match &self.value {
            DynValue::Map(entries) => {
                let mut new_entries: Vec<(String, Dynamic)> = Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k.as_str() == key {
                        new_entries.push((k.clone(), val.clone()));
                        replaced = true;
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                if !replaced {
                    new_entries.push((key.to_string(), val));
                }
                Self {
                    value: DynValue::Map(new_entries),
                }
            }
            _ => self.clone(),
        }
    }
}

pub struct EntityCustomNameToComponentFix;

impl EntityCustomNameToComponentFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityCustomNameToComponentFix#EntityCustomNameToComponentFix
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityCustomNameToComponentFix#makeRule
    pub fn fix(&self, entity: Dynamic) -> Dynamic {
        Self::fix_entity(entity)
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityCustomNameToComponentFix#fixEntity
    fn fix_entity(entity: Dynamic) -> Dynamic {
        let custom_name = entity.get("CustomName").and_then(|d| d.as_string());
        let Some(name) = custom_name else {
            return entity;
        };
        if name.is_empty() {
            return entity.remove("CustomName");
        }
        let id = entity
            .get("id")
            .and_then(|d| d.as_string())
            .unwrap_or_default();
        let fixed = Self::fix_custom_name(&name, &id);
        entity.set("CustomName", fixed)
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityCustomNameToComponentFix#fixCustomName
    fn fix_custom_name(name: &str, entity_id: &str) -> Dynamic {
        if entity_id == "minecraft:commandblock_minecart" {
            Dynamic {
                value: DynValue::String(name.to_string()),
            }
        } else {
            create_plain_text_component(name)
        }
    }
}

/// Minimal port of `LegacyComponentDataFixUtils.createPlainTextComponent` behavior.
fn create_plain_text_component(text: &str) -> Dynamic {
    Dynamic {
        value: DynValue::Map(vec![(
            "text".to_string(),
            Dynamic {
                value: DynValue::String(text.to_string()),
            },
        )]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic {
            value: DynValue::Map(
                entries
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
            ),
        }
    }

    fn str_val(s: &str) -> Dynamic {
        Dynamic {
            value: DynValue::String(s.to_string()),
        }
    }

    #[test]
    fn missing_custom_name_unchanged() {
        let fix = EntityCustomNameToComponentFix::new(());
        let entity = map(vec![("id", str_val("minecraft:pig"))]);
        let out = fix.fix(entity.clone());
        assert_eq!(out, entity);
    }

    #[test]
    fn empty_custom_name_removed() {
        let fix = EntityCustomNameToComponentFix::new(());
        let entity = map(vec![
            ("id", str_val("minecraft:pig")),
            ("CustomName", str_val("")),
        ]);
        let out = fix.fix(entity);
        assert!(out.get("CustomName").is_none());
    }

    #[test]
    fn normal_entity_becomes_text_component() {
        let fix = EntityCustomNameToComponentFix::new(());
        let entity = map(vec![
            ("id", str_val("minecraft:zombie")),
            ("CustomName", str_val("Bob")),
        ]);
        let out = fix.fix(entity);
        let cn = out.get("CustomName").unwrap();
        assert_eq!(
            cn.get("text").and_then(|d| d.as_string()),
            Some("Bob".to_string())
        );
    }

    #[test]
    fn commandblock_minecart_keeps_string() {
        let fix = EntityCustomNameToComponentFix::new(());
        let entity = map(vec![
            ("id", str_val("minecraft:commandblock_minecart")),
            ("CustomName", str_val("cmd")),
        ]);
        let out = fix.fix(entity);
        let cn = out.get("CustomName").unwrap();
        assert_eq!(cn.as_string(), Some("cmd".to_string()));
    }

    #[test]
    fn missing_id_treated_as_non_commandblock() {
        let fix = EntityCustomNameToComponentFix::new(());
        let entity = map(vec![("CustomName", str_val("X"))]);
        let out = fix.fix(entity);
        let cn = out.get("CustomName").unwrap();
        assert_eq!(
            cn.get("text").and_then(|d| d.as_string()),
            Some("X".to_string())
        );
    }
}
