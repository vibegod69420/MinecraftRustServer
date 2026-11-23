//! Port of `net.minecraft.util.datafix.fixes.SpawnerDataFix`.

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Null,
    Int(i32),
    String(String),
    Map(Vec<(String, Dynamic)>),
    List(Vec<Dynamic>),
    /// DFU-style pair representation used by typed spawner data.
    Pair(Box<Dynamic>, Box<Dynamic>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynValue,
}

impl Dynamic {
    pub fn empty() -> Self {
        Self {
            value: DynValue::Map(Vec::new()),
        }
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        match &self.value {
            DynValue::Map(entries) => entries
                .iter()
                .find(|(k, _)| k.as_str() == key)
                .map(|(_, v)| OptionalDynamic {
                    inner: Some(v.clone()),
                })
                .unwrap_or(OptionalDynamic { inner: None }),
            _ => OptionalDynamic { inner: None },
        }
    }

    pub fn set(&self, key: &str, val: Dynamic) -> Self {
        match &self.value {
            DynValue::Map(entries) => {
                let mut out: Vec<(String, Dynamic)> = Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k.as_str() == key {
                        out.push((k.clone(), val.clone()));
                        replaced = true;
                    } else {
                        out.push((k.clone(), v.clone()));
                    }
                }
                if !replaced {
                    out.push((key.to_string(), val));
                }
                Self {
                    value: DynValue::Map(out),
                }
            }
            _ => self.clone(),
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

    pub fn as_int(&self, default: i32) -> i32 {
        match &self.value {
            DynValue::Int(v) => *v,
            _ => default,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionalDynamic {
    inner: Option<Dynamic>,
}

impl OptionalDynamic {
    pub fn result(&self) -> Option<Dynamic> {
        self.inner.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

pub struct SpawnerDataFix;

impl SpawnerDataFix {
    // JAVA: net.minecraft.util.datafix.fixes.SpawnerDataFix#SpawnerDataFix
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.SpawnerDataFix#makeRule
    pub fn fix_spawner(&self, spawner: Dynamic) -> Dynamic {
        let spawner = if let Some(spawn_data) = spawner.get("SpawnData").result() {
            let wrapped = self.wrap_entity_to_spawn_data(Typed { value: spawn_data });
            spawner.set("SpawnData", wrapped.value)
        } else {
            spawner
        };
        if let Some(spawn_potentials) = spawner.get("SpawnPotentials").result() {
            let wrapped =
                self.wrap_spawn_potentials_to_weighted_entries(Typed { value: spawn_potentials });
            spawner.set("SpawnPotentials", wrapped.value)
        } else {
            spawner
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.SpawnerDataFix#wrapEntityToSpawnData
    fn wrap_entity_to_spawn_data(&self, typed: Typed) -> Typed {
        Typed {
            value: Dynamic {
                value: DynValue::Pair(Box::new(typed.value), Box::new(Dynamic::empty())),
            },
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.SpawnerDataFix#wrapSpawnPotentialsToWeightedEntries
    fn wrap_spawn_potentials_to_weighted_entries(&self, typed: Typed) -> Typed {
        let items = match &typed.value.value {
            DynValue::List(items) => items.clone(),
            _ => Vec::new(),
        };
        let out: Vec<Dynamic> = items
            .into_iter()
            .map(|entry| {
                let (first, second) = match &entry.value {
                    DynValue::Pair(a, b) => (a.as_ref().clone(), b.as_ref().clone()),
                    _ => (entry.clone(), Dynamic::empty()),
                };
                let weight = second
                    .get("Weight")
                    .result()
                    .map(|d| d.as_int(1))
                    .unwrap_or(1);
                let mut weight_dyn = Dynamic::empty();
                weight_dyn = weight_dyn.set(
                    "weight",
                    Dynamic {
                        value: DynValue::Int(weight),
                    },
                );
                let entity_data = second.remove("Weight").remove("Entity");
                Dynamic {
                    value: DynValue::Pair(
                        Box::new(Dynamic {
                            value: DynValue::Pair(Box::new(first), Box::new(entity_data)),
                        }),
                        Box::new(weight_dyn),
                    ),
                }
            })
            .collect();
        Typed {
            value: Dynamic {
                value: DynValue::List(out),
            },
        }
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

    fn int_val(v: i32) -> Dynamic {
        Dynamic {
            value: DynValue::Int(v),
        }
    }

    fn str_val(s: &str) -> Dynamic {
        Dynamic {
            value: DynValue::String(s.to_string()),
        }
    }

    fn pair(a: Dynamic, b: Dynamic) -> Dynamic {
        Dynamic {
            value: DynValue::Pair(Box::new(a), Box::new(b)),
        }
    }

    fn list(items: Vec<Dynamic>) -> Dynamic {
        Dynamic {
            value: DynValue::List(items),
        }
    }

    #[test]
    fn wraps_spawn_data_entity() {
        let fix = SpawnerDataFix::new(());
        let entity = map(vec![("id", str_val("minecraft:pig"))]);
        let spawner = map(vec![("SpawnData", entity.clone())]);
        let out = fix.fix_spawner(spawner);
        let sd = out.get("SpawnData").result().unwrap();
        match &sd.value {
            DynValue::Pair(e, extra) => {
                assert_eq!(**e, entity);
                assert!(matches!(&extra.value, DynValue::Map(m) if m.is_empty()));
            }
            _ => panic!("expected pair"),
        }
    }

    #[test]
    fn wraps_spawn_potentials() {
        let fix = SpawnerDataFix::new(());
        let entity = map(vec![("id", str_val("minecraft:zombie"))]);
        let entry_dyn = map(vec![
            ("Weight", int_val(5)),
            ("Entity", str_val("ignored")),
            ("custom", int_val(1)),
        ]);
        let entry = pair(entity.clone(), entry_dyn);
        let spawner = map(vec![("SpawnPotentials", list(vec![entry]))]);
        let out = fix.fix_spawner(spawner);
        let pots = out.get("SpawnPotentials").result().unwrap();
        let items = match &pots.value {
            DynValue::List(v) => v,
            _ => panic!("expected list"),
        };
        assert_eq!(items.len(), 1);
        match &items[0].value {
            DynValue::Pair(inner, weight_dyn) => {
                match &inner.value {
                    DynValue::Pair(e, data) => {
                        assert_eq!(**e, entity);
                        assert!(data.get("Weight").result().is_none());
                        assert!(data.get("Entity").result().is_none());
                        assert_eq!(data.get("custom").result().unwrap().as_int(0), 1);
                    }
                    _ => panic!("expected inner pair"),
                }
                assert_eq!(weight_dyn.get("weight").result().unwrap().as_int(0), 5);
            }
            _ => panic!("expected pair"),
        }
    }

    #[test]
    fn spawn_potential_default_weight() {
        let fix = SpawnerDataFix::new(());
        let entity = map(vec![("id", str_val("minecraft:pig"))]);
        let entry = pair(entity, map(vec![]));
        let spawner = map(vec![("SpawnPotentials", list(vec![entry]))]);
        let out = fix.fix_spawner(spawner);
        let pots = out.get("SpawnPotentials").result().unwrap();
        let items = match &pots.value {
            DynValue::List(v) => v,
            _ => panic!("expected list"),
        };
        match &items[0].value {
            DynValue::Pair(_, weight_dyn) => {
                assert_eq!(weight_dyn.get("weight").result().unwrap().as_int(0), 1);
            }
            _ => panic!("expected pair"),
        }
    }

    #[test]
    fn missing_fields_unchanged() {
        let fix = SpawnerDataFix::new(());
        let spawner = map(vec![("Delay", int_val(20))]);
        let out = fix.fix_spawner(spawner);
        assert_eq!(out.get("Delay").result().unwrap().as_int(0), 20);
        assert!(out.get("SpawnData").result().is_none());
        assert!(out.get("SpawnPotentials").result().is_none());
    }
}
