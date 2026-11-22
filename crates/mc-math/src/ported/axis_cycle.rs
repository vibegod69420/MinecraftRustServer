/// Minimal self-contained representation of `net.minecraft.core.Direction.Axis`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub const VALUES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

    // JAVA: net.minecraft.core.Direction$Axis#choose(int,int,int)
    pub fn choose_i32(self, x: i32, y: i32, z: i32) -> i32 {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }

    // JAVA: net.minecraft.core.Direction$Axis#choose(double,double,double)
    pub fn choose_f64(self, x: f64, y: f64, z: f64) -> f64 {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }

    // JAVA: java.lang.Enum#ordinal()
    pub fn ordinal(self) -> i32 {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AxisCycle {
    None,
    Forward,
    Backward,
}

pub const AXIS_VALUES: [Axis; 3] = Axis::VALUES;
pub const VALUES: [AxisCycle; 3] = [AxisCycle::None, AxisCycle::Forward, AxisCycle::Backward];

impl AxisCycle {
    // JAVA: net.minecraft.core.AxisCycle#cycle(int,int,int,Direction$Axis)
    pub fn cycle_i32(self, x: i32, y: i32, z: i32, axis: Axis) -> i32 {
        match self {
            AxisCycle::None => axis.choose_i32(x, y, z),
            AxisCycle::Forward => axis.choose_i32(z, x, y),
            AxisCycle::Backward => axis.choose_i32(y, z, x),
        }
    }

    // JAVA: net.minecraft.core.AxisCycle#cycle(double,double,double,Direction$Axis)
    pub fn cycle_f64(self, x: f64, y: f64, z: f64, axis: Axis) -> f64 {
        match self {
            AxisCycle::None => axis.choose_f64(x, y, z),
            AxisCycle::Forward => axis.choose_f64(z, x, y),
            AxisCycle::Backward => axis.choose_f64(y, z, x),
        }
    }

    // JAVA: net.minecraft.core.AxisCycle#cycle(Direction$Axis)
    pub fn cycle_axis(self, axis: Axis) -> Axis {
        match self {
            AxisCycle::None => axis,
            AxisCycle::Forward => {
                AXIS_VALUES[(axis.ordinal().wrapping_add(1)).rem_euclid(3) as usize]
            }
            AxisCycle::Backward => {
                AXIS_VALUES[(axis.ordinal().wrapping_sub(1)).rem_euclid(3) as usize]
            }
        }
    }

    // JAVA: net.minecraft.core.AxisCycle#inverse()
    pub fn inverse(self) -> AxisCycle {
        match self {
            AxisCycle::None => AxisCycle::None,
            AxisCycle::Forward => AxisCycle::Backward,
            AxisCycle::Backward => AxisCycle::Forward,
        }
    }

    // JAVA: net.minecraft.core.AxisCycle#between(Direction$Axis,Direction$Axis)
    pub fn between(from: Axis, to: Axis) -> AxisCycle {
        VALUES[(to.ordinal().wrapping_sub(from.ordinal())).rem_euclid(3) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_choose_and_ordinal() {
        assert_eq!(Axis::X.choose_i32(1, 2, 3), 1);
        assert_eq!(Axis::Y.choose_i32(1, 2, 3), 2);
        assert_eq!(Axis::Z.choose_i32(1, 2, 3), 3);
        assert_eq!(Axis::X.ordinal(), 0);
        assert_eq!(Axis::Y.ordinal(), 1);
        assert_eq!(Axis::Z.ordinal(), 2);
        assert_eq!(Axis::X.choose_f64(1.5, -2.0, 3.25).to_bits(), 1.5f64.to_bits());
    }

    #[test]
    fn cycle_none_identity() {
        for &a in &AXIS_VALUES {
            assert_eq!(AxisCycle::None.cycle_axis(a), a);
        }
        assert_eq!(AxisCycle::None.cycle_i32(10, 20, 30, Axis::X), 10);
        assert_eq!(AxisCycle::None.cycle_i32(10, 20, 30, Axis::Y), 20);
        assert_eq!(AxisCycle::None.cycle_i32(10, 20, 30, Axis::Z), 30);
    }

    #[test]
    fn cycle_forward_backward_axes() {
        assert_eq!(AxisCycle::Forward.cycle_axis(Axis::X), Axis::Y);
        assert_eq!(AxisCycle::Forward.cycle_axis(Axis::Y), Axis::Z);
        assert_eq!(AxisCycle::Forward.cycle_axis(Axis::Z), Axis::X);
        assert_eq!(AxisCycle::Backward.cycle_axis(Axis::X), Axis::Z);
        assert_eq!(AxisCycle::Backward.cycle_axis(Axis::Y), Axis::X);
        assert_eq!(AxisCycle::Backward.cycle_axis(Axis::Z), Axis::Y);
    }

    #[test]
    fn cycle_i32_and_f64_values() {
        // FORWARD: choose(z, x, y)
        assert_eq!(AxisCycle::Forward.cycle_i32(1, 2, 3, Axis::X), 3);
        assert_eq!(AxisCycle::Forward.cycle_i32(1, 2, 3, Axis::Y), 1);
        assert_eq!(AxisCycle::Forward.cycle_i32(1, 2, 3, Axis::Z), 2);
        // BACKWARD: choose(y, z, x)
        assert_eq!(AxisCycle::Backward.cycle_i32(1, 2, 3, Axis::X), 2);
        assert_eq!(AxisCycle::Backward.cycle_i32(1, 2, 3, Axis::Y), 3);
        assert_eq!(AxisCycle::Backward.cycle_i32(1, 2, 3, Axis::Z), 1);

        let v = AxisCycle::Forward.cycle_f64(1.5, -2.0, 3.25, Axis::X);
        assert_eq!(v.to_bits(), 3.25f64.to_bits());
    }

    #[test]
    fn inverse_and_between() {
        for &c in &VALUES {
            assert_eq!(c.inverse().inverse(), c);
            for &a in &AXIS_VALUES {
                assert_eq!(c.inverse().cycle_axis(c.cycle_axis(a)), a);
            }
        }
        assert_eq!(AxisCycle::between(Axis::X, Axis::X), AxisCycle::None);
        assert_eq!(AxisCycle::between(Axis::X, Axis::Y), AxisCycle::Forward);
        assert_eq!(AxisCycle::between(Axis::X, Axis::Z), AxisCycle::Backward);
        assert_eq!(AxisCycle::between(Axis::Z, Axis::X), AxisCycle::Forward);
    }
}
