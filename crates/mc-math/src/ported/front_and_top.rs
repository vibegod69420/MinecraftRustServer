#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

impl Direction {
    pub const VALUES: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    // JAVA: net.minecraft.core.Direction#ordinal
    pub const fn ordinal(self) -> usize {
        self as usize
    }
}

const NUM_DIRECTIONS: usize = Direction::VALUES.len();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrontAndTop {
    DownEast,
    DownNorth,
    DownSouth,
    DownWest,
    UpEast,
    UpNorth,
    UpSouth,
    UpWest,
    WestUp,
    EastUp,
    NorthUp,
    SouthUp,
}

impl FrontAndTop {
    pub const VALUES: [FrontAndTop; 12] = [
        FrontAndTop::DownEast,
        FrontAndTop::DownNorth,
        FrontAndTop::DownSouth,
        FrontAndTop::DownWest,
        FrontAndTop::UpEast,
        FrontAndTop::UpNorth,
        FrontAndTop::UpSouth,
        FrontAndTop::UpWest,
        FrontAndTop::WestUp,
        FrontAndTop::EastUp,
        FrontAndTop::NorthUp,
        FrontAndTop::SouthUp,
    ];

    // JAVA: net.minecraft.core.FrontAndTop#lookupKey
    const fn lookup_key(front: Direction, top: Direction) -> usize {
        front.ordinal() * NUM_DIRECTIONS + top.ordinal()
    }

    // JAVA: net.minecraft.core.FrontAndTop#getSerializedName
    pub const fn get_serialized_name(self) -> &'static str {
        match self {
            FrontAndTop::DownEast => "down_east",
            FrontAndTop::DownNorth => "down_north",
            FrontAndTop::DownSouth => "down_south",
            FrontAndTop::DownWest => "down_west",
            FrontAndTop::UpEast => "up_east",
            FrontAndTop::UpNorth => "up_north",
            FrontAndTop::UpSouth => "up_south",
            FrontAndTop::UpWest => "up_west",
            FrontAndTop::WestUp => "west_up",
            FrontAndTop::EastUp => "east_up",
            FrontAndTop::NorthUp => "north_up",
            FrontAndTop::SouthUp => "south_up",
        }
    }

    // JAVA: net.minecraft.core.FrontAndTop#fromFrontAndTop
    pub fn from_front_and_top(front: Direction, top: Direction) -> FrontAndTop {
        BY_TOP_FRONT[Self::lookup_key(front, top)]
    }

    // JAVA: net.minecraft.core.FrontAndTop#front
    pub const fn front(self) -> Direction {
        match self {
            FrontAndTop::DownEast => Direction::Down,
            FrontAndTop::DownNorth => Direction::Down,
            FrontAndTop::DownSouth => Direction::Down,
            FrontAndTop::DownWest => Direction::Down,
            FrontAndTop::UpEast => Direction::Up,
            FrontAndTop::UpNorth => Direction::Up,
            FrontAndTop::UpSouth => Direction::Up,
            FrontAndTop::UpWest => Direction::Up,
            FrontAndTop::WestUp => Direction::West,
            FrontAndTop::EastUp => Direction::East,
            FrontAndTop::NorthUp => Direction::North,
            FrontAndTop::SouthUp => Direction::South,
        }
    }

    // JAVA: net.minecraft.core.FrontAndTop#top
    pub const fn top(self) -> Direction {
        match self {
            FrontAndTop::DownEast => Direction::East,
            FrontAndTop::DownNorth => Direction::North,
            FrontAndTop::DownSouth => Direction::South,
            FrontAndTop::DownWest => Direction::West,
            FrontAndTop::UpEast => Direction::East,
            FrontAndTop::UpNorth => Direction::North,
            FrontAndTop::UpSouth => Direction::South,
            FrontAndTop::UpWest => Direction::West,
            FrontAndTop::WestUp => Direction::Up,
            FrontAndTop::EastUp => Direction::Up,
            FrontAndTop::NorthUp => Direction::Up,
            FrontAndTop::SouthUp => Direction::Up,
        }
    }
}

const fn build_by_top_front() -> [FrontAndTop; NUM_DIRECTIONS * NUM_DIRECTIONS] {
    let mut table = [FrontAndTop::DownEast; NUM_DIRECTIONS * NUM_DIRECTIONS];
    let mut i = 0;
    while i < FrontAndTop::VALUES.len() {
        let v = FrontAndTop::VALUES[i];
        table[FrontAndTop::lookup_key(v.front(), v.top())] = v;
        i += 1;
    }
    table
}

static BY_TOP_FRONT: [FrontAndTop; NUM_DIRECTIONS * NUM_DIRECTIONS] = build_by_top_front();

#[cfg(test)]
mod tests {
    use super::{Direction, FrontAndTop};

    #[test]
    fn roundtrip_all_variants() {
        for &v in &FrontAndTop::VALUES {
            let got = FrontAndTop::from_front_and_top(v.front(), v.top());
            assert_eq!(got, v);
        }
    }

    #[test]
    fn serialized_names_match_java() {
        assert_eq!(FrontAndTop::DownEast.get_serialized_name(), "down_east");
        assert_eq!(FrontAndTop::UpWest.get_serialized_name(), "up_west");
        assert_eq!(FrontAndTop::WestUp.get_serialized_name(), "west_up");
        assert_eq!(FrontAndTop::SouthUp.get_serialized_name(), "south_up");
    }

    #[test]
    fn front_and_top_directions() {
        assert_eq!(FrontAndTop::NorthUp.front(), Direction::North);
        assert_eq!(FrontAndTop::NorthUp.top(), Direction::Up);
        assert_eq!(FrontAndTop::DownSouth.front(), Direction::Down);
        assert_eq!(FrontAndTop::DownSouth.top(), Direction::South);
    }

    #[test]
    fn lookup_key_covers_all_directions() {
        assert_eq!(Direction::VALUES.len(), 6);
        assert_eq!(FrontAndTop::VALUES.len(), 12);
        // Spot-check a few known keys
        let k = FrontAndTop::from_front_and_top(Direction::East, Direction::Up);
        assert_eq!(k, FrontAndTop::EastUp);
    }
}
