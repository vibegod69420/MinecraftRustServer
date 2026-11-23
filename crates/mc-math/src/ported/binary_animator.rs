#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingType {
    Linear,
}

impl EasingType {
    // JAVA: net.minecraft.util.EasingType#apply
    pub fn apply(self, value: f32) -> f32 {
        match self {
            Self::Linear => value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryAnimator {
    animation_length: i32,
    easing: EasingType,
    ticks: i32,
    ticks_old: i32,
}

impl BinaryAnimator {
    // JAVA: net.minecraft.util.BinaryAnimator#BinaryAnimator
    pub fn with_easing(animation_length: i32, easing: EasingType) -> Self {
        Self {
            animation_length,
            easing,
            ticks: 0,
            ticks_old: 0,
        }
    }

    // JAVA: net.minecraft.util.BinaryAnimator#BinaryAnimator
    pub fn new(animation_length: i32) -> Self {
        Self::with_easing(animation_length, EasingType::Linear)
    }

    // JAVA: net.minecraft.util.BinaryAnimator#tick
    pub fn tick(&mut self, active: bool) {
        self.ticks_old = self.ticks;
        if active {
            if self.ticks < self.animation_length {
                self.ticks = self.ticks.wrapping_add(1);
            }
        } else if self.ticks > 0 {
            self.ticks = self.ticks.wrapping_sub(1);
        }
    }

    // JAVA: net.minecraft.util.BinaryAnimator#getFactor
    pub fn get_factor(&self, partial_tick: f32) -> f32 {
        let t = crate::mth::lerp_f32(
            partial_tick,
            self.ticks_old as f32,
            self.ticks as f32,
        ) / (self.animation_length as f32);
        self.easing.apply(t)
    }
}

#[cfg(test)]
mod tests {
    use super::{BinaryAnimator, EasingType};

    #[test]
    fn tick_up_clamps_at_animation_length() {
        let mut anim = BinaryAnimator::new(3);
        for _ in 0..10 {
            anim.tick(true);
        }
        assert_eq!(anim.ticks, 3);
        assert_eq!(anim.ticks_old, 3);
    }

    #[test]
    fn tick_down_clamps_at_zero() {
        let mut anim = BinaryAnimator::new(3);
        anim.tick(true);
        anim.tick(true);
        anim.tick(false);
        anim.tick(false);
        anim.tick(false);
        anim.tick(false);
        assert_eq!(anim.ticks, 0);
    }

    #[test]
    fn get_factor_interpolates_with_partial_tick() {
        let mut anim = BinaryAnimator::new(4);
        anim.tick(true); // ticks_old=0, ticks=1
        let f0 = anim.get_factor(0.0);
        let f05 = anim.get_factor(0.5);
        let f1 = anim.get_factor(1.0);
        assert_eq!(f0.to_bits(), 0.0f32.to_bits());
        assert_eq!(f05.to_bits(), (0.125f32).to_bits()); // lerp(0.5,0,1)/4 = 0.125
        assert_eq!(f1.to_bits(), (0.25f32).to_bits());
    }

    #[test]
    fn linear_easing_is_identity() {
        assert_eq!(EasingType::Linear.apply(0.0).to_bits(), 0.0f32.to_bits());
        assert_eq!(EasingType::Linear.apply(0.5).to_bits(), 0.5f32.to_bits());
        assert_eq!(EasingType::Linear.apply(1.0).to_bits(), 1.0f32.to_bits());
    }

    #[test]
    fn with_easing_constructor_preserves_easing() {
        let anim = BinaryAnimator::with_easing(5, EasingType::Linear);
        let mut anim2 = anim.clone();
        anim2.tick(true);
        assert_eq!(anim2.get_factor(1.0).to_bits(), (1.0f32 / 5.0f32).to_bits());
    }

    #[test]
    fn negative_animation_length_does_not_increment() {
        let mut anim = BinaryAnimator::new(-1);
        anim.tick(true);
        assert_eq!(anim.ticks, 0);
        // get_factor: 0 / -1.0 = -0.0 in IEEE float; behavior is Java-faithful float division
        let f = anim.get_factor(1.0);
        assert!(f == 0.0);
    }
}
