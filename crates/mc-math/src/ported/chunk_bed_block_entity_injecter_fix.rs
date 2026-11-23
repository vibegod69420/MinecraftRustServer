use std::collections::BTreeMap;

/// Minimal self-contained dynamic value representation for datafix porting.
///
/// This stands in for DFU `Dynamic<?>` in the narrow set of operations used by
/// `ChunkBedBlockEntityInjecterFix`.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Compound(BTreeMap<String, Dynamic>),
    Map(Vec<(Dynamic, Dynamic)>),
    List(Vec<Dynamic>),
    String(String),
    Int(i32),
    Short(i16),
    IntArray(Vec<i32>),
    Empty,
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Compound(m) => m.get(key).cloned().unwrap_or(Dynamic::Empty),
            Dynamic::Map(entries) => {
                for (k, v) in entries {
                    if let Dynamic::String(s) = k {
                        if s == key {
                            return v.clone();
                        }
                    }
                }
                Dynamic::Empty
            }
            _ => Dynamic::Empty,
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            Dynamic::Short(v) => i32::from(*v),
            _ => default,
        }
    }

    pub fn as_list(&self) -> Vec<Dynamic> {
        match self {
            Dynamic::List(v) => v.clone(),
            _ => Vec::new(),
        }
    }

    pub fn as_int_stream(&self) -> Vec<i32> {
        match self {
            Dynamic::IntArray(v) => v.clone(),
            Dynamic::List(v) => v.iter().map(|d| d.as_int(0)).collect(),
            _ => Vec::new(),
        }
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_owned())
    }

    pub fn create_int(&self, v: i32) -> Dynamic {
        Dynamic::Int(v)
    }

    pub fn create_short(&self, v: i16) -> Dynamic {
        Dynamic::Short(v)
    }

    pub fn create_map(&self, entries: Vec<(Dynamic, Dynamic)>) -> Dynamic {
        Dynamic::Map(entries)
    }
}

/// Stub for DFU `Schema`.
#[derive(Clone, Debug, Default)]
pub struct Schema;

/// Stub for DFU `TypeRewriteRule`.
#[derive(Clone, Debug, Default)]
pub struct TypeRewriteRule;

/// Port of `net.minecraft.util.datafix.fixes.ChunkBedBlockEntityInjecterFix`.
///
/// DFU rule wiring is represented by stubs; the core chunk transformation logic
/// is faithfully implemented in `cap`.
pub struct ChunkBedBlockEntityInjecterFix {
    _schema: Schema,
    _changes_type: bool,
}

