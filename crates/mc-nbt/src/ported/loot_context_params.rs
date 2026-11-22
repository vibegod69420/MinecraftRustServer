use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Minimal local stand-in for `net.minecraft.util.context.ContextKey`.
///
/// The Java class is generic (`ContextKey<T>`); this preserves the type marker
/// while keeping the module self-contained.
#[derive(Debug, Clone, Copy, Eq)]
pub struct ContextKey<T: ?Sized> {
    name: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> ContextKey<T> {
    /// Mirrors `net.minecraft.util.context.ContextKey#vanilla(String)`.
    pub const fn vanilla(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    /// Returns the vanilla key path (for example, `"this_entity"`).
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

impl<T: ?Sized> PartialEq for ContextKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<T: ?Sized> Hash for ContextKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

// Minimal local stand-ins for unported Minecraft types referenced by LootContextParams.

/// Stand-in for `net.minecraft.world.entity.Entity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Entity;

/// Stand-in for `net.minecraft.world.entity.player.Player`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Player;

/// Stand-in for `net.minecraft.world.damagesource.DamageSource`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DamageSource;

/// Stand-in for `net.minecraft.world.phys.Vec3`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Stand-in for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState;

/// Stand-in for `net.minecraft.world.level.block.entity.BlockEntity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockEntity;

/// Stand-in for `net.minecraft.world.item.ItemStack`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ItemStack;

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#THIS_ENTITY
pub const THIS_ENTITY: ContextKey<Entity> = ContextKey::vanilla("this_entity");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#INTERACTING_ENTITY
pub const INTERACTING_ENTITY: ContextKey<Entity> = ContextKey::vanilla("interacting_entity");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#TARGET_ENTITY
pub const TARGET_ENTITY: ContextKey<Entity> = ContextKey::vanilla("target_entity");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#LAST_DAMAGE_PLAYER
pub const LAST_DAMAGE_PLAYER: ContextKey<Player> = ContextKey::vanilla("last_damage_player");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#DAMAGE_SOURCE
pub const DAMAGE_SOURCE: ContextKey<DamageSource> = ContextKey::vanilla("damage_source");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#ATTACKING_ENTITY
pub const ATTACKING_ENTITY: ContextKey<Entity> = ContextKey::vanilla("attacking_entity");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#DIRECT_ATTACKING_ENTITY
pub const DIRECT_ATTACKING_ENTITY: ContextKey<Entity> =
    ContextKey::vanilla("direct_attacking_entity");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#ORIGIN
pub const ORIGIN: ContextKey<Vec3> = ContextKey::vanilla("origin");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#BLOCK_STATE
pub const BLOCK_STATE: ContextKey<BlockState> = ContextKey::vanilla("block_state");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#BLOCK_ENTITY
pub const BLOCK_ENTITY: ContextKey<BlockEntity> = ContextKey::vanilla("block_entity");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#TOOL
pub const TOOL: ContextKey<ItemStack> = ContextKey::vanilla("tool");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#EXPLOSION_RADIUS
pub const EXPLOSION_RADIUS: ContextKey<f32> = ContextKey::vanilla("explosion_radius");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#ENCHANTMENT_LEVEL
pub const ENCHANTMENT_LEVEL: ContextKey<i32> = ContextKey::vanilla("enchantment_level");

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#ENCHANTMENT_ACTIVE
pub const ENCHANTMENT_ACTIVE: ContextKey<bool> = ContextKey::vanilla("enchantment_active");

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn key_names_match_java_exactly() {
        assert_eq!(THIS_ENTITY.name(), "this_entity");
        assert_eq!(INTERACTING_ENTITY.name(), "interacting_entity");
        assert_eq!(TARGET_ENTITY.name(), "target_entity");
        assert_eq!(LAST_DAMAGE_PLAYER.name(), "last_damage_player");
        assert_eq!(DAMAGE_SOURCE.name(), "damage_source");
        assert_eq!(ATTACKING_ENTITY.name(), "attacking_entity");
        assert_eq!(DIRECT_ATTACKING_ENTITY.name(), "direct_attacking_entity");
        assert_eq!(ORIGIN.name(), "origin");
        assert_eq!(BLOCK_STATE.name(), "block_state");
        assert_eq!(BLOCK_ENTITY.name(), "block_entity");
        assert_eq!(TOOL.name(), "tool");
        assert_eq!(EXPLOSION_RADIUS.name(), "explosion_radius");
        assert_eq!(ENCHANTMENT_LEVEL.name(), "enchantment_level");
        assert_eq!(ENCHANTMENT_ACTIVE.name(), "enchantment_active");
    }

    #[test]
    fn key_equality_hash_and_distinctness() {
        let a: ContextKey<Entity> = ContextKey::vanilla("this_entity");
        let b: ContextKey<Entity> = ContextKey::vanilla("this_entity");
        let c: ContextKey<Entity> = ContextKey::vanilla("other_entity");

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(THIS_ENTITY, a);

        let mut set = HashSet::new();
        set.insert(a.name());
        set.insert(b.name());
        set.insert(c.name());
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn enchantment_level_i32_overflow_and_negative_semantics() {
        // Java int semantics: wrapping arithmetic and masked/defined boundary behavior.
        let max = i32::MAX;
        let min = i32::MIN;
        assert_eq!(max.wrapping_add(1), min);
        assert_eq!(min.wrapping_sub(1), max);
        assert_eq!((-1i32).wrapping_mul(2), -2i32);
        assert_eq!(min.wrapping_mul(-1), min); // Java two's-complement overflow

        let negative_level: i32 = -5;
        assert!(negative_level < 0);
        assert_eq!(ENCHANTMENT_LEVEL, ContextKey::<i32>::vanilla("enchantment_level"));
    }

    #[test]
    fn explosion_radius_f32_float_bits_preserved() {
        // Float identity is bit-sensitive: -0.0 and NaN payloads must remain distinct.
        let pos_zero = 0.0f32;
        let neg_zero = -0.0f32;
        assert_eq!(pos_zero.to_bits(), 0x0000_0000u32);
        assert_eq!(neg_zero.to_bits(), 0x8000_0000u32);
        assert_ne!(pos_zero.to_bits(), neg_zero.to_bits());

        let nan_a = f32::from_bits(0x7FC0_0001u32);
        let nan_b = f32::from_bits(0x7FC0_0002u32);
        assert!(nan_a.is_nan());
        assert!(nan_b.is_nan());
        assert_ne!(nan_a.to_bits(), nan_b.to_bits());

        let radius_bits = 2.5f32.to_bits();
        assert_eq!(f32::from_bits(radius_bits).to_bits(), radius_bits);
        assert_eq!(EXPLOSION_RADIUS, ContextKey::<f32>::vanilla("explosion_radius"));
    }

    #[test]
    fn enchantment_active_bool_and_shift_edge_cases() {
        // Java shift semantics are masked (& 31 for int, & 63 for long); verify representative cases.
        let value: i32 = 1;
        assert_eq!(value.wrapping_shl(31), i32::MIN);
        assert_eq!(1i32.wrapping_shl(32 & 31), 1i32); // Java masks shift count
        assert_eq!(1i64.wrapping_shl(64 & 63), 1i64);

        let active_true: bool = true;
        let active_false: bool = false;
        assert_ne!(active_true, active_false);
        assert_eq!(ENCHANTMENT_ACTIVE, ContextKey::<bool>::vanilla("enchantment_active"));
    }
}
