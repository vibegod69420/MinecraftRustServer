/// Minimal view over a JFR `RecordedEvent` for `CpuLoadStat` extraction.
pub trait CpuLoadEvent {
    fn get_float(&self, name: &str) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpuLoadStat {
    pub jvm: f64,
    pub user_jvm: f64,
    pub system: f64,
}

impl CpuLoadStat {
    // JAVA: net.minecraft.util.profiling.jfr.stats.CpuLoadStat#from
    pub fn from<E: CpuLoadEvent + ?Sized>(event: &E) -> Self {
        Self {
            jvm: f64::from(event.get_float("jvmSystem")),
            user_jvm: f64::from(event.get_float("jvmUser")),
            system: f64::from(event.get_float("machineTotal")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEvent {
        jvm_system: f32,
        jvm_user: f32,
        machine_total: f32,
    }

    impl CpuLoadEvent for MockEvent {
        fn get_float(&self, name: &str) -> f32 {
            match name {
                "jvmSystem" => self.jvm_system,
                "jvmUser" => self.jvm_user,
                "machineTotal" => self.machine_total,
                _ => panic!("unexpected field: {name}"),
            }
        }
    }

    #[test]
    fn from_basic_values() {
        let stat = CpuLoadStat::from(&MockEvent {
            jvm_system: 0.5,
            jvm_user: 0.25,
            machine_total: 0.75,
        });
        assert_eq!(stat.jvm, 0.5_f64);
        assert_eq!(stat.user_jvm, 0.25_f64);
        assert_eq!(stat.system, 0.75_f64);
    }

    #[test]
    fn from_negative_and_zero() {
        let stat = CpuLoadStat::from(&MockEvent {
            jvm_system: -1.0,
            jvm_user: 0.0,
            machine_total: -0.0,
        });
        assert_eq!(stat.jvm.to_bits(), (-1.0_f64).to_bits());
        assert_eq!(stat.user_jvm.to_bits(), 0.0_f64.to_bits());
        assert!(stat.system.is_sign_negative());
        assert_eq!(stat.system.to_bits(), (-0.0_f64).to_bits());
    }

    #[test]
    fn from_special_float_values() {
        let stat = CpuLoadStat::from(&MockEvent {
            jvm_system: f32::INFINITY,
            jvm_user: f32::NEG_INFINITY,
            machine_total: f32::NAN,
        });
        assert!(stat.jvm.is_infinite() && stat.jvm.is_sign_positive());
        assert!(stat.user_jvm.is_infinite() && stat.user_jvm.is_sign_negative());
        assert!(stat.system.is_nan());
    }

    #[test]
    fn from_preserves_f32_widening_bits() {
        let v = f32::from_bits(0x3F80_0001);
        let stat = CpuLoadStat::from(&MockEvent {
            jvm_system: v,
            jvm_user: f32::MIN_POSITIVE,
            machine_total: f32::MAX,
        });
        assert_eq!(stat.jvm, f64::from(v));
        assert_eq!(stat.jvm.to_bits(), f64::from(v).to_bits());
        assert_eq!(stat.user_jvm.to_bits(), f64::from(f32::MIN_POSITIVE).to_bits());
        assert_eq!(stat.system.to_bits(), f64::from(f32::MAX).to_bits());
    }
}
