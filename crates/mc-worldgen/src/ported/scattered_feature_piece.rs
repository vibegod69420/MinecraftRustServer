use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub fn is_inside(&self, pos: &MutableBlockPos) -> bool {
        pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.y >= self.min_y
            && pos.y <= self.max_y
            && pos.z >= self.min_z
            && pos.z <= self.max_z
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, dz: i32) {
        self.min_x = self.min_x.wrapping_add(dx);
        self.min_y = self.min_y.wrapping_add(dy);
        self.min_z = self.min_z.wrapping_add(dz);
        self.max_x = self.max_x.wrapping_add(dx);
        self.max_y = self.max_y.wrapping_add(dy);
        self.max_z = self.max_z.wrapping_add(dz);
    }

    pub fn min_x(&self) -> i32 {
        self.min_x
    }
    pub fn min_y(&self) -> i32 {
        self.min_y
    }
    pub fn min_z(&self) -> i32 {
        self.min_z
    }
    pub fn max_x(&self) -> i32 {
        self.max_x
    }
    pub fn max_y(&self) -> i32 {
        self.max_y
    }
    pub fn max_z(&self) -> i32 {
        self.max_z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    MotionBlockingNoLeaves,
}

pub trait LevelAccessor {
    fn get_heightmap_pos(&self, heightmap: HeightmapType, pos: &MutableBlockPos) -> BlockPos;
    fn get_max_y(&self) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StructurePieceType(pub u32);

#[derive(Debug, Clone, Copy, Default)]
pub struct StructurePieceSerializationContext;

#[derive(Debug, Clone, Default)]
pub struct CompoundTag {
    ints: HashMap<String, i32>,
}

impl CompoundTag {
    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        self.ints.get(key).copied().unwrap_or(default)
    }

