#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagKey {
    pub id: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub name: &'static str,
    pub tags: &'static [&'static str],
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockState#is(TagKey)
    pub fn is(&self, tag: &TagKey) -> bool {
        self.tags.iter().any(|t| *t == tag.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    pub state: BlockState,
    pub nbt: Option<Vec<u8>>,
}

pub trait LevelReader {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

#[derive(Debug, Clone, Default)]
pub struct StructurePlaceSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureProcessorType {
    ProtectedBlocks,
}

// JAVA: net.minecraft.world.level.levelgen.feature.Feature#isReplaceable
pub fn is_replaceable(cannot_replace: &TagKey, state: &BlockState) -> bool {
    !state.is(cannot_replace)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedBlockProcessor {
    pub cannot_replace: TagKey,
}

impl ProtectedBlockProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.ProtectedBlockProcessor#<init>
    pub fn new(cannot_replace: TagKey) -> Self {
        Self { cannot_replace }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.ProtectedBlockProcessor#processBlock
    pub fn process_block(
        &self,
        level: &dyn LevelReader,
        _pos1: &BlockPos,
        _pos2: &BlockPos,
        _info3: &StructureBlockInfo,
        info4: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let state = level.get_block_state(&info4.pos);
        if is_replaceable(&self.cannot_replace, &state) {
            Some(info4.clone())
        } else {
            None
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.ProtectedBlockProcessor#getType
    pub fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::ProtectedBlocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        state: BlockState,
    }

    impl LevelReader for MockLevel {
        fn get_block_state(&self, _pos: &BlockPos) -> BlockState {
            self.state.clone()
        }
    }

    struct PositionalLevel;

    impl LevelReader for PositionalLevel {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            if pos.x == 7 {
                BlockState {
                    name: "minecraft:bedrock",
                    tags: &["prot"],
                }
            } else {
                BlockState {
                    name: "minecraft:stone",
                    tags: &[],
                }
            }
        }
    }

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    fn info_at(p: BlockPos, name: &'static str) -> StructureBlockInfo {
        StructureBlockInfo {
            pos: p,
            state: BlockState { name, tags: &[] },
            nbt: None,
        }
    }

    #[test]
    fn replaceable_when_not_in_protected_tag() {
        let tag = TagKey {
            id: "minecraft:features_cannot_replace",
        };
        let proc = ProtectedBlockProcessor::new(tag);
        let level = MockLevel {
            state: BlockState {
                name: "minecraft:stone",
                tags: &[],
            },
        };
        let p = pos(1, 2, 3);
        let i3 = info_at(pos(0, 0, 0), "minecraft:air");
        let i4 = info_at(p, "minecraft:oak_planks");
        let result = proc.process_block(
            &level,
            &pos(0, 0, 0),
            &pos(0, 0, 0),
            &i3,
            &i4,
            &StructurePlaceSettings,
        );
        assert_eq!(result, Some(i4));
    }

    #[test]
    fn blocked_when_in_protected_tag() {
        let tag = TagKey {
            id: "minecraft:features_cannot_replace",
        };
        let proc = ProtectedBlockProcessor::new(tag);
        let level = MockLevel {
            state: BlockState {
                name: "minecraft:bedrock",
                tags: &["minecraft:features_cannot_replace"],
            },
        };
        let i4 = info_at(pos(5, -1, 10), "minecraft:stone");
        let i3 = info_at(pos(0, 0, 0), "minecraft:air");
        let result = proc.process_block(
            &level,
            &pos(0, 0, 0),
            &pos(0, 0, 0),
            &i3,
            &i4,
            &StructurePlaceSettings,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn uses_level_state_at_destination_pos() {
        let proc = ProtectedBlockProcessor::new(TagKey { id: "prot" });
        let level = PositionalLevel;
        let i3 = info_at(pos(0, 0, 0), "minecraft:air");

        let protected_pos = pos(7, 0, 0);
        let i4_prot = info_at(protected_pos, "minecraft:stone");
        assert_eq!(
            proc.process_block(
                &level,
                &pos(0, 0, 0),
                &pos(0, 0, 0),
                &i3,
                &i4_prot,
                &StructurePlaceSettings
            ),
            None
        );

        let free_pos = pos(8, 0, 0);
        let i4_free = info_at(free_pos, "minecraft:stone");
        assert_eq!(
            proc.process_block(
                &level,
                &pos(0, 0, 0),
                &pos(0, 0, 0),
                &i3,
                &i4_free,
                &StructurePlaceSettings
            ),
            Some(i4_free)
        );
    }

    #[test]
    fn different_tag_does_not_block() {
        let proc = ProtectedBlockProcessor::new(TagKey { id: "tag_a" });
        let level = MockLevel {
            state: BlockState {
                name: "minecraft:stone",
                tags: &["tag_b"],
            },
        };
        let i4 = info_at(pos(-3, -64, 99), "minecraft:dirt");
        let i3 = info_at(pos(0, 0, 0), "minecraft:air");
        let result = proc.process_block(
            &level,
            &pos(0, 0, 0),
            &pos(0, 0, 0),
            &i3,
            &i4,
            &StructurePlaceSettings,
        );
        assert_eq!(result, Some(i4));
    }

    #[test]
    fn get_type_is_protected_blocks() {
        let proc = ProtectedBlockProcessor::new(TagKey { id: "any" });
        assert_eq!(proc.get_type(), StructureProcessorType::ProtectedBlocks);
    }
}
