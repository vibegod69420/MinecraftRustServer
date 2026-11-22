use std::sync::atomic::{AtomicBool, Ordering};

/// Minimal self-contained stub for `net.minecraft.SharedConstants#DEBUG_OPEN_INCOMPATIBLE_WORLDS`.
pub static DEBUG_OPEN_INCOMPATIBLE_WORLDS: AtomicBool = AtomicBool::new(false);

pub const MAIN_SERIES: &str = "main";

/// Port of `net.minecraft.world.level.storage.DataVersion`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataVersion {
    pub version: i32,
    pub series: String,
}

impl DataVersion {
    // JAVA: net.minecraft.world.level.storage.DataVersion#<init>
    pub fn new(version: i32, series: String) -> Self {
        Self { version, series }
    }

    // JAVA: net.minecraft.world.level.storage.DataVersion#version
    pub fn version(&self) -> i32 {
        self.version
    }

    // JAVA: net.minecraft.world.level.storage.DataVersion#series
    pub fn series(&self) -> &str {
        &self.series
    }

    // JAVA: net.minecraft.world.level.storage.DataVersion#isSideSeries
    pub fn is_side_series(&self) -> bool {
        self.series != "main"
    }

    // JAVA: net.minecraft.world.level.storage.DataVersion#isCompatible
    pub fn is_compatible(&self, other: &DataVersion) -> bool {
        if DEBUG_OPEN_INCOMPATIBLE_WORLDS.load(Ordering::Relaxed) {
            true
        } else {
            self.series() == other.series()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_debug_flag<F: FnOnce()>(value: bool, f: F) {
        let _guard = TEST_LOCK.lock().unwrap();
        let prev = DEBUG_OPEN_INCOMPATIBLE_WORLDS.load(Ordering::Relaxed);
        DEBUG_OPEN_INCOMPATIBLE_WORLDS.store(value, Ordering::Relaxed);
        f();
        DEBUG_OPEN_INCOMPATIBLE_WORLDS.store(prev, Ordering::Relaxed);
    }

    #[test]
    fn main_series_not_side_series() {
        let dv = DataVersion::new(1, MAIN_SERIES.to_owned());
        assert!(!dv.is_side_series());
        assert_eq!(dv.version(), 1);
        assert_eq!(dv.series(), "main");
    }

    #[test]
    fn side_series_detected_with_negative_version() {
        let dv = DataVersion::new(-1, "snapshot".to_owned());
        assert!(dv.is_side_series());
        assert_eq!(dv.version(), -1);
    }

    #[test]
    fn compatible_same_series_when_debug_off() {
        with_debug_flag(false, || {
            let a = DataVersion::new(100, "main".to_owned());
            let b = DataVersion::new(200, "main".to_owned());
            assert!(a.is_compatible(&b));
        });
    }

    #[test]
    fn incompatible_different_series_when_debug_off() {
        with_debug_flag(false, || {
            let a = DataVersion::new(1, "main".to_owned());
            let b = DataVersion::new(1, "side".to_owned());
            assert!(!a.is_compatible(&b));
        });
    }

    #[test]
    fn debug_flag_forces_compatibility() {
        with_debug_flag(true, || {
            let a = DataVersion::new(i32::MIN, "main".to_owned());
            let b = DataVersion::new(i32::MAX, "other".to_owned());
            assert!(a.is_compatible(&b));
        });
    }

    #[test]
    fn version_extremes_overflow_and_float_bits_sanity() {
        let dv = DataVersion::new(i32::MAX, MAIN_SERIES.to_owned());
        assert_eq!(dv.version(), i32::MAX);
        assert_eq!(dv.version().wrapping_add(1), i32::MIN);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);

        let f_bits = 1.5f32.to_bits();
        let f_back = f32::from_bits(f_bits);
        assert_eq!(f_back.to_bits(), f_bits);
        assert_ne!((-0.0f64).to_bits(), 0.0f64.to_bits());
    }
}
