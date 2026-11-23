/// Placeholder for unported `net.minecraft.world.level.pathfinder.Path`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Path;

#[derive(Debug, Clone, PartialEq)]
pub struct DebugPathInfo {
    path: Path,
    max_node_distance: f32,
}

impl DebugPathInfo {
    // JAVA: net.minecraft.util.debug.DebugPathInfo#<init>
    pub fn new(path: Path, max_node_distance: f32) -> Self {
        Self {
            path,
            max_node_distance,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugPathInfo#path
    pub fn path(&self) -> &Path {
        &self.path
    }

    // JAVA: net.minecraft.util.debug.DebugPathInfo#maxNodeDistance
    pub fn max_node_distance(&self) -> f32 {
        self.max_node_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_accessors() {
        let info = DebugPathInfo::new(Path, 1.5_f32);
        assert_eq!(info.max_node_distance().to_bits(), 1.5_f32.to_bits());
        assert_eq!(info.path(), &Path);
    }

    #[test]
    fn negative_max_node_distance() {
        let info = DebugPathInfo::new(Path, -2.0_f32);
        assert_eq!(info.max_node_distance().to_bits(), (-2.0_f32).to_bits());
        assert!(info.max_node_distance() < 0.0);
    }

    #[test]
    fn nan_float_bits() {
        let nan = f32::from_bits(0x7FC0_0001);
        let info = DebugPathInfo::new(Path, nan);
        assert_eq!(info.max_node_distance().to_bits(), nan.to_bits());
        assert!(info.max_node_distance().is_nan());
    }

    #[test]
    fn infinity_float_bits() {
        let pos_inf = DebugPathInfo::new(Path, f32::INFINITY);
        let neg_inf = DebugPathInfo::new(Path, f32::NEG_INFINITY);
        assert_eq!(
            pos_inf.max_node_distance().to_bits(),
            f32::INFINITY.to_bits()
        );
        assert_eq!(
            neg_inf.max_node_distance().to_bits(),
            f32::NEG_INFINITY.to_bits()
        );
    }

    #[test]
    fn zero_and_negative_zero_bits() {
        let pos = DebugPathInfo::new(Path, 0.0_f32);
        let neg = DebugPathInfo::new(Path, -0.0_f32);
        assert_eq!(pos.max_node_distance().to_bits(), 0.0_f32.to_bits());
        assert_eq!(neg.max_node_distance().to_bits(), (-0.0_f32).to_bits());
        assert_ne!(
            pos.max_node_distance().to_bits(),
            neg.max_node_distance().to_bits()
        );
    }

    #[test]
    fn max_and_min_float_bits() {
        let max_info = DebugPathInfo::new(Path, f32::MAX);
        let min_info = DebugPathInfo::new(Path, f32::MIN);
        assert_eq!(max_info.max_node_distance().to_bits(), f32::MAX.to_bits());
        assert_eq!(min_info.max_node_distance().to_bits(), f32::MIN.to_bits());
    }
}
