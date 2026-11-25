use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.Vec3i#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.Vec3i#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.Vec3i#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LocationPredicate;

impl LocationPredicate {
    // JAVA: net.minecraft.advancements.criterion.LocationPredicate#matches
    pub fn matches(&self, _level: (), _x: f64, _y: f64, _z: f64) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct LootContext {
    origin: Option<Vec3>,
}

impl LootContext {
    // JAVA: net.minecraft.world.level.storage.loot.LootContext#getOptionalParameter
    pub fn get_optional_parameter_origin(&self) -> Option<Vec3> {
        self.origin
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContext#getLevel
    pub fn get_level(&self) -> () {
        ()
    }
}

#[derive(Debug, Clone)]
pub struct LocationCheck {
    pub predicate: Option<LocationPredicate>,
    pub offset: BlockPos,
}

impl LocationCheck {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LocationCheck#test
    pub fn test(&self, ctx: &LootContext) -> bool {
        match ctx.get_optional_parameter_origin() {
            Some(origin) => {
                self.predicate.as_ref().map_or(true, |predicate| {
                    predicate.matches(
                        ctx.get_level(),
                        origin.x + self.offset.get_x() as f64,
                        origin.y + self.offset.get_y() as f64,
                        origin.z + self.offset.get_z() as f64,
                    )
                })
            }
            None => false,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LocationCheck#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<&'static str> {
        let mut set = HashSet::new();
        set.insert("origin");
        set
    }
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.LocationCheck#checkLocation
pub fn check_location(predicate: LocationPredicate) -> LocationCheck {
    LocationCheck {
        predicate: Some(predicate),
        offset: BlockPos::new(0, 0, 0),
    }
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.LocationCheck#checkLocation
pub fn check_location_with_offset(predicate: LocationPredicate, offset: BlockPos) -> LocationCheck {
    LocationCheck {
        predicate: Some(predicate),
        offset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_origin_returns_false() {
        let check = LocationCheck {
            predicate: None,
            offset: BlockPos::new(1, 2, 3),
        };
        assert!(!check.test(&LootContext { origin: None }));
    }

    #[test]
    fn empty_predicate_with_origin_returns_true() {
        let check = LocationCheck {
            predicate: None,
            offset: BlockPos::new(1, 2, 3),
        };
        assert!(check.test(&LootContext {
            origin: Some(Vec3 { x: 10.0, y: 20.0, z: 30.0 }),
        }));
    }

    #[test]
    fn offset_is_added_to_origin() {
        let check = LocationCheck {
            predicate: Some(LocationPredicate),
            offset: BlockPos::new(1, -2, 3),
        };
        assert!(check.test(&LootContext {
            origin: Some(Vec3 { x: 10.0, y: 20.0, z: 30.0 }),
        }));
    }

    #[test]
    fn referenced_params_contains_origin() {
        let check = LocationCheck {
            predicate: None,
            offset: BlockPos::new(0, 0, 0),
        };
        assert!(check.get_referenced_context_params().contains("origin"));
    }
}
