use std::sync::{Arc, LazyLock, OnceLock};

pub const INVALID_START_ID: &str = "INVALID";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub fn get_center(&self) -> BlockPos {
        BlockPos {
            x: self
                .min_x
                .wrapping_add(self.max_x.wrapping_sub(self.min_x).wrapping_add(1) / 2),
            y: self
                .min_y
                .wrapping_add(self.max_y.wrapping_sub(self.min_y).wrapping_add(1) / 2),
            z: self
                .min_z
                .wrapping_add(self.max_z.wrapping_sub(self.min_z).wrapping_add(1) / 2),
        }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.max_x >= other.min_x
            && self.min_x <= other.max_x
            && self.max_z >= other.min_z
            && self.min_z <= other.max_z
            && self.max_y >= other.min_y
            && self.min_y <= other.max_y
    }
}

#[derive(Clone, Debug, Default)]
pub struct ListTag {
    pub elements: Vec<CompoundTag>,
}

#[derive(Clone, Debug)]
pub enum TagValue {
    String(String),
    Int(i32),
    List(ListTag),
}

#[derive(Clone, Debug, Default)]
pub struct CompoundTag {
    pub values: std::collections::HashMap<String, TagValue>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self {
            values: std::collections::HashMap::new(),
        }
    }

    pub fn get_string_or(&self, key: &str, default: &str) -> String {
        match self.values.get(key) {
            Some(TagValue::String(s)) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        match self.values.get(key) {
            Some(TagValue::Int(v)) => *v,
            _ => default,
        }
    }

    pub fn get_list_or_empty(&self, key: &str) -> ListTag {
        match self.values.get(key) {
            Some(TagValue::List(l)) => l.clone(),
            _ => ListTag::default(),
        }
    }

    pub fn put_string(&mut self, key: &str, value: String) {
        self.values.insert(key.to_string(), TagValue::String(value));
    }

    pub fn put_int(&mut self, key: &str, value: i32) {
        self.values.insert(key.to_string(), TagValue::Int(value));
    }

    pub fn put(&mut self, key: &str, value: ListTag) {
        self.values.insert(key.to_string(), TagValue::List(value));
    }
}

pub trait WorldGenLevel: Send + Sync {}
pub trait StructureManager: Send + Sync {}
pub trait ChunkGenerator: Send + Sync {}
pub trait RandomSource: Send + Sync {}

pub trait StructurePiece: Send + Sync {
    fn bounding_box(&self) -> &BoundingBox;
    fn get_bounding_box(&self) -> &BoundingBox {
        self.bounding_box()
    }
    fn post_process(
        &mut self,
        level: &mut dyn WorldGenLevel,
        structure_manager: &dyn StructureManager,
        generator: &dyn ChunkGenerator,
        random: &mut dyn RandomSource,
        box_limit: &BoundingBox,
        chunk_pos: ChunkPos,
        pos: BlockPos,
    );
}

pub struct PiecesContainer {
    inner: Vec<Box<dyn StructurePiece>>,
}

impl PiecesContainer {
    pub fn new(pieces: Vec<Box<dyn StructurePiece>>) -> Self {
        Self { inner: pieces }
    }

    pub fn pieces(&self) -> &[Box<dyn StructurePiece>] {
        &self.inner
    }

