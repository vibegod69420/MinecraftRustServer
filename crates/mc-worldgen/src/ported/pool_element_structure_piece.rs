//! Port of `net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece`.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.core.BlockPos#offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
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

impl BoundingBox {
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructurePieceType {
    Jigsaw,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JigsawJunction {
    pub source_x: i32,
    pub source_ground_y: i32,
    pub source_z: i32,
    pub delta_y: i32,
    pub dest_projection: i32,
}

impl JigsawJunction {
    pub fn new(
        source_x: i32,
        source_ground_y: i32,
        source_z: i32,
        delta_y: i32,
        dest_projection: i32,
    ) -> Self {
        Self {
            source_x,
            source_ground_y,
            source_z,
            delta_y,
            dest_projection,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiquidSettings {
    IgnoreWaterlogging,
    ApplyWaterlogging,
}

pub const DEFAULT_LIQUID_SETTINGS: LiquidSettings = LiquidSettings::ApplyWaterlogging;

#[derive(Clone, Debug, Default)]
pub struct StructureTemplateManager;

#[derive(Clone, Debug, Default)]
pub struct WorldGenLevel;

#[derive(Clone, Debug, Default)]
pub struct StructureManager;

#[derive(Clone, Debug, Default)]
pub struct ChunkGenerator;

#[derive(Clone, Debug, Default)]
pub struct RandomSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Clone, Debug, Default)]
pub struct StructurePieceSerializationContext;

#[derive(Clone, Debug)]
pub struct StructurePoolElement {
    pub name: String,
}

impl StructurePoolElement {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#getBoundingBox
    pub fn get_bounding_box(
        &self,
        _mgr: &StructureTemplateManager,
        pos: BlockPos,
        _rot: Rotation,
    ) -> BoundingBox {
        BoundingBox::new(pos.x, pos.y, pos.z, pos.x, pos.y, pos.z)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#place
    pub fn place(
        &self,
        _tmpl_mgr: &StructureTemplateManager,
        _level: &mut WorldGenLevel,
        _mgr: &StructureManager,
        _gen: &ChunkGenerator,
        _pos: BlockPos,
        _origin: BlockPos,
        _rot: Rotation,
        _bb: &BoundingBox,
        _rand: &mut RandomSource,
        _liquid: LiquidSettings,
        _keep: bool,
    ) {
        // no-op faithful stub
    }
}

#[derive(Clone, Debug)]
pub struct StructurePiece {
    pub piece_type: StructurePieceType,
    pub gen_depth: i32,
    pub bounding_box: BoundingBox,
}

impl StructurePiece {
    pub fn new(piece_type: StructurePieceType, gen_depth: i32, bounding_box: BoundingBox) -> Self {
        Self {
            piece_type,
            gen_depth,
            bounding_box,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#move
    pub fn move_piece(&mut self, dx: i32, dy: i32, dz: i32) {
        self.bounding_box.min_x = self.bounding_box.min_x.wrapping_add(dx);
        self.bounding_box.min_y = self.bounding_box.min_y.wrapping_add(dy);
        self.bounding_box.min_z = self.bounding_box.min_z.wrapping_add(dz);
        self.bounding_box.max_x = self.bounding_box.max_x.wrapping_add(dx);
        self.bounding_box.max_y = self.bounding_box.max_y.wrapping_add(dy);
        self.bounding_box.max_z = self.bounding_box.max_z.wrapping_add(dz);
    }

    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
}

#[derive(Clone, Debug)]
pub struct PoolElementStructurePiece {
    pub base: StructurePiece,
    pub element: StructurePoolElement,
    pub position: BlockPos,
    pub ground_level_delta: i32,
    pub rotation: Rotation,
    pub junctions: Vec<JigsawJunction>,
    pub structure_template_manager: StructureTemplateManager,
    pub liquid_settings: LiquidSettings,
}

impl PoolElementStructurePiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#<init>
    pub fn new(
        structure_template_manager: StructureTemplateManager,
        element: StructurePoolElement,
        position: BlockPos,
        ground_level_delta: i32,
        rotation: Rotation,
        bounding_box: BoundingBox,
        liquid_settings: LiquidSettings,
    ) -> Self {
        Self {
            base: StructurePiece::new(StructurePieceType::Jigsaw, 0, bounding_box),
            element,
            position,
            ground_level_delta,
            rotation,
            junctions: Vec::new(),
            structure_template_manager,
            liquid_settings,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#postProcess
    pub fn post_process(
        &mut self,
        level: &mut WorldGenLevel,
        structure_manager: &StructureManager,
        generator: &ChunkGenerator,
        random: &mut RandomSource,
        box_: &BoundingBox,
        chunk_pos: ChunkPos,
        pos: BlockPos,
    ) {
        self.place(level, structure_manager, generator, random, box_, pos, false);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#place
    pub fn place(
        &mut self,
        level: &mut WorldGenLevel,
        structure_manager: &StructureManager,
        generator: &ChunkGenerator,
        random: &mut RandomSource,
        box_: &BoundingBox,
        pos: BlockPos,
        keep_jigsaws: bool,
    ) {
        self.element.place(
            &self.structure_template_manager,
            level,
            structure_manager,
            generator,
            self.position,
            pos,
            self.rotation,
            box_,
            random,
            self.liquid_settings,
            keep_jigsaws,
        );
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#move
    pub fn move_by(&mut self, dx: i32, dy: i32, dz: i32) {
        self.base.move_piece(dx, dy, dz);
        self.position = self.position.offset(dx, dy, dz);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#getRotation
    pub fn get_rotation(&self) -> Rotation {
        self.rotation
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#getElement
    pub fn get_element(&self) -> &StructurePoolElement {
        &self.element
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#getPosition
    pub fn get_position(&self) -> BlockPos {
        self.position
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#getGroundLevelDelta
    pub fn get_ground_level_delta(&self) -> i32 {
        self.ground_level_delta
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#addJunction
    pub fn add_junction(&mut self, junction: JigsawJunction) {
        self.junctions.push(junction);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#getJunctions
    pub fn get_junctions(&self) -> &Vec<JigsawJunction> {
        &self.junctions
    }
}

impl fmt::Display for PoolElementStructurePiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<PoolElementStructurePiece | ({}, {}, {}) | {:?} | {}>",
            self.position.x, self.position.y, self.position.z, self.rotation, self.element.name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_getters_basic() {
        let mgr = StructureTemplateManager;
        let elem = StructurePoolElement::new("test_element");
        let pos = BlockPos::new(10, 20, 30);
        let bb = BoundingBox::new(10, 20, 30, 12, 25, 33);
        let piece = PoolElementStructurePiece::new(
            mgr,
            elem,
            pos,
            5,
            Rotation::Clockwise90,
            bb.clone(),
            DEFAULT_LIQUID_SETTINGS,
        );
        assert_eq!(piece.get_position(), BlockPos::new(10, 20, 30));
        assert_eq!(piece.get_ground_level_delta(), 5);
        assert_eq!(piece.get_rotation(), Rotation::Clockwise90);
        assert_eq!(piece.get_element().name, "test_element");
        assert_eq!(piece.get_junctions().len(), 0);
        assert_eq!(piece.base.bounding_box(), &bb);
    }

    #[test]
    fn test_move_by_updates_position_and_bbox_wrapping() {
        let mgr = StructureTemplateManager;
        let elem = StructurePoolElement::new("e");
        let pos = BlockPos::new(1, 2, 3);
        let bb = BoundingBox::new(1, 2, 3, 4, 5, 6);
        let mut piece = PoolElementStructurePiece::new(
            mgr,
            elem,
            pos,
            0,
            Rotation::None,
            bb,
            DEFAULT_LIQUID_SETTINGS,
        );
        piece.move_by(-2, 10, -5);
        assert_eq!(piece.get_position(), BlockPos::new(-1, 12, -2));
        assert_eq!(piece.base.bounding_box.min_x, -1);
        assert_eq!(piece.base.bounding_box.min_y, 12);
        assert_eq!(piece.base.bounding_box.min_z, -2);
        assert_eq!(piece.base.bounding_box.max_x, 2);
        assert_eq!(piece.base.bounding_box.max_y, 15);
        assert_eq!(piece.base.bounding_box.max_z, 1);

        // wrapping behavior
        let pos2 = BlockPos::new(i32::MAX, 0, 0);
        let bb2 = BoundingBox::new(i32::MAX, 0, 0, i32::MAX, 0, 0);
        let mut piece2 = PoolElementStructurePiece::new(
            StructureTemplateManager,
            StructurePoolElement::new("w"),
            pos2,
            0,
            Rotation::None,
            bb2,
            DEFAULT_LIQUID_SETTINGS,
        );
        piece2.move_by(1, 0, 0);
        assert_eq!(piece2.get_position().x, i32::MIN);
        assert_eq!(piece2.base.bounding_box.min_x, i32::MIN);
    }

    #[test]
    fn test_junctions_add_and_get() {
        let mut piece = PoolElementStructurePiece::new(
            StructureTemplateManager,
            StructurePoolElement::new("j"),
            BlockPos::new(0, 0, 0),
            0,
            Rotation::None,
            BoundingBox::new(0, 0, 0, 0, 0, 0),
            DEFAULT_LIQUID_SETTINGS,
        );
        assert!(piece.get_junctions().is_empty());
        piece.add_junction(JigsawJunction::new(1, 2, 3, -4, 7));
        piece.add_junction(JigsawJunction::new(-5, 6, 7, 8, 9));
        assert_eq!(piece.get_junctions().len(), 2);
        assert_eq!(piece.get_junctions()[0].source_x, 1);
        assert_eq!(piece.get_junctions()[1].delta_y, 8);
    }

    #[test]
    fn test_to_string_format() {
        let piece = PoolElementStructurePiece::new(
            StructureTemplateManager,
            StructurePoolElement::new("road_piece"),
            BlockPos::new(7, -3, 11),
            2,
            Rotation::Clockwise180,
            BoundingBox::new(0, 0, 0, 1, 1, 1),
            DEFAULT_LIQUID_SETTINGS,
        );
        let s = piece.to_string();
        assert!(s.contains("PoolElementStructurePiece"));
        assert!(s.contains("(7, -3, 11)"));
        assert!(s.contains("Clockwise180"));
        assert!(s.contains("road_piece"));
    }
}
