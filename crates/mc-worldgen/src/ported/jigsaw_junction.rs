use std::fmt;

/// Minimal local representation of `net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool.Projection`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Projection {
    TerrainMatching,
    Rigid,
}

impl Projection {
    /// Returns the serialized name used by Minecraft.
    pub fn get_name(&self) -> &'static str {
        match self {
            Projection::TerrainMatching => "terrain_matching",
            Projection::Rigid => "rigid",
        }
    }

    /// Parses a projection by name, defaulting to `TerrainMatching` for unknown values.
    pub fn by_name(name: &str) -> Self {
        if name == "rigid" {
            Projection::Rigid
        } else {
            Projection::TerrainMatching
        }
    }

    /// Deterministic hash contribution for this projection.
    pub fn hash_code(&self) -> i32 {
        match self {
            Projection::TerrainMatching => 1,
            Projection::Rigid => 2,
        }
    }
}

impl fmt::Display for Projection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Projection::TerrainMatching => write!(f, "TERRAIN_MATCHING"),
            Projection::Rigid => write!(f, "RIGID"),
        }
    }
}

/// Minimal dynamic-ops abstraction for self-contained serialization/deserialization,
/// modeling the operations used by Mojang's `DynamicOps<T>`.
pub trait DynamicOps<T: Clone> {
    fn create_string(&self, value: &str) -> T;
    fn create_int(&self, value: i32) -> T;
    fn create_map(&self, entries: Vec<(T, T)>) -> T;
    fn get(&self, value: &T, key: &str) -> Option<T>;
    fn empty(&self) -> T;
    fn as_int(&self, value: &T, default: i32) -> i32;
    fn as_string(&self, value: &T, default: &str) -> String;
}

/// Minimal dynamic value wrapper, modeling Mojang's `Dynamic<T>`.
#[derive(Clone, Debug)]
pub struct Dynamic<T: Clone, O: DynamicOps<T> + Clone> {
    pub ops: O,
    pub value: T,
}

