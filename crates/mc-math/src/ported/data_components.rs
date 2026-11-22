use std::marker::PhantomData;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Unit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CustomData;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Component(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Identifier(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemLore;
impl ItemLore {
    pub const EMPTY: Self = Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemEnchantments;
impl ItemEnchantments {
    pub const EMPTY: Self = Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemAttributeModifiers;
impl ItemAttributeModifiers {
    pub const EMPTY: Self = Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UseEffects;
impl UseEffects {
    pub const DEFAULT: Self = Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TooltipDisplay;
impl TooltipDisplay {
    pub const DEFAULT: Self = Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SwingAnimation;
impl SwingAnimation {
    pub const DEFAULT: Self = Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
}
impl Rarity {
    pub const COMMON: Self = Self::Common;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SoundEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Holder<T> {
    id: &'static str,
    _marker: PhantomData<fn() -> T>,
}
impl<T> Holder<T> {
    pub const fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
    pub const fn id(&self) -> &'static str {
        self.id
    }
}

pub struct SoundEvents;
impl SoundEvents {
    pub const ITEM_BREAK: Holder<SoundEvent> = Holder::new("minecraft:item.break");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EitherHolder<T>(PhantomData<fn() -> T>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TypedEntityData<T>(PhantomData<fn() -> T>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TagKey<T>(PhantomData<fn() -> T>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceKey<T>(PhantomData<fn() -> T>);

#[derive(Debug, Default)]
pub struct Registry<T>(PhantomData<T>);
impl<T> Registry<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

macro_rules! stub_types {
    ($($name:ident),* $(,)?) => {
        $( #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub struct $name; )*
    };
}

stub_types!(
    AdventureModePredicate,
    CustomModelData,
    FoodProperties,
    Consumable,
    UseRemainder,
    UseCooldown,
    DamageResistant,
    Tool,
    Weapon,
    AttackRange,
    Enchantable,
    Equippable,
    Repairable,
    DeathProtection,
    BlocksAttacks,
    PiercingWeapon,
    KineticWeapon,
    DyedItemColor,
    MapItemColor,
    MapId,
    MapDecorations,
    MapPostProcessing,
    ChargedProjectiles,
    BundleContents,
    PotionContents,
    SuspiciousStewEffects,
    WritableBookContent,
    WrittenBookContent,
    ArmorTrim,
    DebugStickState,
    InstrumentComponent,
    ProvidesTrimMaterial,
    OminousBottleAmplifier,
    JukeboxPlayable,
    LodestoneTracker,
    FireworkExplosion,
    Fireworks,
    ResolvableProfile,
    BannerPatternLayers,
    DyeColor,
    PotDecorations,
    ItemContainerContents,
    BlockItemStateProperties,
    Bees,
    LockCode,
    SeededContainerLoot,
    DamageType,
    EntityType,
    BlockEntityType,
    VillagerType,
    WolfVariant,
    WolfSoundVariant,
    FoxVariant,
    SalmonVariant,
    ParrotVariant,
    TropicalFishPattern,
    MooshroomVariant,
    RabbitVariant,
    PigVariant,
    CowVariant,
    ChickenVariant,
    ZombieNautilusVariant,
    FrogVariant,
    HorseVariant,
    PaintingVariant,
    LlamaVariant,
    AxolotlVariant,
    CatVariant,
    BannerPattern,
    Recipe
);

// ---------------------------------------------------------------------------
// Core DataComponentType / Builder
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct DataComponentType<T> {
    name: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T> DataComponentType<T> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> Clone for DataComponentType<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for DataComponentType<T> {}
impl<T> PartialEq for DataComponentType<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl<T> Eq for DataComponentType<T> {}

pub struct DataComponentBuilder<T> {
    name: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T> DataComponentBuilder<T> {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    pub fn persistent<C>(self, _codec: C) -> Self {
        self
    }

    pub fn network_synchronized<C>(self, _codec: C) -> Self {
        self
    }

    pub fn cache_encoding(self) -> Self {
        self
    }

    pub fn ignore_swap_animation(self) -> Self {
        self
    }

    fn build(self) -> DataComponentType<T> {
        DataComponentType::new(self.name)
    }
}

// JAVA: net.minecraft.core.component.DataComponents#register
fn register<T, F>(name: &'static str, f: F) -> DataComponentType<T>
where
    F: FnOnce(DataComponentBuilder<T>) -> DataComponentBuilder<T>,
{
    // Faithful to Java shape; actual BuiltInRegistries insertion is omitted in this self-contained port.
    f(DataComponentBuilder::new(name)).build()
}

macro_rules! component {
    ($name:ident, $ty:ty, $id:expr, |$b:ident| $body:expr) => {
        pub static $name: LazyLock<DataComponentType<$ty>> =
            LazyLock::new(|| register($id, |$b| $body));
    };
}

// ---------------------------------------------------------------------------
// Data component constants
// ---------------------------------------------------------------------------

component!(CUSTOM_DATA, CustomData, "custom_data", |b| b.persistent(()));
component!(MAX_STACK_SIZE, i32, "max_stack_size", |b| b.persistent(()).network_synchronized(()));
component!(MAX_DAMAGE, i32, "max_damage", |b| b.persistent(()).network_synchronized(()));
component!(DAMAGE, i32, "damage", |b| b.persistent(()).ignore_swap_animation().network_synchronized(()));
component!(UNBREAKABLE, Unit, "unbreakable", |b| b.persistent(()).network_synchronized(()));
component!(USE_EFFECTS, UseEffects, "use_effects", |b| b.persistent(()).network_synchronized(()));
component!(CUSTOM_NAME, Component, "custom_name", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(MINIMUM_ATTACK_CHARGE, f32, "minimum_attack_charge", |b| b.persistent(()).network_synchronized(()));
component!(DAMAGE_TYPE, EitherHolder<DamageType>, "damage_type", |b| b.persistent(()).network_synchronized(()));
component!(ITEM_NAME, Component, "item_name", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(ITEM_MODEL, Identifier, "item_model", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(LORE, ItemLore, "lore", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(RARITY, Rarity, "rarity", |b| b.persistent(()).network_synchronized(()));
component!(ENCHANTMENTS, ItemEnchantments, "enchantments", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(CAN_PLACE_ON, AdventureModePredicate, "can_place_on", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(CAN_BREAK, AdventureModePredicate, "can_break", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(ATTRIBUTE_MODIFIERS, ItemAttributeModifiers, "attribute_modifiers", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(CUSTOM_MODEL_DATA, CustomModelData, "custom_model_data", |b| b.persistent(()).network_synchronized(()));
component!(TOOLTIP_DISPLAY, TooltipDisplay, "tooltip_display", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(REPAIR_COST, i32, "repair_cost", |b| b.persistent(()).network_synchronized(()));
component!(CREATIVE_SLOT_LOCK, Unit, "creative_slot_lock", |b| b.network_synchronized(()));
component!(ENCHANTMENT_GLINT_OVERRIDE, bool, "enchantment_glint_override", |b| b.persistent(()).network_synchronized(()));
component!(INTANGIBLE_PROJECTILE, Unit, "intangible_projectile", |b| b.persistent(()));
component!(FOOD, FoodProperties, "food", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(CONSUMABLE, Consumable, "consumable", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(USE_REMAINDER, UseRemainder, "use_remainder", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(USE_COOLDOWN, UseCooldown, "use_cooldown", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(DAMAGE_RESISTANT, DamageResistant, "damage_resistant", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(TOOL, Tool, "tool", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(WEAPON, Weapon, "weapon", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(ATTACK_RANGE, AttackRange, "attack_range", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(ENCHANTABLE, Enchantable, "enchantable", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(EQUIPPABLE, Equippable, "equippable", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(REPAIRABLE, Repairable, "repairable", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(GLIDER, Unit, "glider", |b| b.persistent(()).network_synchronized(()));
component!(TOOLTIP_STYLE, Identifier, "tooltip_style", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(DEATH_PROTECTION, DeathProtection, "death_protection", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(BLOCKS_ATTACKS, BlocksAttacks, "blocks_attacks", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(PIERCING_WEAPON, PiercingWeapon, "piercing_weapon", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(KINETIC_WEAPON, KineticWeapon, "kinetic_weapon", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(SWING_ANIMATION, SwingAnimation, "swing_animation", |b| b.persistent(()).network_synchronized(()));
component!(STORED_ENCHANTMENTS, ItemEnchantments, "stored_enchantments", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(DYED_COLOR, DyedItemColor, "dyed_color", |b| b.persistent(()).network_synchronized(()));
component!(MAP_COLOR, MapItemColor, "map_color", |b| b.persistent(()).network_synchronized(()));
component!(MAP_ID, MapId, "map_id", |b| b.persistent(()).network_synchronized(()));
component!(MAP_DECORATIONS, MapDecorations, "map_decorations", |b| b.persistent(()).cache_encoding());
component!(MAP_POST_PROCESSING, MapPostProcessing, "map_post_processing", |b| b.network_synchronized(()));
component!(CHARGED_PROJECTILES, ChargedProjectiles, "charged_projectiles", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(BUNDLE_CONTENTS, BundleContents, "bundle_contents", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(POTION_CONTENTS, PotionContents, "potion_contents", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(POTION_DURATION_SCALE, f32, "potion_duration_scale", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(SUSPICIOUS_STEW_EFFECTS, SuspiciousStewEffects, "suspicious_stew_effects", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(WRITABLE_BOOK_CONTENT, WritableBookContent, "writable_book_content", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(WRITTEN_BOOK_CONTENT, WrittenBookContent, "written_book_content", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(TRIM, ArmorTrim, "trim", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(DEBUG_STICK_STATE, DebugStickState, "debug_stick_state", |b| b.persistent(()).cache_encoding());
component!(ENTITY_DATA, TypedEntityData<EntityType>, "entity_data", |b| b.persistent(()).network_synchronized(()));
component!(BUCKET_ENTITY_DATA, CustomData, "bucket_entity_data", |b| b.persistent(()).network_synchronized(()));
component!(BLOCK_ENTITY_DATA, TypedEntityData<BlockEntityType>, "block_entity_data", |b| b.persistent(()).network_synchronized(()));
component!(INSTRUMENT, InstrumentComponent, "instrument", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(PROVIDES_TRIM_MATERIAL, ProvidesTrimMaterial, "provides_trim_material", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(OMINOUS_BOTTLE_AMPLIFIER, OminousBottleAmplifier, "ominous_bottle_amplifier", |b| b.persistent(()).network_synchronized(()));
component!(JUKEBOX_PLAYABLE, JukeboxPlayable, "jukebox_playable", |b| b.persistent(()).network_synchronized(()));
component!(PROVIDES_BANNER_PATTERNS, TagKey<BannerPattern>, "provides_banner_patterns", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(RECIPES, Vec<ResourceKey<Recipe>>, "recipes", |b| b.persistent(()).cache_encoding());
component!(LODESTONE_TRACKER, LodestoneTracker, "lodestone_tracker", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(FIREWORK_EXPLOSION, FireworkExplosion, "firework_explosion", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(FIREWORKS, Fireworks, "fireworks", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(PROFILE, ResolvableProfile, "profile", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(NOTE_BLOCK_SOUND, Identifier, "note_block_sound", |b| b.persistent(()).network_synchronized(()));
component!(BANNER_PATTERNS, BannerPatternLayers, "banner_patterns", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(BASE_COLOR, DyeColor, "base_color", |b| b.persistent(()).network_synchronized(()));
component!(POT_DECORATIONS, PotDecorations, "pot_decorations", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(CONTAINER, ItemContainerContents, "container", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(BLOCK_STATE, BlockItemStateProperties, "block_state", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(BEES, Bees, "bees", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(LOCK, LockCode, "lock", |b| b.persistent(()));
component!(CONTAINER_LOOT, SeededContainerLoot, "container_loot", |b| b.persistent(()));
component!(BREAK_SOUND, Holder<SoundEvent>, "break_sound", |b| b.persistent(()).network_synchronized(()).cache_encoding());
component!(VILLAGER_VARIANT, Holder<VillagerType>, "villager/variant", |b| b.persistent(()).network_synchronized(()));
component!(WOLF_VARIANT, Holder<WolfVariant>, "wolf/variant", |b| b.persistent(()).network_synchronized(()));
component!(WOLF_SOUND_VARIANT, Holder<WolfSoundVariant>, "wolf/sound_variant", |b| b.persistent(()).network_synchronized(()));
component!(WOLF_COLLAR, DyeColor, "wolf/collar", |b| b.persistent(()).network_synchronized(()));
component!(FOX_VARIANT, FoxVariant, "fox/variant", |b| b.persistent(()).network_synchronized(()));
component!(SALMON_SIZE, SalmonVariant, "salmon/size", |b| b.persistent(()).network_synchronized(()));
component!(PARROT_VARIANT, ParrotVariant, "parrot/variant", |b| b.persistent(()).network_synchronized(()));
component!(TROPICAL_FISH_PATTERN, TropicalFishPattern, "tropical_fish/pattern", |b| b.persistent(()).network_synchronized(()));
component!(TROPICAL_FISH_BASE_COLOR, DyeColor, "tropical_fish/base_color", |b| b.persistent(()).network_synchronized(()));
component!(TROPICAL_FISH_PATTERN_COLOR, DyeColor, "tropical_fish/pattern_color", |b| b.persistent(()).network_synchronized(()));
component!(MOOSHROOM_VARIANT, MooshroomVariant, "mooshroom/variant", |b| b.persistent(()).network_synchronized(()));
component!(RABBIT_VARIANT, RabbitVariant, "rabbit/variant", |b| b.persistent(()).network_synchronized(()));
component!(PIG_VARIANT, Holder<PigVariant>, "pig/variant", |b| b.persistent(()).network_synchronized(()));
component!(COW_VARIANT, Holder<CowVariant>, "cow/variant", |b| b.persistent(()).network_synchronized(()));
component!(CHICKEN_VARIANT, EitherHolder<ChickenVariant>, "chicken/variant", |b| b.persistent(()).network_synchronized(()));
component!(ZOMBIE_NAUTILUS_VARIANT, EitherHolder<ZombieNautilusVariant>, "zombie_nautilus/variant", |b| b.persistent(()).network_synchronized(()));
component!(FROG_VARIANT, Holder<FrogVariant>, "frog/variant", |b| b.persistent(()).network_synchronized(()));
component!(HORSE_VARIANT, HorseVariant, "horse/variant", |b| b.persistent(()).network_synchronized(()));
component!(PAINTING_VARIANT, Holder<PaintingVariant>, "painting/variant", |b| b.persistent(()).network_synchronized(()));
component!(LLAMA_VARIANT, LlamaVariant, "llama/variant", |b| b.persistent(()).network_synchronized(()));
component!(AXOLOTL_VARIANT, AxolotlVariant, "axolotl/variant", |b| b.persistent(()).network_synchronized(()));
component!(CAT_VARIANT, Holder<CatVariant>, "cat/variant", |b| b.persistent(()).network_synchronized(()));
component!(CAT_COLLAR, DyeColor, "cat/collar", |b| b.persistent(()).network_synchronized(()));
component!(SHEEP_COLOR, DyeColor, "sheep/color", |b| b.persistent(()).network_synchronized(()));
component!(SHULKER_COLOR, DyeColor, "shulker/color", |b| b.persistent(()).network_synchronized(()));

// JAVA: net.minecraft.core.component.DataComponents#bootstrap
pub fn bootstrap(_registry: &mut Registry<DataComponentType<()>>) -> DataComponentType<CustomData> {
    *CUSTOM_DATA
}

// ---------------------------------------------------------------------------
// Common item components map
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct DataComponentMap {
    pub max_stack_size: i32,
    pub lore: ItemLore,
    pub enchantments: ItemEnchantments,
    pub repair_cost: i32,
    pub use_effects: UseEffects,
    pub attribute_modifiers: ItemAttributeModifiers,
    pub rarity: Rarity,
    pub break_sound: Holder<SoundEvent>,
    pub tooltip_display: TooltipDisplay,
    pub swing_animation: SwingAnimation,
}

pub struct DataComponentMapBuilder {
    map: DataComponentMap,
}

impl DataComponentMapBuilder {
    pub fn new() -> Self {
        Self {
            map: DataComponentMap {
                max_stack_size: 0,
                lore: ItemLore::EMPTY,
                enchantments: ItemEnchantments::EMPTY,
                repair_cost: 0,
                use_effects: UseEffects::DEFAULT,
                attribute_modifiers: ItemAttributeModifiers::EMPTY,
                rarity: Rarity::COMMON,
                break_sound: SoundEvents::ITEM_BREAK,
                tooltip_display: TooltipDisplay::DEFAULT,
                swing_animation: SwingAnimation::DEFAULT,
            },
        }
    }

    pub fn set_max_stack_size(mut self, v: i32) -> Self {
        self.map.max_stack_size = v;
        self
    }
    pub fn set_lore(mut self, v: ItemLore) -> Self {
        self.map.lore = v;
        self
    }
    pub fn set_enchantments(mut self, v: ItemEnchantments) -> Self {
        self.map.enchantments = v;
        self
    }
    pub fn set_repair_cost(mut self, v: i32) -> Self {
        self.map.repair_cost = v;
        self
    }
    pub fn set_use_effects(mut self, v: UseEffects) -> Self {
        self.map.use_effects = v;
        self
    }
    pub fn set_attribute_modifiers(mut self, v: ItemAttributeModifiers) -> Self {
        self.map.attribute_modifiers = v;
        self
    }
    pub fn set_rarity(mut self, v: Rarity) -> Self {
        self.map.rarity = v;
        self
    }
    pub fn set_break_sound(mut self, v: Holder<SoundEvent>) -> Self {
        self.map.break_sound = v;
        self
    }
    pub fn set_tooltip_display(mut self, v: TooltipDisplay) -> Self {
        self.map.tooltip_display = v;
        self
    }
    pub fn set_swing_animation(mut self, v: SwingAnimation) -> Self {
        self.map.swing_animation = v;
        self
    }

    pub fn build(self) -> DataComponentMap {
        self.map
    }
}

impl Default for DataComponentMapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DataComponentMap {
    pub fn builder() -> DataComponentMapBuilder {
        DataComponentMapBuilder::new()
    }
}

pub static COMMON_ITEM_COMPONENTS: LazyLock<DataComponentMap> = LazyLock::new(|| {
    DataComponentMap::builder()
        .set_max_stack_size(64)
        .set_lore(ItemLore::EMPTY)
        .set_enchantments(ItemEnchantments::EMPTY)
        .set_repair_cost(0)
        .set_use_effects(UseEffects::DEFAULT)
        .set_attribute_modifiers(ItemAttributeModifiers::EMPTY)
        .set_rarity(Rarity::COMMON)
        .set_break_sound(SoundEvents::ITEM_BREAK)
        .set_tooltip_display(TooltipDisplay::DEFAULT)
        .set_swing_animation(SwingAnimation::DEFAULT)
        .build()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_returns_custom_data() {
        let mut reg = Registry::<DataComponentType<()>>::new();
        let got = bootstrap(&mut reg);
        assert_eq!(got.name(), "custom_data");
        assert_eq!(got, *CUSTOM_DATA);
    }

    #[test]
    fn register_creates_named_component_with_builder_chain() {
        let c: DataComponentType<i32> =
            register("test_component", |b| b.persistent(()).network_synchronized(()).cache_encoding());
        assert_eq!(c.name(), "test_component");
        let c2: DataComponentType<Unit> = register("swap_anim", |b| b.persistent(()).ignore_swap_animation());
        assert_eq!(c2.name(), "swap_anim");
    }

    #[test]
    fn common_item_components_defaults() {
        let c = &*COMMON_ITEM_COMPONENTS;
        assert_eq!(c.max_stack_size, 64);
        assert_eq!(c.repair_cost, 0);
        assert_eq!(c.lore, ItemLore::EMPTY);
        assert_eq!(c.enchantments, ItemEnchantments::EMPTY);
        assert_eq!(c.use_effects, UseEffects::DEFAULT);
        assert_eq!(c.attribute_modifiers, ItemAttributeModifiers::EMPTY);
        assert_eq!(c.rarity, Rarity::COMMON);
        assert_eq!(c.break_sound, SoundEvents::ITEM_BREAK);
        assert_eq!(c.tooltip_display, TooltipDisplay::DEFAULT);
        assert_eq!(c.swing_animation, SwingAnimation::DEFAULT);
    }

    #[test]
    fn integer_edge_cases_and_overflow() {
        let m = DataComponentMap::builder()
            .set_max_stack_size(i32::MAX)
            .set_repair_cost(i32::MIN)
            .build();
        assert_eq!(m.max_stack_size, i32::MAX);
        assert_eq!(m.repair_cost, i32::MIN);
        assert_eq!(m.max_stack_size.wrapping_add(1), i32::MIN);
        assert_eq!(m.repair_cost.wrapping_sub(1), i32::MAX);
        assert_eq!((-1i32).wrapping_mul(i32::MIN), i32::MIN);

        assert_eq!(MAX_STACK_SIZE.name(), "max_stack_size");
        assert_eq!(REPAIR_COST.name(), "repair_cost");
    }

    #[test]
    fn component_names_and_float_bits() {
        assert_eq!(CUSTOM_DATA.name(), "custom_data");
        assert_eq!(DAMAGE.name(), "damage");
        assert_eq!(MINIMUM_ATTACK_CHARGE.name(), "minimum_attack_charge");
        assert_eq!(POTION_DURATION_SCALE.name(), "potion_duration_scale");

        assert_eq!(0.0f32.to_bits(), 0x0000_0000);
        assert_eq!((-0.0f32).to_bits(), 0x8000_0000);
        assert_eq!(1.0f32.to_bits(), 0x3f80_0000);
        assert_eq!(f32::INFINITY.to_bits(), 0x7f80_0000);
        assert!(f32::from_bits(0x7fc0_0000).is_nan());
    }
}
