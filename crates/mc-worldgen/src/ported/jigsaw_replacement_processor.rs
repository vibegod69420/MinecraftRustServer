use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub name: String,
}

impl BlockState {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn is(&self, block_name: &str) -> bool {
        self.name == block_name
    }
}

pub mod blocks {
    pub const JIGSAW: &str = "minecraft:jigsaw";
    pub const STRUCTURE_VOID: &str = "minecraft:structure_void";
}

pub mod shared_constants {
    pub const DEBUG_KEEP_JIGSAW_BLOCKS_DURING_STRUCTURE_GEN: bool = false;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundTag {
    entries: BTreeMap<String, String>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(key.into(), value.into());
    }

    pub fn get_string_or(&self, key: &str, default: &str) -> String {
        self.entries
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    pub state: BlockState,
    pub nbt: Option<CompoundTag>,
}

#[derive(Debug, Clone, Default)]
pub struct StructurePlaceSettings;

pub trait BlockStateLookup {
    fn parse_block_state(&self, input: &str, allow_nbt: bool) -> Result<BlockState, String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureProcessorTypeId {
    JigsawReplacement,
}

pub struct JigsawReplacementProcessor;

pub static INSTANCE: JigsawReplacementProcessor = JigsawReplacementProcessor;

impl JigsawReplacementProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.JigsawReplacementProcessor#<init>
    fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.JigsawReplacementProcessor#processBlock
    pub fn process_block(
        &self,
        level: &dyn BlockStateLookup,
        block_pos: &BlockPos,
        _pivot_pos: &BlockPos,
        _original_block_info: &StructureBlockInfo,
        block_info: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let state = &block_info.state;
        if !state.is(blocks::JIGSAW)
            || shared_constants::DEBUG_KEEP_JIGSAW_BLOCKS_DURING_STRUCTURE_GEN
        {
            return Some(block_info.clone());
        }

        let Some(nbt) = &block_info.nbt else {
            eprintln!(
                "Jigsaw block at {} is missing nbt, will not replace",
                block_pos
            );
            return Some(block_info.clone());
        };

        let final_state_str = nbt.get_string_or("final_state", "minecraft:air");

        let replacement_state = match level.parse_block_state(&final_state_str, true) {
            Ok(result) => result,
            Err(err) => {
                eprintln!(
                    "Failed to parse jigsaw replacement state '{}' at {}: {}",
                    final_state_str, block_pos, err
                );
                return None;
            }
        };

        if replacement_state.is(blocks::STRUCTURE_VOID) {
            None
        } else {
            Some(StructureBlockInfo {
                pos: block_info.pos.clone(),
                state: replacement_state,
                nbt: None,
            })
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.JigsawReplacementProcessor#getType
    pub fn get_type(&self) -> StructureProcessorTypeId {
        StructureProcessorTypeId::JigsawReplacement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLookup;
    impl BlockStateLookup for MockLookup {
        fn parse_block_state(&self, input: &str, _allow_nbt: bool) -> Result<BlockState, String> {
            if input == "minecraft:invalid" {
                Err("bad state".to_string())
            } else {
                Ok(BlockState::new(input))
            }
        }
    }

    fn make_jigsaw(pos: BlockPos, final_state: Option<&str>) -> StructureBlockInfo {
        let nbt = final_state.map(|s| {
            let mut c = CompoundTag::new();
            c.insert_string("final_state", s);
            c
        });
        StructureBlockInfo {
            pos,
            state: BlockState::new(blocks::JIGSAW),
            nbt,
        }
    }

    fn settings() -> StructurePlaceSettings {
        StructurePlaceSettings
    }

    #[test]
    fn non_jigsaw_returns_unchanged() {
        let proc = JigsawReplacementProcessor::new();
        let info = StructureBlockInfo {
            pos: BlockPos::new(0, 0, 0),
            state: BlockState::new("minecraft:stone"),
            nbt: None,
        };
        let out = proc.process_block(&MockLookup, &info.pos, &info.pos, &info, &info, &settings());
        assert_eq!(out, Some(info));
    }

    #[test]
    fn missing_nbt_returns_unchanged() {
        let proc = JigsawReplacementProcessor;
        let info = make_jigsaw(BlockPos::new(1, 2, 3), None);
        let out = proc.process_block(&MockLookup, &info.pos, &info.pos, &info, &info, &settings());
        assert_eq!(out, Some(info));
    }

    #[test]
    fn valid_replacement_returns_new_state_with_no_nbt() {
        let proc = JigsawReplacementProcessor;
        let info = make_jigsaw(BlockPos::new(-5, 64, 7), Some("minecraft:stone"));
        let out = proc
            .process_block(&MockLookup, &info.pos, &info.pos, &info, &info, &settings())
            .expect("should replace");
        assert_eq!(out.pos, BlockPos::new(-5, 64, 7));
        assert_eq!(out.state, BlockState::new("minecraft:stone"));
        assert_eq!(out.nbt, None);
    }

    #[test]
    fn structure_void_replacement_returns_none() {
        let proc = JigsawReplacementProcessor;
        let info = make_jigsaw(BlockPos::new(0, 0, 0), Some("minecraft:structure_void"));
        let out = proc.process_block(&MockLookup, &info.pos, &info.pos, &info, &info, &settings());
        assert_eq!(out, None);
    }

    #[test]
    fn parse_error_returns_none() {
        let proc = JigsawReplacementProcessor;
        let info = make_jigsaw(BlockPos::new(0, -1, 0), Some("minecraft:invalid"));
        let out = proc.process_block(&MockLookup, &info.pos, &info.pos, &info, &info, &settings());
        assert_eq!(out, None);
    }

    #[test]
    fn default_final_state_is_air_and_type_is_correct() {
        let proc = JigsawReplacementProcessor;
        let mut nbt = CompoundTag::new();
        nbt.insert_string("other", "value");
        let info = StructureBlockInfo {
            pos: BlockPos::new(i32::MIN, 0, i32::MAX),
            state: BlockState::new(blocks::JIGSAW),
            nbt: Some(nbt),
        };
        let out = proc
            .process_block(&MockLookup, &info.pos, &info.pos, &info, &info, &settings())
            .expect("should replace");
        assert_eq!(out.state, BlockState::new("minecraft:air"));
        assert_eq!(proc.get_type(), StructureProcessorTypeId::JigsawReplacement);
    }
}