impl<T: Clone, O: DynamicOps<T> + Clone> Dynamic<T, O> {
    pub fn get(&self, key: &str) -> Self {
        let value = self
            .ops
            .get(&self.value, key)
            .unwrap_or_else(|| self.ops.empty());
        Dynamic {
            ops: self.ops.clone(),
            value,
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        self.ops.as_int(&self.value, default)
    }

    pub fn as_string(&self, default: &str) -> String {
        self.ops.as_string(&self.value, default)
    }
}

/// Port of `net.minecraft.world.level.levelgen.structure.pools.JigsawJunction`.
#[derive(Clone, Debug)]
pub struct JigsawJunction {
    source_x: i32,
    source_ground_y: i32,
    source_z: i32,
    delta_y: i32,
    dest_projection: Projection,
}

impl JigsawJunction {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#<init>
    pub fn new(
        source_x: i32,
        source_ground_y: i32,
        source_z: i32,
        delta_y: i32,
        dest_projection: Projection,
    ) -> Self {
        Self {
            source_x,
            source_ground_y,
            source_z,
            delta_y,
            dest_projection,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#getSourceX
    pub fn get_source_x(&self) -> i32 {
        self.source_x
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#getSourceGroundY
    pub fn get_source_ground_y(&self) -> i32 {
        self.source_ground_y
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#getSourceZ
    pub fn get_source_z(&self) -> i32 {
        self.source_z
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#getDeltaY
    pub fn get_delta_y(&self) -> i32 {
        self.delta_y
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#getDestProjection
    pub fn get_dest_projection(&self) -> Projection {
        self.dest_projection
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#serialize
    pub fn serialize<T: Clone, O: DynamicOps<T> + Clone>(&self, ops: &O) -> Dynamic<T, O> {
        let entries = vec![
            (ops.create_string("source_x"), ops.create_int(self.source_x)),
            (
                ops.create_string("source_ground_y"),
                ops.create_int(self.source_ground_y),
            ),
            (ops.create_string("source_z"), ops.create_int(self.source_z)),
            (ops.create_string("delta_y"), ops.create_int(self.delta_y)),
            (
                ops.create_string("dest_proj"),
                ops.create_string(self.dest_projection.get_name()),
            ),
        ];
        Dynamic {
            ops: ops.clone(),
            value: ops.create_map(entries),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#deserialize
    pub fn deserialize<T: Clone, O: DynamicOps<T> + Clone>(dynamic: &Dynamic<T, O>) -> Self {
        Self::new(
            dynamic.get("source_x").as_int(0),
            dynamic.get("source_ground_y").as_int(0),
            dynamic.get("source_z").as_int(0),
            dynamic.get("delta_y").as_int(0),
            Projection::by_name(&dynamic.get("dest_proj").as_string("")),
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#equals
    pub fn equals(&self, other: &Self) -> bool {
        if self.source_x != other.source_x {
            return false;
        }
        if self.source_z != other.source_z {
            return false;
        }
        if self.delta_y != other.delta_y {
            return false;
        }
        self.dest_projection == other.dest_projection
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#hashCode
    pub fn hash_code(&self) -> i32 {
        let mut h: i32 = self.source_x;
        h = h.wrapping_mul(31).wrapping_add(self.source_ground_y);
        h = h.wrapping_mul(31).wrapping_add(self.source_z);
        h = h.wrapping_mul(31).wrapping_add(self.delta_y);
        h.wrapping_mul(31)
            .wrapping_add(self.dest_projection.hash_code())
    }
}

impl PartialEq for JigsawJunction {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Eq for JigsawJunction {}

impl std::hash::Hash for JigsawJunction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash_code().hash(state);
    }
}

impl fmt::Display for JigsawJunction {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.JigsawJunction#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JigsawJunction{{sourceX={}, sourceGroundY={}, sourceZ={}, deltaY={}, destProjection={}}}",
            self.source_x, self.source_ground_y, self.source_z, self.delta_y, self.dest_projection
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug)]
    enum TestVal {
        Int(i32),
        Str(String),
        Map(HashMap<String, TestVal>),
        Empty,
    }

    #[derive(Clone, Debug, Default)]
    struct TestOps;

    impl DynamicOps<TestVal> for TestOps {
        fn create_string(&self, value: &str) -> TestVal {
            TestVal::Str(value.to_string())
        }

        fn create_int(&self, value: i32) -> TestVal {
            TestVal::Int(value)
        }

        fn create_map(&self, entries: Vec<(TestVal, TestVal)>) -> TestVal {
            let mut map = HashMap::new();
            for (k, v) in entries {
                if let TestVal::Str(s) = k {
                    map.insert(s, v);
                }
            }
            TestVal::Map(map)
        }

        fn get(&self, value: &TestVal, key: &str) -> Option<TestVal> {
            match value {
                TestVal::Map(map) => map.get(key).cloned(),
                _ => None,
            }
        }

        fn empty(&self) -> TestVal {
            TestVal::Empty
        }

        fn as_int(&self, value: &TestVal, default: i32) -> i32 {
            match value {
                TestVal::Int(v) => *v,
                _ => default,
            }
        }

        fn as_string(&self, value: &TestVal, default: &str) -> String {
            match value {
                TestVal::Str(v) => v.clone(),
                _ => default.to_string(),
            }
        }
    }

    #[test]
    fn constructor_getters_and_to_string() {
        let j = JigsawJunction::new(1, 2, 3, 4, Projection::Rigid);
        assert_eq!(j.get_source_x(), 1);
        assert_eq!(j.get_source_ground_y(), 2);
        assert_eq!(j.get_source_z(), 3);
        assert_eq!(j.get_delta_y(), 4);
        assert_eq!(j.get_dest_projection(), Projection::Rigid);
        assert_eq!(
            j.to_string(),
            "JigsawJunction{sourceX=1, sourceGroundY=2, sourceZ=3, deltaY=4, destProjection=RIGID}"
        );
    }

    #[test]
    fn equals_ignores_source_ground_y_like_java() {
        let a = JigsawJunction::new(10, -5, 20, 3, Projection::TerrainMatching);
        let b = JigsawJunction::new(10, 999, 20, 3, Projection::TerrainMatching);
        let c = JigsawJunction::new(10, -5, 20, 3, Projection::Rigid);
        assert!(a.equals(&b));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn hash_code_uses_wrapping_and_negative_inputs() {
        let j = JigsawJunction::new(i32::MAX, -1, i32::MIN, -100, Projection::TerrainMatching);
        let actual = j.hash_code();

        let mut expected: i32 = i32::MAX;
        expected = expected.wrapping_mul(31).wrapping_add(-1);
        expected = expected.wrapping_mul(31).wrapping_add(i32::MIN);
        expected = expected.wrapping_mul(31).wrapping_add(-100);
        expected = expected
            .wrapping_mul(31)
            .wrapping_add(Projection::TerrainMatching.hash_code());

        assert_eq!(actual, expected);
        assert_eq!(actual, j.hash_code());
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let ops = TestOps;
        let original = JigsawJunction::new(-7, 64, 128, -3, Projection::Rigid);
        let dynamic = original.serialize(&ops);
        let restored = JigsawJunction::deserialize(&dynamic);

        assert_eq!(restored.get_source_x(), original.get_source_x());
        assert_eq!(restored.get_source_ground_y(), original.get_source_ground_y());
        assert_eq!(restored.get_source_z(), original.get_source_z());
        assert_eq!(restored.get_delta_y(), original.get_delta_y());
        assert_eq!(restored.get_dest_projection(), original.get_dest_projection());
    }

    #[test]
    fn deserialize_missing_fields_uses_defaults() {
        let ops = TestOps;
        let empty = Dynamic {
            ops: ops.clone(),
            value: TestVal::Map(HashMap::new()),
        };
        let j = JigsawJunction::deserialize(&empty);

        assert_eq!(j.get_source_x(), 0);
        assert_eq!(j.get_source_ground_y(), 0);
        assert_eq!(j.get_source_z(), 0);
        assert_eq!(j.get_delta_y(), 0);
        assert_eq!(j.get_dest_projection(), Projection::TerrainMatching);

        let unknown_proj = JigsawJunction::new(0, 0, 0, 0, Projection::TerrainMatching);
        let mut map = HashMap::new();
        map.insert("dest_proj".to_string(), TestVal::Str("unknown".to_string()));
        let dyn_unknown = Dynamic {
            ops,
            value: TestVal::Map(map),
        };
        assert_eq!(JigsawJunction::deserialize(&dyn_unknown), unknown_proj);
    }
}
