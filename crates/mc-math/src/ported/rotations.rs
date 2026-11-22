/// Faithful port of `net.minecraft.core.Rotations`.
///
/// Codec/StreamCodec fields from Java are omitted for self-containment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotations {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Rotations {
    // JAVA: net.minecraft.core.Rotations#<init>
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: normalize_rotation(x),
            y: normalize_rotation(y),
            z: normalize_rotation(z),
        }
    }

    // JAVA: net.minecraft.core.Rotations#x
    pub fn x(&self) -> f32 {
        self.x
    }

    // JAVA: net.minecraft.core.Rotations#y
    pub fn y(&self) -> f32 {
        self.y
    }

    // JAVA: net.minecraft.core.Rotations#z
    pub fn z(&self) -> f32 {
        self.z
    }
}

fn normalize_rotation(v: f32) -> f32 {
    if !v.is_infinite() && !v.is_nan() {
        v % 360.0f32
    } else {
        0.0f32
    }
}

#[cfg(test)]
mod tests {
    use super::Rotations;

    #[test]
    fn normalizes_finite_values() {
        let r = Rotations::new(370.0, -10.0, 360.0);
        assert_eq!(r.x().to_bits(), (370.0f32 % 360.0f32).to_bits());
        assert_eq!(r.y().to_bits(), (-10.0f32 % 360.0f32).to_bits());
        assert_eq!(r.z().to_bits(), (360.0f32 % 360.0f32).to_bits());
    }

    #[test]
    fn normalizes_nan_and_infinity_to_zero() {
        let r = Rotations::new(f32::NAN, f32::INFINITY, f32::NEG_INFINITY);
        assert_eq!(r.x().to_bits(), 0.0f32.to_bits());
        assert_eq!(r.y().to_bits(), 0.0f32.to_bits());
        assert_eq!(r.z().to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn preserves_negative_zero_and_small_values() {
        let r = Rotations::new(-0.0, 0.5, -359.5);
        assert_eq!(r.x().to_bits(), (-0.0f32 % 360.0f32).to_bits());
        assert_eq!(r.y().to_bits(), 0.5f32.to_bits());
        assert_eq!(r.z().to_bits(), (-359.5f32 % 360.0f32).to_bits());
    }

    #[test]
    fn handles_large_magnitude() {
        let v = 1_000_000.0f32;
        let r = Rotations::new(v, -v, 720.25);
        assert_eq!(r.x().to_bits(), (v % 360.0f32).to_bits());
        assert_eq!(r.y().to_bits(), ((-v) % 360.0f32).to_bits());
        assert_eq!(r.z().to_bits(), (720.25f32 % 360.0f32).to_bits());
    }
}
