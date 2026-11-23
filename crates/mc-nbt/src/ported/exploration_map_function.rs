use std::collections::HashSet;
use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

pub struct LootContextParams;
impl LootContextParams {
    pub const ORIGIN: ContextKey = ContextKey("minecraft:origin");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagKey<T> {
    pub name: &'static str,
    _pd: PhantomData<fn() -> T>,
}

impl<T> TagKey<T> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _pd: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Holder<T>(pub T);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapDecorationType(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Structure;

pub struct StructureTags;
impl StructureTags {
    pub const ON_TREASURE_MAPS: TagKey<Structure> =
        TagKey::new("minecraft:on_treasure_maps");
}

pub struct MapDecorationTypes;
impl MapDecorationTypes {
    pub const WOODLAND_MANSION: Holder<MapDecorationType> =
        Holder(MapDecorationType("minecraft:woodland_mansion"));
}

pub const DEFAULT_DESTINATION: TagKey<Structure> = StructureTags::ON_TREASURE_MAPS;
pub const DEFAULT_DECORATION: Holder<MapDecorationType> = MapDecorationTypes::WOODLAND_MANSION;
pub const DEFAULT_ZOOM: i8 = 2;
pub const DEFAULT_SEARCH_RADIUS: i32 = 50;
pub const DEFAULT_SKIP_EXISTING: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LootItemFunctionType(pub &'static str);

pub struct LootItemFunctions;
impl LootItemFunctions {
    pub const EXPLORATION_MAP: LootItemFunctionType =
        LootItemFunctionType("minecraft:exploration_map");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

pub struct Items;
impl Items {
    pub const MAP: &'static str = "minecraft:map";
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item: String,
    pub count: i32,
}

impl ItemStack {
    pub fn new(item: impl Into<String>, count: i32) -> Self {
        Self {
            item: item.into(),
            count,
        }
    }

    pub fn is(&self, item: &str) -> bool {
        self.item == item
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

// JAVA: net.minecraft.util.Mth#floor
pub fn mth_floor(v: f64) -> i32 {
    let i = v as i32;
    if v < (i as f64) {
        i.wrapping_sub(1)
    } else {
        i
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#containing
    pub fn containing(pos: Vec3) -> Self {
        Self {
            x: mth_floor(pos.x),
            y: mth_floor(pos.y),
            z: mth_floor(pos.z),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.BlockPos#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }
}

/// Minimal level capability surface required by ExplorationMapFunction.
pub trait ExplorationMapLevel {
    fn find_nearest_map_structure(
        &mut self,
        destination: &TagKey<Structure>,
        pos: BlockPos,
        search_radius: i32,
        skip_known_structures: bool,
    ) -> Option<BlockPos>;

    fn map_item_create(
        &mut self,
        x: i32,
        z: i32,
        zoom: i8,
        tracking: bool,
        unlimited: bool,
    ) -> ItemStack;

    fn map_item_render_biome_preview_map(&mut self, stack: &mut ItemStack);

    fn map_item_saved_data_add_target_decoration(
        &mut self,
        stack: &mut ItemStack,
        pos: BlockPos,
        label: &str,
        decoration: &Holder<MapDecorationType>,
    );
}

/// Minimal loot context surface required by ExplorationMapFunction.
pub struct LootContext<'a> {
    pub origin: Option<Vec3>,
    pub level: &'a mut dyn ExplorationMapLevel,
}

impl<'a> LootContext<'a> {
    pub fn new(origin: Option<Vec3>, level: &'a mut dyn ExplorationMapLevel) -> Self {
        Self { origin, level }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContext#getOptionalParameter
    pub fn get_optional_parameter_origin(&self) -> Option<Vec3> {
        self.origin
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContext#getLevel
    pub fn get_level(&mut self) -> &mut dyn ExplorationMapLevel {
        self.level
    }
}

// ---------------------------------------------------------------------------
// Ported: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ExplorationMapFunction {
    pub conditions: Vec<LootItemCondition>,
    pub destination: TagKey<Structure>,
    pub map_decoration: Holder<MapDecorationType>,
    pub zoom: i8,
    pub search_radius: i32,
    pub skip_known_structures: bool,
}

impl ExplorationMapFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction#<init>
    pub fn new(
        conditions: Vec<LootItemCondition>,
        destination: TagKey<Structure>,
        map_decoration: Holder<MapDecorationType>,
        zoom: i8,
        search_radius: i32,
        skip_known_structures: bool,
    ) -> Self {
        Self {
            conditions,
            destination,
            map_decoration,
            zoom,
            search_radius,
            skip_known_structures,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::EXPLORATION_MAP
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        let mut set = HashSet::new();
        set.insert(LootContextParams::ORIGIN);
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction#run
    pub fn run(&self, stack: ItemStack, ctx: &mut LootContext<'_>) -> ItemStack {
        if !stack.is(Items::MAP) {
            return stack;
        }

        if let Some(origin) = ctx.get_optional_parameter_origin() {
            let level = ctx.get_level();
            let search_pos = BlockPos::containing(origin);
            if let Some(found) = level.find_nearest_map_structure(
                &self.destination,
                search_pos,
                self.search_radius,
                self.skip_known_structures,
            ) {
                let mut map_stack =
                    level.map_item_create(found.get_x(), found.get_z(), self.zoom, true, true);
                level.map_item_render_biome_preview_map(&mut map_stack);
                level.map_item_saved_data_add_target_decoration(
                    &mut map_stack,
                    found,
                    "+",
                    &self.map_decoration,
                );
                return map_stack;
            }
        }

        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction#makeExplorationMap
    pub fn make_exploration_map() -> Builder {
        Builder::new()
    }
}

#[derive(Debug, Clone)]
pub struct Builder {
    pub conditions: Vec<LootItemCondition>,
    pub destination: TagKey<Structure>,
    pub map_decoration: Holder<MapDecorationType>,
    pub zoom: i8,
    pub search_radius: i32,
    pub skip_known_structures: bool,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction$Builder#<init>
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            destination: DEFAULT_DESTINATION,
            map_decoration: DEFAULT_DECORATION,
            zoom: DEFAULT_ZOOM,
            search_radius: DEFAULT_SEARCH_RADIUS,
            skip_known_structures: DEFAULT_SKIP_EXISTING,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction$Builder#getThis
    pub fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction$Builder#setDestination
    pub fn set_destination(&mut self, destination: TagKey<Structure>) -> &mut Self {
        self.destination = destination;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction$Builder#setMapDecoration
    pub fn set_map_decoration(&mut self, map_decoration: Holder<MapDecorationType>) -> &mut Self {
        self.map_decoration = map_decoration;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction$Builder#setZoom
    pub fn set_zoom(&mut self, zoom: i8) -> &mut Self {
        self.zoom = zoom;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction$Builder#setSearchRadius
    pub fn set_search_radius(&mut self, search_radius: i32) -> &mut Self {
        self.search_radius = search_radius;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction$Builder#setSkipKnownStructures
    pub fn set_skip_known_structures(&mut self, skip_known_structures: bool) -> &mut Self {
        self.skip_known_structures = skip_known_structures;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction$Builder#build
    pub fn build(&self) -> ExplorationMapFunction {
        ExplorationMapFunction::new(
            self.conditions.clone(),
            self.destination,
            self.map_decoration.clone(),
            self.zoom,
            self.search_radius,
            self.skip_known_structures,
        )
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        found: Option<BlockPos>,
        last_search: Option<(TagKey<Structure>, BlockPos, i32, bool)>,
        created: Vec<(i32, i32, i8, bool, bool)>,
        rendered: usize,
        decorations: Vec<(BlockPos, String, Holder<MapDecorationType>)>,
    }

    impl MockLevel {
        fn new(found: Option<BlockPos>) -> Self {
            Self {
                found,
                last_search: None,
                created: Vec::new(),
                rendered: 0,
                decorations: Vec::new(),
            }
        }
    }

    impl ExplorationMapLevel for MockLevel {
        fn find_nearest_map_structure(
            &mut self,
            destination: &TagKey<Structure>,
            pos: BlockPos,
            search_radius: i32,
            skip_known_structures: bool,
        ) -> Option<BlockPos> {
            self.last_search = Some((*destination, pos, search_radius, skip_known_structures));
            self.found
        }

        fn map_item_create(
            &mut self,
            x: i32,
            z: i32,
            zoom: i8,
            tracking: bool,
            unlimited: bool,
        ) -> ItemStack {
            self.created.push((x, z, zoom, tracking, unlimited));
            ItemStack::new("minecraft:filled_map", 1)
        }

        fn map_item_render_biome_preview_map(&mut self, _stack: &mut ItemStack) {
            self.rendered = self.rendered.wrapping_add(1);
        }

        fn map_item_saved_data_add_target_decoration(
            &mut self,
            _stack: &mut ItemStack,
            pos: BlockPos,
            label: &str,
            decoration: &Holder<MapDecorationType>,
        ) {
            self.decorations.push((pos, label.to_string(), decoration.clone()));
        }
    }

    #[test]
    fn run_non_map_returns_unchanged() {
        let mut level = MockLevel::new(Some(BlockPos::new(10, 64, 20)));
        let mut ctx = LootContext::new(Some(Vec3::new(1.0, 2.0, 3.0)), &mut level);
        let f = ExplorationMapFunction::new(
            vec![],
            DEFAULT_DESTINATION,
            DEFAULT_DECORATION,
            DEFAULT_ZOOM,
            DEFAULT_SEARCH_RADIUS,
            DEFAULT_SKIP_EXISTING,
        );
        let stack = ItemStack::new("minecraft:stone", 1);
        let out = f.run(stack.clone(), &mut ctx);
        assert_eq!(out, stack);
        assert!(level.last_search.is_none());
    }

    #[test]
    fn run_no_origin_or_no_structure_returns_original() {
        let mut level = MockLevel::new(None);
        let mut ctx = LootContext::new(None, &mut level);
        let f = ExplorationMapFunction::new(
            vec![],
            DEFAULT_DESTINATION,
            DEFAULT_DECORATION,
            DEFAULT_ZOOM,
            DEFAULT_SEARCH_RADIUS,
            DEFAULT_SKIP_EXISTING,
        );
        let stack = ItemStack::new(Items::MAP, 1);
        let out = f.run(stack.clone(), &mut ctx);
        assert_eq!(out, stack);

        let mut level2 = MockLevel::new(None);
        let mut ctx2 = LootContext::new(Some(Vec3::new(0.0, 0.0, 0.0)), &mut level2);
        let out2 = f.run(ItemStack::new(Items::MAP, 1), &mut ctx2);
        assert_eq!(out2.item, Items::MAP);
        assert!(level2.last_search.is_some());
    }

    #[test]
    fn run_found_structure_creates_decorated_map() {
        let mut level = MockLevel::new(Some(BlockPos::new(100, 64, -200)));
        let mut ctx = LootContext::new(Some(Vec3::new(1.5, 70.0, -3.7)), &mut level);
        let dest = TagKey::new("minecraft:village");
        let deco = Holder(MapDecorationType("minecraft:target_point"));
        let f = ExplorationMapFunction::new(vec![], dest, deco.clone(), 3, 123, false);
        let out = f.run(ItemStack::new(Items::MAP, 1), &mut ctx);
        assert_eq!(out.item, "minecraft:filled_map");

        let (sd, sp, sr, ss) = level.last_search.as_ref().unwrap();
        assert_eq!(*sd, dest);
        assert_eq!(*sp, BlockPos::new(1, 70, -4)); // floor(-3.7) == -4
        assert_eq!(*sr, 123);
        assert!(!*ss);

        assert_eq!(level.created, vec![(100, -200, 3, true, true)]);
        assert_eq!(level.rendered, 1);
        assert_eq!(level.decorations.len(), 1);
        assert_eq!(level.decorations[0].0, BlockPos::new(100, 64, -200));
        assert_eq!(level.decorations[0].1, "+");
        assert_eq!(level.decorations[0].2, deco);
    }

    #[test]
    fn block_pos_containing_negative_overflow_and_float_bits() {
        assert_eq!(
            BlockPos::containing(Vec3::new(-0.1, -1.0, -1.9)),
            BlockPos::new(-1, -1, -2)
        );

        let neg_zero = f64::from_bits(0x8000_0000_0000_0000u64);
        assert_ne!(neg_zero.to_bits(), 0.0f64.to_bits());
        let p = BlockPos::containing(Vec3::new(neg_zero, 0.0, 1.5));
        assert_eq!(p.x, 0); // Java/Mth.floor(-0.0) == 0

        // Rust float-to-int cast saturates like Java narrowing conversion;
        // Mth.floor then applies i-1 when d < i, with wrapping int semantics.
        let big = BlockPos::containing(Vec3::new(1e20, -1e20, f64::INFINITY));
        assert_eq!(big.x, i32::MAX);
        assert_eq!(big.y, i32::MAX); // saturated MIN, then MIN-1 wraps to MAX
        assert_eq!(big.z, i32::MAX);

        let neg_inf = BlockPos::containing(Vec3::new(f64::NEG_INFINITY, 0.0, 0.0));
        assert_eq!(neg_inf.x, i32::MAX); // saturated MIN, then MIN-1 wraps to MAX
    }

    #[test]
    fn run_with_negative_and_extreme_params() {
        let mut level = MockLevel::new(Some(BlockPos::new(i32::MIN, 0, i32::MAX)));
        let mut ctx = LootContext::new(Some(Vec3::new(-999.9, -64.5, 999.9)), &mut level);
        let f = ExplorationMapFunction::new(
            vec![],
            TagKey::new("minecraft:test"),
            Holder(MapDecorationType("minecraft:target_x")),
            i8::MIN,
            i32::MIN,
            false,
        );
        let out = f.run(ItemStack::new(Items::MAP, 1), &mut ctx);
        assert_eq!(out.item, "minecraft:filled_map");

        let (_, sp, sr, ss) = level.last_search.as_ref().unwrap();
        assert_eq!(sp.x, -1000);
        assert_eq!(sp.y, -65);
        assert_eq!(sp.z, 999);
        assert_eq!(*sr, i32::MIN);
        assert!(!*ss);

        assert_eq!(level.created[0].0, i32::MIN);
        assert_eq!(level.created[0].1, i32::MAX);
        assert_eq!(level.created[0].2, i8::MIN);
        assert!(level.created[0].3);
        assert!(level.created[0].4);
    }

    #[test]
    fn builder_defaults_setters_and_make_exploration_map() {
        let f = Builder::new().build();
        assert_eq!(f.destination, DEFAULT_DESTINATION);
        assert_eq!(f.map_decoration, DEFAULT_DECORATION);
        assert_eq!(f.zoom, DEFAULT_ZOOM);
        assert_eq!(f.search_radius, DEFAULT_SEARCH_RADIUS);
        assert_eq!(f.skip_known_structures, DEFAULT_SKIP_EXISTING);
        assert_eq!(f.get_type(), LootItemFunctions::EXPLORATION_MAP);

        let refs = f.get_referenced_context_params();
        assert_eq!(refs.len(), 1);
        assert!(refs.contains(&LootContextParams::ORIGIN));

        let mut b = ExplorationMapFunction::make_exploration_map();
        b.get_this()
            .set_destination(TagKey::new("minecraft:custom"))
            .set_map_decoration(Holder(MapDecorationType("minecraft:custom_deco")))
            .set_zoom(i8::MAX)
            .set_search_radius(-1)
            .set_skip_known_structures(false);
        let f2 = b.build();
        assert_eq!(f2.destination.name, "minecraft:custom");
        assert_eq!(f2.map_decoration.0 .0, "minecraft:custom_deco");
        assert_eq!(f2.zoom, i8::MAX);
        assert_eq!(f2.search_radius, -1);
        assert!(!f2.skip_known_structures);
    }
}
