use std::fmt;

/// Minimal stub for `net.minecraft.nbt.CompoundTag`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompoundTag(pub i32);

/// Minimal stub for `net.minecraft.world.level.levelgen.structure.StructurePiece`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructurePiece(pub i32);

/// Minimal stub for
/// `net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructureTemplateManager(pub i32);

/// Minimal stub for
/// `net.minecraft.world.level.levelgen.structure.pieces.StructurePieceSerializationContext`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructurePieceSerializationContext {
    structure_template_manager: StructureTemplateManager,
}

impl StructurePieceSerializationContext {
    pub fn new(structure_template_manager: StructureTemplateManager) -> Self {
        Self {
            structure_template_manager,
        }
    }

    pub fn structure_template_manager(&self) -> &StructureTemplateManager {
        &self.structure_template_manager
    }
}

pub type FullContextLoader =
    fn(&StructurePieceSerializationContext, &CompoundTag) -> StructurePiece;
pub type ContextlessLoader = fn(&CompoundTag) -> StructurePiece;
pub type TemplateLoader = fn(&StructureTemplateManager, &CompoundTag) -> StructurePiece;

#[derive(Clone, Copy)]
enum LoaderKind {
    Full(FullContextLoader),
    Contextless(ContextlessLoader),
    Template(TemplateLoader),
}

/// Port of `net.minecraft.world.level.levelgen.structure.pieces.StructurePieceType`.
#[derive(Clone, Copy)]
pub struct StructurePieceType {
    id: &'static str,
    loader: LoaderKind,
}

impl fmt::Debug for StructurePieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StructurePieceType")
            .field("id", &self.id)
            .finish()
    }
}

impl PartialEq for StructurePieceType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for StructurePieceType {}

impl StructurePieceType {
    pub const fn new_full(id: &'static str, loader: FullContextLoader) -> Self {
        Self {
            id,
            loader: LoaderKind::Full(loader),
        }
    }

    pub const fn new_contextless(id: &'static str, loader: ContextlessLoader) -> Self {
        Self {
            id,
            loader: LoaderKind::Contextless(loader),
        }
    }

    pub const fn new_template(id: &'static str, loader: TemplateLoader) -> Self {
        Self {
            id,
            loader: LoaderKind::Template(loader),
        }
    }

