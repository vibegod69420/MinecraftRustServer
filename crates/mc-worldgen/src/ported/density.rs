// JAVA: net.minecraft.world.level.levelgen.Density
// Ported from net.minecraft.world.level.levelgen.Density

// JAVA: net.minecraft.world.level.levelgen.Density#surface
pub fn surface() -> f64 {
    0.0
}

// JAVA: net.minecraft.world.level.levelgen.Density#unrecoverablyDense
pub fn unrecoverably_dense() -> f64 {
    64.0
}

// JAVA: net.minecraft.world.level.levelgen.Density#unrecoverablyThin
pub fn unrecoverably_thin() -> f64 {
    -64.0
}

// Java static final double fields
pub const SURFACE: f64 = 0.0;
pub const UNRECOVERABLY_DENSE: f64 = 64.0;
pub const UNRECOVERABLY_THIN: f64 = -64.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_return() {
        assert_eq!(surface(), 0.0);
    }

    #[test]
    fn test_dense_return() {
        assert_eq!(unrecoverably_dense(), 64.0);
    }

    #[test]
    fn test_thin_return() {
        assert_eq!(unrecoverably_thin(), -64.0);
    }

    #[test]
    fn test_surface_bits() {
        // Java double 0.0 bit representation is 0
        assert_eq!(surface().to_bits(), 0u64);
    }

    #[test]
    fn test_dense_bits() {
        let expected = 64.0_f64.to_bits();
        assert_eq!(unrecoverably_dense().to_bits(), expected);
    }

    #[test]
    fn test_thin_bits() {
        let expected = (-64.0_f64).to_bits();
        assert_eq!(unrecoverably_thin().to_bits(), expected);
    }

    #[test]
    fn test_constants_match_functions() {
        assert_eq!(SURFACE, surface());
        assert_eq!(UNRECOVERABLY_DENSE, unrecoverably_dense());
        assert_eq!(UNRECOVERABLY_THIN, unrecoverably_thin());
    }
}
