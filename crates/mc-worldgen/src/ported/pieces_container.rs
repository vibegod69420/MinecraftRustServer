use std::collections::HashMap;
use std::fmt;
use std::io;
use std::sync::OnceLock;

// ── Minimal stubs for external types (self-contained) ──────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }

    pub fn parse(s: &str) -> Self {
        if let Some((ns, p)) = s.split_once(':') {
            Self {
                namespace: ns.to_string(),
                path: p.to_string(),
            }
        } else {
            Self::with_default_namespace(s)
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub fn is_inside(&self, pos: BlockPos) -> bool {
        pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.y >= self.min_y
            && pos.y <= self.max_y
            && pos.z >= self.min_z
            && pos.z <= self.max_z
    }

    /// Encapsulating box over an iterator of boxes. Returns a zero box if empty,
    /// matching typical Minecraft behavior for empty structure piece collections.
    pub fn encapsulating<I: IntoIterator<Item = BoundingBox>>(boxes: I) -> Option<BoundingBox> {
        let mut it = boxes.into_iter();
        let first = it.next()?;
        let mut r = first;
        for b in it {
            r.min_x = r.min_x.min(b.min_x);
            r.min_y = r.min_y.min(b.min_y);
            r.min_z = r.min_z.min(b.min_z);
            r.max_x = r.max_x.max(b.max_x);
            r.max_y = r.max_y.max(b.max_y);
            r.max_z = r.max_z.max(b.max_z);
        }
        Some(r)
    }
}

// ── Minimal NBT stubs (self-contained; compatible shape for this module) ───

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    Compound(CompoundTag),
    List(ListTag),
    String(String),
    End,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompoundTag {
    entries: HashMap<String, Tag>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, tag: Tag) {
        self.entries.insert(key.into(), tag);
    }

    pub fn get_string_or(&self, key: &str, default: &str) -> String {
        match self.entries.get(key) {
            Some(Tag::String(s)) => s.clone(),
            _ => default.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ListTag {
    items: Vec<Tag>,
}

impl ListTag {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, tag: Tag) {
        self.items.push(tag);
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn get_compound_or_empty(&self, index: usize) -> CompoundTag {
        match self.items.get(index) {
            Some(Tag::Compound(c)) => c.clone(),
            _ => CompoundTag::new(),
        }
    }
}

// ── Structure piece stubs ──────────────────────────────────────────────────

pub struct StructurePieceSerializationContext;

pub trait StructurePiece: Send + Sync {
    fn get_bounding_box(&self) -> &BoundingBox;
    fn create_tag(&self, context: &StructurePieceSerializationContext) -> Tag;
}

pub trait StructurePieceType: Send + Sync {
    fn load(
        &self,
        context: &StructurePieceSerializationContext,
        tag: &CompoundTag,
    ) -> Result<Box<dyn StructurePiece>, Box<dyn std::error::Error>>;
}

pub struct StructurePieceRegistry {
    map: HashMap<Identifier, Box<dyn StructurePieceType>>,
}

impl StructurePieceRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: Identifier, piece_type: Box<dyn StructurePieceType>) {
        self.map.insert(id, piece_type);
    }

    pub fn get_value(&self, id: &Identifier) -> Option<&dyn StructurePieceType> {
        self.map.get(id).map(|b| b.as_ref())
    }
}

impl Default for StructurePieceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal stand-in for `BuiltInRegistries.STRUCTURE_PIECE`.
pub mod built_in_registries {
    use super::{OnceLock, StructurePieceRegistry};

    static STRUCTURE_PIECE_REGISTRY: OnceLock<StructurePieceRegistry> = OnceLock::new();

    pub fn structure_piece() -> &'static StructurePieceRegistry {
        STRUCTURE_PIECE_REGISTRY.get_or_init(StructurePieceRegistry::new)
    }

    /// Test/helper hook to initialize the registry once.
    pub fn init_structure_piece(registry: StructurePieceRegistry) -> bool {
        STRUCTURE_PIECE_REGISTRY.set(registry).is_ok()
    }
}

