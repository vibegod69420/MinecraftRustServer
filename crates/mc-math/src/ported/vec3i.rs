use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Minimal position trait for self-contained distance queries.
/// Mirrors `net.minecraft.core.Position`.
pub trait Position {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn z(&self) -> f64;
}

/// Minimal axis enum for self-contained `Vec3i` operations.
/// Mirrors `net.minecraft.core.Direction.Axis`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    // JAVA: net.minecraft.core.Direction$Axis#choose
    pub fn choose<T>(self, x: T, y: T, z: T) -> T {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }
}

/// Minimal direction enum for self-contained `Vec3i` operations.
/// Mirrors `net.minecraft.core.Direction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    // JAVA: net.minecraft.core.Direction#getStepX
    pub fn step_x(self) -> i32 {
        match self {
            Direction::West => -1,
            Direction::East => 1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction#getStepY
    pub fn step_y(self) -> i32 {
        match self {
            Direction::Down => -1,
            Direction::Up => 1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction#getStepZ
    pub fn step_z(self) -> i32 {
        match self {
            Direction::North => -1,
            Direction::South => 1,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vec3i {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3i {
    pub const ZERO: Vec3i = Vec3i { x: 0, y: 0, z: 0 };

    // JAVA: net.minecraft.core.Vec3i#<init>
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

    // JAVA: net.minecraft.core.Vec3i#setX
    pub fn set_x(&mut self, x: i32) -> &mut Self {
        self.x = x;
        self
    }

    // JAVA: net.minecraft.core.Vec3i#setY
    pub fn set_y(&mut self, y: i32) -> &mut Self {
        self.y = y;
        self
    }

    // JAVA: net.minecraft.core.Vec3i#setZ
    pub fn set_z(&mut self, z: i32) -> &mut Self {
        self.z = z;
        self
    }

    // JAVA: net.minecraft.core.Vec3i#hashCode
    pub fn hash_code(&self) -> i32 {
        self.get_y()
            .wrapping_add(self.get_z().wrapping_mul(31))
            .wrapping_mul(31)
            .wrapping_add(self.get_x())
    }

    // JAVA: net.minecraft.core.Vec3i#compareTo
    pub fn compare_to(&self, other: &Vec3i) -> i32 {
        if self.get_y() == other.get_y() {
            if self.get_z() == other.get_z() {
                self.get_x().wrapping_sub(other.get_x())
            } else {
                self.get_z().wrapping_sub(other.get_z())
            }
        } else {
            self.get_y().wrapping_sub(other.get_y())
        }
    }

    // JAVA: net.minecraft.core.Vec3i#offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Vec3i {
        if dx == 0 && dy == 0 && dz == 0 {
            *self
        } else {
            Vec3i::new(
                self.get_x().wrapping_add(dx),
                self.get_y().wrapping_add(dy),
                self.get_z().wrapping_add(dz),
            )
        }
    }

    // JAVA: net.minecraft.core.Vec3i#offset
    pub fn offset_vec(&self, other: &Vec3i) -> Vec3i {
        self.offset(other.get_x(), other.get_y(), other.get_z())
    }

    // JAVA: net.minecraft.core.Vec3i#subtract
    pub fn subtract(&self, other: &Vec3i) -> Vec3i {
        self.offset(
            other.get_x().wrapping_neg(),
            other.get_y().wrapping_neg(),
            other.get_z().wrapping_neg(),
        )
    }

    // JAVA: net.minecraft.core.Vec3i#multiply
    pub fn multiply_scalar(&self, scalar: i32) -> Vec3i {
        if scalar == 1 {
            *self
        } else if scalar == 0 {
            Self::ZERO
        } else {
            Vec3i::new(
                self.get_x().wrapping_mul(scalar),
                self.get_y().wrapping_mul(scalar),
                self.get_z().wrapping_mul(scalar),
            )
        }
    }

    // JAVA: net.minecraft.core.Vec3i#multiply
    pub fn multiply(&self, sx: i32, sy: i32, sz: i32) -> Vec3i {
        Vec3i::new(
            self.get_x().wrapping_mul(sx),
            self.get_y().wrapping_mul(sy),
            self.get_z().wrapping_mul(sz),
        )
    }

    // JAVA: net.minecraft.core.Vec3i#above
    pub fn above(&self) -> Vec3i {
        self.above_n(1)
    }

    // JAVA: net.minecraft.core.Vec3i#above
    pub fn above_n(&self, n: i32) -> Vec3i {
        self.relative(Direction::Up, n)
    }

    // JAVA: net.minecraft.core.Vec3i#below
    pub fn below(&self) -> Vec3i {
        self.below_n(1)
    }

    // JAVA: net.minecraft.core.Vec3i#below
    pub fn below_n(&self, n: i32) -> Vec3i {
        self.relative(Direction::Down, n)
    }

    // JAVA: net.minecraft.core.Vec3i#north
    pub fn north(&self) -> Vec3i {
        self.north_n(1)
    }

    // JAVA: net.minecraft.core.Vec3i#north
    pub fn north_n(&self, n: i32) -> Vec3i {
        self.relative(Direction::North, n)
    }

    // JAVA: net.minecraft.core.Vec3i#south
    pub fn south(&self) -> Vec3i {
        self.south_n(1)
    }

    // JAVA: net.minecraft.core.Vec3i#south
    pub fn south_n(&self, n: i32) -> Vec3i {
        self.relative(Direction::South, n)
    }

    // JAVA: net.minecraft.core.Vec3i#west
    pub fn west(&self) -> Vec3i {
        self.west_n(1)
    }

    // JAVA: net.minecraft.core.Vec3i#west
    pub fn west_n(&self, n: i32) -> Vec3i {
        self.relative(Direction::West, n)
    }

    // JAVA: net.minecraft.core.Vec3i#east
    pub fn east(&self) -> Vec3i {
        self.east_n(1)
    }

    // JAVA: net.minecraft.core.Vec3i#east
    pub fn east_n(&self, n: i32) -> Vec3i {
        self.relative(Direction::East, n)
    }

    // JAVA: net.minecraft.core.Vec3i#relative
    pub fn relative(&self, dir: Direction, n: i32) -> Vec3i {
        if n == 0 {
            *self
        } else {
            Vec3i::new(
                self.get_x()
                    .wrapping_add(dir.step_x().wrapping_mul(n)),
                self.get_y()
                    .wrapping_add(dir.step_y().wrapping_mul(n)),
                self.get_z()
                    .wrapping_add(dir.step_z().wrapping_mul(n)),
            )
        }
    }

    // JAVA: net.minecraft.core.Vec3i#relative
    pub fn relative_axis(&self, axis: Axis, n: i32) -> Vec3i {
        if n == 0 {
            *self
        } else {
            let dx = if axis == Axis::X { n } else { 0 };
            let dy = if axis == Axis::Y { n } else { 0 };
            let dz = if axis == Axis::Z { n } else { 0 };
            Vec3i::new(
                self.get_x().wrapping_add(dx),
                self.get_y().wrapping_add(dy),
                self.get_z().wrapping_add(dz),
            )
        }
    }

    // JAVA: net.minecraft.core.Vec3i#cross
    pub fn cross(&self, other: &Vec3i) -> Vec3i {
        Vec3i::new(
            self.get_y()
                .wrapping_mul(other.get_z())
                .wrapping_sub(self.get_z().wrapping_mul(other.get_y())),
            self.get_z()
                .wrapping_mul(other.get_x())
                .wrapping_sub(self.get_x().wrapping_mul(other.get_z())),
            self.get_x()
                .wrapping_mul(other.get_y())
                .wrapping_sub(self.get_y().wrapping_mul(other.get_x())),
        )
    }

    // JAVA: net.minecraft.core.Vec3i#closerThan
    pub fn closer_than(&self, other: &Vec3i, distance: f64) -> bool {
        self.dist_sqr(other) < distance * distance
    }

    // JAVA: net.minecraft.core.Vec3i#closerToCenterThan
    pub fn closer_to_center_than<P: Position>(&self, pos: &P, distance: f64) -> bool {
        self.dist_to_center_sqr_pos(pos) < distance * distance
    }

    // JAVA: net.minecraft.core.Vec3i#distSqr
    pub fn dist_sqr(&self, other: &Vec3i) -> f64 {
        self.dist_to_low_corner_sqr(
            other.get_x() as f64,
            other.get_y() as f64,
            other.get_z() as f64,
        )
    }

    // JAVA: net.minecraft.core.Vec3i#distToCenterSqr
    pub fn dist_to_center_sqr_pos<P: Position>(&self, pos: &P) -> f64 {
        self.dist_to_center_sqr(pos.x(), pos.y(), pos.z())
    }

    // JAVA: net.minecraft.core.Vec3i#distToCenterSqr
    pub fn dist_to_center_sqr(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.get_x() as f64 + 0.5 - x;
        let dy = self.get_y() as f64 + 0.5 - y;
        let dz = self.get_z() as f64 + 0.5 - z;
        dx * dx + dy * dy + dz * dz
    }

    // JAVA: net.minecraft.core.Vec3i#distToLowCornerSqr
    pub fn dist_to_low_corner_sqr(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.get_x() as f64 - x;
        let dy = self.get_y() as f64 - y;
        let dz = self.get_z() as f64 - z;
        dx * dx + dy * dy + dz * dz
    }

    // JAVA: net.minecraft.core.Vec3i#distManhattan
    pub fn dist_manhattan(&self, other: &Vec3i) -> i32 {
        let dx = (other.get_x().wrapping_sub(self.get_x())).wrapping_abs() as f32;
        let dy = (other.get_y().wrapping_sub(self.get_y())).wrapping_abs() as f32;
        let dz = (other.get_z().wrapping_sub(self.get_z())).wrapping_abs() as f32;
        (dx + dy + dz) as i32
    }

    // JAVA: net.minecraft.core.Vec3i#distChessboard
    pub fn dist_chessboard(&self, other: &Vec3i) -> i32 {
        let dx = (self.get_x().wrapping_sub(other.get_x())).wrapping_abs();
        let dy = (self.get_y().wrapping_sub(other.get_y())).wrapping_abs();
        let dz = (self.get_z().wrapping_sub(other.get_z())).wrapping_abs();
        dx.max(dy).max(dz)
    }

    // JAVA: net.minecraft.core.Vec3i#get
    pub fn get(&self, axis: Axis) -> i32 {
        axis.choose(self.x, self.y, self.z)
    }

    // JAVA: net.minecraft.core.Vec3i#toShortString
    pub fn to_short_string(&self) -> String {
        format!("{}, {}, {}", self.get_x(), self.get_y(), self.get_z())
    }
}

impl PartialEq for Vec3i {
    // JAVA: net.minecraft.core.Vec3i#equals
    fn eq(&self, other: &Self) -> bool {
        self.get_x() == other.get_x()
            && self.get_y() == other.get_y()
            && self.get_z() == other.get_z()
    }
}

impl Eq for Vec3i {}

impl Hash for Vec3i {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_code().hash(state);
    }
}

impl PartialOrd for Vec3i {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Vec3i {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare_to(other).cmp(&0)
    }
}

impl fmt::Display for Vec3i {
    // JAVA: net.minecraft.core.Vec3i#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vec3i{{x={}, y={}, z={}}}",
            self.get_x(),
            self.get_y(),
            self.get_z()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimplePos(f64, f64, f64);
    impl Position for SimplePos {
        fn x(&self) -> f64 {
            self.0
        }
        fn y(&self) -> f64 {
            self.1
        }
        fn z(&self) -> f64 {
            self.2
        }
    }

    #[test]
    fn basic_getters_setters_hash_compare() {
        let mut v = Vec3i::new(1, 2, 3);
        assert_eq!(v.get_x(), 1);
        assert_eq!(v.get_y(), 2);
        assert_eq!(v.get_z(), 3);
        v.set_x(10).set_y(20).set_z(30);
        assert_eq!(v, Vec3i::new(10, 20, 30));
        assert_eq!(Vec3i::new(1, 2, 3).hash_code(), (2 + 3 * 31) * 31 + 1);
        assert!(Vec3i::new(0, 0, 1).compare_to(&Vec3i::new(0, 0, 0)) > 0);
        assert!(Vec3i::new(0, 1, 0).compare_to(&Vec3i::new(0, 0, 5)) > 0);
        assert_eq!(Vec3i::new(1, 2, 3).to_short_string(), "1, 2, 3");
        assert_eq!(Vec3i::new(1, 2, 3).to_string(), "Vec3i{x=1, y=2, z=3}");
    }

    #[test]
    fn offset_subtract_multiply_overflow() {
        let v = Vec3i::new(i32::MAX, i32::MIN, -1);
        let o = v.offset(1, -1, 2);
        assert_eq!(o, Vec3i::new(i32::MIN, i32::MAX, 1));
        assert_eq!(v.offset(0, 0, 0), v);
        let s = Vec3i::new(1, 2, 3).subtract(&Vec3i::new(2, -3, i32::MIN));
        assert_eq!(s, Vec3i::new(-1, 5, 3i32.wrapping_add(i32::MIN)));
        assert_eq!(Vec3i::new(2, 3, 4).multiply_scalar(0), Vec3i::ZERO);
        assert_eq!(Vec3i::new(2, 3, 4).multiply_scalar(1), Vec3i::new(2, 3, 4));
        assert_eq!(
            Vec3i::new(2, 3, 4).multiply_scalar(2),
            Vec3i::new(4, 6, 8)
        );
        assert_eq!(
            Vec3i::new(i32::MAX, 2, -3).multiply(2, 3, 4),
            Vec3i::new(i32::MAX.wrapping_mul(2), 6, -12)
        );
    }

    #[test]
    fn manhattan_chessboard_negative_and_min() {
        let a = Vec3i::new(0, 0, 0);
        let b = Vec3i::new(i32::MIN, 0, 0);
        // Java: Math.abs(i32::MIN) == i32::MIN, float sum -2147483648.0f, (int) -> i32::MIN
        assert_eq!(a.dist_manhattan(&b), i32::MIN);
        assert_eq!(b.dist_manhattan(&a), i32::MIN);
        let c = Vec3i::new(-1, -2, -3);
        let d = Vec3i::new(1, 2, 3);
        assert_eq!(c.dist_manhattan(&d), 12);
        assert_eq!(c.dist_chessboard(&d), 6);
        // Java: Math.abs(i32::MIN) == i32::MIN, Math.max(i32::MIN, 0) == 0
        assert_eq!(a.dist_chessboard(&b), 0);
    }

    #[test]
    fn dist_sqr_and_center_float_bits() {
        let v = Vec3i::new(1, 2, 3);
        let other = Vec3i::new(4, 6, 3);
        assert_eq!(v.dist_sqr(&other).to_bits(), 25.0f64.to_bits());
        let center = v.dist_to_center_sqr(1.0, 2.0, 3.0);
        assert_eq!(center.to_bits(), 0.75f64.to_bits());
        let pos = SimplePos(1.0, 2.0, 3.0);
        assert_eq!(v.dist_to_center_sqr_pos(&pos).to_bits(), 0.75f64.to_bits());
        assert!(v.closer_than(&other, 6.0));
        assert!(!v.closer_than(&other, 5.0));
        assert!(v.closer_to_center_than(&pos, 1.0));
    }

    #[test]
    fn relative_cross_and_axis() {
        let v = Vec3i::new(1, 2, 3);
        assert_eq!(v.above(), Vec3i::new(1, 3, 3));
        assert_eq!(v.below_n(2), Vec3i::new(1, 0, 3));
        assert_eq!(v.north(), Vec3i::new(1, 2, 2));
        assert_eq!(v.south_n(2), Vec3i::new(1, 2, 5));
        assert_eq!(v.west(), Vec3i::new(0, 2, 3));
        assert_eq!(v.east_n(3), Vec3i::new(4, 2, 3));
        assert_eq!(v.relative_axis(Axis::X, 5), Vec3i::new(6, 2, 3));
        assert_eq!(v.relative_axis(Axis::Y, 0), v);
        assert_eq!(v.get(Axis::Y), 2);
        let cross = Vec3i::new(1, 0, 0).cross(&Vec3i::new(0, 1, 0));
        assert_eq!(cross, Vec3i::new(0, 0, 1));
    }
}
