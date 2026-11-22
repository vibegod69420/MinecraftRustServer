use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    North,
    South,
    West,
    East,
}

/// Placeholder for Mojang Codec; serialization framework not ported.
#[derive(Debug, Clone, Copy)]
pub struct Codec<T>(pub PhantomData<T>);

impl<T> Codec<T> {
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaveSurface {
    Ceiling,
    Floor,
}

/// Placeholder for StringRepresentable.fromEnum(CaveSurface::values).
/// JAVA: net.minecraft.world.level.levelgen.placement.CaveSurface#CODEC
pub const CAVE_SURFACE_CODEC: Codec<CaveSurface> = Codec::placeholder();

impl CaveSurface {
    // JAVA: net.minecraft.world.level.levelgen.placement.CaveSurface#getDirection
    pub fn get_direction(&self) -> Direction {
        match self {
            CaveSurface::Ceiling => Direction::Up,
            CaveSurface::Floor => Direction::Down,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CaveSurface#getY
    pub fn get_y(&self) -> i32 {
        match self {
            CaveSurface::Ceiling => 1,
            CaveSurface::Floor => -1,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CaveSurface#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            CaveSurface::Ceiling => "ceiling",
            CaveSurface::Floor => "floor",
        }
    }

    pub fn values() -> [CaveSurface; 2] {
        [CaveSurface::Ceiling, CaveSurface::Floor]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceiling_properties() {
        let c = CaveSurface::Ceiling;
        assert_eq!(c.get_direction(), Direction::Up);
        assert_eq!(c.get_y(), 1);
        assert_eq!(c.get_serialized_name(), "ceiling");
    }

    #[test]
    fn floor_properties() {
        let f = CaveSurface::Floor;
        assert_eq!(f.get_direction(), Direction::Down);
        assert_eq!(f.get_y(), -1);
        assert_eq!(f.get_serialized_name(), "floor");
    }

    #[test]
    fn values_contains_both() {
        let vals = CaveSurface::values();
        assert_eq!(vals.len(), 2);
        assert_eq!(vals[0], CaveSurface::Ceiling);
        assert_eq!(vals[1], CaveSurface::Floor);
    }

    #[test]
    fn codec_placeholder_exists() {
        let _c: Codec<CaveSurface> = CAVE_SURFACE_CODEC;
    }
}
