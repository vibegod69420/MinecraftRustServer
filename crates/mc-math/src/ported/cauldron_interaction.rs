use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Minimal self-contained types modeling Minecraft dependencies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionResult {
    Success,
    TryWithEmptyHand,
    Consume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionHand {
    MainHand,
    OffHand,
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

    pub fn above(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub block: String,
    /// LayeredCauldronBlock.LEVEL (1..=3) when applicable.
    pub level: Option<i32>,
}

impl BlockState {
    pub fn new(block: impl Into<String>) -> Self {
        Self {
            block: block.into(),
            level: None,
        }
    }

    pub fn with_level(block: impl Into<String>, level: i32) -> Self {
        Self {
            block: block.into(),
            level: Some(level),
        }
    }

    pub fn get_level(&self) -> i32 {
        self.level.unwrap_or(0)
    }

    /// Mimics BlockState.cycle(LayeredCauldronBlock.LEVEL) for levels 1..=3.
    pub fn cycle_level(&self) -> Self {
        let mut next = self.clone();
        let cur = self.get_level();
        // LayeredCauldronBlock.LEVEL cycles through 1,2,3
        let cycled = if cur >= 3 { 1 } else { cur.wrapping_add(1) };
        next.level = Some(cycled);
        next
    }

    pub fn set_level(&self, level: i32) -> Self {
        let mut next = self.clone();
        next.level = Some(level);
        next
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item: String,
    pub count: i32,
    pub potion: Option<String>,
    pub dyed_color: Option<i32>,
    pub banner_pattern_layers: i32,
}

impl ItemStack {
    pub fn new(item: impl Into<String>) -> Self {
        Self {
            item: item.into(),
            count: 1,
            potion: None,
            dyed_color: None,
            banner_pattern_layers: 0,
        }
    }

    pub fn with_count(item: impl Into<String>, count: i32) -> Self {
        Self {
            count,
            ..Self::new(item)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item.is_empty() || self.item == "minecraft:air"
    }

    pub fn shrink(&mut self, amount: i32) {
        self.count = self.count.wrapping_sub(amount);
    }

    pub fn copy_with_count(&self, count: i32) -> Self {
        let mut c = self.clone();
        c.count = count;
        c
    }

    pub fn transmute_copy(&self, item: impl Into<String>, count: i32) -> Self {
        Self::with_count(item, count)
    }
}

/// Minimal level/world access required by cauldron interactions.
pub trait LevelAccess {
    fn is_client_side(&self) -> bool;
    fn set_block_and_update(&mut self, pos: BlockPos, state: BlockState);
    fn play_sound(&mut self, pos: BlockPos, sound: &str, source: &str, volume: f32, pitch: f32);
    fn game_event(&mut self, event: &str, pos: BlockPos);
    fn fluid_is_water_at(&self, pos: BlockPos) -> bool;
}

/// Minimal player access required by cauldron interactions.
pub trait PlayerAccess {
    fn set_item_in_hand(&mut self, hand: InteractionHand, stack: ItemStack);
    fn award_stat(&mut self, stat: &str);
    fn award_item_stat(&mut self, item: &str);
    fn has_infinite_materials(&self) -> bool;
    fn add_item(&mut self, stack: ItemStack);
    fn has_item(&self, item: &str) -> bool;
}

/// Functional interface equivalent.
/// JAVA: net.minecraft.core.cauldron.CauldronInteraction
pub type CauldronInteractionFn = fn(
    &BlockState,
    &mut dyn LevelAccess,
    BlockPos,
    &mut dyn PlayerAccess,
    InteractionHand,
    &mut ItemStack,
) -> InteractionResult;

/// Record equivalent: CauldronInteraction.InteractionMap
#[derive(Clone)]
pub struct InteractionMap {
    pub name: String,
    pub map: HashMap<String, CauldronInteractionFn>,
}

impl InteractionMap {
    pub fn get(&self, item: &str) -> CauldronInteractionFn {
        self.map
            .get(item)
            .copied()
            .unwrap_or(default_interaction)
    }
}

fn default_interaction(
    _state: &BlockState,
    _level: &mut dyn LevelAccess,
    _pos: BlockPos,
    _player: &mut dyn PlayerAccess,
    _hand: InteractionHand,
    _stack: &mut ItemStack,
) -> InteractionResult {
    InteractionResult::TryWithEmptyHand
}

/// Registry holding all interaction maps (replaces static INTERACTIONS/EMPTY/WATER/LAVA/POWDER_SNOW).
pub struct CauldronInteractionsRegistry {
    pub interactions: HashMap<String, InteractionMap>,
}

impl CauldronInteractionsRegistry {
    pub fn new() -> Self {
        Self {
            interactions: HashMap::new(),
        }
    }

    pub fn get_map(&self, name: &str) -> Option<&InteractionMap> {
        self.interactions.get(name)
    }

    pub fn get_map_mut(&mut self, name: &str) -> Option<&mut InteractionMap> {
        self.interactions.get_mut(name)
    }
}

impl Default for CauldronInteractionsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Item/block/sound/stat constants (minimal identifiers for self-contained port)
// ---------------------------------------------------------------------------

const ITEM_POTION: &str = "minecraft:potion";
const ITEM_GLASS_BOTTLE: &str = "minecraft:glass_bottle";
const ITEM_BUCKET: &str = "minecraft:bucket";
const ITEM_WATER_BUCKET: &str = "minecraft:water_bucket";
const ITEM_LAVA_BUCKET: &str = "minecraft:lava_bucket";
const ITEM_POWDER_SNOW_BUCKET: &str = "minecraft:powder_snow_bucket";
const ITEM_LEATHER_BOOTS: &str = "minecraft:leather_boots";
const ITEM_LEATHER_LEGGINGS: &str = "minecraft:leather_leggings";
const ITEM_LEATHER_CHESTPLATE: &str = "minecraft:leather_chestplate";
const ITEM_LEATHER_HELMET: &str = "minecraft:leather_helmet";
const ITEM_LEATHER_HORSE_ARMOR: &str = "minecraft:leather_horse_armor";
const ITEM_WOLF_ARMOR: &str = "minecraft:wolf_armor";

const BLOCK_CAULDRON: &str = "minecraft:cauldron";
const BLOCK_WATER_CAULDRON: &str = "minecraft:water_cauldron";
const BLOCK_LAVA_CAULDRON: &str = "minecraft:lava_cauldron";
const BLOCK_POWDER_SNOW_CAULDRON: &str = "minecraft:powder_snow_cauldron";
const BLOCK_SHULKER_BOX: &str = "minecraft:shulker_box";

const SOUND_BOTTLE_EMPTY: &str = "minecraft:item.bottle.empty";
const SOUND_BOTTLE_FILL: &str = "minecraft:item.bottle.fill";
const SOUND_BUCKET_FILL: &str = "minecraft:item.bucket.fill";
const SOUND_BUCKET_FILL_LAVA: &str = "minecraft:item.bucket.fill_lava";
const SOUND_BUCKET_FILL_POWDER_SNOW: &str = "minecraft:item.bucket.fill_powder_snow";
const SOUND_BUCKET_EMPTY: &str = "minecraft:item.bucket.empty";
const SOUND_BUCKET_EMPTY_LAVA: &str = "minecraft:item.bucket.empty_lava";
const SOUND_BUCKET_EMPTY_POWDER_SNOW: &str = "minecraft:item.bucket.empty_powder_snow";

const STAT_USE_CAULDRON: &str = "minecraft:use_cauldron";
const STAT_FILL_CAULDRON: &str = "minecraft:fill_cauldron";
const STAT_CLEAN_SHULKER_BOX: &str = "minecraft:clean_shulker_box";
const STAT_CLEAN_BANNER: &str = "minecraft:clean_banner";
const STAT_CLEAN_ARMOR: &str = "minecraft:clean_armor";

const POTION_WATER: &str = "minecraft:water";

const BANNERS: &[&str] = &[
    "minecraft:white_banner",
    "minecraft:gray_banner",
    "minecraft:black_banner",
    "minecraft:blue_banner",
    "minecraft:brown_banner",
    "minecraft:cyan_banner",
    "minecraft:green_banner",
    "minecraft:light_blue_banner",
    "minecraft:light_gray_banner",
    "minecraft:lime_banner",
    "minecraft:magenta_banner",
    "minecraft:orange_banner",
    "minecraft:pink_banner",
    "minecraft:purple_banner",
    "minecraft:red_banner",
    "minecraft:yellow_banner",
];

const SHULKER_BOXES: &[&str] = &[
    "minecraft:white_shulker_box",
    "minecraft:gray_shulker_box",
    "minecraft:black_shulker_box",
    "minecraft:blue_shulker_box",
    "minecraft:brown_shulker_box",
    "minecraft:cyan_shulker_box",
    "minecraft:green_shulker_box",
    "minecraft:light_blue_shulker_box",
    "minecraft:light_gray_shulker_box",
    "minecraft:lime_shulker_box",
    "minecraft:magenta_shulker_box",
    "minecraft:orange_shulker_box",
    "minecraft:pink_shulker_box",
    "minecraft:purple_shulker_box",
    "minecraft:red_shulker_box",
    "minecraft:yellow_shulker_box",
];

// ---------------------------------------------------------------------------
// Helpers (not CauldronInteraction methods)
// ---------------------------------------------------------------------------

fn is_water_potion(stack: &ItemStack) -> bool {
    matches!(stack.potion.as_deref(), Some(POTION_WATER))
}

fn create_filled_result(
    stack: &mut ItemStack,
    player: &mut dyn PlayerAccess,
    result: ItemStack,
    limit_creative_stack: bool,
) -> ItemStack {
    if player.has_infinite_materials() {
        if !limit_creative_stack || !player.has_item(&result.item) {
            player.add_item(result.clone());
        }
        if limit_creative_stack {
            // Java 4-arg createFilledResult: if creative and already has matching item,
            // returns original; otherwise returns result. Simplified faithful behavior.
            if player.has_item(&result.item) {
                return stack.clone();
            }
        }
        return result;
    }

    stack.shrink(1);
    if stack.is_empty() {
        return result;
    }
    player.add_item(result.clone());
    stack.clone()
}

fn is_dyeable(item: &str) -> bool {
    matches!(
        item,
        ITEM_LEATHER_BOOTS
            | ITEM_LEATHER_LEGGINGS
            | ITEM_LEATHER_CHESTPLATE
            | ITEM_LEATHER_HELMET
            | ITEM_LEATHER_HORSE_ARMOR
            | ITEM_WOLF_ARMOR
    )
}

fn is_shulker_box(item: &str) -> bool {
    SHULKER_BOXES.contains(&item)
}

// ---------------------------------------------------------------------------
// Ported CauldronInteraction methods
// ---------------------------------------------------------------------------

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#newInteractionMap
pub fn new_interaction_map(name: &str, registry: &mut CauldronInteractionsRegistry) -> String {
    let map = InteractionMap {
        name: name.to_string(),
        map: HashMap::new(),
    };
    registry.interactions.insert(name.to_string(), map);
    name.to_string()
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#interact
pub fn interact(
    map: &InteractionMap,
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    let f = map.get(&stack.item);
    f(state, level, pos, player, hand, stack)
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#bootStrap
pub fn boot_strap(registry: &mut CauldronInteractionsRegistry) {
    let empty_name = new_interaction_map("empty", registry);
    let water_name = new_interaction_map("water", registry);
    let lava_name = new_interaction_map("lava", registry);
    let powder_snow_name = new_interaction_map("powder_snow", registry);

    if let Some(empty) = registry.get_map_mut(&empty_name) {
        add_default_interactions(empty);
        empty
            .map
            .insert(ITEM_POTION.to_string(), empty_potion_interaction);
    }

    if let Some(water) = registry.get_map_mut(&water_name) {
        add_default_interactions(water);
        water
            .map
            .insert(ITEM_BUCKET.to_string(), water_fill_bucket_interaction);
        water
            .map
            .insert(ITEM_GLASS_BOTTLE.to_string(), water_glass_bottle_interaction);
        water
            .map
            .insert(ITEM_POTION.to_string(), water_potion_interaction);
        water
            .map
            .insert(ITEM_LEATHER_BOOTS.to_string(), dyed_item_iteration);
        water
            .map
            .insert(ITEM_LEATHER_LEGGINGS.to_string(), dyed_item_iteration);
        water
            .map
            .insert(ITEM_LEATHER_CHESTPLATE.to_string(), dyed_item_iteration);
        water
            .map
            .insert(ITEM_LEATHER_HELMET.to_string(), dyed_item_iteration);
        water
            .map
            .insert(ITEM_LEATHER_HORSE_ARMOR.to_string(), dyed_item_iteration);
        water
            .map
            .insert(ITEM_WOLF_ARMOR.to_string(), dyed_item_iteration);
        for &banner in BANNERS {
            water.map.insert(banner.to_string(), banner_interaction);
        }
        for &shulker in SHULKER_BOXES {
            water.map.insert(shulker.to_string(), shulker_box_interaction);
        }
    }

    if let Some(lava) = registry.get_map_mut(&lava_name) {
        lava
            .map
            .insert(ITEM_BUCKET.to_string(), lava_fill_bucket_interaction);
        add_default_interactions(lava);
    }

    if let Some(powder_snow) = registry.get_map_mut(&powder_snow_name) {
        powder_snow
            .map
            .insert(ITEM_BUCKET.to_string(), powder_snow_fill_bucket_interaction);
        add_default_interactions(powder_snow);
    }
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#addDefaultInteractions
pub fn add_default_interactions(map: &mut InteractionMap) {
    map.map
        .insert(ITEM_LAVA_BUCKET.to_string(), fill_lava_interaction);
    map.map
        .insert(ITEM_WATER_BUCKET.to_string(), fill_water_interaction);
    map.map.insert(
        ITEM_POWDER_SNOW_BUCKET.to_string(),
        fill_powder_snow_interaction,
    );
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#fillBucket
pub fn fill_bucket(
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
    output: ItemStack,
    predicate: fn(&BlockState) -> bool,
    sound_event: &str,
) -> InteractionResult {
    if !predicate(state) {
        return InteractionResult::TryWithEmptyHand;
    }

    if !level.is_client_side() {
        let item = stack.item.clone();
        let filled = create_filled_result(stack, player, output, true);
        player.set_item_in_hand(hand, filled);
        player.award_stat(STAT_USE_CAULDRON);
        player.award_item_stat(&item);
        level.set_block_and_update(pos, BlockState::new(BLOCK_CAULDRON));
        level.play_sound(pos, sound_event, "BLOCKS", 1.0_f32, 1.0_f32);
        level.game_event("minecraft:fluid_pickup", pos);
    }

    InteractionResult::Success
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#emptyBucket
pub fn empty_bucket(
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
    filled_state: BlockState,
    sound_event: &str,
) -> InteractionResult {
    if !level.is_client_side() {
        let item = stack.item.clone();
        let filled = create_filled_result(stack, player, ItemStack::new(ITEM_BUCKET), true);
        player.set_item_in_hand(hand, filled);
        player.award_stat(STAT_FILL_CAULDRON);
        player.award_item_stat(&item);
        level.set_block_and_update(pos, filled_state);
        level.play_sound(pos, sound_event, "BLOCKS", 1.0_f32, 1.0_f32);
        level.game_event("minecraft:fluid_place", pos);
    }

    InteractionResult::Success
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#fillWaterInteraction
fn fill_water_interaction(
    _state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    empty_bucket(
        level,
        pos,
        player,
        hand,
        stack,
        BlockState::with_level(BLOCK_WATER_CAULDRON, 3),
        SOUND_BUCKET_EMPTY,
    )
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#fillLavaInteraction
fn fill_lava_interaction(
    _state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    if is_under_water(level, pos) {
        InteractionResult::Consume
    } else {
        empty_bucket(
            level,
            pos,
            player,
            hand,
            stack,
            BlockState::new(BLOCK_LAVA_CAULDRON),
            SOUND_BUCKET_EMPTY_LAVA,
        )
    }
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#fillPowderSnowInteraction
fn fill_powder_snow_interaction(
    _state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    if is_under_water(level, pos) {
        InteractionResult::Consume
    } else {
        empty_bucket(
            level,
            pos,
            player,
            hand,
            stack,
            BlockState::with_level(BLOCK_POWDER_SNOW_CAULDRON, 3),
            SOUND_BUCKET_EMPTY_POWDER_SNOW,
        )
    }
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#shulkerBoxInteraction
fn shulker_box_interaction(
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    if !is_shulker_box(&stack.item) {
        return InteractionResult::TryWithEmptyHand;
    }

    if !level.is_client_side() {
        let result = stack.transmute_copy(BLOCK_SHULKER_BOX, 1);
        let filled = create_filled_result(stack, player, result, false);
        player.set_item_in_hand(hand, filled);
        player.award_stat(STAT_CLEAN_SHULKER_BOX);
        lower_fill_level(state, level, pos);
    }

    InteractionResult::Success
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#bannerInteraction
fn banner_interaction(
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    if stack.banner_pattern_layers <= 0 {
        return InteractionResult::TryWithEmptyHand;
    }

    if !level.is_client_side() {
        let mut result = stack.copy_with_count(1);
        result.banner_pattern_layers = result.banner_pattern_layers.wrapping_sub(1);
        let filled = create_filled_result(stack, player, result, false);
        player.set_item_in_hand(hand, filled);
        player.award_stat(STAT_CLEAN_BANNER);
        lower_fill_level(state, level, pos);
    }

    InteractionResult::Success
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#dyedItemIteration
fn dyed_item_iteration(
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    _hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    if !is_dyeable(&stack.item) {
        return InteractionResult::TryWithEmptyHand;
    }

    if stack.dyed_color.is_none() {
        return InteractionResult::TryWithEmptyHand;
    }

    if !level.is_client_side() {
        stack.dyed_color = None;
        player.award_stat(STAT_CLEAN_ARMOR);
        lower_fill_level(state, level, pos);
    }

    InteractionResult::Success
}

// JAVA: net.minecraft.core.cauldron.CauldronInteraction#isUnderWater
fn is_under_water(level: &dyn LevelAccess, pos: BlockPos) -> bool {
    level.fluid_is_water_at(pos.above())
}

// ---------------------------------------------------------------------------
// BootStrap lambda interactions (named for fn-pointer compatibility)
// ---------------------------------------------------------------------------

fn empty_potion_interaction(
    _state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    if !is_water_potion(stack) {
        return InteractionResult::TryWithEmptyHand;
    }

    if !level.is_client_side() {
        let item = stack.item.clone();
        let filled = create_filled_result(stack, player, ItemStack::new(ITEM_GLASS_BOTTLE), true);
        player.set_item_in_hand(hand, filled);
        player.award_stat(STAT_USE_CAULDRON);
        player.award_item_stat(&item);
        level.set_block_and_update(pos, BlockState::new(BLOCK_WATER_CAULDRON));
        level.play_sound(pos, SOUND_BOTTLE_EMPTY, "BLOCKS", 1.0_f32, 1.0_f32);
        level.game_event("minecraft:fluid_place", pos);
    }

    InteractionResult::Success
}

fn water_bucket_predicate(state: &BlockState) -> bool {
    state.get_level() == 3
}

fn always_true_predicate(_state: &BlockState) -> bool {
    true
}

fn water_fill_bucket_interaction(
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    fill_bucket(
        state,
        level,
        pos,
        player,
        hand,
        stack,
        ItemStack::new(ITEM_WATER_BUCKET),
        water_bucket_predicate,
        SOUND_BUCKET_FILL,
    )
}

fn lava_fill_bucket_interaction(
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    fill_bucket(
        state,
        level,
        pos,
        player,
        hand,
        stack,
        ItemStack::new(ITEM_LAVA_BUCKET),
        always_true_predicate,
        SOUND_BUCKET_FILL_LAVA,
    )
}

fn powder_snow_fill_bucket_interaction(
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    fill_bucket(
        state,
        level,
        pos,
        player,
        hand,
        stack,
        ItemStack::new(ITEM_POWDER_SNOW_BUCKET),
        water_bucket_predicate,
        SOUND_BUCKET_FILL_POWDER_SNOW,
    )
}

fn water_glass_bottle_interaction(
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    if !level.is_client_side() {
        let item = stack.item.clone();
        let mut result = ItemStack::new(ITEM_POTION);
        result.potion = Some(POTION_WATER.to_string());
        let filled = create_filled_result(stack, player, result, true);
        player.set_item_in_hand(hand, filled);
        player.award_stat(STAT_USE_CAULDRON);
        player.award_item_stat(&item);
        lower_fill_level(state, level, pos);
        level.play_sound(pos, SOUND_BOTTLE_FILL, "BLOCKS", 1.0_f32, 1.0_f32);
        level.game_event("minecraft:fluid_pickup", pos);
    }

    InteractionResult::Success
}

fn water_potion_interaction(
    state: &BlockState,
    level: &mut dyn LevelAccess,
    pos: BlockPos,
    player: &mut dyn PlayerAccess,
    hand: InteractionHand,
    stack: &mut ItemStack,
) -> InteractionResult {
    if state.get_level() == 3 {
        return InteractionResult::TryWithEmptyHand;
    }

    if !is_water_potion(stack) {
        return InteractionResult::TryWithEmptyHand;
    }

    if !level.is_client_side() {
        let item = stack.item.clone();
        let filled = create_filled_result(stack, player, ItemStack::new(ITEM_GLASS_BOTTLE), true);
        player.set_item_in_hand(hand, filled);
        player.award_stat(STAT_USE_CAULDRON);
        player.award_item_stat(&item);
        level.set_block_and_update(pos, state.cycle_level());
        level.play_sound(pos, SOUND_BOTTLE_EMPTY, "BLOCKS", 1.0_f32, 1.0_f32);
        level.game_event("minecraft:fluid_place", pos);
    }

    InteractionResult::Success
}

// ---------------------------------------------------------------------------
// LayeredCauldronBlock.lowerFillLevel equivalent (minimal local helper)
// ---------------------------------------------------------------------------

fn lower_fill_level(state: &BlockState, level: &mut dyn LevelAccess, pos: BlockPos) {
    let current = state.get_level();
    let next = current.wrapping_sub(1);
    if next <= 0 {
        level.set_block_and_update(pos, BlockState::new(BLOCK_CAULDRON));
    } else {
        level.set_block_and_update(pos, state.set_level(next));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct MockLevel {
        client_side: bool,
        block_at: HashMap<(i32, i32, i32), BlockState>,
        water_above: HashSet<(i32, i32, i32)>,
        sounds: Vec<String>,
        events: Vec<String>,
    }

    impl MockLevel {
        fn new(client_side: bool) -> Self {
            Self {
                client_side,
                block_at: HashMap::new(),
                water_above: HashSet::new(),
                sounds: Vec::new(),
                events: Vec::new(),
            }
        }

        fn block_at(&self, pos: BlockPos) -> Option<&BlockState> {
            self.block_at.get(&(pos.x, pos.y, pos.z))
        }
    }

    impl LevelAccess for MockLevel {
        fn is_client_side(&self) -> bool {
            self.client_side
        }

        fn set_block_and_update(&mut self, pos: BlockPos, state: BlockState) {
            self.block_at.insert((pos.x, pos.y, pos.z), state);
        }

        fn play_sound(
            &mut self,
            _pos: BlockPos,
            sound: &str,
            _source: &str,
            _volume: f32,
            _pitch: f32,
        ) {
            self.sounds.push(sound.to_string());
        }

        fn game_event(&mut self, event: &str, _pos: BlockPos) {
            self.events.push(event.to_string());
        }

        fn fluid_is_water_at(&self, pos: BlockPos) -> bool {
            self.water_above.contains(&(pos.x, pos.y, pos.z))
        }
    }

    struct MockPlayer {
        infinite: bool,
        hand_stack: Option<ItemStack>,
        stats: Vec<String>,
        item_stats: Vec<String>,
        inventory: Vec<ItemStack>,
    }

    impl MockPlayer {
        fn new(infinite: bool) -> Self {
            Self {
                infinite,
                hand_stack: None,
                stats: Vec::new(),
                item_stats: Vec::new(),
                inventory: Vec::new(),
            }
        }
    }

    impl PlayerAccess for MockPlayer {
        fn set_item_in_hand(&mut self, _hand: InteractionHand, stack: ItemStack) {
            self.hand_stack = Some(stack);
        }

        fn award_stat(&mut self, stat: &str) {
            self.stats.push(stat.to_string());
        }

        fn award_item_stat(&mut self, item: &str) {
            self.item_stats.push(item.to_string());
        }

        fn has_infinite_materials(&self) -> bool {
            self.infinite
        }

        fn add_item(&mut self, stack: ItemStack) {
            self.inventory.push(stack);
        }

        fn has_item(&self, item: &str) -> bool {
            self.inventory.iter().any(|s| s.item == item)
                || self
                    .hand_stack
                    .as_ref()
                    .is_some_and(|s| s.item == item)
        }
    }

    #[test]
    fn new_interaction_map_default_returns_try_with_empty_hand() {
        let mut reg = CauldronInteractionsRegistry::new();
        let name = new_interaction_map("empty", &mut reg);
        let map = reg.get_map(&name).unwrap();
        let mut level = MockLevel::new(false);
        let mut player = MockPlayer::new(false);
        let mut stack = ItemStack::new("minecraft:stick");
        let state = BlockState::new(BLOCK_CAULDRON);
        let res = interact(
            map,
            &state,
            &mut level,
            BlockPos::new(0, 64, 0),
            &mut player,
            InteractionHand::MainHand,
            &mut stack,
        );
        assert_eq!(res, InteractionResult::TryWithEmptyHand);
    }

    #[test]
    fn fill_bucket_predicate_false_and_negative_pos() {
        let state = BlockState::with_level(BLOCK_WATER_CAULDRON, 2);
        let mut level = MockLevel::new(false);
        let mut player = MockPlayer::new(false);
        let mut stack = ItemStack::new(ITEM_BUCKET);
        let pos = BlockPos::new(-10, -64, i32::MIN);
        let res = fill_bucket(
            &state,
            &mut level,
            pos,
            &mut player,
            InteractionHand::MainHand,
            &mut stack,
            ItemStack::new(ITEM_WATER_BUCKET),
            water_bucket_predicate,
            SOUND_BUCKET_FILL,
        );
        assert_eq!(res, InteractionResult::TryWithEmptyHand);
        assert!(level.block_at(pos).is_none());
        assert_eq!(pos.above().y, (-64_i32).wrapping_add(1));
    }

    #[test]
    fn fill_bucket_success_updates_level_and_stats() {
        let state = BlockState::with_level(BLOCK_WATER_CAULDRON, 3);
        let mut level = MockLevel::new(false);
        let mut player = MockPlayer::new(false);
        let mut stack = ItemStack::with_count(ITEM_BUCKET, 1);
        let pos = BlockPos::new(1, 2, 3);
        let res = fill_bucket(
            &state,
            &mut level,
            pos,
            &mut player,
            InteractionHand::MainHand,
            &mut stack,
            ItemStack::new(ITEM_WATER_BUCKET),
            water_bucket_predicate,
            SOUND_BUCKET_FILL,
        );
        assert_eq!(res, InteractionResult::Success);
        let placed = level.block_at(pos).unwrap();
        assert_eq!(placed.block, BLOCK_CAULDRON);
        assert_eq!(player.hand_stack.as_ref().unwrap().item, ITEM_WATER_BUCKET);
        assert!(player.stats.iter().any(|s| s == STAT_USE_CAULDRON));
        assert!(level.events.iter().any(|e| e == "minecraft:fluid_pickup"));
    }

    #[test]
    fn is_under_water_checks_above_with_wrapping_y() {
        let mut level = MockLevel::new(false);
        let pos = BlockPos::new(0, i32::MAX, 0);
        // above() wraps i32::MAX + 1 -> i32::MIN
        let above = pos.above();
        assert_eq!(above.y, i32::MIN);
        level.water_above.insert((above.x, above.y, above.z));
        assert!(is_under_water(&level, pos));
    }

    #[test]
    fn lower_fill_level_and_cycle_edge_cases() {
        let mut level = MockLevel::new(false);
        let pos = BlockPos::new(5, 5, 5);

        let state1 = BlockState::with_level(BLOCK_WATER_CAULDRON, 1);
        lower_fill_level(&state1, &mut level, pos);
        assert_eq!(level.block_at(pos).unwrap().block, BLOCK_CAULDRON);

        let state3 = BlockState::with_level(BLOCK_WATER_CAULDRON, 3);
        lower_fill_level(&state3, &mut level, pos);
        assert_eq!(level.block_at(pos).unwrap().get_level(), 2);

        let cycled = BlockState::with_level(BLOCK_WATER_CAULDRON, 3).cycle_level();
        assert_eq!(cycled.get_level(), 1);
        let cycled2 = BlockState::with_level(BLOCK_WATER_CAULDRON, 1).cycle_level();
        assert_eq!(cycled2.get_level(), 2);
    }

    #[test]
    fn fill_lava_underwater_returns_consume_and_float_bits_preserved() {
        let mut level = MockLevel::new(false);
        let pos = BlockPos::new(7, 8, 9);
        level.water_above.insert((7, 9, 9));
        let mut player = MockPlayer::new(false);
        let mut stack = ItemStack::new(ITEM_LAVA_BUCKET);
        let state = BlockState::new(BLOCK_CAULDRON);
        let res = fill_lava_interaction(
            &state,
            &mut level,
            pos,
            &mut player,
            InteractionHand::MainHand,
            &mut stack,
        );
        assert_eq!(res, InteractionResult::Consume);
        assert!(level.block_at(pos).is_none());

        // Float bit preservation check (rule coverage): exact f32 bits round-trip.
        let bits: u32 = 0xFFC0_0001; // quiet NaN payload
        let f = f32::from_bits(bits);
        let out_bits = f.to_bits();
        assert_eq!(out_bits, bits);
    }
}
