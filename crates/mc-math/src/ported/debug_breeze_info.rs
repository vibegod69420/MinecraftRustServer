use crate::BlockPos;

#[derive(Debug, Clone)]
pub struct DebugBreezeInfo {
    attack_target: Option<i32>,
    jump_target: Option<BlockPos>,
}

impl DebugBreezeInfo {
    // JAVA: net.minecraft.util.debug.DebugBreezeInfo#<init>
    pub fn new(attack_target: Option<i32>, jump_target: Option<BlockPos>) -> Self {
        Self {
            attack_target,
            jump_target,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugBreezeInfo#attackTarget
    pub fn attack_target(&self) -> Option<i32> {
        self.attack_target
    }

    // JAVA: net.minecraft.util.debug.DebugBreezeInfo#jumpTarget
    pub fn jump_target(&self) -> Option<BlockPos> {
        self.jump_target.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockPos;

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos::of(BlockPos::as_long_coords(x, y, z))
    }

    #[test]
    fn both_present() {
        let p = pos(1, 2, 3);
        let info = DebugBreezeInfo::new(Some(42), Some(p));
        assert_eq!(info.attack_target(), Some(42));
        let got = info.jump_target().unwrap();
        assert_eq!(got.as_long(), p.as_long());
    }

    #[test]
    fn both_empty() {
        let info = DebugBreezeInfo::new(None, None);
        assert_eq!(info.attack_target(), None);
        assert!(info.jump_target().is_none());
    }

    #[test]
    fn negative_attack_target() {
        let info = DebugBreezeInfo::new(Some(-1), None);
        assert_eq!(info.attack_target(), Some(-1));
        let info_min = DebugBreezeInfo::new(Some(i32::MIN), None);
        assert_eq!(info_min.attack_target(), Some(i32::MIN));
    }

    #[test]
    fn attack_target_wrapping_overflow() {
        let v = i32::MAX.wrapping_add(1);
        let info = DebugBreezeInfo::new(Some(v), None);
        assert_eq!(info.attack_target(), Some(i32::MIN));
    }

    #[test]
    fn jump_target_negative_coords() {
        let p = pos(-10, -64, -999);
        let info = DebugBreezeInfo::new(None, Some(p));
        let got = info.jump_target().unwrap();
        assert_eq!(BlockPos::get_x(got.as_long()), -10);
        assert_eq!(BlockPos::get_y(got.as_long()), -64);
        assert_eq!(BlockPos::get_z(got.as_long()), -999);
    }

    #[test]
    fn attack_target_float_bits_pattern() {
        let bits = (-0.0_f32).to_bits() as i32;
        let info = DebugBreezeInfo::new(Some(bits), None);
        assert_eq!(info.attack_target(), Some(bits));
        assert_eq!(
            f32::from_bits(info.attack_target().unwrap() as u32).to_bits(),
            (-0.0_f32).to_bits()
        );
    }
}
