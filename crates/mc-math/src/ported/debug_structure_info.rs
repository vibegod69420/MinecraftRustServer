/// Minimal self-contained placeholder for `net.minecraft.world.level.levelgen.structure.BoundingBox`.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Faithful port of `net.minecraft.util.debug.DebugStructureInfo.Piece`.
/// STREAM_CODEC is omitted because the Minecraft networking codec framework is not ported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugStructurePiece {
    pub bounding_box: BoundingBox,
    pub is_start: bool,
}

impl DebugStructurePiece {
    // JAVA: net.minecraft.util.debug.DebugStructureInfo$Piece#<init>
    pub fn new(bounding_box: BoundingBox, is_start: bool) -> Self {
        Self {
            bounding_box,
            is_start,
        }
    }
}

/// Faithful port of `net.minecraft.util.debug.DebugStructureInfo`.
/// STREAM_CODEC is omitted because the Minecraft networking codec framework is not ported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugStructureInfo {
    pub bounding_box: BoundingBox,
    pub pieces: Vec<DebugStructurePiece>,
}

impl DebugStructureInfo {
    // JAVA: net.minecraft.util.debug.DebugStructureInfo#<init>
    pub fn new(bounding_box: BoundingBox, pieces: Vec<DebugStructurePiece>) -> Self {
        Self {
            bounding_box,
            pieces,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bb(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> BoundingBox {
        BoundingBox::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    #[test]
    fn piece_new_stores_fields() {
        let piece = DebugStructurePiece::new(bb(0, 1, 2, 3, 4, 5), true);
        assert_eq!(piece.bounding_box, bb(0, 1, 2, 3, 4, 5));
        assert!(piece.is_start);
    }

    #[test]
    fn info_new_stores_bounding_box_and_pieces() {
        let pieces = vec![
            DebugStructurePiece::new(bb(0, 0, 0, 1, 1, 1), true),
            DebugStructurePiece::new(bb(2, 0, 0, 3, 1, 1), false),
        ];
        let info = DebugStructureInfo::new(bb(-1, -2, -3, 10, 20, 30), pieces.clone());
        assert_eq!(info.bounding_box, bb(-1, -2, -3, 10, 20, 30));
        assert_eq!(info.pieces, pieces);
    }

    #[test]
    fn info_with_empty_pieces() {
        let info = DebugStructureInfo::new(bb(0, 0, 0, 0, 0, 0), vec![]);
        assert!(info.pieces.is_empty());
    }

    #[test]
    fn bounding_box_handles_negative_coordinates() {
        let piece = DebugStructurePiece::new(bb(-100, -64, -200, -50, 0, -150), false);
        assert_eq!(piece.bounding_box.min_x, -100);
        assert_eq!(piece.bounding_box.max_y, 0);
        assert!(!piece.is_start);
    }
}