impl ChunkBedBlockEntityInjecterFix {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkBedBlockEntityInjecterFix#ChunkBedBlockEntityInjecterFix
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            _schema: schema,
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkBedBlockEntityInjecterFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        // DFU rule composition is not modeled in this self-contained port.
        // The faithful data transformation is exposed via `cap`.
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkBedBlockEntityInjecterFix#cap
    pub fn cap(&self, chunk: &Dynamic) -> Dynamic {
        let level = chunk.get("Level");
        let x_pos: i32 = level.get("xPos").as_int(0);
        let z_pos: i32 = level.get("zPos").as_int(0);
        let mut tile_entities: Vec<Dynamic> = level.get("TileEntities").as_list();

        for section in level.get("Sections").as_list() {
            let section_y: i32 = section.get("Y").as_int(0);
            let blocks: Vec<i32> = section.get("Blocks").as_int_stream();
            for (index, &block) in blocks.iter().enumerate() {
                // 416 == ($$4xx & 0xFF) << 4
                if 416 == ((block & 0xFF) << 4) {
                    let idx: i32 = index as i32;
                    let local_x: i32 = idx & 15;
                    let local_y: i32 = (idx >> 8) & 15;
                    let local_z: i32 = (idx >> 4) & 15;
                    let bed_entity = section.create_map(vec![
                        (
                            section.create_string("id"),
                            section.create_string("minecraft:bed"),
                        ),
                        (
                            section.create_string("x"),
                            section.create_int(local_x.wrapping_add(x_pos.wrapping_shl(4))),
                        ),
                        (
                            section.create_string("y"),
                            section.create_int(local_y.wrapping_add(section_y.wrapping_shl(4))),
                        ),
                        (
                            section.create_string("z"),
                            section.create_int(local_z.wrapping_add(z_pos.wrapping_shl(4))),
                        ),
                        (
                            section.create_string("color"),
                            section.create_short(14i16),
                        ),
                    ]);
                    tile_entities.push(bed_entity);
                }
            }
        }

        if !tile_entities.is_empty() {
            let mut new_level_map = match &level {
                Dynamic::Compound(m) => m.clone(),
                _ => BTreeMap::new(),
            };
            new_level_map.insert("TileEntities".to_string(), Dynamic::List(tile_entities));

            let mut new_chunk_map = match chunk {
                Dynamic::Compound(m) => m.clone(),
                _ => BTreeMap::new(),
            };
            new_chunk_map.insert("Level".to_string(), Dynamic::Compound(new_level_map));
            Dynamic::Compound(new_chunk_map)
        } else {
            chunk.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chunk(
        x_pos: i32,
        z_pos: i32,
        sections: Vec<Dynamic>,
        tile_entities: Vec<Dynamic>,
    ) -> Dynamic {
        let mut level = BTreeMap::new();
        level.insert("xPos".to_string(), Dynamic::Int(x_pos));
        level.insert("zPos".to_string(), Dynamic::Int(z_pos));
        level.insert("Sections".to_string(), Dynamic::List(sections));
        level.insert("TileEntities".to_string(), Dynamic::List(tile_entities));
        let mut chunk = BTreeMap::new();
        chunk.insert("Level".to_string(), Dynamic::Compound(level));
        Dynamic::Compound(chunk)
    }

    fn make_section(y: i32, blocks: Vec<i32>) -> Dynamic {
        let mut m = BTreeMap::new();
        m.insert("Y".to_string(), Dynamic::Int(y));
        m.insert("Blocks".to_string(), Dynamic::IntArray(blocks));
        Dynamic::Compound(m)
    }

    #[test]
    fn injects_bed_block_entity_with_correct_coords() {
        let fix = ChunkBedBlockEntityInjecterFix::new(Schema, false);
        let mut blocks = vec![0i32; 4096];
        // index 0 => local_x=0, local_y=0, local_z=0; block 26 => bed
        blocks[0] = 26;
        let chunk = make_chunk(1, 2, vec![make_section(3, blocks)], vec![]);
        let fixed = fix.cap(&chunk);
        let tes = fixed.get("Level").get("TileEntities").as_list();
        assert_eq!(tes.len(), 1);
        let te = &tes[0];
        assert_eq!(te.get("id"), Dynamic::String("minecraft:bed".to_string()));
        assert_eq!(te.get("x").as_int(i32::MIN), 16); // 0 + (1 << 4)
        assert_eq!(te.get("y").as_int(i32::MIN), 48); // 0 + (3 << 4)
        assert_eq!(te.get("z").as_int(i32::MIN), 32); // 0 + (2 << 4)
        assert_eq!(te.get("color").as_int(i32::MIN), 14);
    }

    #[test]
    fn no_beds_and_no_tile_entities_returns_unchanged() {
        let fix = ChunkBedBlockEntityInjecterFix::new(Schema, false);
        let chunk = make_chunk(0, 0, vec![make_section(0, vec![0i32; 4096])], vec![]);
        let fixed = fix.cap(&chunk);
        assert_eq!(fixed, chunk);
    }

    #[test]
    fn preserves_existing_tile_entities_and_appends_beds() {
        let fix = ChunkBedBlockEntityInjecterFix::new(Schema, false);
        let mut blocks = vec![0i32; 4096];
        blocks[5] = 26;
        let existing = Dynamic::Compound(BTreeMap::from([(
            "id".to_string(),
            Dynamic::String("minecraft:chest".to_string()),
        )]));
        let chunk = make_chunk(0, 0, vec![make_section(0, blocks)], vec![existing.clone()]);
        let fixed = fix.cap(&chunk);
        let tes = fixed.get("Level").get("TileEntities").as_list();
        assert_eq!(tes.len(), 2);
        assert_eq!(tes[0], existing);
        assert_eq!(tes[1].get("id"), Dynamic::String("minecraft:bed".to_string()));
    }

    #[test]
    fn index_decomposition_and_negative_coordinates() {
        let fix = ChunkBedBlockEntityInjecterFix::new(Schema, false);
        let mut blocks = vec![0i32; 4096];
        // index = y*256 + z*16 + x = 1*256 + 10*16 + 5 = 421
        blocks[421] = 26;
        let chunk = make_chunk(-1, -2, vec![make_section(-3, blocks)], vec![]);
        let fixed = fix.cap(&chunk);
        let tes = fixed.get("Level").get("TileEntities").as_list();
        assert_eq!(tes.len(), 1);
        let te = &tes[0];
        assert_eq!(
            te.get("x").as_int(0),
            5i32.wrapping_add((-1i32).wrapping_shl(4))
        );
        assert_eq!(
            te.get("y").as_int(0),
            1i32.wrapping_add((-3i32).wrapping_shl(4))
        );
        assert_eq!(
            te.get("z").as_int(0),
            10i32.wrapping_add((-2i32).wrapping_shl(4))
        );
    }

    #[test]
    fn matches_bed_by_low_byte_only() {
        let fix = ChunkBedBlockEntityInjecterFix::new(Schema, false);
        let mut blocks = vec![0i32; 4096];
        blocks[0] = 26 | (0x5A << 8); // high bits set, low byte is 26 -> match
        blocks[1] = 26; // exact match
        blocks[2] = 416; // 416 & 0xFF = 160, not a bed match
        let chunk = make_chunk(0, 0, vec![make_section(0, blocks)], vec![]);
        let fixed = fix.cap(&chunk);
        let tes = fixed.get("Level").get("TileEntities").as_list();
        assert_eq!(tes.len(), 2);
    }

    #[test]
    fn coordinate_wrapping_on_overflow() {
        let fix = ChunkBedBlockEntityInjecterFix::new(Schema, false);
        let mut blocks = vec![0i32; 4096];
        blocks[0] = 26;
        let chunk = make_chunk(
            i32::MAX,
            i32::MIN,
            vec![make_section(i32::MAX, blocks)],
            vec![],
        );
        let fixed = fix.cap(&chunk);
        let tes = fixed.get("Level").get("TileEntities").as_list();
        assert_eq!(tes.len(), 1);
        let te = &tes[0];
        assert_eq!(
            te.get("x").as_int(0),
            0i32.wrapping_add(i32::MAX.wrapping_shl(4))
        );
        assert_eq!(
            te.get("y").as_int(0),
            0i32.wrapping_add(i32::MAX.wrapping_shl(4))
        );
        assert_eq!(
            te.get("z").as_int(0),
            0i32.wrapping_add(i32::MIN.wrapping_shl(4))
        );
    }
}