    pub fn put_int(&mut self, key: &str, value: i32) {
        self.ints.insert(key.to_owned(), value);
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#makeBoundingBox
pub fn make_bounding_box(
    x: i32,
    y: i32,
    z: i32,
    orientation: Direction,
    width: i32,
    height: i32,
    depth: i32,
) -> BoundingBox {
    match orientation {
        Direction::North | Direction::South => BoundingBox {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x.wrapping_add(width).wrapping_sub(1),
            max_y: y.wrapping_add(height).wrapping_sub(1),
            max_z: z.wrapping_add(depth).wrapping_sub(1),
        },
        Direction::West | Direction::East => BoundingBox {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x.wrapping_add(depth).wrapping_sub(1),
            max_y: y.wrapping_add(height).wrapping_sub(1),
            max_z: z.wrapping_add(width).wrapping_sub(1),
        },
        _ => BoundingBox {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x.wrapping_add(width).wrapping_sub(1),
            max_y: y.wrapping_add(height).wrapping_sub(1),
            max_z: z.wrapping_add(depth).wrapping_sub(1),
        },
    }
}

#[derive(Debug, Clone)]
pub struct StructurePieceBase {
    pub piece_type: StructurePieceType,
    pub gen_depth: i32,
    pub bounding_box: BoundingBox,
    pub orientation: Option<Direction>,
}

impl StructurePieceBase {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#<init>
    pub fn new(piece_type: StructurePieceType, gen_depth: i32, bounding_box: BoundingBox) -> Self {
        Self {
            piece_type,
            gen_depth,
            bounding_box,
            orientation: None,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#<init>
    pub fn from_nbt(piece_type: StructurePieceType, _tag: &CompoundTag) -> Self {
        Self {
            piece_type,
            gen_depth: 0,
            bounding_box: BoundingBox::default(),
            orientation: None,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#setOrientation
    pub fn set_orientation(&mut self, orientation: Direction) {
        self.orientation = Some(orientation);
    }
}

#[derive(Debug, Clone)]
pub struct ScatteredFeaturePiece {
    pub base: StructurePieceBase,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub height_position: i32,
}

impl ScatteredFeaturePiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece#<init>
    pub fn new(
        piece_type: StructurePieceType,
        x: i32,
        y: i32,
        z: i32,
        width: i32,
        height: i32,
        depth: i32,
        orientation: Direction,
    ) -> Self {
        let mut base = StructurePieceBase::new(
            piece_type,
            0,
            make_bounding_box(x, y, z, orientation, width, height, depth),
        );
        base.set_orientation(orientation);
        Self {
            base,
            width,
            height,
            depth,
            height_position: -1,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece#<init>
    pub fn from_nbt(piece_type: StructurePieceType, tag: &CompoundTag) -> Self {
        let base = StructurePieceBase::from_nbt(piece_type, tag);
        Self {
            base,
            width: tag.get_int_or("Width", 0),
            height: tag.get_int_or("Height", 0),
            depth: tag.get_int_or("Depth", 0),
            height_position: tag.get_int_or("HPos", 0),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece#addAdditionalSaveData
    pub fn add_additional_save_data(
        &self,
        _context: &StructurePieceSerializationContext,
        tag: &mut CompoundTag,
    ) {
        tag.put_int("Width", self.width);
        tag.put_int("Height", self.height);
        tag.put_int("Depth", self.depth);
        tag.put_int("HPos", self.height_position);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece#updateAverageGroundHeight
    pub fn update_average_ground_height(
        &mut self,
        level: &dyn LevelAccessor,
        bb: &BoundingBox,
        offset: i32,
    ) -> bool {
        if self.height_position >= 0 {
            return true;
        }

        let mut total: i32 = 0;
        let mut count: i32 = 0;
        let mut pos = MutableBlockPos::new();

        let min_z = self.base.bounding_box.min_z();
        let max_z = self.base.bounding_box.max_z();
        let mut z = min_z;
        while z <= max_z {
            let min_x = self.base.bounding_box.min_x();
            let max_x = self.base.bounding_box.max_x();
            let mut x = min_x;
            while x <= max_x {
                pos.set(x, 64, z);
                if bb.is_inside(&pos) {
                    let height = level
                        .get_heightmap_pos(HeightmapType::MotionBlockingNoLeaves, &pos)
                        .get_y();
                    total = total.wrapping_add(height);
                    count = count.wrapping_add(1);
                }
                if x == max_x {
                    break;
                }
                x = x.wrapping_add(1);
            }
            if z == max_z {
                break;
            }
            z = z.wrapping_add(1);
        }

        if count == 0 {
            return false;
        }

        self.height_position = total / count;
        let dy = self
            .height_position
            .wrapping_sub(self.base.bounding_box.min_y())
            .wrapping_add(offset);
        self.base.bounding_box.move_by(0, dy, 0);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece#updateHeightPositionToLowestGroundHeight
    pub fn update_height_position_to_lowest_ground_height(
        &mut self,
        level: &dyn LevelAccessor,
        offset: i32,
    ) -> bool {
        if self.height_position >= 0 {
            return true;
        }

        let mut min_y: i32 = level.get_max_y().wrapping_add(1);
        let mut found = false;
        let mut pos = MutableBlockPos::new();

        let min_z = self.base.bounding_box.min_z();
        let max_z = self.base.bounding_box.max_z();
        let mut z = min_z;
        while z <= max_z {
            let min_x = self.base.bounding_box.min_x();
            let max_x = self.base.bounding_box.max_x();
            let mut x = min_x;
            while x <= max_x {
                pos.set(x, 0, z);
                let height = level
                    .get_heightmap_pos(HeightmapType::MotionBlockingNoLeaves, &pos)
                    .get_y();
                min_y = min_y.min(height);
                found = true;
                if x == max_x {
                    break;
                }
                x = x.wrapping_add(1);
            }
            if z == max_z {
                break;
            }
            z = z.wrapping_add(1);
        }

        if !found {
            return false;
        }

        self.height_position = min_y;
        let dy = self
            .height_position
            .wrapping_sub(self.base.bounding_box.min_y())
            .wrapping_add(offset);
        self.base.bounding_box.move_by(0, dy, 0);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        heights: HashMap<(i32, i32), i32>,
        max_y: i32,
    }

    impl MockLevel {
        fn new(max_y: i32) -> Self {
            Self {
                heights: HashMap::new(),
                max_y,
            }
        }

        fn with_height(mut self, x: i32, z: i32, y: i32) -> Self {
            self.heights.insert((x, z), y);
            self
        }
    }

    impl LevelAccessor for MockLevel {
        fn get_heightmap_pos(&self, _heightmap: HeightmapType, pos: &MutableBlockPos) -> BlockPos {
            let y = self.heights.get(&(pos.x, pos.z)).copied().unwrap_or(64);
            BlockPos::new(pos.x, y, pos.z)
        }

        fn get_max_y(&self) -> i32 {
            self.max_y
        }
    }

    #[test]
    fn constructor_and_save_data() {
        let piece =
            ScatteredFeaturePiece::new(StructurePieceType(1), 10, 20, 30, 5, 6, 7, Direction::North);
        assert_eq!(piece.width, 5);
        assert_eq!(piece.height, 6);
        assert_eq!(piece.depth, 7);
        assert_eq!(piece.height_position, -1);
        assert_eq!(piece.base.orientation, Some(Direction::North));
        assert_eq!(piece.base.gen_depth, 0);

        let mut tag = CompoundTag::default();
        let ctx = StructurePieceSerializationContext;
        piece.add_additional_save_data(&ctx, &mut tag);
        assert_eq!(tag.get_int_or("Width", -1), 5);
        assert_eq!(tag.get_int_or("Height", -1), 6);
        assert_eq!(tag.get_int_or("Depth", -1), 7);
        assert_eq!(tag.get_int_or("HPos", -1), -1);
    }

    #[test]
    fn from_nbt_reads_fields_with_defaults() {
        let mut tag = CompoundTag::default();
        tag.put_int("Width", 11);
        tag.put_int("Height", 12);
        tag.put_int("Depth", 13);
        tag.put_int("HPos", 5);
        let piece = ScatteredFeaturePiece::from_nbt(StructurePieceType(2), &tag);
        assert_eq!(piece.width, 11);
        assert_eq!(piece.height, 12);
        assert_eq!(piece.depth, 13);
        assert_eq!(piece.height_position, 5);

        let empty = CompoundTag::default();
        let piece2 = ScatteredFeaturePiece::from_nbt(StructurePieceType(3), &empty);
        assert_eq!(piece2.width, 0);
        assert_eq!(piece2.height, 0);
        assert_eq!(piece2.depth, 0);
        assert_eq!(piece2.height_position, 0);
    }

    #[test]
    fn update_average_ground_height_already_set_returns_true() {
        let mut piece =
            ScatteredFeaturePiece::new(StructurePieceType(0), 0, 0, 0, 3, 3, 3, Direction::North);
        piece.height_position = 10;
        let level = MockLevel::new(255);
        let bb = BoundingBox {
            min_x: -100,
            min_y: -100,
            min_z: -100,
            max_x: 100,
            max_y: 100,
            max_z: 100,
        };
        assert!(piece.update_average_ground_height(&level, &bb, 0));
        assert_eq!(piece.height_position, 10);
    }

    #[test]
    fn update_average_ground_height_computes_average_and_moves_box() {
        let mut piece =
            ScatteredFeaturePiece::new(StructurePieceType(0), 0, 0, 0, 2, 3, 2, Direction::North);
        let level = MockLevel::new(255)
            .with_height(0, 0, 64)
            .with_height(1, 0, 68)
            .with_height(0, 1, 72)
            .with_height(1, 1, 76);
        let bb = BoundingBox {
            min_x: -10,
            min_y: -10,
            min_z: -10,
            max_x: 10,
            max_y: 300,
            max_z: 10,
        };
        let old_min_y = piece.base.bounding_box.min_y();
        assert!(piece.update_average_ground_height(&level, &bb, 1));
        assert_eq!(piece.height_position, 70); // (64+68+72+76)/4
        let expected_dy = 70_i32.wrapping_sub(old_min_y).wrapping_add(1);
        assert_eq!(piece.base.bounding_box.min_y(), old_min_y.wrapping_add(expected_dy));
    }

    #[test]
    fn update_average_ground_height_returns_false_when_no_positions_inside() {
        let mut piece =
            ScatteredFeaturePiece::new(StructurePieceType(0), 0, 0, 0, 2, 3, 2, Direction::North);
        let level = MockLevel::new(255);
        let bb = BoundingBox {
            min_x: 100,
            min_y: 100,
            min_z: 100,
            max_x: 200,
            max_y: 200,
            max_z: 200,
        };
        assert!(!piece.update_average_ground_height(&level, &bb, 0));
        assert_eq!(piece.height_position, -1);
    }

    #[test]
    fn update_height_position_to_lowest_ground_height_with_negative_offset() {
        let mut piece =
            ScatteredFeaturePiece::new(StructurePieceType(0), 0, 5, 0, 2, 3, 2, Direction::North);
        let level = MockLevel::new(255)
            .with_height(0, 0, 70)
            .with_height(1, 0, 65)
            .with_height(0, 1, 68)
            .with_height(1, 1, 72);
        let old_min_y = piece.base.bounding_box.min_y();
        assert!(piece.update_height_position_to_lowest_ground_height(&level, -2));
        assert_eq!(piece.height_position, 65);
        let expected_dy = 65_i32.wrapping_sub(old_min_y).wrapping_add(-2);
        assert_eq!(piece.base.bounding_box.min_y(), old_min_y.wrapping_add(expected_dy));
    }
}