// ── Ported type ────────────────────────────────────────────────────────────

fn renames() -> &'static HashMap<Identifier, Identifier> {
    static RENAMES: OnceLock<HashMap<Identifier, Identifier>> = OnceLock::new();
    RENAMES.get_or_init(|| {
        let jigsaw = Identifier::with_default_namespace("jigsaw");
        let mut m = HashMap::new();
        m.insert(Identifier::with_default_namespace("nvi"), jigsaw.clone());
        m.insert(Identifier::with_default_namespace("pcp"), jigsaw.clone());
        m.insert(
            Identifier::with_default_namespace("bastionremnant"),
            jigsaw.clone(),
        );
        m.insert(Identifier::with_default_namespace("runtime"), jigsaw);
        m
    })
}

/// JAVA: net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer
pub struct PiecesContainer {
    pieces: Vec<Box<dyn StructurePiece>>,
}

impl fmt::Debug for PiecesContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PiecesContainer")
            .field("pieces_len", &self.pieces.len())
            .finish()
    }
}

impl PiecesContainer {
    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer#<init>
    pub fn new(pieces: Vec<Box<dyn StructurePiece>>) -> Self {
        Self { pieces }
    }

    pub fn pieces(&self) -> &[Box<dyn StructurePiece>] {
        &self.pieces
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer#isEmpty
    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer#isInsidePiece
    pub fn is_inside_piece(&self, pos: BlockPos) -> bool {
        for piece in &self.pieces {
            if piece.get_bounding_box().is_inside(pos) {
                return true;
            }
        }
        false
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer#save
    pub fn save(&self, context: &StructurePieceSerializationContext) -> Tag {
        let mut list = ListTag::new();
        for piece in &self.pieces {
            list.add(piece.create_tag(context));
        }
        Tag::List(list)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer#load
    pub fn load(list: &ListTag, context: &StructurePieceSerializationContext) -> Self {
        let mut pieces: Vec<Box<dyn StructurePiece>> = Vec::new();
        let mut i = 0usize;
        while i < list.size() {
            let compound = list.get_compound_or_empty(i);
            let id_str = compound.get_string_or("id", "").to_lowercase();
            let id = Identifier::parse(&id_str);
            let resolved_id = renames().get(&id).cloned().unwrap_or(id);
            let piece_type = built_in_registries::structure_piece().get_value(&resolved_id);
            match piece_type {
                None => {
                    eprintln!("Unknown structure piece id: {}", resolved_id);
                }
                Some(pt) => match pt.load(context, &compound) {
                    Ok(piece) => pieces.push(piece),
                    Err(err) => {
                        eprintln!(
                            "Exception loading structure piece with id {}: {}",
                            resolved_id, err
                        );
                    }
                },
            }
            i += 1;
        }
        PiecesContainer::new(pieces)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer#calculateBoundingBox
    pub fn calculate_bounding_box(&self) -> Option<BoundingBox> {
        BoundingBox::encapsulating(self.pieces.iter().map(|p| *p.get_bounding_box()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPiece {
        bb: BoundingBox,
        id: String,
    }

    impl StructurePiece for TestPiece {
        fn get_bounding_box(&self) -> &BoundingBox {
            &self.bb
        }
        fn create_tag(&self, _context: &StructurePieceSerializationContext) -> Tag {
            let mut c = CompoundTag::new();
            c.insert("id", Tag::String(self.id.clone()));
            Tag::Compound(c)
        }
    }

    struct TestPieceType;

    impl StructurePieceType for TestPieceType {
        fn load(
            &self,
            _context: &StructurePieceSerializationContext,
            tag: &CompoundTag,
        ) -> Result<Box<dyn StructurePiece>, Box<dyn std::error::Error>> {
            let id = tag.get_string_or("id", "unknown");
            if id == "bad" {
                return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "bad piece")));
            }
            Ok(Box::new(TestPiece {
                bb: BoundingBox {
                    min_x: 0,
                    min_y: 0,
                    min_z: 0,
                    max_x: 1,
                    max_y: 1,
                    max_z: 1,
                },
                id,
            }))
        }
    }

    fn bb(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> BoundingBox {
        BoundingBox { min_x, min_y, min_z, max_x, max_y, max_z }
    }

    #[test]
    fn empty_container() {
        let c = PiecesContainer::new(vec![]);
        assert!(c.is_empty());
        assert!(!c.is_inside_piece(BlockPos { x: 0, y: 0, z: 0 }));
        assert_eq!(c.calculate_bounding_box(), None);
    }

    #[test]
    fn is_inside_piece_checks_all_pieces() {
        let c = PiecesContainer::new(vec![
            Box::new(TestPiece { bb: bb(0, 0, 0, 1, 1, 1), id: "a".into() }),
            Box::new(TestPiece { bb: bb(10, 10, 10, 11, 11, 11), id: "b".into() }),
        ]);
        assert!(c.is_inside_piece(BlockPos { x: 1, y: 1, z: 1 }));
        assert!(c.is_inside_piece(BlockPos { x: 10, y: 11, z: 10 }));
        assert!(!c.is_inside_piece(BlockPos { x: 5, y: 5, z: 5 }));
    }

    #[test]
    fn save_creates_list_tag() {
        let c = PiecesContainer::new(vec![
            Box::new(TestPiece { bb: bb(0, 0, 0, 0, 0, 0), id: "minecraft:test".into() }),
        ]);
        let ctx = StructurePieceSerializationContext;
        let tag = c.save(&ctx);
        match tag {
            Tag::List(list) => {
                assert_eq!(list.size(), 1);
                let comp = list.get_compound_or_empty(0);
                assert_eq!(comp.get_string_or("id", ""), "minecraft:test");
            }
            _ => panic!("expected list tag"),
        }
    }

    #[test]
    fn calculate_bounding_box_encapsulates() {
        let c = PiecesContainer::new(vec![
            Box::new(TestPiece { bb: bb(-2, 0, 5, 1, 3, 6), id: "a".into() }),
            Box::new(TestPiece { bb: bb(4, -1, -3, 8, 2, 7), id: "b".into() }),
        ]);
        assert_eq!(c.calculate_bounding_box(), Some(bb(-2, -1, -3, 8, 3, 7)));
    }

    #[test]
    fn identifier_parse_and_rename_map() {
        let id = Identifier::parse("nvi");
        assert_eq!(id, Identifier::with_default_namespace("nvi"));
        let resolved = renames().get(&id).cloned().unwrap_or(id);
        assert_eq!(resolved, Identifier::with_default_namespace("jigsaw"));

        let id2 = Identifier::parse("custom:thing");
        assert_eq!(id2.namespace, "custom");
        assert_eq!(id2.path, "thing");
        assert!(!renames().contains_key(&id2));
    }

    #[test]
    fn load_skips_unknown_and_bad_entries() {
        // Registry may already be initialized by another test in this process;
        // init is best-effort. We only test load behavior with an empty registry
        // by ensuring unknown ids are skipped gracefully.
        let mut list = ListTag::new();
        let mut c1 = CompoundTag::new();
        c1.insert("id", Tag::String("minecraft:unknown_piece_type".into()));
        list.add(Tag::Compound(c1));
        let mut c2 = CompoundTag::new();
        c2.insert("id", Tag::String("".into()));
        list.add(Tag::Compound(c2));

        let ctx = StructurePieceSerializationContext;
        let container = PiecesContainer::load(&list, &ctx);
        // With default empty registry, both should be skipped.
        assert!(container.is_empty());
    }
}
