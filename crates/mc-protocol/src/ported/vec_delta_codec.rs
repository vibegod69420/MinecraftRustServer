const TRUNCATION_STEPS: f64 = 4096.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn subtract(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VecDeltaCodec {
    base: Vec3,
}

impl Default for VecDeltaCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl VecDeltaCodec {
    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#<init>
    pub fn new() -> Self {
        Self { base: Vec3::ZERO }
    }

    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#encode
    pub fn encode(value: f64) -> i64 {
        (value * TRUNCATION_STEPS + 0.5).floor() as i64
    }

    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#decode
    pub fn decode_coord(value: i64) -> f64 {
        (value as f64) / TRUNCATION_STEPS
    }

    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#decode
    pub fn decode(&self, x: i64, y: i64, z: i64) -> Vec3 {
        if x == 0_i64 && y == 0_i64 && z == 0_i64 {
            return self.base;
        }

        let dx = if x == 0_i64 {
            self.base.x
        } else {
            Self::decode_coord(Self::encode(self.base.x).wrapping_add(x))
        };
        let dy = if y == 0_i64 {
            self.base.y
        } else {
            Self::decode_coord(Self::encode(self.base.y).wrapping_add(y))
        };
        let dz = if z == 0_i64 {
            self.base.z
        } else {
            Self::decode_coord(Self::encode(self.base.z).wrapping_add(z))
        };

        Vec3 {
            x: dx,
            y: dy,
            z: dz,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#encodeX
    pub fn encode_x(&self, pos: Vec3) -> i64 {
        Self::encode(pos.x).wrapping_sub(Self::encode(self.base.x))
    }

    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#encodeY
    pub fn encode_y(&self, pos: Vec3) -> i64 {
        Self::encode(pos.y).wrapping_sub(Self::encode(self.base.y))
    }

    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#encodeZ
    pub fn encode_z(&self, pos: Vec3) -> i64 {
        Self::encode(pos.z).wrapping_sub(Self::encode(self.base.z))
    }

    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#delta
    pub fn delta(&self, pos: Vec3) -> Vec3 {
        pos.subtract(self.base)
    }

    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#setBase
    pub fn set_base(&mut self, base: Vec3) {
        self.base = base;
    }

    // JAVA: net.minecraft.network.protocol.game.VecDeltaCodec#getBase
    pub fn get_base(&self) -> Vec3 {
        self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_matches_java_math_round_behavior() {
        assert_eq!(VecDeltaCodec::encode(0.0), 0_i64);
        assert_eq!(VecDeltaCodec::encode(1.0 / 4096.0), 1_i64);
        assert_eq!(VecDeltaCodec::encode(-1.5 / 4096.0), -1_i64);
        assert_eq!(VecDeltaCodec::encode(-2.5 / 4096.0), -2_i64);
        assert_eq!(VecDeltaCodec::encode(0.5 / 4096.0), 1_i64);
    }

    #[test]
    fn decode_coord_and_float_bits() {
        let decoded = VecDeltaCodec::decode_coord(1_i64);
        assert_eq!(decoded.to_bits(), (1.0_f64 / 4096.0_f64).to_bits());
        let neg = VecDeltaCodec::decode_coord(-4096_i64);
        assert_eq!(neg.to_bits(), (-1.0_f64).to_bits());
    }

    #[test]
    fn zero_delta_returns_exact_base_bits() {
        let mut codec = VecDeltaCodec::new();
        let base = Vec3 {
            x: 1.234_567_89_f64,
            y: -9.876_543_21_f64,
            z: 0.000_1_f64,
        };
        codec.set_base(base);
        let decoded = codec.decode(0_i64, 0_i64, 0_i64);
        assert_eq!(decoded.x.to_bits(), base.x.to_bits());
        assert_eq!(decoded.y.to_bits(), base.y.to_bits());
        assert_eq!(decoded.z.to_bits(), base.z.to_bits());
    }

    #[test]
    fn encode_decode_roundtrip_with_negative_values() {
        let mut codec = VecDeltaCodec::new();
        codec.set_base(Vec3 {
            x: -100.5_f64,
            y: 64.0_f64,
            z: -0.125_f64,
        });
        let pos = Vec3 {
            x: -100.25_f64,
            y: 64.5_f64,
            z: -0.125_f64,
        };
        let ex = codec.encode_x(pos);
        let ey = codec.encode_y(pos);
        let ez = codec.encode_z(pos);
        let decoded = codec.decode(ex, ey, ez);
        assert_eq!(decoded.x.to_bits(), pos.x.to_bits());
        assert_eq!(decoded.y.to_bits(), pos.y.to_bits());
        assert_eq!(decoded.z.to_bits(), pos.z.to_bits());
    }

    #[test]
    fn delta_and_wrapping_sub_behavior() {
        let mut codec = VecDeltaCodec::new();
        let base = Vec3 {
            x: 10.0_f64,
            y: -20.0_f64,
            z: 30.0_f64,
        };
        codec.set_base(base);
        let pos = Vec3 {
            x: 11.5_f64,
            y: -20.25_f64,
            z: 29.75_f64,
        };
        let delta = codec.delta(pos);
        assert_eq!(delta.x.to_bits(), (pos.x - base.x).to_bits());
        assert_eq!(delta.y.to_bits(), (pos.y - base.y).to_bits());
        assert_eq!(delta.z.to_bits(), (pos.z - base.z).to_bits());

        let expected_x = VecDeltaCodec::encode(pos.x).wrapping_sub(VecDeltaCodec::encode(base.x));
        assert_eq!(codec.encode_x(pos), expected_x);

        let large = Vec3 {
            x: f64::from(i32::MAX),
            y: 0.0_f64,
            z: 0.0_f64,
        };
        let expected_large =
            VecDeltaCodec::encode(large.x).wrapping_sub(VecDeltaCodec::encode(base.x));
        assert_eq!(codec.encode_x(large), expected_large);
    }
}