    pub fn pieces_mut(&mut self) -> &mut [Box<dyn StructurePiece>] {
        &mut self.inner
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn calculate_bounding_box(&self) -> BoundingBox {
        if self.inner.is_empty() {
            return BoundingBox {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 0,
                max_y: 0,
                max_z: 0,
            };
        }
        let mut bb = *self.inner[0].bounding_box();
        for p in &self.inner[1..] {
            let b = p.bounding_box();
            if b.min_x < bb.min_x {
                bb.min_x = b.min_x;
            }
            if b.min_y < bb.min_y {
                bb.min_y = b.min_y;
            }
            if b.min_z < bb.min_z {
                bb.min_z = b.min_z;
            }
            if b.max_x > bb.max_x {
                bb.max_x = b.max_x;
            }
            if b.max_y > bb.max_y {
                bb.max_y = b.max_y;
            }
            if b.max_z > bb.max_z {
                bb.max_z = b.max_z;
            }
        }
        bb
    }

    pub fn load(
        list: &ListTag,
        ctx: &dyn StructurePieceSerializationContext,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pieces = ctx.load_pieces(list)?;
        Ok(Self::new(pieces))
    }

    pub fn save(&self, ctx: &dyn StructurePieceSerializationContext) -> ListTag {
        ctx.save_pieces(&self.inner)
    }
}

pub trait Structure: Send + Sync {
    fn adjust_bounding_box(&self, bb: BoundingBox) -> BoundingBox;
    fn after_place(
        &self,
        level: &mut dyn WorldGenLevel,
        structure_manager: &dyn StructureManager,
        generator: &dyn ChunkGenerator,
        random: &mut dyn RandomSource,
        box_limit: &BoundingBox,
        chunk_pos: ChunkPos,
        pieces: &PiecesContainer,
    );
    fn regenerate_pieces_after_load(
        &self,
        _chunk_pos: ChunkPos,
        _seed: i64,
        pieces: PiecesContainer,
    ) -> PiecesContainer {
        pieces
    }
}

pub trait StructurePieceSerializationContext: Send + Sync {
    fn lookup_structure(&self, id: &str) -> Option<Arc<dyn Structure>>;
    fn structure_key(&self, structure: &dyn Structure) -> Option<String>;
    fn load_pieces(
        &self,
        list: &ListTag,
    ) -> Result<Vec<Box<dyn StructurePiece>>, Box<dyn std::error::Error + Send + Sync>>;
    fn save_pieces(&self, pieces: &[Box<dyn StructurePiece>]) -> ListTag;
}

pub struct StructureStart {
    structure: Option<Arc<dyn Structure>>,
    piece_container: PiecesContainer,
    chunk_pos: ChunkPos,
    references: i32,
    cached_bounding_box: OnceLock<BoundingBox>,
}

pub static INVALID_START: LazyLock<StructureStart> = LazyLock::new(|| {
    StructureStart::new(None, ChunkPos::new(0, 0), 0, PiecesContainer::new(Vec::new()))
});

impl StructureStart {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#<init>
    pub fn new(
        structure: Option<Arc<dyn Structure>>,
        chunk_pos: ChunkPos,
        references: i32,
        piece_container: PiecesContainer,
    ) -> Self {
        Self {
            structure,
            piece_container,
            chunk_pos,
            references,
            cached_bounding_box: OnceLock::new(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#loadStaticStart
    pub fn load_static_start(
        ctx: &dyn StructurePieceSerializationContext,
        tag: &CompoundTag,
        seed: i64,
    ) -> Option<Self> {
        let id = tag.get_string_or("id", "");
        if INVALID_START_ID == id {
            return Some(Self::new(
                None,
                ChunkPos::new(0, 0),
                0,
                PiecesContainer::new(Vec::new()),
            ));
        }

        let structure = match ctx.lookup_structure(&id) {
            Some(s) => s,
            None => {
                eprintln!("Unknown stucture id: {}", id);
                return None;
            }
        };

        let chunk_pos = ChunkPos::new(tag.get_int_or("ChunkX", 0), tag.get_int_or("ChunkZ", 0));
        let references = tag.get_int_or("references", 0);
        let children = tag.get_list_or_empty("Children");

        match PiecesContainer::load(&children, ctx) {
            Ok(mut pieces) => {
                pieces = structure.regenerate_pieces_after_load(chunk_pos, seed, pieces);
                Some(Self::new(Some(structure), chunk_pos, references, pieces))
            }
            Err(e) => {
                eprintln!("Failed Start with id {}", id);
                eprintln!("{}", e);
                None
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#getBoundingBox
    pub fn get_bounding_box(&self) -> BoundingBox {
        *self.cached_bounding_box.get_or_init(|| {
            self.structure
                .as_ref()
                .expect("missing structure")
                .adjust_bounding_box(self.piece_container.calculate_bounding_box())
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#placeInChunk
    pub fn place_in_chunk(
        &mut self,
        level: &mut dyn WorldGenLevel,
        structure_manager: &dyn StructureManager,
        generator: &dyn ChunkGenerator,
        random: &mut dyn RandomSource,
        box_limit: &BoundingBox,
        chunk_pos: ChunkPos,
    ) {
        if self.piece_container.is_empty() {
            return;
        }

        let first_bbox = *self.piece_container.pieces()[0].bounding_box();
        let center = first_bbox.get_center();
        let anchor = BlockPos::new(center.x, first_bbox.min_y, center.z);

        let len = self.piece_container.pieces().len();
        for i in 0..len {
            let intersects = self.piece_container.pieces()[i]
                .get_bounding_box()
                .intersects(box_limit);
            if intersects {
                self.piece_container.pieces_mut()[i].post_process(
                    level,
                    structure_manager,
                    generator,
                    random,
                    box_limit,
                    chunk_pos,
                    anchor,
                );
            }
        }

        let structure = self.structure.clone();
        if let Some(structure) = structure {
            structure.after_place(
                level,
                structure_manager,
                generator,
                random,
                box_limit,
                chunk_pos,
                &self.piece_container,
            );
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#createTag
    pub fn create_tag(
        &self,
        ctx: &dyn StructurePieceSerializationContext,
        chunk_pos: ChunkPos,
    ) -> CompoundTag {
        let mut tag = CompoundTag::new();
        if self.is_valid() {
            let key = ctx
                .structure_key(self.structure.as_ref().expect("missing structure").as_ref())
                .expect("missing structure key");
            tag.put_string("id", key);
            tag.put_int("ChunkX", chunk_pos.x);
            tag.put_int("ChunkZ", chunk_pos.z);
            tag.put_int("references", self.references);
            tag.put("Children", self.piece_container.save(ctx));
            tag
        } else {
            tag.put_string("id", INVALID_START_ID.to_string());
            tag
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#isValid
    pub fn is_valid(&self) -> bool {
        !self.piece_container.is_empty()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#getChunkPos
    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#canBeReferenced
    pub fn can_be_referenced(&self) -> bool {
        self.references < self.get_max_references()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#addReference
    pub fn add_reference(&mut self) {
        self.references = self.references.wrapping_add(1);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#getReferences
    pub fn get_references(&self) -> i32 {
        self.references
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#getMaxReferences
    pub fn get_max_references(&self) -> i32 {
        1
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#getStructure
    pub fn get_structure(&self) -> Option<&Arc<dyn Structure>> {
        self.structure.as_ref()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureStart#getPieces
    pub fn get_pieces(&self) -> &[Box<dyn StructurePiece>] {
        self.piece_container.pieces()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPiece {
        bb: BoundingBox,
    }

    impl StructurePiece for MockPiece {
        fn bounding_box(&self) -> &BoundingBox {
            &self.bb
        }
        fn post_process(
            &mut self,
            _: &mut dyn WorldGenLevel,
            _: &dyn StructureManager,
            _: &dyn ChunkGenerator,
            _: &mut dyn RandomSource,
            _: &BoundingBox,
            _: ChunkPos,
            _: BlockPos,
        ) {
        }
    }

    struct MockStructure;

    impl Structure for MockStructure {
        fn adjust_bounding_box(&self, bb: BoundingBox) -> BoundingBox {
            bb
        }
        fn after_place(
            &self,
            _: &mut dyn WorldGenLevel,
            _: &dyn StructureManager,
            _: &dyn ChunkGenerator,
            _: &mut dyn RandomSource,
            _: &BoundingBox,
            _: ChunkPos,
            _: &PiecesContainer,
        ) {
        }
    }

    struct MockCtx;

    impl StructurePieceSerializationContext for MockCtx {
        fn lookup_structure(&self, id: &str) -> Option<Arc<dyn Structure>> {
            if id == "test:s" {
                Some(Arc::new(MockStructure))
            } else {
                None
            }
        }
        fn structure_key(&self, _: &dyn Structure) -> Option<String> {
            Some("test:s".to_string())
        }
        fn load_pieces(
            &self,
            _: &ListTag,
        ) -> Result<Vec<Box<dyn StructurePiece>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(vec![Box::new(MockPiece {
                bb: BoundingBox {
                    min_x: 1,
                    min_y: 2,
                    min_z: 3,
                    max_x: 4,
                    max_y: 5,
                    max_z: 6,
                },
            })])
        }
        fn save_pieces(&self, _: &[Box<dyn StructurePiece>]) -> ListTag {
            ListTag::default()
        }
    }

    #[test]
    fn invalid_start_and_validity() {
        let s = StructureStart::new(None, ChunkPos::new(0, 0), 0, PiecesContainer::new(vec![]));
        assert!(!s.is_valid());
        assert_eq!(s.get_references(), 0);
        assert_eq!(s.get_chunk_pos(), ChunkPos::new(0, 0));
        assert!(!s.can_be_referenced() == false); // references 0 < 1
        assert!(s.can_be_referenced());
    }

    #[test]
    fn references_increment_and_wrapping_overflow() {
        let mut s = StructureStart::new(
            Some(Arc::new(MockStructure)),
            ChunkPos::new(1, -2),
            i32::MAX,
            PiecesContainer::new(vec![Box::new(MockPiece {
                bb: BoundingBox {
                    min_x: 0,
                    min_y: 0,
                    min_z: 0,
                    max_x: 1,
                    max_y: 1,
                    max_z: 1,
                },
            })]),
        );
        assert!(!s.can_be_referenced());
        s.add_reference();
        assert_eq!(s.get_references(), i32::MIN);
        assert!(s.can_be_referenced());
    }

    #[test]
    fn bounding_box_center_negative_and_intersects_edge() {
        let bb = BoundingBox {
            min_x: -10,
            min_y: -20,
            min_z: -30,
            max_x: -2,
            max_y: -4,
            max_z: -6,
        };
        assert_eq!(bb.get_center(), BlockPos::new(-6, -12, -18));

        let a = BoundingBox {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 5,
            max_y: 5,
            max_z: 5,
        };
        let b = BoundingBox {
            min_x: 6,
            min_y: 0,
            min_z: 0,
            max_x: 10,
            max_y: 5,
            max_z: 5,
        };
        let c = BoundingBox {
            min_x: 5,
            min_y: 5,
            min_z: 5,
            max_x: 10,
            max_y: 10,
            max_z: 10,
        };
        assert!(!a.intersects(&b));
        assert!(a.intersects(&c));
    }

    #[test]
    fn create_tag_invalid_and_valid() {
        let ctx = MockCtx;
        let invalid = StructureStart::new(None, ChunkPos::new(0, 0), 0, PiecesContainer::new(vec![]));
        let tag = invalid.create_tag(&ctx, ChunkPos::new(1, 2));
        assert_eq!(tag.get_string_or("id", ""), "INVALID");

        let valid = StructureStart::new(
            Some(Arc::new(MockStructure)),
            ChunkPos::new(1, 2),
            3,
            PiecesContainer::new(vec![Box::new(MockPiece {
                bb: BoundingBox {
                    min_x: 0,
                    min_y: 0,
                    min_z: 0,
                    max_x: 0,
                    max_y: 0,
                    max_z: 0,
                },
            })]),
        );
        let tag = valid.create_tag(&ctx, ChunkPos::new(5, -6));
        assert_eq!(tag.get_string_or("id", ""), "test:s");
        assert_eq!(tag.get_int_or("ChunkX", -1), 5);
        assert_eq!(tag.get_int_or("ChunkZ", -1), -6);
        assert_eq!(tag.get_int_or("references", -1), 3);
    }

    #[test]
    fn load_static_start_paths() {
        let ctx = MockCtx;

        let mut invalid_tag = CompoundTag::new();
        invalid_tag.put_string("id", "INVALID".to_string());
        let s = StructureStart::load_static_start(&ctx, &invalid_tag, 0).unwrap();
        assert!(!s.is_valid());

        let mut unknown_tag = CompoundTag::new();
        unknown_tag.put_string("id", "minecraft:unknown".to_string());
        assert!(StructureStart::load_static_start(&ctx, &unknown_tag, 0).is_none());

        let mut valid_tag = CompoundTag::new();
        valid_tag.put_string("id", "test:s".to_string());
        valid_tag.put_int("ChunkX", 7);
        valid_tag.put_int("ChunkZ", -8);
        valid_tag.put_int("references", 2);
        let s = StructureStart::load_static_start(&ctx, &valid_tag, 42).unwrap();
        assert!(s.is_valid());
        assert_eq!(s.get_chunk_pos(), ChunkPos::new(7, -8));
        assert_eq!(s.get_references(), 2);
        assert_eq!(s.get_pieces().len(), 1);
        let bb = s.get_bounding_box();
        assert_eq!(bb, BoundingBox { min_x: 1, min_y: 2, min_z: 3, max_x: 4, max_y: 5, max_z: 6 });
    }

    #[test]
    fn float_bits_sanity() {
        // No float operations in StructureStart; verify canonical NaN bit patterns.
        assert_eq!(f32::NAN.to_bits() & 0x7FC0_0000, 0x7FC0_0000);
        assert_eq!(
            f64::NAN.to_bits() & 0x7FF8_0000_0000_0000,
            0x7FF8_0000_0000_0000
        );
    }
}
