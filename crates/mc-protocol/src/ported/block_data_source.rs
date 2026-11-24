use std::fmt;
use std::hash::{Hash, Hasher};

/// Minimal NBT compound tag representation for this self-contained port.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundTag {
    pub id: i32,
}

impl CompoundTag {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
}

/// Minimal registry access marker.
pub trait RegistryAccess: fmt::Debug {}

/// Block position (x, y, z).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Minimal block entity abstraction.
pub trait BlockEntity {
    fn save_with_full_metadata(&self, registry_access: &dyn RegistryAccess) -> CompoundTag;
}

/// Minimal server level abstraction.
pub trait ServerLevel {
    fn is_loaded(&self, pos: BlockPos) -> bool;
    fn get_block_entity<'a>(&'a self, pos: BlockPos) -> Option<&'a dyn BlockEntity>;
}

/// Minimal command source stack abstraction.
pub trait CommandSourceStack {
    fn get_level(&self) -> &dyn ServerLevel;
    fn registry_access(&self) -> &dyn RegistryAccess;
    /// Returns (x, y, z) source position.
    fn position(&self) -> (f64, f64, f64);
    /// Returns (y_rot_degrees, x_rot_degrees).
    fn rotation(&self) -> (f32, f32);
}

/// Parsed coordinates that can resolve to a block position.
pub trait Coordinates: fmt::Debug {
    fn get_block_pos(&self, source: &dyn CommandSourceStack) -> BlockPos;
}

/// Minimal data source abstraction.
pub trait DataSource {
    fn get_data(&self, source: &dyn CommandSourceStack) -> Vec<CompoundTag>;
}

/// Placeholder for Mojang's MapCodec, not available in this self-contained port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodecPlaceholder;

/// Static placeholder corresponding to `MAP_CODEC`.
pub static MAP_CODEC: MapCodecPlaceholder = MapCodecPlaceholder;

#[derive(Debug, Clone, Copy, PartialEq)]
struct WorldCoordinates {
    x: f64,
    rx: bool,
    y: f64,
    ry: bool,
    z: f64,
    rz: bool,
}

impl Coordinates for WorldCoordinates {
    fn get_block_pos(&self, source: &dyn CommandSourceStack) -> BlockPos {
        let (sx, sy, sz) = source.position();
        let x = self.x + if self.rx { sx } else { 0.0 };
        let y = self.y + if self.ry { sy } else { 0.0 };
        let z = self.z + if self.rz { sz } else { 0.0 };
        BlockPos::new(floor_to_i32(x), floor_to_i32(y), floor_to_i32(z))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LocalCoordinates {
    left: f64,
    up: f64,
    forwards: f64,
}

impl Coordinates for LocalCoordinates {
    fn get_block_pos(&self, source: &dyn CommandSourceStack) -> BlockPos {
        let (y_rot, x_rot) = source.rotation();
        let (px, py, pz) = source.position();
        let pi = std::f32::consts::PI;
        let deg_to_rad = pi / 180.0f32;

        let yaw = (y_rot + 90.0f32) * deg_to_rad;
        let pitch_neg = (-x_rot) * deg_to_rad;
        let pitch_neg_plus_90 = ((-x_rot) + 90.0f32) * deg_to_rad;

        let f = yaw.cos();
        let g = yaw.sin();
        let h = pitch_neg.cos();
        let i = pitch_neg.sin();
        let j = pitch_neg_plus_90.cos();
        let k = pitch_neg_plus_90.sin();

        // vec2 = (f*h, i, g*h), vec3 = (f*j, k, g*j)
        // vec4 = cross(vec2, vec3).scale(-1.0)
        let v2x = (f * h) as f64;
        let v2y = i as f64;
        let v2z = (g * h) as f64;
        let v3x = (f * j) as f64;
        let v3y = k as f64;
        let v3z = (g * j) as f64;

        let cx = v2y * v3z - v2z * v3y;
        let cy = v2z * v3x - v2x * v3z;
        let cz = v2x * v3y - v2y * v3x;
        let v4x = cx * -1.0;
        let v4y = cy * -1.0;
        let v4z = cz * -1.0;

        let dx = v2x * self.forwards + v3x * self.up + v4x * self.left;
        let dy = v2y * self.forwards + v3y * self.up + v4y * self.left;
        let dz = v2z * self.forwards + v3z * self.up + v4z * self.left;

        BlockPos::new(
            floor_to_i32(px + dx),
            floor_to_i32(py + dy),
            floor_to_i32(pz + dz),
        )
    }
}

fn floor_to_i32(v: f64) -> i32 {
    // Java: Mth.floor / (int)Math.floor(v) semantics, saturating on overflow like Java cast.
    v.floor() as i32
}

fn parse_world_component(s: &str) -> Option<(f64, bool)> {
    if let Some(rest) = s.strip_prefix('~') {
        if rest.is_empty() {
            Some((0.0, true))
        } else {
            let value: f64 = rest.parse().ok()?;
            if !value.is_finite() {
                return None;
            }
            Some((value, true))
        }
    } else {
        let value: f64 = s.parse().ok()?;
        if !value.is_finite() {
            return None;
        }
        Some((value, false))
    }
}

fn parse_local_component(s: &str) -> Option<f64> {
    let rest = s.strip_prefix('^')?;
    if rest.is_empty() {
        Some(0.0)
    } else {
        let value: f64 = rest.parse().ok()?;
        if !value.is_finite() {
            return None;
        }
        Some(value)
    }
}

/// Port of `net.minecraft.network.chat.contents.data.BlockDataSource`.
pub struct BlockDataSource {
    pos_pattern: String,
    compiled_pos: Option<Box<dyn Coordinates>>,
}

impl BlockDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.BlockDataSource#<init>
    pub fn new(pos_pattern: String) -> Self {
        let compiled_pos = Self::compile_pos(&pos_pattern);
        Self {
            pos_pattern,
            compiled_pos,
        }
    }

    // JAVA: net.minecraft.network.chat.contents.data.BlockDataSource#<init>
    pub fn with_compiled(pos_pattern: String, compiled_pos: Option<Box<dyn Coordinates>>) -> Self {
        Self {
            pos_pattern,
            compiled_pos,
        }
    }

    // JAVA: net.minecraft.network.chat.contents.data.BlockDataSource#compilePos
    fn compile_pos(s: &str) -> Option<Box<dyn Coordinates>> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 3 {
            return None;
        }

        let any_local = parts.iter().any(|p| p.starts_with('^'));
        let all_local = parts.iter().all(|p| p.starts_with('^'));
        if any_local {
            if !all_local {
                return None;
            }
            let left = parse_local_component(parts[0])?;
            let up = parse_local_component(parts[1])?;
            let forwards = parse_local_component(parts[2])?;
            return Some(Box::new(LocalCoordinates { left, up, forwards }));
        }

        let (x, rx) = parse_world_component(parts[0])?;
        let (y, ry) = parse_world_component(parts[1])?;
        let (z, rz) = parse_world_component(parts[2])?;
        Some(Box::new(WorldCoordinates { x, rx, y, ry, z, rz }))
    }

