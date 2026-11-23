// JAVA: net.minecraft.util.datafix.fixes.BlockPosFormatAndRenamesFix

use std::collections::BTreeMap;

use crate::packing::BlockPos;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i64),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn rename_field(mut self, from: &str, to: &str) -> Self {
        if let Dynamic::Map(map) = &mut self {
            if let Some(value) = map.remove(from) {
                map.insert(to.to_string(), value);
            }
        }
        self
    }

    pub fn rename_and_fix_field(mut self, from: &str, to: &str) -> Self {
        if let Dynamic::Map(map) = &mut self {
            if let Some(value) = map.remove(from) {
                map.insert(to.to_string(), BlockPosFormatAndRenamesFix::fix_block_pos(value));
            }
        }
        self
    }

    pub fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(mut map) => {
                let old = map.remove(key).unwrap_or(Dynamic::Null);
                map.insert(key.to_string(), f(old));
                Dynamic::Map(map)
            }
            other => other,
        }
    }

    pub fn create_list(values: Vec<Dynamic>) -> Self {
        Dynamic::List(values)
    }

    pub fn as_stream(self) -> Vec<Dynamic> {
        match self {
            Dynamic::List(values) => values,
            _ => Vec::new(),
        }
    }
}

pub struct BlockPosFormatAndRenamesFix;

impl BlockPosFormatAndRenamesFix {
    pub const PATROLLING_MOBS: [&'static str; 6] = [
        "minecraft:witch",
        "minecraft:ravager",
        "minecraft:pillager",
        "minecraft:illusioner",
        "minecraft:evoker",
        "minecraft:vindicator",
    ];

    pub fn new() -> Self {
        Self
    }

    // JAVA: ExtraDataFixUtils#fixBlockPos (used by this fix)
    pub fn fix_block_pos(value: Dynamic) -> Dynamic {
        match value {
            Dynamic::Map(map) => {
                let x = map.get("X").and_then(|v| match v {
                    Dynamic::Int(i) => Some(*i as i32),
                    _ => None,
                });
                let y = map.get("Y").and_then(|v| match v {
                    Dynamic::Int(i) => Some(*i as i32),
                    _ => None,
                });
                let z = map.get("Z").and_then(|v| match v {
                    Dynamic::Int(i) => Some(*i as i32),
                    _ => None,
                });
                if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                    return Dynamic::Int(BlockPos::as_long_coords(x, y, z));
                }
                Dynamic::Map(map)
            }
            Dynamic::List(values) if values.len() == 3 => {
                let mut ints = [0i32; 3];
                let mut ok = true;
                for (i, value) in values.iter().enumerate() {
                    match value {
                        Dynamic::Int(v) => ints[i] = *v as i32,
                        _ => {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    return Dynamic::Int(BlockPos::as_long_coords(ints[0], ints[1], ints[2]));
                }
                Dynamic::List(values)
            }
            other => other,
        }
    }

    pub fn fix_fields(value: Dynamic, renames: &[(String, String)]) -> Dynamic {
        let mut value = value;
        for (from, to) in renames {
            value = value.rename_and_fix_field(from, to);
        }
        value
    }

    pub fn fix_map_saved_data(value: Dynamic) -> Dynamic {
        value
            .update("frames", |frames| {
                Dynamic::List(
                    frames
                        .as_stream()
                        .into_iter()
                        .map(|frame| {
                            frame
                                .rename_and_fix_field("Pos", "pos")
                                .rename_field("Rotation", "rotation")
                                .rename_field("EntityId", "entity_id")
                        })
                        .collect(),
                )
            })
            .update("banners", |banners| {
                Dynamic::List(
                    banners
                        .as_stream()
                        .into_iter()
                        .map(|banner| {
                            banner
                                .rename_field("Pos", "pos")
                                .rename_field("Color", "color")
                                .rename_field("Name", "name")
                        })
                        .collect(),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixes_map_block_pos() {
        let mut map = BTreeMap::new();
        map.insert("X".to_string(), Dynamic::Int(1));
        map.insert("Y".to_string(), Dynamic::Int(2));
        map.insert("Z".to_string(), Dynamic::Int(3));
        let fixed = BlockPosFormatAndRenamesFix::fix_block_pos(Dynamic::Map(map));
        assert_eq!(fixed, Dynamic::Int(BlockPos::as_long_coords(1, 2, 3)));
    }

    #[test]
    fn fixes_list_block_pos() {
        let value = Dynamic::List(vec![Dynamic::Int(4), Dynamic::Int(5), Dynamic::Int(6)]);
        let fixed = BlockPosFormatAndRenamesFix::fix_block_pos(value);
        assert_eq!(fixed, Dynamic::Int(BlockPos::as_long_coords(4, 5, 6)));
    }

    #[test]
    fn renames_and_fixes_fields() {
        let mut map = BTreeMap::new();
        let mut pos = BTreeMap::new();
        pos.insert("X".to_string(), Dynamic::Int(7));
        pos.insert("Y".to_string(), Dynamic::Int(8));
        pos.insert("Z".to_string(), Dynamic::Int(9));
        map.insert("HivePos".to_string(), Dynamic::Map(pos));
        let fixed = BlockPosFormatAndRenamesFix::fix_fields(
            Dynamic::Map(map),
            &[("HivePos".to_string(), "hive_pos".to_string())],
        );
        let mut expected_map = BTreeMap::new();
        expected_map.insert("hive_pos".to_string(), Dynamic::Int(BlockPos::as_long_coords(7, 8, 9)));
        assert_eq!(fixed, Dynamic::Map(expected_map));
    }

    #[test]
    fn fixes_map_saved_data() {
        let mut frame_pos = BTreeMap::new();
        frame_pos.insert("X".to_string(), Dynamic::Int(1));
        frame_pos.insert("Y".to_string(), Dynamic::Int(2));
        frame_pos.insert("Z".to_string(), Dynamic::Int(3));
        let mut frame = BTreeMap::new();
        frame.insert("Pos".to_string(), Dynamic::Map(frame_pos));
        frame.insert("Rotation".to_string(), Dynamic::Int(4));
        frame.insert("EntityId".to_string(), Dynamic::Int(5));

        let mut banner = BTreeMap::new();
        banner.insert("Pos".to_string(), Dynamic::Int(99));
        banner.insert("Color".to_string(), Dynamic::Int(10));
        banner.insert("Name".to_string(), Dynamic::Int(11));

        let mut data = BTreeMap::new();
        data.insert("frames".to_string(), Dynamic::List(vec![Dynamic::Map(frame)]));
        data.insert("banners".to_string(), Dynamic::List(vec![Dynamic::Map(banner)]));

        let fixed = BlockPosFormatAndRenamesFix::fix_map_saved_data(Dynamic::Map(data));
        let Dynamic::Map(fixed_map) = fixed else { panic!("expected map") };
        let Dynamic::List(frames) = fixed_map.get("frames").unwrap() else { panic!("expected frames") };
        let Dynamic::Map(frame) = &frames[0] else { panic!("expected frame") };
        assert!(frame.contains_key("pos"));
        assert!(frame.contains_key("rotation"));
        assert!(frame.contains_key("entity_id"));
        assert!(!frame.contains_key("Pos"));
    }

    #[test]
    fn leaves_unknown_values_unchanged() {
        let value = Dynamic::Int(42);
        assert_eq!(BlockPosFormatAndRenamesFix::fix_block_pos(value.clone()), value);
    }
}
