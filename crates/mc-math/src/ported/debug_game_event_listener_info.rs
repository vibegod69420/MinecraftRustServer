#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebugGameEventListenerInfo {
    listener_radius: i32,
}

impl DebugGameEventListenerInfo {
    // JAVA: net.minecraft.util.debug.DebugGameEventListenerInfo#<init>
    pub fn new(listener_radius: i32) -> Self {
        Self { listener_radius }
    }

    // JAVA: net.minecraft.util.debug.DebugGameEventListenerInfo#listenerRadius
    pub fn listener_radius(&self) -> i32 {
        self.listener_radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listener_radius_basic() {
        let info = DebugGameEventListenerInfo::new(16);
        assert_eq!(info.listener_radius(), 16);
    }

    #[test]
    fn listener_radius_negative() {
        let info = DebugGameEventListenerInfo::new(-1);
        assert_eq!(info.listener_radius(), -1);
    }

    #[test]
    fn listener_radius_extremes() {
        let min_info = DebugGameEventListenerInfo::new(i32::MIN);
        let max_info = DebugGameEventListenerInfo::new(i32::MAX);
        assert_eq!(min_info.listener_radius(), i32::MIN);
        assert_eq!(max_info.listener_radius(), i32::MAX);
    }

    #[test]
    fn listener_radius_wrapping_overflow() {
        let wrapped = i32::MAX.wrapping_add(1);
        let info = DebugGameEventListenerInfo::new(wrapped);
        assert_eq!(info.listener_radius(), i32::MIN);
    }

    #[test]
    fn listener_radius_float_bits_pattern() {
        let bits = (-0.0_f32).to_bits() as i32;
        let info = DebugGameEventListenerInfo::new(bits);
        assert_eq!(info.listener_radius(), bits);
        assert_eq!(
            f32::from_bits(info.listener_radius() as u32).to_bits(),
            (-0.0_f32).to_bits()
        );
    }

    #[test]
    fn copy_and_equality() {
        let a = DebugGameEventListenerInfo::new(5);
        let b = a;
        assert_eq!(a, b);
        assert_eq!(a.listener_radius(), b.listener_radius());
    }
}