    pub fn id(&self) -> &'static str {
        self.id
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceType#load
    pub fn load(
        &self,
        context: &StructurePieceSerializationContext,
        tag: &CompoundTag,
    ) -> StructurePiece {
        match self.loader {
            LoaderKind::Full(f) => f(context, tag),
            LoaderKind::Contextless(f) => load_contextless(f, context, tag),
            LoaderKind::Template(f) => load_template(f, context, tag),
        }
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceType$ContextlessType#load
fn load_contextless(
    loader: ContextlessLoader,
    _context: &StructurePieceSerializationContext,
    tag: &CompoundTag,
) -> StructurePiece {
    loader(tag)
}

// JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceType$StructureTemplateType#load
fn load_template(
    loader: TemplateLoader,
    context: &StructurePieceSerializationContext,
    tag: &CompoundTag,
) -> StructurePiece {
    loader(context.structure_template_manager(), tag)
}

fn register_piece(id: &str, loader: LoaderKind) -> StructurePieceType {
    // Java: Registry.register(BuiltInRegistries.STRUCTURE_PIECE, id.toLowerCase(Locale.ROOT), type)
    // Registry side effect omitted in this self-contained port; ID normalization preserved.
    StructurePieceType {
        id: Box::leak(id.to_ascii_lowercase().into_boxed_str()),
        loader,
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceType#setFullContextPieceId
fn set_full_context_piece_id(loader: FullContextLoader, id: &str) -> StructurePieceType {
    register_piece(id, LoaderKind::Full(loader))
}

// JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceType#setPieceId
fn set_piece_id(loader: ContextlessLoader, id: &str) -> StructurePieceType {
    register_piece(id, LoaderKind::Contextless(loader))
}

// JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceType#setTemplatePieceId
fn set_template_piece_id(loader: TemplateLoader, id: &str) -> StructurePieceType {
    register_piece(id, LoaderKind::Template(loader))
}

fn stub_full(_context: &StructurePieceSerializationContext, _tag: &CompoundTag) -> StructurePiece {
    StructurePiece(1)
}

fn stub_contextless(_tag: &CompoundTag) -> StructurePiece {
    StructurePiece(2)
}

fn stub_template(
    _manager: &StructureTemplateManager,
    _tag: &CompoundTag,
) -> StructurePiece {
    StructurePiece(3)
}

pub const MINE_SHAFT_CORRIDOR: StructurePieceType =
    StructurePieceType::new_contextless("mscorridor", stub_contextless);
pub const MINE_SHAFT_CROSSING: StructurePieceType =
    StructurePieceType::new_contextless("mscrossing", stub_contextless);
pub const MINE_SHAFT_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("msroom", stub_contextless);
pub const MINE_SHAFT_STAIRS: StructurePieceType =
    StructurePieceType::new_contextless("msstairs", stub_contextless);
pub const NETHER_FORTRESS_BRIDGE_CROSSING: StructurePieceType =
    StructurePieceType::new_contextless("nebcr", stub_contextless);
pub const NETHER_FORTRESS_BRIDGE_END_FILLER: StructurePieceType =
    StructurePieceType::new_contextless("nebef", stub_contextless);
pub const NETHER_FORTRESS_BRIDGE_STRAIGHT: StructurePieceType =
    StructurePieceType::new_contextless("nebs", stub_contextless);
pub const NETHER_FORTRESS_CASTLE_CORRIDOR_STAIRS: StructurePieceType =
    StructurePieceType::new_contextless("neccs", stub_contextless);
pub const NETHER_FORTRESS_CASTLE_CORRIDOR_T_BALCONY: StructurePieceType =
    StructurePieceType::new_contextless("nectb", stub_contextless);
pub const NETHER_FORTRESS_CASTLE_ENTRANCE: StructurePieceType =
    StructurePieceType::new_contextless("nece", stub_contextless);
pub const NETHER_FORTRESS_CASTLE_SMALL_CORRIDOR_CROSSING: StructurePieceType =
    StructurePieceType::new_contextless("nescsc", stub_contextless);
pub const NETHER_FORTRESS_CASTLE_SMALL_CORRIDOR_LEFT_TURN: StructurePieceType =
    StructurePieceType::new_contextless("nesclt", stub_contextless);
pub const NETHER_FORTRESS_CASTLE_SMALL_CORRIDOR: StructurePieceType =
    StructurePieceType::new_contextless("nesc", stub_contextless);
pub const NETHER_FORTRESS_CASTLE_SMALL_CORRIDOR_RIGHT_TURN: StructurePieceType =
    StructurePieceType::new_contextless("nescrt", stub_contextless);
pub const NETHER_FORTRESS_CASTLE_STALK_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("necsr", stub_contextless);
pub const NETHER_FORTRESS_MONSTER_THRONE: StructurePieceType =
    StructurePieceType::new_contextless("nemt", stub_contextless);
pub const NETHER_FORTRESS_ROOM_CROSSING: StructurePieceType =
    StructurePieceType::new_contextless("nerc", stub_contextless);
pub const NETHER_FORTRESS_STAIRS_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("nesr", stub_contextless);
pub const NETHER_FORTRESS_START: StructurePieceType =
    StructurePieceType::new_contextless("nestart", stub_contextless);
pub const STRONGHOLD_CHEST_CORRIDOR: StructurePieceType =
    StructurePieceType::new_contextless("shcc", stub_contextless);
pub const STRONGHOLD_FILLER_CORRIDOR: StructurePieceType =
    StructurePieceType::new_contextless("shfc", stub_contextless);
pub const STRONGHOLD_FIVE_CROSSING: StructurePieceType =
    StructurePieceType::new_contextless("sh5c", stub_contextless);
pub const STRONGHOLD_LEFT_TURN: StructurePieceType =
    StructurePieceType::new_contextless("shlt", stub_contextless);
pub const STRONGHOLD_LIBRARY: StructurePieceType =
    StructurePieceType::new_contextless("shli", stub_contextless);
pub const STRONGHOLD_PORTAL_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("shpr", stub_contextless);
pub const STRONGHOLD_PRISON_HALL: StructurePieceType =
    StructurePieceType::new_contextless("shph", stub_contextless);
pub const STRONGHOLD_RIGHT_TURN: StructurePieceType =
    StructurePieceType::new_contextless("shrt", stub_contextless);
pub const STRONGHOLD_ROOM_CROSSING: StructurePieceType =
    StructurePieceType::new_contextless("shrc", stub_contextless);
pub const STRONGHOLD_STAIRS_DOWN: StructurePieceType =
    StructurePieceType::new_contextless("shsd", stub_contextless);
pub const STRONGHOLD_START: StructurePieceType =
    StructurePieceType::new_contextless("shstart", stub_contextless);
pub const STRONGHOLD_STRAIGHT: StructurePieceType =
    StructurePieceType::new_contextless("shs", stub_contextless);
pub const STRONGHOLD_STRAIGHT_STAIRS_DOWN: StructurePieceType =
    StructurePieceType::new_contextless("shssd", stub_contextless);
pub const JUNGLE_PYRAMID_PIECE: StructurePieceType =
    StructurePieceType::new_contextless("tejp", stub_contextless);
pub const OCEAN_RUIN: StructurePieceType =
    StructurePieceType::new_template("orp", stub_template);
pub const IGLOO: StructurePieceType = StructurePieceType::new_template("iglu", stub_template);
pub const RUINED_PORTAL: StructurePieceType =
    StructurePieceType::new_template("rupo", stub_template);
pub const SWAMPLAND_HUT: StructurePieceType =
    StructurePieceType::new_contextless("tesh", stub_contextless);
pub const DESERT_PYRAMID_PIECE: StructurePieceType =
    StructurePieceType::new_contextless("tedp", stub_contextless);
pub const OCEAN_MONUMENT_BUILDING: StructurePieceType =
    StructurePieceType::new_contextless("omb", stub_contextless);
pub const OCEAN_MONUMENT_CORE_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omcr", stub_contextless);
pub const OCEAN_MONUMENT_DOUBLE_X_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omdxr", stub_contextless);
pub const OCEAN_MONUMENT_DOUBLE_XY_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omdxyr", stub_contextless);
pub const OCEAN_MONUMENT_DOUBLE_Y_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omdyr", stub_contextless);
pub const OCEAN_MONUMENT_DOUBLE_YZ_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omdyzr", stub_contextless);
pub const OCEAN_MONUMENT_DOUBLE_Z_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omdzr", stub_contextless);
pub const OCEAN_MONUMENT_ENTRY_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omentry", stub_contextless);
pub const OCEAN_MONUMENT_PENTHOUSE: StructurePieceType =
    StructurePieceType::new_contextless("ompenthouse", stub_contextless);
pub const OCEAN_MONUMENT_SIMPLE_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omsimple", stub_contextless);
pub const OCEAN_MONUMENT_SIMPLE_TOP_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omsimplet", stub_contextless);
pub const OCEAN_MONUMENT_WING_ROOM: StructurePieceType =
    StructurePieceType::new_contextless("omwr", stub_contextless);
pub const END_CITY_PIECE: StructurePieceType =
    StructurePieceType::new_template("ecp", stub_template);
pub const WOODLAND_MANSION_PIECE: StructurePieceType =
    StructurePieceType::new_template("wmp", stub_template);
pub const BURIED_TREASURE_PIECE: StructurePieceType =
    StructurePieceType::new_contextless("btp", stub_contextless);
pub const SHIPWRECK_PIECE: StructurePieceType =
    StructurePieceType::new_template("shipwreck", stub_template);
pub const NETHER_FOSSIL: StructurePieceType =
    StructurePieceType::new_template("nefos", stub_template);
pub const JIGSAW: StructurePieceType = StructurePieceType::new_full("jigsaw", stub_full);

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(manager_id: i32) -> StructurePieceSerializationContext {
        StructurePieceSerializationContext::new(StructureTemplateManager(manager_id))
    }

    #[test]
    fn set_helpers_normalize_ids_to_ascii_lowercase() {
        fn cl(_tag: &CompoundTag) -> StructurePiece {
            StructurePiece(0)
        }
        fn tl(_mgr: &StructureTemplateManager, _tag: &CompoundTag) -> StructurePiece {
            StructurePiece(0)
        }
        fn fl(_ctx: &StructurePieceSerializationContext, _tag: &CompoundTag) -> StructurePiece {
            StructurePiece(0)
        }

        assert_eq!(set_piece_id(cl, "MSCorridor").id(), "mscorridor");
        assert_eq!(set_template_piece_id(tl, "TeJP").id(), "tejp");
        assert_eq!(set_full_context_piece_id(fl, "JIGSAW").id(), "jigsaw");
        assert_eq!(set_piece_id(cl, "already").id(), "already");
    }

    #[test]
    fn contextless_load_dispatches_without_context() {
        fn loader(tag: &CompoundTag) -> StructurePiece {
            StructurePiece(tag.0)
        }
        let t = set_piece_id(loader, "Test");
        let ctx = make_context(99);
        assert_eq!(t.load(&ctx, &CompoundTag(42)), StructurePiece(42));
        assert_eq!(t.load(&ctx, &CompoundTag(-1)), StructurePiece(-1));
    }

    #[test]
    fn template_load_dispatches_with_template_manager() {
        fn loader(mgr: &StructureTemplateManager, tag: &CompoundTag) -> StructurePiece {
            StructurePiece(mgr.0.wrapping_add(tag.0))
        }
        let t = set_template_piece_id(loader, "Tpl");
        let ctx = make_context(10);
        assert_eq!(t.load(&ctx, &CompoundTag(7)), StructurePiece(17));
        assert_eq!(t.load(&ctx, &CompoundTag(i32::MIN)), StructurePiece(10i32.wrapping_add(i32::MIN)));
    }

    #[test]
    fn full_context_load_receives_context_and_tag() {
        fn loader(ctx: &StructurePieceSerializationContext, tag: &CompoundTag) -> StructurePiece {
            StructurePiece(
                ctx.structure_template_manager()
                    .0
                    .wrapping_mul(100)
                    .wrapping_add(tag.0),
            )
        }
        let t = set_full_context_piece_id(loader, "Full");
        let ctx = make_context(3);
        assert_eq!(t.load(&ctx, &CompoundTag(5)), StructurePiece(305));
        assert_eq!(t.load(&ctx, &CompoundTag(i32::MAX)), StructurePiece(300i32.wrapping_add(i32::MAX)));
    }

    #[test]
    fn constants_have_expected_ids_and_loader_kinds() {
        let ctx = make_context(0);
        let tag = CompoundTag(0);

        assert_eq!(MINE_SHAFT_CORRIDOR.id(), "mscorridor");
        assert_eq!(NETHER_FORTRESS_BRIDGE_CROSSING.id(), "nebcr");
        assert_eq!(STRONGHOLD_START.id(), "shstart");
        assert_eq!(OCEAN_MONUMENT_ENTRY_ROOM.id(), "omentry");
        assert_eq!(RUINED_PORTAL.id(), "rupo");
        assert_eq!(JIGSAW.id(), "jigsaw");

        // Contextless, template, full dispatch markers from stub loaders.
        assert_eq!(MINE_SHAFT_CORRIDOR.load(&ctx, &tag), StructurePiece(2));
        assert_eq!(OCEAN_RUIN.load(&ctx, &tag), StructurePiece(3));
        assert_eq!(JIGSAW.load(&ctx, &tag), StructurePiece(1));
    }

    #[test]
    fn load_passes_boundary_values() {
        fn loader(tag: &CompoundTag) -> StructurePiece {
            StructurePiece(tag.0)
        }
        let t = set_piece_id(loader, "boundary");
        let ctx = make_context(0);
        assert_eq!(t.load(&ctx, &CompoundTag(i32::MIN)), StructurePiece(i32::MIN));
        assert_eq!(t.load(&ctx, &CompoundTag(i32::MAX)), StructurePiece(i32::MAX));
        assert_eq!(t.load(&ctx, &CompoundTag(0)), StructurePiece(0));
    }
}
