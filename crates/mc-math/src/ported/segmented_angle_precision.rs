pub trait DirectionLike {
    fn axis_is_vertical(&self) -> bool;
    fn two_d_data_value(&self) -> i32;
}

#[derive(Debug, Clone)]
pub struct SegmentedAnglePrecision {
    mask: i32,
    precision: i32,
    degree_to_angle: f32,
    angle_to_degree: f32,
}

impl SegmentedAnglePrecision {
    // JAVA: net.minecraft.util.SegmentedAnglePrecision#<init>
    pub fn new(precision: i32) -> Self {
        if precision < 2 {
            panic!("IllegalArgumentException: Precision cannot be less than 2 bits");
        }
        if precision > 30 {
            panic!("IllegalArgumentException: Precision cannot be greater than 30 bits");
        }

        let size = 1i32.wrapping_shl((precision as u32) & 31);
        let mask = size.wrapping_sub(1);
        let degree_to_angle = (size as f32) / 360.0_f32;
        let angle_to_degree = 360.0_f32 / (size as f32);

        Self {
            mask,
            precision,
            degree_to_angle,
            angle_to_degree,
        }
    }

    // JAVA: net.minecraft.util.SegmentedAnglePrecision#isSameAxis
    pub fn is_same_axis(&self, a: i32, b: i32) -> bool {
        let half_mask = self.get_mask() >> 1;
        (a & half_mask) == (b & half_mask)
    }

    // JAVA: net.minecraft.util.SegmentedAnglePrecision#fromDirection
    pub fn from_direction<D: DirectionLike>(&self, dir: &D) -> i32 {
        if dir.axis_is_vertical() {
            return 0;
        }
        let value = dir.two_d_data_value();
        value.wrapping_shl(((self.precision - 2) as u32) & 31)
    }

    // JAVA: net.minecraft.util.SegmentedAnglePrecision#fromDegreesWithTurns
    pub fn from_degrees_with_turns(&self, degrees: f32) -> i32 {
        let v = degrees * self.degree_to_angle;
        (v + 0.5_f32).floor() as i32
    }

    // JAVA: net.minecraft.util.SegmentedAnglePrecision#fromDegrees
    pub fn from_degrees(&self, degrees: f32) -> i32 {
        self.normalize(self.from_degrees_with_turns(degrees))
    }

    // JAVA: net.minecraft.util.SegmentedAnglePrecision#toDegreesWithTurns
    pub fn to_degrees_with_turns(&self, angle: i32) -> f32 {
        (angle as f32) * self.angle_to_degree
    }

    // JAVA: net.minecraft.util.SegmentedAnglePrecision#toDegrees
    pub fn to_degrees(&self, angle: i32) -> f32 {
        let degrees = self.to_degrees_with_turns(self.normalize(angle));
        if degrees >= 180.0_f32 {
            degrees - 360.0_f32
        } else {
            degrees
        }
    }

    // JAVA: net.minecraft.util.SegmentedAnglePrecision#normalize
    pub fn normalize(&self, angle: i32) -> i32 {
        angle & self.mask
    }

    // JAVA: net.minecraft.util.SegmentedAnglePrecision#getMask
    pub fn get_mask(&self) -> i32 {
        self.mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDir {
        vertical: bool,
        data_2d: i32,
    }

    impl DirectionLike for MockDir {
        fn axis_is_vertical(&self) -> bool {
            self.vertical
        }

        fn two_d_data_value(&self) -> i32 {
            self.data_2d
        }
    }

    #[test]
    fn constructor_sets_mask_and_factors() {
        let p = SegmentedAnglePrecision::new(8);
        assert_eq!(p.get_mask(), 255);
        assert_eq!(p.to_degrees_with_turns(256), 360.0_f32);
    }

    #[test]
    #[should_panic(expected = "Precision cannot be less than 2 bits")]
    fn constructor_rejects_low_precision() {
        let _ = SegmentedAnglePrecision::new(1);
    }

    #[test]
    #[should_panic(expected = "Precision cannot be greater than 30 bits")]
    fn constructor_rejects_high_precision() {
        let _ = SegmentedAnglePrecision::new(31);
    }

    #[test]
    fn degrees_roundtrip_and_normalize() {
        let p = SegmentedAnglePrecision::new(8);
        let angle = p.from_degrees(90.0_f32);
        assert_eq!(angle, 64);
        assert_eq!(p.to_degrees(angle), 90.0_f32);
        assert_eq!(p.normalize(256 + 5), 5);
        assert_eq!(p.normalize(-1), 255);
    }

    #[test]
    fn from_direction_and_same_axis() {
        let p = SegmentedAnglePrecision::new(4); // size 16
        let vertical = MockDir { vertical: true, data_2d: 3 };
        let south = MockDir { vertical: false, data_2d: 0 }; // 0 << 2 = 0
        let east = MockDir { vertical: false, data_2d: 3 };  // 3 << 2 = 12
        assert_eq!(p.from_direction(&vertical), 0);
        assert_eq!(p.from_direction(&south), 0);
        assert_eq!(p.from_direction(&east), 12);
        assert!(p.is_same_axis(0, 8)); // same axis in 16-segment circle
        assert!(!p.is_same_axis(0, 4));
    }

    #[test]
    fn to_degrees_wraps_at_180() {
        let p = SegmentedAnglePrecision::new(8);
        assert_eq!(p.to_degrees(128), -180.0_f32);
        assert_eq!(p.to_degrees(192), -90.0_f32);
    }
}
