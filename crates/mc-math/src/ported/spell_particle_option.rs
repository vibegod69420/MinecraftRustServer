/// Port of `net.minecraft.core.particles.SpellParticleOption`.
///
/// Codec/stream codec factories are omitted because this module is self-contained
/// and does not depend on Mojang serialization infrastructure.

/// Minimal self-contained stand-in for particle type identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleTypeId(pub u32);

fn as_8bit_channel(value: f32) -> i32 {
    crate::mth::floor_f32(value * 255.0_f32)
}

fn argb_color(a: i32, r: i32, g: i32, b: i32) -> i32 {
    (((a & 0xFF) as u32) << 24
        | ((r & 0xFF) as u32) << 16
        | ((g & 0xFF) as u32) << 8
        | ((b & 0xFF) as u32)) as i32
}

fn argb_color_from_float(a: f32, r: f32, g: f32, b: f32) -> i32 {
    argb_color(
        as_8bit_channel(a),
        as_8bit_channel(r),
        as_8bit_channel(g),
        as_8bit_channel(b),
    )
}

fn argb_red(color: i32) -> i32 {
    (color >> 16) & 0xFF
}

fn argb_green(color: i32) -> i32 {
    (color >> 8) & 0xFF
}

fn argb_blue(color: i32) -> i32 {
    color & 0xFF
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpellParticleOption {
    type_id: ParticleTypeId,
    color: i32,
    power: f32,
}

impl SpellParticleOption {
    // JAVA: net.minecraft.core.particles.SpellParticleOption#<init>
    fn new(type_id: ParticleTypeId, color: i32, power: f32) -> Self {
        Self {
            type_id,
            color,
            power,
        }
    }

    // JAVA: net.minecraft.core.particles.SpellParticleOption#getType
    pub fn get_type(&self) -> ParticleTypeId {
        self.type_id
    }

    // JAVA: net.minecraft.core.particles.SpellParticleOption#getRed
    pub fn get_red(&self) -> f32 {
        argb_red(self.color) as f32 / 255.0_f32
    }

    // JAVA: net.minecraft.core.particles.SpellParticleOption#getGreen
    pub fn get_green(&self) -> f32 {
        argb_green(self.color) as f32 / 255.0_f32
    }

    // JAVA: net.minecraft.core.particles.SpellParticleOption#getBlue
    pub fn get_blue(&self) -> f32 {
        argb_blue(self.color) as f32 / 255.0_f32
    }

    // JAVA: net.minecraft.core.particles.SpellParticleOption#getPower
    pub fn get_power(&self) -> f32 {
        self.power
    }

    // JAVA: net.minecraft.core.particles.SpellParticleOption#create
    pub fn create(type_id: ParticleTypeId, color: i32, power: f32) -> Self {
        Self::new(type_id, color, power)
    }

    // JAVA: net.minecraft.core.particles.SpellParticleOption#create
    pub fn create_from_floats(type_id: ParticleTypeId, r: f32, g: f32, b: f32, power: f32) -> Self {
        Self::create(type_id, argb_color_from_float(1.0_f32, r, g, b), power)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TYPE_A: ParticleTypeId = ParticleTypeId(1);

    #[test]
    fn create_and_getters() {
        let opt = SpellParticleOption::create(TYPE_A, 0x1122_3344_u32 as i32, 1.5_f32);
        assert_eq!(opt.get_type(), TYPE_A);
        assert_eq!(opt.get_power().to_bits(), 1.5_f32.to_bits());
    }

    #[test]
    fn color_channel_extraction() {
        let color = 0xFF80_4020_u32 as i32;
        let opt = SpellParticleOption::create(TYPE_A, color, 1.0_f32);
        assert_eq!(opt.get_red(), 128.0_f32 / 255.0_f32);
        assert_eq!(opt.get_green(), 64.0_f32 / 255.0_f32);
        assert_eq!(opt.get_blue(), 32.0_f32 / 255.0_f32);
    }

    #[test]
    fn create_from_floats() {
        let opt = SpellParticleOption::create_from_floats(TYPE_A, 1.0_f32, 0.5_f32, 0.0_f32, 0.75_f32);
        assert_eq!(opt.get_red(), 1.0_f32);
        // 0.5 * 255.0 = 127.5, floor = 127
        assert_eq!(opt.get_green(), 127.0_f32 / 255.0_f32);
        assert_eq!(opt.get_blue(), 0.0_f32);
        assert_eq!(opt.get_power().to_bits(), 0.75_f32.to_bits());
    }

    #[test]
    fn negative_and_edge_color_values() {
        let all_ones = SpellParticleOption::create(TYPE_A, -1, 1.0_f32);
        assert_eq!(all_ones.get_red(), 1.0_f32);
        assert_eq!(all_ones.get_green(), 1.0_f32);
        assert_eq!(all_ones.get_blue(), 1.0_f32);

        let min = SpellParticleOption::create(TYPE_A, i32::MIN, 0.0_f32);
        // i32::MIN = 0x80000000: red=0x00, green=0x00, blue=0x00
        assert_eq!(min.get_red(), 0.0_f32);
        assert_eq!(min.get_green(), 0.0_f32);
        assert_eq!(min.get_blue(), 0.0_f32);
    }

    #[test]
    fn power_float_bits_and_negative() {
        let neg = SpellParticleOption::create(TYPE_A, 0, -2.5_f32);
        assert_eq!(neg.get_power().to_bits(), (-2.5_f32).to_bits());

        let nan_bits = 0x7FC0_0001_u32;
        let nan = f32::from_bits(nan_bits);
        let opt = SpellParticleOption::create(TYPE_A, 0, nan);
        assert_eq!(opt.get_power().to_bits(), nan_bits);
    }
}
