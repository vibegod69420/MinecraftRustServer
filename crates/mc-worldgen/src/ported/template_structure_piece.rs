use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureMode {
    Save,
    Load,
    Corner,
    Data,
    Legacy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub name: String,
}

impl BlockState {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn air() -> Self {
        Self {
            name: "minecraft:air".to_string(),
        }
    }

    pub fn structure_block() -> Self {
        Self {
            name: "minecraft:structure_block".to_string(),
        }
    }

    pub fn jigsaw() -> Self {
        Self {
            name: "minecraft:jigsaw".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn parse(s: &str) -> Self {
        if let Some(colon) = s.find(':') {
            let (ns, rest) = s.split_at(colon);
            let path = &rest[1..];
            Self {
                namespace: ns.to_string(),
                path: path.to_string(),
            }
        } else {
            Self {
                namespace: "minecraft".to_string(),
                path: s.to_string(),
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CompoundTag {
    pub strings: HashMap<String, String>,
    pub ints: HashMap<String, i32>,
    pub mode: Option<StructureMode>,
}

impl CompoundTag {
    pub fn get_string_or(&self, key: &str, default: &str) -> String {
        self.strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        self.ints.get(key).copied().unwrap_or(default)
    }

    pub fn get_mode(&self) -> Option<StructureMode> {
        self.mode
    }

    pub fn set_mode(&mut self, mode: StructureMode) {
        self.mode = Some(mode);
    }
}

#[derive(Clone, Debug)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    pub state: BlockState,
    pub nbt: Option<CompoundTag>,
}

pub type StructurePieceTypeId = i32;

#[derive(Clone, Debug)]
pub struct StructurePieceBase {
    pub piece_type: StructurePieceTypeId,
    pub gen_depth: i32,
    pub bounding_box: BoundingBox,
    pub orientation: Option<Direction>,
}

impl StructurePieceBase {
    pub fn move_piece(&mut self, dx: i32, dy: i32, dz: i32) {
        self.bounding_box.min_x = self.bounding_box.min_x.wrapping_add(dx);
        self.bounding_box.min_y = self.bounding_box.min_y.wrapping_add(dy);
        self.bounding_box.min_z = self.bounding_box.min_z.wrapping_add(dz);
        self.bounding_box.max_x = self.bounding_box.max_x.wrapping_add(dx);
        self.bounding_box.max_y = self.bounding_box.max_y.wrapping_add(dy);
        self.bounding_box.max_z = self.bounding_box.max_z.wrapping_add(dz);
    }
}

#[derive(Clone, Debug)]
pub struct StructurePlaceSettings {
    pub rotation: Rotation,
    pub bounding_box: Option<BoundingBox>,
}

impl StructurePlaceSettings {
    pub fn new(rotation: Rotation) -> Self {
        Self {
            rotation,
            bounding_box: None,
        }
    }

    pub fn set_bounding_box(&mut self, bb: BoundingBox) {
        self.bounding_box = Some(bb);
    }

    pub fn get_rotation(&self) -> Rotation {
        self.rotation
    }
}

pub trait RandomSourceLike {}

pub trait WorldGenLevelLike {
    fn set_block(&mut self, pos: BlockPos, state: &BlockState, flags: i32) -> bool;
}

pub trait ServerLevelAccessorLike: WorldGenLevelLike {}

pub trait StructureTemplateLike {
    fn get_bounding_box(&self, settings: &StructurePlaceSettings, pos: BlockPos) -> BoundingBox;

    fn place_in_world(
        &self,
        level: &mut dyn WorldGenLevelLike,
        pos: BlockPos,
        pivot: BlockPos,
        settings: &StructurePlaceSettings,
        rand: &mut dyn RandomSourceLike,
        flags: i32,
    ) -> bool;

    fn filter_blocks(
        &self,
        pos: BlockPos,
        settings: &StructurePlaceSettings,
        block: &BlockState,
    ) -> Vec<StructureBlockInfo>;
}

pub trait StructureTemplateManagerLike {
    fn get_or_create(&mut self, id: &Identifier) -> Box<dyn StructureTemplateLike>;
}

pub trait DataMarkerHandler {
    // JAVA: net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#handleDataMarker
    fn handle_data_marker(
        &mut self,
        metadata: &str,
        pos: BlockPos,
        level: &mut dyn WorldGenLevelLike,
        rand: &mut dyn RandomSourceLike,
        bbox: &BoundingBox,
    );
}

pub struct TemplateStructurePiece {
    pub base: StructurePieceBase,
    pub template_name: String,
    pub template: Box<dyn StructureTemplateLike>,
    pub place_settings: StructurePlaceSettings,
    pub template_position: BlockPos,
}

impl TemplateStructurePiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#makeTemplateLocation
    pub fn make_template_location_from_name(template_name: &str) -> Identifier {
        Identifier::parse(template_name)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#TemplateStructurePiece
    pub fn new_from_parts(
        piece_type: StructurePieceTypeId,
        gen_depth: i32,
        manager: &mut dyn StructureTemplateManagerLike,
        id: Identifier,
        template_name: String,
        place_settings: StructurePlaceSettings,
        template_position: BlockPos,
    ) -> Self {
        let template = manager.get_or_create(&id);
        let bounding_box = template.get_bounding_box(&place_settings, template_position);
        let base = StructurePieceBase {
            piece_type,
            gen_depth,
            bounding_box,
            orientation: Some(Direction::North),
        };
        Self {
            base,
            template_name,
            template,
            place_settings,
            template_position,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#TemplateStructurePiece
    pub fn new_from_nbt<F>(
        piece_type: StructurePieceTypeId,
        nbt: &CompoundTag,
        manager: &mut dyn StructureTemplateManagerLike,
        mut place_settings_fn: F,
        mut base: StructurePieceBase,
    ) -> Self
    where
        F: FnMut(&Identifier) -> StructurePlaceSettings,
    {
        base.orientation = Some(Direction::North);
        let template_name = nbt.get_string_or("Template", "");
        let tpx = nbt.get_int_or("TPX", 0);
        let tpy = nbt.get_int_or("TPY", 0);
        let tpz = nbt.get_int_or("TPZ", 0);
        let template_position = BlockPos::new(tpx, tpy, tpz);
        let id = Self::make_template_location_from_name(&template_name);
        let template = manager.get_or_create(&id);
        let place_settings = place_settings_fn(&id);
        let bounding_box = template.get_bounding_box(&place_settings, template_position);
        base.bounding_box = bounding_box;
        Self {
            base,
            template_name,
            template,
            place_settings,
            template_position,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#addAdditionalSaveData
    pub fn add_additional_save_data(&self, nbt: &mut CompoundTag) {
        nbt.ints.insert("TPX".to_string(), self.template_position.x);
        nbt.ints.insert("TPY".to_string(), self.template_position.y);
        nbt.ints.insert("TPZ".to_string(), self.template_position.z);
        nbt.strings
            .insert("Template".to_string(), self.template_name.clone());
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#postProcess
    pub fn post_process<FParse>(
        &mut self,
        level: &mut dyn WorldGenLevelLike,
        rand: &mut dyn RandomSourceLike,
        bbox: BoundingBox,
        pivot: BlockPos,
        handler: &mut dyn DataMarkerHandler,
        parse_block_state: FParse,
    ) where
        FParse: Fn(&str) -> BlockState,
    {
        self.place_settings.set_bounding_box(bbox);
        self.base.bounding_box = self
            .template
            .get_bounding_box(&self.place_settings, self.template_position);

        let placed = self.template.place_in_world(
            level,
            self.template_position,
            pivot,
            &self.place_settings,
            rand,
            2,
        );
        if placed {
            let sb = BlockState::structure_block();
            let infos_sb = self
                .template
                .filter_blocks(self.template_position, &self.place_settings, &sb);
            for info in infos_sb {
                if let Some(nbt) = &info.nbt {
                    let mode = nbt.get_mode().unwrap_or(StructureMode::Data);
                    if mode == StructureMode::Data {
                        let meta = nbt.get_string_or("metadata", "");
                        handler.handle_data_marker(
                            &meta,
                            info.pos,
                            level,
                            rand,
                            &self.base.bounding_box,
                        );
                    }
                }
            }

            let jb = BlockState::jigsaw();
            let infos_jb = self
                .template
                .filter_blocks(self.template_position, &self.place_settings, &jb);
            for info in infos_jb {
                if let Some(nbt) = &info.nbt {
                    let final_state_str = nbt.get_string_or("final_state", "minecraft:air");
                    let state = parse_block_state(&final_state_str);
                    let _ = level.set_block(info.pos, &state, 3);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#move
    pub fn move_piece(&mut self, dx: i32, dy: i32, dz: i32) {
        self.base.move_piece(dx, dy, dz);
        self.template_position = self.template_position.offset(dx, dy, dz);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#getRotation
    pub fn get_rotation(&self) -> Rotation {
        self.place_settings.get_rotation()
    }

    pub fn template(&self) -> &dyn StructureTemplateLike {
        &*self.template
    }

    pub fn template_position(&self) -> BlockPos {
        self.template_position
    }

    pub fn place_settings(&self) -> &StructurePlaceSettings {
        &self.place_settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyRand;

    impl RandomSourceLike for DummyRand {}

    struct DummyLevel {
        pub set_blocks: Vec<(BlockPos, BlockState, i32)>,
    }

    impl DummyLevel {
        fn new() -> Self {
            Self {
                set_blocks: Vec::new(),
            }
        }
    }

    impl WorldGenLevelLike for DummyLevel {
        fn set_block(&mut self, pos: BlockPos, state: &BlockState, flags: i32) -> bool {
            self.set_blocks.push((pos, state.clone(), flags));
            true
        }
    }

    struct DummyTemplate {
        bb: BoundingBox,
        place_ok: bool,
        sb_infos: Vec<StructureBlockInfo>,
        jb_infos: Vec<StructureBlockInfo>,
    }

    impl StructureTemplateLike for DummyTemplate {
        fn get_bounding_box(
            &self,
            _settings: &StructurePlaceSettings,
            pos: BlockPos,
        ) -> BoundingBox {
            BoundingBox {
                min_x: self.bb.min_x.wrapping_add(pos.x),
                min_y: self.bb.min_y.wrapping_add(pos.y),
                min_z: self.bb.min_z.wrapping_add(pos.z),
                max_x: self.bb.max_x.wrapping_add(pos.x),
                max_y: self.bb.max_y.wrapping_add(pos.y),
                max_z: self.bb.max_z.wrapping_add(pos.z),
            }
        }

        fn place_in_world(
            &self,
            _level: &mut dyn WorldGenLevelLike,
            _pos: BlockPos,
            _pivot: BlockPos,
            _settings: &StructurePlaceSettings,
            _rand: &mut dyn RandomSourceLike,
            _flags: i32,
        ) -> bool {
            self.place_ok
        }

        fn filter_blocks(
            &self,
            _pos: BlockPos,
            _settings: &StructurePlaceSettings,
            block: &BlockState,
        ) -> Vec<StructureBlockInfo> {
            if block.name == BlockState::structure_block().name {
                self.sb_infos.clone()
            } else if block.name == BlockState::jigsaw().name {
                self.jb_infos.clone()
            } else {
                Vec::new()
            }
        }
    }

    struct DummyMgr {
        tpl: Option<Box<dyn StructureTemplateLike>>,
    }

    impl StructureTemplateManagerLike for DummyMgr {
        fn get_or_create(&mut self, _id: &Identifier) -> Box<dyn StructureTemplateLike> {
            if let Some(t) = self.tpl.take() {
                t
            } else {
                Box::new(DummyTemplate {
                    bb: BoundingBox {
                        min_x: 0,
                        min_y: 0,
                        min_z: 0,
                        max_x: 1,
                        max_y: 1,
                        max_z: 1,
                    },
                    place_ok: true,
                    sb_infos: Vec::new(),
                    jb_infos: Vec::new(),
                })
            }
        }
    }

    struct DummyHandler {
        pub calls: Vec<(String, BlockPos)>,
    }

    impl DataMarkerHandler for DummyHandler {
        fn handle_data_marker(
            &mut self,
            metadata: &str,
            pos: BlockPos,
            _level: &mut dyn WorldGenLevelLike,
            _rand: &mut dyn RandomSourceLike,
            _bbox: &BoundingBox,
        ) {
            self.calls.push((metadata.to_string(), pos));
        }
    }

    #[test]
    fn test_identifier_parse_and_offset_wrapping() {
        let id = Identifier::parse("mymod:test_path");
        assert_eq!(id.namespace, "mymod");
        assert_eq!(id.path, "test_path");
        let id2 = Identifier::parse("noprefix");
        assert_eq!(id2.namespace, "minecraft");
        assert_eq!(id2.path, "noprefix");

        let p = BlockPos::new(i32::MAX, -5, 0);
        let p2 = p.offset(1, -1, 0);
        assert_eq!(p2.x, i32::MIN);
        assert_eq!(p2.y, -6);
    }

    #[test]
    fn test_move_piece_wrapping() {
        let bb = BoundingBox {
            min_x: i32::MAX - 1,
            min_y: 0,
            min_z: 0,
            max_x: i32::MAX,
            max_y: 1,
            max_z: 1,
        };
        let base = StructurePieceBase {
            piece_type: 1,
            gen_depth: 0,
            bounding_box: bb,
            orientation: Some(Direction::North),
        };
        let tpl = Box::new(DummyTemplate {
            bb: BoundingBox {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 0,
                max_y: 0,
                max_z: 0,
            },
            place_ok: false,
            sb_infos: Vec::new(),
            jb_infos: Vec::new(),
        });
        let mut piece = TemplateStructurePiece {
            base,
            template_name: "t".to_string(),
            template: tpl,
            place_settings: StructurePlaceSettings::new(Rotation::None),
            template_position: BlockPos::new(10, 20, 30),
        };
        piece.move_piece(5, -100, 0);
        assert_eq!(piece.template_position.x, 15);
        assert_eq!(piece.template_position.y, 20i32.wrapping_sub(100));
        assert_eq!(
            piece.base.bounding_box.min_x,
            (i32::MAX - 1).wrapping_add(5)
        );
    }

    #[test]
    fn test_additional_save_and_nbt_ctor() {
        let mut piece = TemplateStructurePiece {
            base: StructurePieceBase {
                piece_type: 2,
                gen_depth: 1,
                bounding_box: BoundingBox {
                    min_x: 0,
                    min_y: 0,
                    min_z: 0,
                    max_x: 0,
                    max_y: 0,
                    max_z: 0,
                },
                orientation: Some(Direction::North),
            },
            template_name: "ns:tpl".to_string(),
            template: Box::new(DummyTemplate {
                bb: BoundingBox {
                    min_x: 0,
                    min_y: 0,
                    min_z: 0,
                    max_x: 2,
                    max_y: 2,
                    max_z: 2,
                },
                place_ok: true,
                sb_infos: Vec::new(),
                jb_infos: Vec::new(),
            }),
            place_settings: StructurePlaceSettings::new(Rotation::None),
            template_position: BlockPos::new(7, -3, 9),
        };
        let mut nbt = CompoundTag::default();
        piece.add_additional_save_data(&mut nbt);
        assert_eq!(nbt.get_int_or("TPX", 0), 7);
        assert_eq!(nbt.get_int_or("TPY", 0), -3);
        assert_eq!(nbt.get_int_or("TPZ", 0), 9);
        assert_eq!(nbt.get_string_or("Template", ""), "ns:tpl");

        let mut mgr = DummyMgr { tpl: None };
        let base2 = StructurePieceBase {
            piece_type: 2,
            gen_depth: 1,
            bounding_box: BoundingBox {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 0,
                max_y: 0,
                max_z: 0,
            },
            orientation: None,
        };
        let piece2 = TemplateStructurePiece::new_from_nbt(
            2,
            &nbt,
            &mut mgr,
            |_| StructurePlaceSettings::new(Rotation::Clockwise90),
            base2,
        );
        assert_eq!(piece2.template_position, BlockPos::new(7, -3, 9));
        assert_eq!(piece2.template_name, "ns:tpl");
        assert_eq!(piece2.get_rotation(), Rotation::Clockwise90);
    }

    #[test]
    fn test_post_process_data_and_jigsaw() {
        let mut sb_nbt = CompoundTag::default();
        sb_nbt.set_mode(StructureMode::Data);
        sb_nbt.strings.insert("metadata".to_string(), "mymeta".to_string());

        let mut jb_nbt = CompoundTag::default();
        jb_nbt
            .strings
            .insert("final_state".to_string(), "minecraft:stone".to_string());

        let tpl = DummyTemplate {
            bb: BoundingBox {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 1,
                max_y: 1,
                max_z: 1,
            },
            place_ok: true,
            sb_infos: vec![StructureBlockInfo {
                pos: BlockPos::new(1, 2, 3),
                state: BlockState::structure_block(),
                nbt: Some(sb_nbt),
            }],
            jb_infos: vec![StructureBlockInfo {
                pos: BlockPos::new(4, 5, 6),
                state: BlockState::jigsaw(),
                nbt: Some(jb_nbt),
            }],
        };
        let mut piece = TemplateStructurePiece {
            base: StructurePieceBase {
                piece_type: 3,
                gen_depth: 0,
                bounding_box: BoundingBox {
                    min_x: 0,
                    min_y: 0,
                    min_z: 0,
                    max_x: 0,
                    max_y: 0,
                    max_z: 0,
                },
                orientation: Some(Direction::North),
            },
            template_name: "t".to_string(),
            template: Box::new(tpl),
            place_settings: StructurePlaceSettings::new(Rotation::None),
            template_position: BlockPos::new(0, 0, 0),
        };
        let mut level = DummyLevel::new();
        let mut rand = DummyRand;
        let mut handler = DummyHandler { calls: Vec::new() };

        piece.post_process(
            &mut level,
            &mut rand,
            BoundingBox {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 5,
                max_y: 5,
                max_z: 5,
            },
            BlockPos::new(0, 0, 0),
            &mut handler,
            |s| BlockState::new(s),
        );

        assert_eq!(handler.calls.len(), 1);
        assert_eq!(handler.calls[0].0, "mymeta");
        assert_eq!(handler.calls[0].1, BlockPos::new(1, 2, 3));
        assert_eq!(level.set_blocks.len(), 1);
        assert_eq!(level.set_blocks[0].0, BlockPos::new(4, 5, 6));
        assert_eq!(level.set_blocks[0].1.name, "minecraft:stone");
        assert_eq!(level.set_blocks[0].2, 3);
    }
}