    // JAVA: net.minecraft.network.chat.contents.data.BlockDataSource#codec
    pub fn codec(&self) -> MapCodecPlaceholder {
        MAP_CODEC
    }

    pub fn pos_pattern(&self) -> &str {
        &self.pos_pattern
    }

    pub fn compiled_pos(&self) -> Option<&dyn Coordinates> {
        self.compiled_pos.as_deref()
    }
}

impl DataSource for BlockDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.BlockDataSource#getData
    fn get_data(&self, source: &dyn CommandSourceStack) -> Vec<CompoundTag> {
        if let Some(ref compiled_pos) = self.compiled_pos {
            let level = source.get_level();
            let pos = compiled_pos.get_block_pos(source);
            if level.is_loaded(pos) {
                if let Some(block_entity) = level.get_block_entity(pos) {
                    return vec![block_entity.save_with_full_metadata(source.registry_access())];
                }
            }
        }
        Vec::new()
    }
}

impl fmt::Display for BlockDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.BlockDataSource#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "block={}", self.pos_pattern)
    }
}

impl fmt::Debug for BlockDataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockDataSource")
            .field("pos_pattern", &self.pos_pattern)
            .field("compiled_pos", &self.compiled_pos)
            .finish()
    }
}

impl PartialEq for BlockDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.BlockDataSource#equals
    fn eq(&self, other: &Self) -> bool {
        self.pos_pattern == other.pos_pattern
    }
}

impl Eq for BlockDataSource {}

