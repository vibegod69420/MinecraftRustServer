/// Minimal self-contained stub for `net.minecraft.core.Direction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn get_step_x(self) -> i32 {
        match self {
            Direction::East => 1,
            Direction::West => -1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction#getStepY
    pub fn get_step_y(self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction#getStepZ
    pub fn get_step_z(self) -> i32 {
        match self {
            Direction::South => 1,
            Direction::North => -1,
            _ => 0,
        }
    }
}

/// Faithful port of `net.minecraft.core.Direction8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction8 {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction8 {
    // JAVA: net.minecraft.core.Direction8#<init>
    fn directions(self) -> &'static [Direction] {
        match self {
            Direction8::North => &[Direction::North],
            Direction8::NorthEast => &[Direction::North, Direction::East],
            Direction8::East => &[Direction::East],
            Direction8::SouthEast => &[Direction::South, Direction::East],
            Direction8::South => &[Direction::South],
            Direction8::SouthWest => &[Direction::South, Direction::West],
            Direction8::West => &[Direction::West],
            Direction8::NorthWest => &[Direction::North, Direction::West],
        }
    }

    // JAVA: net.minecraft.core.Direction8#getDirections
    pub fn get_directions(self) -> &'static [Direction] {
        self.directions()
    }

    // JAVA: net.minecraft.core.Direction8#getStepX
    pub fn get_step_x(self) -> i32 {
        let dirs = self.directions();
        let mut x = 0i32;
        let mut i = 0usize;
        while i < dirs.len() {
            x = x.wrapping_add(dirs[i].get_step_x());
            i += 1;
        }
        x
    }

    // JAVA: net.minecraft.core.Direction8#getStepZ
    pub fn get_step_z(self) -> i32 {
        let dirs = self.directions();
        let mut z = 0i32;
        let mut i = 0usize;
        while i < dirs.len() {
            z = z.wrapping_add(dirs[i].get_step_z());
            i += 1;
        }
        z
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, Direction8};

    #[test]
    fn cardinal_steps() {
        assert_eq!(Direction8::North.get_step_x(), 0);
        assert_eq!(Direction8::North.get_step_z(), -1);
        assert_eq!(Direction8::East.get_step_x(), 1);
        assert_eq!(Direction8::East.get_step_z(), 0);
        assert_eq!(Direction8::South.get_step_x(), 0);
        assert_eq!(Direction8::South.get_step_z(), 1);
        assert_eq!(Direction8::West.get_step_x(), -1);
        assert_eq!(Direction8::West.get_step_z(), 0);
    }

    #[test]
    fn diagonal_steps() {
        assert_eq!(Direction8::NorthEast.get_step_x(), 1);
        assert_eq!(Direction8::NorthEast.get_step_z(), -1);
        assert_eq!(Direction8::SouthEast.get_step_x(), 1);
        assert_eq!(Direction8::SouthEast.get_step_z(), 1);
        assert_eq!(Direction8::SouthWest.get_step_x(), -1);
        assert_eq!(Direction8::SouthWest.get_step_z(), 1);
        assert_eq!(Direction8::NorthWest.get_step_x(), -1);
        assert_eq!(Direction8::NorthWest.get_step_z(), -1);
    }

    #[test]
    fn directions_contents() {
        assert_eq!(Direction8::North.get_directions(), &[Direction::North]);
        assert_eq!(
            Direction8::NorthEast.get_directions(),
            &[Direction::North, Direction::East]
        );
        assert_eq!(
            Direction8::SouthWest.get_directions(),
            &[Direction::South, Direction::West]
        );
    }

    #[test]
    fn direction_stub_steps() {
        assert_eq!(Direction::Up.get_step_y(), 1);
        assert_eq!(Direction::Down.get_step_y(), -1);
        assert_eq!(Direction::North.get_step_y(), 0);
        assert_eq!(Direction::West.get_step_x(), -1);
        assert_eq!(Direction::South.get_step_z(), 1);
    }
}
