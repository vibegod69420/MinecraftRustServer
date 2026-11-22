#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleType {
    pub id: &'static str,
}

pub struct ParticleTypes;

impl ParticleTypes {
    pub const DUST: ParticleType = ParticleType {
        id: "minecraft:dust",
    };
}

// JAVA: net.minecraft.util.ARGB#vector3fFromRGB24
fn vector3f_from_rgb24(color: i32) -> Vector3f {
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    Vector3f::new(r, g, b)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DustParticleOptions {
    scale: f32,
    color: i32,
}

impl DustParticleOptions {
    pub const REDSTONE_PARTICLE_COLOR: i32 = 16_711_680;
    pub const REDSTONE: Self = Self {
        scale: 1.0,
        color: 16_711_680,
    };

    // JAVA: net.minecraft.core.particles.DustParticleOptions#<init>
    pub fn new(color: i32, scale: f32) -> Self {
        Self { scale, color }
    }

    // JAVA: net.minecraft.core.particles.ScalableParticleOptionsBase#getScale
    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    // JAVA: net.minecraft.core.particles.DustParticleOptions#getType
    pub fn get_type(&self) -> ParticleType {
        ParticleTypes::DUST
    }

    // JAVA: net.minecraft.core.particles.DustParticleOptions#getColor
    pub fn get_color(&self) -> Vector3f {
        vector3f_from_rgb24(self.color)
    }

    pub fn color(&self) -> i32 {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redstone_constant() {
        assert_eq!(DustParticleOptions::REDSTONE_PARTICLE_COLOR, 0xFF0000);
        let o = DustParticleOptions::REDSTONE;
        assert_eq!(o.get_scale().to_bits(), 1.0f32.to_bits());
        assert_eq!(o.color(), 0xFF0000);
    }

    #[test]
    fn get_color_rgb24() {
        let o = DustParticleOptions::new(0x00FF00, 1.0);
        let c = o.get_color();
        assert_eq!(c.x.to_bits(), 0.0f32.to_bits());
        assert_eq!(c.y.to_bits(), 1.0f32.to_bits());
        assert_eq!(c.z.to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn get_color_negative_int_bits() {
        let o = DustParticleOptions::new(-1, 0.5);
        let c = o.get_color();
        assert_eq!(c.x.to_bits(), 1.0f32.to_bits());
        assert_eq!(c.y.to_bits(), 1.0f32.to_bits());
        assert_eq!(c.z.to_bits(), 1.0f32.to_bits());
        assert_eq!(o.get_scale().to_bits(), 0.5f32.to_bits());
    }

    #[test]
    fn get_type_is_dust() {
        let o = DustParticleOptions::new(0, 1.0);
        assert_eq!(o.get_type(), ParticleTypes::DUST);
    }

    #[test]
    fn scale_preserves_float_bits() {
        let bits = 0xFFC0_0001u32;
        let scale = f32::from_bits(bits);
        let o = DustParticleOptions::new(0x123456, scale);
        assert_eq!(o.get_scale().to_bits(), bits);
        assert_eq!(o.color(), 0x123456);
    }
}
