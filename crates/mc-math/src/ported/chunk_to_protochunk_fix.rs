use std::collections::BTreeMap;

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i64),
    Short(i16),
    String(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
    ByteBuffer(Vec<u8>),
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
#[derive(Clone, Debug, PartialEq)]
pub struct DataResult<T> {
    value: Option<T>,
}

impl<T> DataResult<T> {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn success(value: T) -> Self {
        Self { value: Some(value) }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn empty() -> Self {
        Self { value: None }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn result(&self) -> Option<T>
    where
        T: Clone,
    {
        self.value.clone()
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn map<U, F>(self, f: F) -> DataResult<U>
    where
        F: FnOnce(T) -> U,
    {
        DataResult {
            value: self.value.map(f),
        }
    }
}

impl Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn null() -> Self {
        Self::Null
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn int(value: i32) -> Self {
        Self::Int(value as i64)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn long(value: i64) -> Self {
        Self::Int(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn short(value: i16) -> Self {
        Self::Short(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn list(value: Vec<Dynamic>) -> Self {
        Self::List(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn map(value: BTreeMap<String, Dynamic>) -> Self {
        Self::Map(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn byte_buffer(value: Vec<u8>) -> Self {
        Self::ByteBuffer(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn from_pairs(pairs: impl IntoIterator<Item = (String, Dynamic)>) -> Self {
        Self::Map(pairs.into_iter().collect())
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn get(&self, name: &str) -> Dynamic {
        match self {
            Self::Map(map) => map.get(name).cloned().unwrap_or(Self::Null),
            _ => Self::Null,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn update<F>(&self, name: &str, f: F) -> Dynamic
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Self::Map(map) => {
                let mut map = map.clone();
                if let Some(value) = map.get(name).cloned() {
                    map.insert(name.to_string(), f(value));
                }
                Self::Map(map)
            }
            _ => self.clone(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn set(&self, name: &str, value: Dynamic) -> Dynamic {
        match self {
            Self::Map(map) => {
                let mut map = map.clone();
                map.insert(name.to_string(), value);
                Self::Map(map)
            }
            _ => {
                let mut map = BTreeMap::new();
                map.insert(name.to_string(), value);
                Self::Map(map)
            }
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn remove(&self, name: &str) -> Dynamic {
        match self {
            Self::Map(map) => {
                let mut map = map.clone();
                map.remove(name);
                Self::Map(map)
            }
            _ => self.clone(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn as_boolean(&self, default: bool) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::Int(value) => *value != 0,
            Self::Short(value) => *value != 0,
            _ => default,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Self::Int(value) => *value as i32,
            Self::Short(value) => *value as i32,
            Self::Bool(value) => {
                if *value {
                    1
                } else {
                    0
                }
            }
            _ => default,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn as_number(&self) -> DataResult<i64> {
        match self {
            Self::Int(value) => DataResult::success(*value),
            Self::Short(value) => DataResult::success(*value as i64),
            _ => DataResult::empty(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn as_byte_buffer_opt(&self) -> DataResult<Vec<u8>> {
        match self {
            Self::ByteBuffer(value) => DataResult::success(value.clone()),
            _ => DataResult::empty(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn as_stream_opt(&self) -> DataResult<Vec<Dynamic>> {
        match self {
            Self::List(value) => DataResult::success(value.clone()),
            _ => DataResult::empty(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn create_string(&self, value: impl Into<String>) -> Dynamic {
        create_string(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn create_boolean(&self, value: bool) -> Dynamic {
        create_boolean(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn create_short(&self, value: i16) -> Dynamic {
        create_short(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn create_int_list(&self, values: impl IntoIterator<Item = i32>) -> Dynamic {
        create_int_list(values)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn create_list(&self, values: impl IntoIterator<Item = Dynamic>) -> Dynamic {
        create_list(values)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
pub fn create_string(value: impl Into<String>) -> Dynamic {
    Dynamic::String(value.into())
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
pub fn create_boolean(value: bool) -> Dynamic {
    Dynamic::Bool(value)
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
pub fn create_short(value: i16) -> Dynamic {
    Dynamic::Short(value)
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
pub fn create_int_list(values: impl IntoIterator<Item = i32>) -> Dynamic {
    Dynamic::List(values.into_iter().map(|value| Dynamic::Int(value as i64)).collect())
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
pub fn create_list(values: impl IntoIterator<Item = Dynamic>) -> Dynamic {
    Dynamic::List(values.into_iter().collect())
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#packOffsetCoordinates
pub fn pack_offset_coordinates(x: i32, y: i32, z: i32) -> i16 {
    ((x & 15) | ((y & 15) << 4) | ((z & 15) << 8)) as i16
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#repackBiomes
pub fn repack_biomes(dynamic: Dynamic) -> Dynamic {
    dynamic.update("Biomes", |biome_dynamic| {
        if let Some(bytes) = biome_dynamic.as_byte_buffer_opt().result() {
            let mut biomes = vec![0i32; 256];
            for index in 0..biomes.len() {
                if index < bytes.len() {
                    biomes[index] = bytes[index] as i32 & 255;
                }
            }
            create_int_list(biomes.into_iter())
        } else {
            biome_dynamic
        }
    })
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#repackTicks
pub fn repack_ticks(dynamic: Dynamic) -> Dynamic {
    if let Some(ticks) = dynamic.get("TileTicks").as_stream_opt().result() {
        let mut sections: Vec<Vec<i16>> = vec![Vec::new(); 16];

        for tick in ticks {
            let x = tick.get("x").as_int(0);
            let y = tick.get("y").as_int(0);
            let z = tick.get("z").as_int(0);
            let packed = pack_offset_coordinates(x, y, z);
            let section_index = (y >> 4) as usize;
            if section_index < sections.len() {
                sections[section_index].push(packed);
            }
        }

        let mut result = dynamic.remove("TileTicks");
        let to_be_ticked = create_list(sections.into_iter().map(|section| {
            create_list(section.into_iter().map(create_short))
        }));
        result.set("ToBeTicked", to_be_ticked)
    } else {
        dynamic
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#fixChunkData
pub fn fix_chunk_data(dynamic: Dynamic) -> Dynamic {
    let terrain_populated = dynamic.get("TerrainPopulated").as_boolean(false);
    let light_populated = dynamic.get("LightPopulated").as_number().result().is_none()
        || dynamic.get("LightPopulated").as_boolean(false);

    let status = if terrain_populated {
        if light_populated {
            "mobs_spawned"
        } else {
            "decorated"
        }
    } else {
        "carved"
    };

    repack_ticks(repack_biomes(dynamic.clone()))
        .set("Status", create_string(status))
        .set("hasLegacyStructureData", create_boolean(true))
}

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
#[allow(dead_code)]
pub struct Schema;

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
#[allow(dead_code)]
pub struct TypeRewriteRule;

// JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
#[allow(dead_code)]
pub struct ChunkToProtochunkFix {
    input_schema: Schema,
    changes_type: bool,
}

impl ChunkToProtochunkFix {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix
    pub fn new(input_schema: Schema, changes_type: bool) -> Self {
        Self {
            input_schema,
            changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#fixChunkData
    pub fn fix_chunk_data(dynamic: Dynamic) -> Dynamic {
        fix_chunk_data(dynamic)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#repackBiomes
    pub fn repack_biomes(dynamic: Dynamic) -> Dynamic {
        repack_biomes(dynamic)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#repackTicks
    pub fn repack_ticks(dynamic: Dynamic) -> Dynamic {
        repack_ticks(dynamic)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#packOffsetCoordinates
    pub fn pack_offset_coordinates(x: i32, y: i32, z: i32) -> i16 {
        pack_offset_coordinates(x, y, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#packOffsetCoordinates
    #[test]
    fn pack_offset_coordinates_basic() {
        assert_eq!(pack_offset_coordinates(1, 2, 3), (1 | (2 << 4) | (3 << 8)) as i16);
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#packOffsetCoordinates
    #[test]
    fn pack_offset_coordinates_masks_negative_and_overflow() {
        assert_eq!(pack_offset_coordinates(17, -1, 272), (1 | (15 << 4) | (0 << 8)) as i16);
        assert_eq!(pack_offset_coordinates(i32::MIN, i32::MAX, 0), (0 | (15 << 4) | 0) as i16);
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#fixChunkData
    #[test]
    fn fix_chunk_data_carved_status() {
        let input = Dynamic::from_pairs([
            ("TerrainPopulated".to_string(), Dynamic::bool(false)),
            ("LightPopulated".to_string(), Dynamic::bool(false)),
        ]);
        let output = fix_chunk_data(input);
        assert_eq!(output.get("Status"), Dynamic::string("carved"));
        assert_eq!(output.get("hasLegacyStructureData"), Dynamic::bool(true));
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#repackBiomes
    #[test]
    fn repack_biomes_byte_buffer() {
        let input = Dynamic::from_pairs([(
            "Biomes".to_string(),
            Dynamic::byte_buffer(vec![1, 2]),
        )]);
        let output = repack_biomes(input);
        match output.get("Biomes") {
            Dynamic::List(values) => {
                assert_eq!(values.len(), 256);
                assert_eq!(values[0], Dynamic::Int(1));
                assert_eq!(values[1], Dynamic::Int(2));
                assert_eq!(values[2], Dynamic::Int(0));
            }
            other => panic!("unexpected biomes value: {other:?}"),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkToProtochunkFix#repackTicks
    #[test]
    fn repack_ticks_groups_by_section() {
        let tick = Dynamic::from_pairs([
            ("x".to_string(), Dynamic::int(1)),
            ("y".to_string(), Dynamic::int(17)),
            ("z".to_string(), Dynamic::int(2)),
        ]);
        let input = Dynamic::from_pairs([(
            "TileTicks".to_string(),
            Dynamic::list(vec![tick]),
        )]);
        let output = repack_ticks(input);
        assert_eq!(output.get("TileTicks"), Dynamic::Null);

        match output.get("ToBeTicked") {
            Dynamic::List(sections) => {
                assert_eq!(sections.len(), 16);
                match &sections[1] {
                    Dynamic::List(ticks) => {
                        assert_eq!(ticks.len(), 1);
                        assert_eq!(ticks[0], Dynamic::Short(pack_offset_coordinates(1, 17, 2)));
                    }
                    other => panic!("unexpected section value: {other:?}"),
                }
                match &sections[0] {
                    Dynamic::List(ticks) => assert!(ticks.is_empty()),
                    other => panic!("unexpected section value: {other:?}"),
                }
            }
            other => panic!("unexpected ToBeTicked value: {other:?}"),
        }
    }
}
