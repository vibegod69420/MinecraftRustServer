use std::fmt;

// ---------------------------------------------------------------------------
// Minimal supporting types (self-contained replacements for Minecraft classes)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn mutable(self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn move_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.y = self.y.wrapping_add(1),
            Direction::Down => self.y = self.y.wrapping_sub(1),
        }
    }

    pub fn as_block_pos(self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
}

pub trait LevelSimulatedReader {
    type State;
    fn is_state_at_position(&self, pos: BlockPos, predicate: &dyn Fn(&Self::State) -> bool) -> bool;
}

// ---------------------------------------------------------------------------
// Column port: net.minecraft.world.level.levelgen.Column
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Column {
    Line(Line),
    Range(Range),
    Ray(Ray),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    floor: i32,
    ceiling: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ray {
    edge: i32,
    pointing_up: bool,
}

impl Column {
    // JAVA: net.minecraft.world.level.levelgen.Column#around
    pub fn around(floor: i32, ceiling: i32) -> Self {
        Self::Range(Range::new(floor.wrapping_sub(1), ceiling.wrapping_add(1)))
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#inside
    pub fn inside(floor: i32, ceiling: i32) -> Self {
        Self::Range(Range::new(floor, ceiling))
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#below
    pub fn below(ceiling: i32) -> Self {
        Self::Ray(Ray::new(ceiling, false))
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#fromHighest
    pub fn from_highest(floor: i32) -> Self {
        Self::Ray(Ray::new(floor.wrapping_add(1), false))
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#above
    pub fn above(floor: i32) -> Self {
        Self::Ray(Ray::new(floor, true))
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#fromLowest
    pub fn from_lowest(ceiling: i32) -> Self {
        Self::Ray(Ray::new(ceiling.wrapping_sub(1), true))
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#line
    pub fn line() -> Self {
        Self::Line(Line)
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#create
    pub fn create(floor: Option<i32>, ceiling: Option<i32>) -> Self {
        match (floor, ceiling) {
            (Some(f), Some(c)) => Self::inside(f, c),
            (Some(f), None) => Self::above(f),
            (None, Some(c)) => Self::below(c),
            (None, None) => Self::line(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#getCeiling
    pub fn get_ceiling(&self) -> Option<i32> {
        match self {
            Self::Line(l) => l.get_ceiling(),
            Self::Range(r) => r.get_ceiling(),
            Self::Ray(r) => r.get_ceiling(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#getFloor
    pub fn get_floor(&self) -> Option<i32> {
        match self {
            Self::Line(l) => l.get_floor(),
            Self::Range(r) => r.get_floor(),
            Self::Ray(r) => r.get_floor(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#getHeight
    pub fn get_height(&self) -> Option<i32> {
        match self {
            Self::Line(l) => l.get_height(),
            Self::Range(r) => r.get_height(),
            Self::Ray(r) => r.get_height(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#withFloor
    pub fn with_floor(&self, floor: Option<i32>) -> Self {
        Self::create(floor, self.get_ceiling())
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#withCeiling
    pub fn with_ceiling(&self, ceiling: Option<i32>) -> Self {
        Self::create(self.get_floor(), ceiling)
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#scan
    pub fn scan<R: LevelSimulatedReader + ?Sized>(
        level: &R,
        pos: BlockPos,
        max_distance: i32,
        column_predicate: &dyn Fn(&R::State) -> bool,
        tip_predicate: &dyn Fn(&R::State) -> bool,
    ) -> Option<Self> {
        let mut mutable = pos.mutable();
        if !level.is_state_at_position(pos, column_predicate) {
            return None;
        }

        let y = pos.get_y();
        let ceiling = Self::scan_direction(
            level,
            max_distance,
            column_predicate,
            tip_predicate,
            &mut mutable,
            y,
            Direction::Up,
        );
        let floor = Self::scan_direction(
            level,
            max_distance,
            column_predicate,
            tip_predicate,
            &mut mutable,
            y,
            Direction::Down,
        );
        Some(Self::create(floor, ceiling))
    }

    // JAVA: net.minecraft.world.level.levelgen.Column#scanDirection
    fn scan_direction<R: LevelSimulatedReader + ?Sized>(
        level: &R,
        max_distance: i32,
        column_predicate: &dyn Fn(&R::State) -> bool,
        tip_predicate: &dyn Fn(&R::State) -> bool,
        pos: &mut MutableBlockPos,
        start_y: i32,
        direction: Direction,
    ) -> Option<i32> {
        pos.set_y(start_y);

        let mut i: i32 = 1;
        while i < max_distance && level.is_state_at_position(pos.as_block_pos(), column_predicate) {
            pos.move_direction(direction);
            i = i.wrapping_add(1);
        }

        if level.is_state_at_position(pos.as_block_pos(), tip_predicate) {
            Some(pos.get_y())
        } else {
            None
        }
    }
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Line(l) => fmt::Display::fmt(l, f),
            Self::Range(r) => fmt::Display::fmt(r, f),
            Self::Ray(r) => fmt::Display::fmt(r, f),
        }
    }
}

impl Line {
    // JAVA: net.minecraft.world.level.levelgen.Column$Line#getCeiling
    pub fn get_ceiling(&self) -> Option<i32> {
        None
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Line#getFloor
    pub fn get_floor(&self) -> Option<i32> {
        None
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Line#getHeight
    pub fn get_height(&self) -> Option<i32> {
        None
    }
}

impl fmt::Display for Line {
    // JAVA: net.minecraft.world.level.levelgen.Column$Line#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C(-)")
    }
}

impl Range {
    // JAVA: net.minecraft.world.level.levelgen.Column$Range#<init>
    pub fn new(floor: i32, ceiling: i32) -> Self {
        let this = Self { floor, ceiling };
        if this.height() < 0 {
            panic!("Column of negative height: {}", this);
        }
        this
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Range#getCeiling
    pub fn get_ceiling(&self) -> Option<i32> {
        Some(self.ceiling)
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Range#getFloor
    pub fn get_floor(&self) -> Option<i32> {
        Some(self.floor)
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Range#getHeight
    pub fn get_height(&self) -> Option<i32> {
        Some(self.height())
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Range#ceiling
    pub fn ceiling(&self) -> i32 {
        self.ceiling
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Range#floor
    pub fn floor(&self) -> i32 {
        self.floor
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Range#height
    pub fn height(&self) -> i32 {
        self.ceiling.wrapping_sub(self.floor).wrapping_sub(1)
    }
}

impl fmt::Display for Range {
    // JAVA: net.minecraft.world.level.levelgen.Column$Range#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C({}-{})", self.ceiling, self.floor)
    }
}

impl Ray {
    // JAVA: net.minecraft.world.level.levelgen.Column$Ray#<init>
    pub fn new(edge: i32, pointing_up: bool) -> Self {
        Self { edge, pointing_up }
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Ray#getCeiling
    pub fn get_ceiling(&self) -> Option<i32> {
        if self.pointing_up {
            None
        } else {
            Some(self.edge)
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Ray#getFloor
    pub fn get_floor(&self) -> Option<i32> {
        if self.pointing_up {
            Some(self.edge)
        } else {
            None
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.Column$Ray#getHeight
    pub fn get_height(&self) -> Option<i32> {
        None
    }
}

impl fmt::Display for Ray {
    // JAVA: net.minecraft.world.level.levelgen.Column$Ray#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pointing_up {
            write!(f, "C({}-)", self.edge)
        } else {
            write!(f, "C(-{})", self.edge)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    struct Block {
        solid: bool,
    }

    struct MockLevel {
        solid_at: fn(i32) -> bool,
    }

    impl LevelSimulatedReader for MockLevel {
        type State = Block;
        fn is_state_at_position(
            &self,
            pos: BlockPos,
            predicate: &dyn Fn(&Self::State) -> bool,
        ) -> bool {
            predicate(&Block {
                solid: (self.solid_at)(pos.get_y()),
            })
        }
    }

    #[test]
    fn factories_create_and_formatting() {
        let c = Column::around(5, 10);
        assert_eq!(c.get_floor(), Some(4));
        assert_eq!(c.get_ceiling(), Some(11));
        assert_eq!(c.get_height(), Some(6));
        assert_eq!(c.to_string(), "C(11-4)");

        assert_eq!(Column::inside(1, 5).to_string(), "C(5-1)");
        assert_eq!(Column::below(10).to_string(), "C(-10)");
        assert_eq!(Column::above(10).to_string(), "C(10-)");
        assert_eq!(Column::from_highest(10).to_string(), "C(-11)");
        assert_eq!(Column::from_lowest(10).to_string(), "C(9-)");
        assert_eq!(Column::line().to_string(), "C(-)");

        assert_eq!(Column::create(Some(2), Some(8)), Column::inside(2, 8));
        assert_eq!(Column::create(Some(2), None), Column::above(2));
        assert_eq!(Column::create(None, Some(8)), Column::below(8));
        assert_eq!(Column::create(None, None), Column::line());
    }

    #[test]
    fn with_floor_and_ceiling() {
        let base = Column::inside(1, 5);
        assert_eq!(base.with_floor(Some(2)), Column::inside(2, 5));
        assert_eq!(base.with_ceiling(Some(10)), Column::inside(1, 10));
        assert_eq!(base.with_floor(None), Column::below(5));
        assert_eq!(base.with_ceiling(None), Column::above(1));
        assert_eq!(Column::line().with_floor(Some(0)), Column::above(0));
    }

    #[test]
    fn wrapping_integer_semantics() {
        let c = Column::around(i32::MIN, i32::MAX);
        assert_eq!(c.get_floor(), Some(i32::MAX));
        assert_eq!(c.get_ceiling(), Some(i32::MIN));
        assert_eq!(c.get_height(), Some(0));

        let c = Column::from_highest(i32::MAX);
        assert_eq!(c.get_ceiling(), Some(i32::MIN));
        assert_eq!(c.get_floor(), None);

        let c = Column::from_lowest(i32::MIN);
        assert_eq!(c.get_floor(), Some(i32::MAX));
        assert_eq!(c.get_ceiling(), None);

        let r = Range::new(i32::MAX, i32::MIN);
        assert_eq!(r.height(), 0);
    }

    #[test]
    #[should_panic(expected = "Column of negative height")]
    fn range_negative_height_panics() {
        let _ = Column::inside(5, 5);
    }

    #[test]
    fn scan_finds_bounds_and_handles_misses() {
        let level = MockLevel {
            solid_at: |y| (2..=5).contains(&y),
        };
        let is_solid = |b: &Block| b.solid;
        let is_air = |b: &Block| !b.solid;

        let col = Column::scan(&level, BlockPos::new(0, 3, 0), 10, &is_solid, &is_air)
            .expect("should find column");
        assert_eq!(col.get_floor(), Some(1));
        assert_eq!(col.get_ceiling(), Some(6));

        let none = Column::scan(&level, BlockPos::new(0, 10, 0), 10, &is_solid, &is_air);
        assert_eq!(none, None);

        let limited = Column::scan(&level, BlockPos::new(0, 3, 0), 2, &is_solid, &is_air)
            .expect("should find limited column");
        assert_eq!(limited.get_floor(), None);
        assert_eq!(limited.get_ceiling(), None);
    }
}
