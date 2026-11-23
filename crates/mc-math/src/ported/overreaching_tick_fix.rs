//! Port of `net.minecraft.util.datafix.fixes.OverreachingTickFix`.

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Null,
    Int(i32),
    String(String),
    List(Vec<Dynamic>),
    Map(Vec<(String, Dynamic)>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynValue,
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Self {
            value: DynValue::Map(Vec::new()),
        }
    }

    pub fn create_list(&self, items: Vec<Dynamic>) -> Self {
        Self {
            value: DynValue::List(items),
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

    pub fn as_int(&self, default: i32) -> i32 {
        match &self.value {
            DynValue::Int(v) => *v,
            _ => default,
        }
    }

    pub fn as_stream(&self) -> Vec<Dynamic> {
        match &self.value {
            DynValue::List(items) => items.clone(),
            _ => Vec::new(),
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

    pub fn or_else_empty_map(&self) -> Dynamic {
        match &self.inner {
            Some(d) => d.clone(),
            None => Dynamic::empty_map(),
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match &self.inner {
            Some(d) => d.as_int(default),
            None => default,
        }
    }
}

pub struct OverreachingTickFix;

impl OverreachingTickFix {
    // JAVA: net.minecraft.util.datafix.fixes.OverreachingTickFix#OverreachingTickFix
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.OverreachingTickFix#makeRule
    pub fn fix_chunk(&self, chunk: Dynamic, block_ticks: Option<Dynamic>) -> Dynamic {
        let chunk_x = chunk.get("xPos").as_int(0);
        let chunk_z = chunk.get("zPos").as_int(0);
        let fluid_ticks = chunk.get("fluid_ticks").result();
        let chunk = Self::extract_overreaching_ticks(
            chunk,
            chunk_x,
            chunk_z,
            block_ticks,
            "neighbor_block_ticks",
        );
        Self::extract_overreaching_ticks(
            chunk,
            chunk_x,
            chunk_z,
            fluid_ticks,
            "neighbor_fluid_ticks",
        )
    }

    // JAVA: net.minecraft.util.datafix.fixes.OverreachingTickFix#extractOverreachingTicks
    fn extract_overreaching_ticks(
        mut chunk: Dynamic,
        chunk_x: i32,
        chunk_z: i32,
        ticks: Option<Dynamic>,
        target_key: &str,
    ) -> Dynamic {
        if let Some(ticks_dyn) = ticks {
            let overreaching: Vec<Dynamic> = ticks_dyn
                .as_stream()
                .into_iter()
                .filter(|tick| {
                    let tick_x = tick.get("x").as_int(0);
                    let tick_z = tick.get("z").as_int(0);
                    let dx = chunk_x.wrapping_sub(tick_x >> 4).unsigned_abs();
                    let dz = chunk_z.wrapping_sub(tick_z >> 4).unsigned_abs();
                    (dx != 0 || dz != 0) && dx <= 1 && dz <= 1
                })
                .collect();
            if !overreaching.is_empty() {
                let upgrade_data = chunk.get("UpgradeData").or_else_empty_map();
                let list = chunk.create_list(overreaching);
                let upgrade_data = upgrade_data.set(target_key, list);
                chunk = chunk.set("UpgradeData", upgrade_data);
            }
        }
        chunk
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

    fn list(items: Vec<Dynamic>) -> Dynamic {
        Dynamic {
            value: DynValue::List(items),
        }
    }

    fn tick(x: i32, z: i32) -> Dynamic {
        map(vec![("x", int_val(x)), ("z", int_val(z))])
    }

    #[test]
    fn extracts_overreaching_block_tick() {
        let fix = OverreachingTickFix::new(());
        let chunk = map(vec![("xPos", int_val(0)), ("zPos", int_val(0))]);
        let block_ticks = Some(list(vec![tick(16, 0), tick(0, 0)]));
        let out = fix.fix_chunk(chunk, block_ticks);
        let upgrade = out.get("UpgradeData").result().unwrap();
        let neighbors = upgrade.get("neighbor_block_ticks").result().unwrap();
        let items = neighbors.as_stream();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].get("x").as_int(-1), 16);
    }

    #[test]
    fn ignores_same_chunk_and_far_ticks() {
        let fix = OverreachingTickFix::new(());
        let chunk = map(vec![("xPos", int_val(0)), ("zPos", int_val(0))]);
        let block_ticks = Some(list(vec![tick(0, 0), tick(32, 0), tick(15, 15)]));
        let out = fix.fix_chunk(chunk, block_ticks);
        assert!(out.get("UpgradeData").result().is_none());
    }

    #[test]
    fn extracts_fluid_ticks_from_chunk() {
        let fix = OverreachingTickFix::new(());
        let chunk = map(vec![
            ("xPos", int_val(1)),
            ("zPos", int_val(1)),
            ("fluid_ticks", list(vec![tick(0, 16)])),
        ]);
        let out = fix.fix_chunk(chunk, None);
        let upgrade = out.get("UpgradeData").result().unwrap();
        let neighbors = upgrade.get("neighbor_fluid_ticks").result().unwrap();
        assert_eq!(neighbors.as_stream().len(), 1);
    }

    #[test]
    fn negative_chunk_coords_extract_adjacent() {
        let fix = OverreachingTickFix::new(());
        let chunk = map(vec![("xPos", int_val(-1)), ("zPos", int_val(0))]);
        // x = -17 => chunk -2 in Java arithmetic shift, adjacent to chunk -1
        let block_ticks = Some(list(vec![tick(-17, 0)]));
        let out = fix.fix_chunk(chunk, block_ticks);
        let upgrade = out.get("UpgradeData").result().unwrap();
        let neighbors = upgrade.get("neighbor_block_ticks").result().unwrap();
        assert_eq!(neighbors.as_stream().len(), 1);
    }

    #[test]
    fn no_ticks_no_change() {
        let fix = OverreachingTickFix::new(());
        let chunk = map(vec![
            ("xPos", int_val(0)),
            ("zPos", int_val(0)),
            ("keep", int_val(1)),
        ]);
        let out = fix.fix_chunk(chunk, None);
        assert_eq!(out.get("keep").as_int(0), 1);
        assert!(out.get("UpgradeData").result().is_none());
    }
}