impl Hash for BlockDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.BlockDataSource#hashCode
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos_pattern.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[derive(Debug)]
    struct MockRegistry;
    impl RegistryAccess for MockRegistry {}

    struct MockBlockEntity {
        tag: CompoundTag,
    }

    impl BlockEntity for MockBlockEntity {
        fn save_with_full_metadata(&self, _registry_access: &dyn RegistryAccess) -> CompoundTag {
            self.tag.clone()
        }
    }

    struct MockLevel {
        loaded: Vec<BlockPos>,
        entity: Option<(BlockPos, MockBlockEntity)>,
    }

    impl ServerLevel for MockLevel {
        fn is_loaded(&self, pos: BlockPos) -> bool {
            self.loaded.contains(&pos)
        }

        fn get_block_entity<'a>(&'a self, pos: BlockPos) -> Option<&'a dyn BlockEntity> {
            match &self.entity {
                Some((p, be)) if *p == pos => Some(be as &dyn BlockEntity),
                _ => None,
            }
        }
    }

    struct MockSource {
        level: MockLevel,
        registry: MockRegistry,
        pos: (f64, f64, f64),
        rot: (f32, f32),
    }

    impl CommandSourceStack for MockSource {
        fn get_level(&self) -> &dyn ServerLevel {
            &self.level
        }

        fn registry_access(&self) -> &dyn RegistryAccess {
            &self.registry
        }

        fn position(&self) -> (f64, f64, f64) {
            self.pos
        }

        fn rotation(&self) -> (f32, f32) {
            self.rot
        }
    }

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn compile_pos_and_to_string_behavior() {
        let ds = BlockDataSource::new("1 2 3".to_string());
        assert!(ds.compiled_pos().is_some());
        assert_eq!(ds.to_string(), "block=1 2 3");
        assert_eq!(ds.codec(), MAP_CODEC);

        let invalid = BlockDataSource::new("not a pos".to_string());
        assert!(invalid.compiled_pos().is_none());
        assert_eq!(invalid.to_string(), "block=not a pos");

        let wrong_arity = BlockDataSource::new("1 2".to_string());
        assert!(wrong_arity.compiled_pos().is_none());

        let mixed_local = BlockDataSource::new("^1 ~2 ^3".to_string());
        assert!(mixed_local.compiled_pos().is_none());
    }

    #[test]
    fn get_data_returns_tag_when_loaded_and_entity_exists() {
        let pos = BlockPos::new(10, 64, -5);
        let tag = CompoundTag::new(42);
        let source = MockSource {
            level: MockLevel {
                loaded: vec![pos],
                entity: Some((pos, MockBlockEntity { tag: tag.clone() })),
            },
            registry: MockRegistry,
            pos: (0.0, 0.0, 0.0),
            rot: (0.0, 0.0),
        };
        let ds = BlockDataSource::new("10 64 -5".to_string());
        let data = ds.get_data(&source);
        assert_eq!(data, vec![tag]);
    }

    #[test]
    fn get_data_empty_for_invalid_unloaded_or_missing_entity() {
        let pos = BlockPos::new(1, 2, 3);
        let ds = BlockDataSource::new("1 2 3".to_string());

        let unloaded = MockSource {
            level: MockLevel {
                loaded: vec![],
                entity: Some((pos, MockBlockEntity { tag: CompoundTag::new(1) })),
            },
            registry: MockRegistry,
            pos: (0.0, 0.0, 0.0),
            rot: (0.0, 0.0),
        };
        assert!(ds.get_data(&unloaded).is_empty());

        let missing_entity = MockSource {
            level: MockLevel {
                loaded: vec![pos],
                entity: None,
            },
            registry: MockRegistry,
            pos: (0.0, 0.0, 0.0),
            rot: (0.0, 0.0),
        };
        assert!(ds.get_data(&missing_entity).is_empty());

        let invalid_ds = BlockDataSource::new("bad".to_string());
        assert!(invalid_ds.get_data(&missing_entity).is_empty());
    }

    #[test]
    fn equals_and_hash_code_use_pos_pattern_only() {
        let a = BlockDataSource::new("~ ~ ~".to_string());
        let b = BlockDataSource::new("~ ~ ~".to_string());
        let c = BlockDataSource::new("1 2 3".to_string());
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(hash_of(&a), hash_of(&b));
        assert_eq!(hash_of(&a), hash_of(&a.pos_pattern().to_string()));
    }

    #[test]
    fn negative_and_relative_coordinates_use_floor_semantics() {
        let source = MockSource {
            level: MockLevel {
                loaded: vec![BlockPos::new(-2, 63, 5)],
                entity: None,
            },
            registry: MockRegistry,
            pos: (0.5, 64.7, 4.2),
            rot: (0.0, 0.0),
        };
        let ds = BlockDataSource::new("~-2 ~-1 ~1".to_string());
        let compiled = ds.compiled_pos().expect("should compile");
        let pos = compiled.get_block_pos(&source);
        assert_eq!(pos, BlockPos::new(-2, 63, 5));
    }

    #[test]
    fn floor_to_i32_handles_negative_inputs_float_bits_and_overflow() {
        assert_eq!(floor_to_i32(-1.0), -1);
        assert_eq!(floor_to_i32(-1.9), -2);

        let neg_zero = f64::from_bits(0x8000_0000_0000_0000u64);
        assert!(neg_zero.is_sign_negative());
        assert_eq!(floor_to_i32(neg_zero), 0);
        assert_eq!(neg_zero.to_bits(), 0x8000_0000_0000_0000u64);

        let big = 3_000_000_000.0f64;
        assert_eq!(floor_to_i32(big), i32::MAX);

        let big_neg = -3_000_000_000.0f64;
        assert_eq!(floor_to_i32(big_neg), i32::MIN);
    }
}
