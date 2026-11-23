#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerTickTimeEvent {
    pub average_tick_duration_nanos: i64,
}

impl ServerTickTimeEvent {
    pub const EVENT_NAME: &'static str = "minecraft.ServerTickTime";

    // JAVA: net.minecraft.util.profiling.jfr.event.ServerTickTimeEvent#<init>
    pub fn new(average_tick_duration_ms: f32) -> Self {
        let average_tick_duration_nanos = (1_000_000.0f32 * average_tick_duration_ms) as i64;
        Self {
            average_tick_duration_nanos,
        }
    }
}

pub struct ServerTickTimeEventFields;

impl ServerTickTimeEventFields {
    pub const AVERAGE_TICK_DURATION: &'static str = "averageTickDuration";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_converts_ms_to_nanos() {
        let ev = ServerTickTimeEvent::new(50.0f32);
        assert_eq!(ev.average_tick_duration_nanos, 50_000_000i64);
    }

    #[test]
    fn constants() {
        assert_eq!(ServerTickTimeEvent::EVENT_NAME, "minecraft.ServerTickTime");
        assert_eq!(
            ServerTickTimeEventFields::AVERAGE_TICK_DURATION,
            "averageTickDuration"
        );
    }

    #[test]
    fn zero_and_negative() {
        assert_eq!(
            ServerTickTimeEvent::new(0.0f32).average_tick_duration_nanos,
            0
        );
        assert_eq!(
            ServerTickTimeEvent::new(-1.5f32).average_tick_duration_nanos,
            -1_500_000i64
        );
    }

    #[test]
    fn float_bits_and_order() {
        // Java: (long)(1000000.0F * $$0)
        let ms = 0.1f32;
        let expected = (1_000_000.0f32 * ms) as i64;
        assert_eq!(
            ServerTickTimeEvent::new(ms).average_tick_duration_nanos,
            expected
        );

        let ms2 = 1.000_000_1f32;
        let expected2 = (1_000_000.0f32 * ms2) as i64;
        assert_eq!(
            ServerTickTimeEvent::new(ms2).average_tick_duration_nanos,
            expected2
        );
    }

    #[test]
    fn nan_and_infinity_cast() {
        assert_eq!(
            ServerTickTimeEvent::new(f32::NAN).average_tick_duration_nanos,
            0
        );
        assert_eq!(
            ServerTickTimeEvent::new(f32::INFINITY).average_tick_duration_nanos,
            i64::MAX
        );
        assert_eq!(
            ServerTickTimeEvent::new(f32::NEG_INFINITY).average_tick_duration_nanos,
            i64::MIN
        );
    }
}
