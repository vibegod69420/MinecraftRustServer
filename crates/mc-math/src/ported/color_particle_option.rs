use crate::mth;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleType {
    pub id: &'static str,
}

// JAVA: net.minecraft.util.ARGB#alpha
fn argb_alpha(color: i32) -> i32 {
    ((color as u32) >> 24) as i32 & 0xFF
}

// JAVA: net.minecraft.util.ARGB#red
fn argb_red(color: i32) -> i32 {
    (color >> 16) & 0xFF
}

// JAVA: net.minecraft.util.ARGB#green
fn argb_green(color: i32) -> i32 {
    (color >> 8) & 0xFF
}

// JAVA: net.minecraft.util.ARGB#blue
fn argb_blue(color: i32) -> i32 {
    color & 0xFF
}

// JAVA: net.minecraft.util.ARGB#color
fn argb_color(a: i32, r: i32, g: i32, b: i32) -> i32 {
    ((a & 0xFF) << 24) | ((r & 0xFF) << 16) | ((g & 0xFF) << 8) | (b & 0xFF)
}

// JAVA: net.minecraft.util.ARGB#as8BitChannel
fn as8bit_channel(v: f32) -> i32 {
    mth::floor_f32(v * 255.0)
}

// JAVA: net.minecraft.util.ARGB#colorFromFloat
fn argb_color_from_float(a: f32, r: f32, g: f32, b: f32) -> i32 {
    argb_color(
        as8bit_channel(a),
        as8bit_channel(r),
        as8bit_channel(g),
        as8bit_channel(b),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorParticleOption {
    type_: ParticleType,
    color: i32,
}

impl ColorParticleOption {
    // JAVA: net.minecraft.core.particles.ColorParticleOption#<init>
    fn new(type_: ParticleType, color: i32) -> Self {
        Self { type_, color }
    }

    // JAVA: net.minecraft.core.particles.ColorParticleOption#getType
    pub fn get_type(&self) -> ParticleType {
        self.type_
    }

    // JAVA: net.minecraft.core.particles.ColorParticleOption#getRed
    pub fn get_red(&self) -> f32 {
        argb_red(self.color) as f32 / 255.0
    }

    // JAVA: net.minecraft.core.particles.ColorParticleOption#getGreen
    pub fn get_green(&self) -> f32 {
        argb_green(self.color) as f32 / 255.0
    }

    // JAVA: net.minecraft.core.particles.ColorParticleOption#getBlue
    pub fn get_blue(&self) -> f32 {
        argb_blue(self.color) as f32 / 255.0
    }

    // JAVA: net.minecraft.core.particles.ColorParticleOption#getAlpha
    pub fn get_alpha(&self) -> f32 {
        argb_alpha(self.color) as f32 / 255.0
    }

    // JAVA: net.minecraft.core.particles.ColorParticleOption#create
    pub fn create(type_: ParticleType, color: i32) -> Self {
        Self::new(type_, color)
    }

    // JAVA: net.minecraft.core.particles.ColorParticleOption#create
    pub fn create_from_floats(type_: ParticleType, r: f32, g: f32, b: f32) -> Self {
        Self::create(type_, argb_color_from_float(1.0, r, g, b))
    }

    pub fn color(&self) -> i32 {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt() -> ParticleType {
        ParticleType {
            id: "minecraft:entity_effect",
        }
    }

    #[test]
    fn create_from_int_color_components() {
        let c = ColorParticleOption::create(pt(), 0x80FF4080u32 as i32);
        assert_eq!(c.get_alpha().to_bits(), (128.0f32 / 255.0).to_bits());
        assert_eq!(c.get_red().to_bits(), 1.0f32.to_bits());
        assert_eq!(c.get_green().to_bits(), (64.0f32 / 255.0).to_bits());
        assert_eq!(c.get_blue().to_bits(), (128.0f32 / 255.0).to_bits());
    }

    #[test]
    fn create_from_floats() {
        let c = ColorParticleOption::create_from_floats(pt(), 1.0, 0.0, 0.5);
        assert_eq!(c.get_alpha().to_bits(), 1.0f32.to_bits());
        assert_eq!(c.get_red().to_bits(), 1.0f32.to_bits());
        assert_eq!(c.get_green().to_bits(), 0.0f32.to_bits());
        let expected_blue = 127.0f32 / 255.0; // floor(0.5 * 255.0) == 127
        assert_eq!(c.get_blue().to_bits(), expected_blue.to_bits());
    }

    #[test]
    fn negative_color_alpha() {
        let c = ColorParticleOption::create(pt(), -1);
        assert_eq!(c.get_alpha().to_bits(), 1.0f32.to_bits());
        assert_eq!(c.get_red().to_bits(), 1.0f32.to_bits());
        assert_eq!(c.get_green().to_bits(), 1.0f32.to_bits());
        assert_eq!(c.get_blue().to_bits(), 1.0f32.to_bits());
    }

    #[test]
    fn create_from_floats_negative_and_overflow_mask() {
        let c = ColorParticleOption::create_from_floats(pt(), -1.0, 2.0, 0.0);
        // floor(-255) = -255, -255 & 0xFF = 1
        // floor(510) = 510, 510 & 0xFF = 254
        assert_eq!(c.get_red().to_bits(), (1.0f32 / 255.0).to_bits());
        assert_eq!(c.get_green().to_bits(), (254.0f32 / 255.0).to_bits());
        assert_eq!(c.get_blue().to_bits(), 0.0f32.to_bits());
        assert_eq!(c.get_alpha().to_bits(), 1.0f32.to_bits());
    }

    #[test]
    fn get_type_preserved() {
        let t = ParticleType { id: "test:particle" };
        let c = ColorParticleOption::create(t, 0);
        assert_eq!(c.get_type(), t);
        assert_eq!(c.color(), 0);
    }
}
