//! Port of `net.minecraft.world.level.levelgen.structure.StructurePieceAccessor`.
//!
//! Defines the accessor interface used by structure generation to register pieces
//! and query for bounding-box collisions.

use std::fmt;

/// An axis-aligned integer bounding box.
///
/// Minimal local representation of
/// `net.minecraft.world.level.levelgen.structure.BoundingBox`,
/// provided to keep this module self-contained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    /// Creates a new bounding box from inclusive minimum and maximum coordinates.
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

    // JAVA: net.minecraft.world.level.levelgen.structure.BoundingBox#intersects
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.max_x >= other.min_x
            && self.min_x <= other.max_x
            && self.max_z >= other.min_z
            && self.min_z <= other.max_z
            && self.max_y >= other.min_y
            && self.min_y <= other.max_y
    }
}

/// Minimal local interface for a structure piece.
///
/// Represents `net.minecraft.world.level.levelgen.structure.StructurePiece`
/// sufficiently for collision queries performed through [`StructurePieceAccessor`].
pub trait StructurePiece: fmt::Debug {
    /// Returns the bounding box of this structure piece.
    fn bounding_box(&self) -> &BoundingBox;
}

/// Port of `net.minecraft.world.level.levelgen.structure.StructurePieceAccessor`.
///
/// Provides operations for registering structure pieces and finding pieces
/// that collide with a given bounding box.
pub trait StructurePieceAccessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePieceAccessor#addPiece
    fn add_piece(&mut self, piece: Box<dyn StructurePiece>);

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePieceAccessor#findCollisionPiece
    fn find_collision_piece(&self, bounding_box: &BoundingBox) -> Option<&dyn StructurePiece>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestPiece {
        bb: BoundingBox,
        id: i32,
    }

    impl TestPiece {
        fn new(bb: BoundingBox, id: i32) -> Self {
            Self { bb, id }
        }

        fn id(&self) -> i32 {
            self.id
        }
    }

    impl StructurePiece for TestPiece {
        fn bounding_box(&self) -> &BoundingBox {
            &self.bb
        }
    }

    #[derive(Debug, Default)]
    struct VecPieceAccessor {
        pieces: Vec<Box<dyn StructurePiece>>,
    }

    impl VecPieceAccessor {
        fn new() -> Self {
            Self { pieces: Vec::new() }
        }

        fn piece_count(&self) -> usize {
            self.pieces.len()
        }
    }

    impl StructurePieceAccessor for VecPieceAccessor {
        // JAVA: net.minecraft.world.level.levelgen.structure.StructurePieceAccessor#addPiece
        fn add_piece(&mut self, piece: Box<dyn StructurePiece>) {
            self.pieces.push(piece);
        }

        // JAVA: net.minecraft.world.level.levelgen.structure.StructurePieceAccessor#findCollisionPiece
        fn find_collision_piece(&self, bounding_box: &BoundingBox) -> Option<&dyn StructurePiece> {
            self.pieces
                .iter()
                .find(|piece| piece.bounding_box().intersects(bounding_box))
                .map(|piece| piece.as_ref())
        }
    }

    fn downcast_test_piece(piece: &dyn StructurePiece) -> Option<&TestPiece> {
        // Use Debug formatting as a lightweight identity check is not possible without Any;
        // instead, tests compare bounding boxes and known IDs through helper accessors.
        // This helper is intentionally unused when not needed.
        let _ = piece;
        None
    }

    fn find_test_piece_id(
        accessor: &dyn StructurePieceAccessor,
        query: &BoundingBox,
        candidates: &[TestPiece],
    ) -> Option<i32> {
        let found_bb = accessor.find_collision_piece(query)?.bounding_box();
        candidates
            .iter()
            .find(|candidate| candidate.bounding_box() == found_bb)
            .map(TestPiece::id)
    }

    #[test]
    fn add_piece_and_find_collision() {
        let mut accessor = VecPieceAccessor::new();
        let piece = TestPiece::new(BoundingBox::new(0, 0, 0, 10, 10, 10), 1);
        let expected_bb = *piece.bounding_box();
        accessor.add_piece(Box::new(piece));

        assert_eq!(accessor.piece_count(), 1);

        let query = BoundingBox::new(5, 5, 5, 15, 15, 15);
        let found = accessor.find_collision_piece(&query);
        assert!(found.is_some());
        assert_eq!(*found.unwrap().bounding_box(), expected_bb);
    }

    #[test]
    fn find_collision_piece_returns_none_when_empty() {
        let accessor = VecPieceAccessor::default();
        let query = BoundingBox::new(0, 0, 0, 1, 1, 1);
        assert!(accessor.find_collision_piece(&query).is_none());
    }

    #[test]
    fn find_collision_piece_returns_none_when_no_intersection() {
        let mut accessor = VecPieceAccessor::new();
        accessor.add_piece(Box::new(TestPiece::new(
            BoundingBox::new(0, 0, 0, 10, 10, 10),
            1,
        )));

        let query = BoundingBox::new(20, 20, 20, 30, 30, 30);
        assert!(accessor.find_collision_piece(&query).is_none());
    }

    #[test]
    fn touching_edges_intersects_like_java() {
        let mut accessor = VecPieceAccessor::new();
        accessor.add_piece(Box::new(TestPiece::new(
            BoundingBox::new(0, 0, 0, 10, 10, 10),
            1,
        )));

        // Touching at max_x == min_x boundary should intersect (Java uses >= / <=).
        let touching = BoundingBox::new(10, 0, 0, 20, 10, 10);
        assert!(accessor.find_collision_piece(&touching).is_some());
    }

    #[test]
    fn separated_by_one_does_not_intersect() {
        let mut accessor = VecPieceAccessor::new();
        accessor.add_piece(Box::new(TestPiece::new(
            BoundingBox::new(0, 0, 0, 10, 10, 10),
            1,
        )));

        let separated = BoundingBox::new(11, 0, 0, 20, 10, 10);
        assert!(accessor.find_collision_piece(&separated).is_none());
    }

    #[test]
    fn negative_coordinates_collision() {
        let mut accessor = VecPieceAccessor::new();
        let bb = BoundingBox::new(-20, -64, -100, -10, -1, -50);
        accessor.add_piece(Box::new(TestPiece::new(bb, 7)));

        let query = BoundingBox::new(-15, -30, -75, -5, 0, -25);
        let found = accessor.find_collision_piece(&query);
        assert!(found.is_some());
        assert_eq!(*found.unwrap().bounding_box(), bb);
    }

    #[test]
    fn negative_coordinates_no_collision() {
        let mut accessor = VecPieceAccessor::new();
        accessor.add_piece(Box::new(TestPiece::new(
            BoundingBox::new(-10, -10, -10, -1, -1, -1),
            1,
        )));

        let query = BoundingBox::new(0, 0, 0, 10, 10, 10);
        assert!(accessor.find_collision_piece(&query).is_none());
    }

    #[test]
    fn multiple_pieces_finds_colliding_piece() {
        let p1 = TestPiece::new(BoundingBox::new(0, 0, 0, 10, 10, 10), 1);
        let p2 = TestPiece::new(BoundingBox::new(100, 100, 100, 110, 110, 110), 2);
        let p3 = TestPiece::new(BoundingBox::new(200, 200, 200, 210, 210, 210), 3);
        let candidates = vec![p1.clone(), p2.clone(), p3.clone()];

        let mut accessor = VecPieceAccessor::new();
        accessor.add_piece(Box::new(p1));
        accessor.add_piece(Box::new(p2));
        accessor.add_piece(Box::new(p3));

        let query = BoundingBox::new(105, 105, 105, 106, 106, 106);
        let found_id = find_test_piece_id(&accessor, &query, &candidates);
        assert_eq!(found_id, Some(2));
    }

    #[test]
    fn multiple_pieces_no_collision() {
        let mut accessor = VecPieceAccessor::new();
        for i in 0..5_i32 {
            let base = i.wrapping_mul(100);
            accessor.add_piece(Box::new(TestPiece::new(
                BoundingBox::new(base, 0, 0, base.wrapping_add(10), 10, 10),
                i,
            )));
        }

        let query = BoundingBox::new(50, 0, 0, 60, 10, 10);
        assert!(accessor.find_collision_piece(&query).is_none());
    }

    #[test]
    fn extreme_i32_coordinates_intersection() {
        let mut accessor = VecPieceAccessor::new();
        let bb = BoundingBox::new(i32::MIN, i32::MIN, i32::MIN, i32::MAX, i32::MAX, i32::MAX);
        accessor.add_piece(Box::new(TestPiece::new(bb, 1)));

        let query = BoundingBox::new(0, 0, 0, 1, 1, 1);
        let found = accessor.find_collision_piece(&query);
        assert!(found.is_some());
        assert_eq!(*found.unwrap().bounding_box(), bb);
    }

    #[test]
    fn extreme_i32_boundary_no_overflow_in_comparisons() {
        let mut accessor = VecPieceAccessor::new();
        let min_piece = BoundingBox::new(i32::MIN, -64, i32::MIN, i32::MIN + 10, 320, i32::MIN + 10);
        let max_piece = BoundingBox::new(i32::MAX - 10, -64, i32::MAX - 10, i32::MAX, 320, i32::MAX);
        accessor.add_piece(Box::new(TestPiece::new(min_piece, 1)));
        accessor.add_piece(Box::new(TestPiece::new(max_piece, 2)));

        let query_min = BoundingBox::new(i32::MIN, 0, i32::MIN, i32::MIN + 5, 5, i32::MIN + 5);
        let found_min = accessor.find_collision_piece(&query_min);
        assert!(found_min.is_some());
        assert_eq!(*found_min.unwrap().bounding_box(), min_piece);

        let query_max = BoundingBox::new(i32::MAX - 5, 0, i32::MAX - 5, i32::MAX, 5, i32::MAX);
        let found_max = accessor.find_collision_piece(&query_max);
        assert!(found_max.is_some());
        assert_eq!(*found_max.unwrap().bounding_box(), max_piece);

        let middle_query = BoundingBox::new(-5, 0, -5, 5, 5, 5);
        assert!(accessor.find_collision_piece(&middle_query).is_none());
    }

    #[test]
    fn integer_precision_preserved_no_float_bits_loss() {
        // 2^24 + 1 is not exactly representable in f32; integer coordinates must be preserved.
        let precise_val: i32 = 16_777_217;
        let neg_precise_val: i32 = -16_777_217;
        let bb = BoundingBox::new(
            precise_val,
            neg_precise_val,
            precise_val,
            precise_val.wrapping_add(1),
            neg_precise_val.wrapping_add(1),
            precise_val.wrapping_add(1),
        );
        let mut accessor = VecPieceAccessor::new();
        accessor.add_piece(Box::new(TestPiece::new(bb, 99)));

        let query = BoundingBox::new(
            precise_val,
            neg_precise_val,
            precise_val,
            precise_val,
            neg_precise_val,
            precise_val,
        );
        let found = accessor
            .find_collision_piece(&query)
            .expect("collision should be found");
        let found_bb = found.bounding_box();
        assert_eq!(found_bb.min_x, precise_val);
        assert_eq!(found_bb.min_y, neg_precise_val);
        assert_eq!(found_bb.min_z, precise_val);
        assert_eq!(found_bb.max_x, precise_val.wrapping_add(1));
        assert_eq!(found_bb.max_y, neg_precise_val.wrapping_add(1));
        assert_eq!(found_bb.max_z, precise_val.wrapping_add(1));

        // Verify exact integer bit patterns are preserved.
        assert_eq!(found_bb.min_x.to_be_bytes(), precise_val.to_be_bytes());
        assert_eq!(found_bb.min_y.to_be_bytes(), neg_precise_val.to_be_bytes());
    }

    #[test]
    fn bounding_box_intersects_self() {
        let bb = BoundingBox::new(1, 2, 3, 4, 5, 6);
        assert!(bb.intersects(&bb));
    }

    #[test]
    fn bounding_box_intersects_contained_and_symmetric() {
        let outer = BoundingBox::new(0, 0, 0, 100, 100, 100);
        let inner = BoundingBox::new(10, 10, 10, 20, 20, 20);
        assert!(outer.intersects(&inner));
        assert!(inner.intersects(&outer));
    }

    #[test]
    fn bounding_box_no_intersection_on_single_axis_gap() {
        let a = BoundingBox::new(0, 0, 0, 10, 10, 10);
        let gap_y = BoundingBox::new(0, 11, 0, 10, 20, 10);
        let gap_x = BoundingBox::new(11, 0, 0, 20, 10, 10);
        let gap_z = BoundingBox::new(0, 0, 11, 10, 10, 20);

        assert!(!a.intersects(&gap_y));
        assert!(!gap_y.intersects(&a));
        assert!(!a.intersects(&gap_x));
        assert!(!a.intersects(&gap_z));
    }

    #[test]
    fn bounding_box_negative_and_extreme_boundary_intersection() {
        let a = BoundingBox::new(i32::MIN, i32::MIN, i32::MIN, -1, -1, -1);
        let b = BoundingBox::new(0, 0, 0, i32::MAX, i32::MAX, i32::MAX);
        assert!(!a.intersects(&b));

        let touching_at_zero = BoundingBox::new(i32::MIN, i32::MIN, i32::MIN, 0, 0, 0);
        assert!(touching_at_zero.intersects(&b));
        assert!(b.intersects(&touching_at_zero));
    }

    #[test]
    fn inverted_bounding_box_uses_java_comparison_semantics() {
        let inverted = BoundingBox::new(10, 10, 10, 0, 0, 0);
        let normal = BoundingBox::new(0, 0, 0, 10, 10, 10);

        let expected = inverted.max_x >= normal.min_x
            && inverted.min_x <= normal.max_x
            && inverted.max_z >= normal.min_z
            && inverted.min_z <= normal.max_z
            && inverted.max_y >= normal.min_y
            && inverted.min_y <= normal.max_y;

        assert_eq!(inverted.intersects(&normal), expected);
    }

    #[test]
    fn accessor_trait_object_is_usable() {
        let mut accessor: Box<dyn StructurePieceAccessor> = Box::new(VecPieceAccessor::default());
        accessor.add_piece(Box::new(TestPiece::new(
            BoundingBox::new(0, 0, 0, 1, 1, 1),
            42,
        )));

        let found = accessor.find_collision_piece(&BoundingBox::new(0, 0, 0, 1, 1, 1));
        assert!(found.is_some());
        assert_eq!(*found.unwrap().bounding_box(), BoundingBox::new(0, 0, 0, 1, 1, 1));
    }

    #[test]
    fn add_piece_preserves_all_inserted_pieces() {
        let mut accessor = VecPieceAccessor::new();
        let boxes = [
            BoundingBox::new(0, 0, 0, 1, 1, 1),
            BoundingBox::new(10, 10, 10, 11, 11, 11),
            BoundingBox::new(-10, -10, -10, -9, -9, -9),
        ];

        for (i, bb) in boxes.iter().copied().enumerate() {
            accessor.add_piece(Box::new(TestPiece::new(bb, i as i32)));
        }

        assert_eq!(accessor.piece_count(), boxes.len());
        for bb in boxes {
            assert!(accessor.find_collision_piece(&bb).is_some());
        }
    }

    #[test]
    fn bounding_box_copy_eq_hash_contract() {
        use std::collections::HashSet;

        let a = BoundingBox::new(1, 2, 3, 4, 5, 6);
        let b = a; // Copy
        let c = BoundingBox::new(1, 2, 3, 4, 5, 6);
        assert_eq!(a, b);
        assert_eq!(a, c);

        let mut set = HashSet::new();
        set.insert(a);
        set.insert(c);
        assert_eq!(set.len(), 1);
        assert!(set.contains(&BoundingBox::new(1, 2, 3, 4, 5, 6)));
    }

    // Silence dead_code warning for the intentionally unused helper kept for clarity.
    #[test]
    fn downcast_helper_is_safe() {
        let piece = TestPiece::new(BoundingBox::new(0, 0, 0, 1, 1, 1), 1);
        let result = downcast_test_piece(&piece);
        assert!(result.is_none());
    }
}
