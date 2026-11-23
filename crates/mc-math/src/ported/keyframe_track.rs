// === FILE: keyframe_track.rs ===

/// Minimal self-contained result type mirroring Mojang's `DataResult` usage
/// in `KeyframeTrack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataResult<T> {
    Success(T),
    Error(String),
}

impl<T> DataResult<T> {
    pub fn success(value: T) -> Self {
        Self::Success(value)
    }

    pub fn error(message: impl FnOnce() -> String) -> Self {
        Self::Error(message())
    }

    pub fn get_or_throw(self) -> T {
        match self {
            Self::Success(value) => value,
            Self::Error(message) => panic!("{}", message),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Error(msg) => Some(msg),
            Self::Success(_) => None,
        }
    }
}

/// Minimal easing type used by `KeyframeTrack`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EasingType {
    #[default]
    Linear,
}

/// Minimal keyframe record: `(ticks, value)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe<T> {
    ticks: i32,
    value: T,
}

impl<T> Keyframe<T> {
    pub fn new(ticks: i32, value: T) -> Self {
        Self { ticks, value }
    }

    pub fn ticks(&self) -> i32 {
        self.ticks
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

/// Minimal lerp function abstraction for `KeyframeTrackSampler`.
pub trait LerpFunction<T> {
    fn lerp(&self, delta: f64, start: &T, end: &T) -> T;
}

impl<T, F> LerpFunction<T> for F
where
    F: Fn(f64, &T, &T) -> T,
{
    fn lerp(&self, delta: f64, start: &T, end: &T) -> T {
        (self)(delta, start, end)
    }
}

/// Port of `net.minecraft.util.KeyframeTrack`.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeTrack<T> {
    keyframes: Vec<Keyframe<T>>,
    easing_type: EasingType,
}

impl<T> KeyframeTrack<T> {
    // JAVA: net.minecraft.util.KeyframeTrack#new
    pub fn new(keyframes: Vec<Keyframe<T>>, easing_type: EasingType) -> Self {
        if keyframes.is_empty() {
            panic!("Track has no keyframes");
        }
        Self {
            keyframes,
            easing_type,
        }
    }

    // JAVA: net.minecraft.util.KeyframeTrack#keyframes
    pub fn keyframes(&self) -> &[Keyframe<T>] {
        &self.keyframes
    }

    // JAVA: net.minecraft.util.KeyframeTrack#easingType
    pub fn easing_type(&self) -> EasingType {
        self.easing_type
    }

    // JAVA: net.minecraft.util.KeyframeTrack#validateKeyframes
    pub fn validate_keyframes(keyframes: Vec<Keyframe<T>>) -> DataResult<Vec<Keyframe<T>>> {
        if keyframes.is_empty() {
            return DataResult::error(|| "Keyframes must not be empty".to_string());
        }

        if !is_in_order_by_ticks(&keyframes) {
            return DataResult::error(|| "Keyframes must be ordered by ticks field".to_string());
        }

        if keyframes.len() > 1 {
            let mut same_tick_run: i32 = 0;
            let mut prev_ticks: i32 = keyframes[keyframes.len() - 1].ticks();

            for kf in &keyframes {
                let ticks = kf.ticks();
                if ticks == prev_ticks {
                    same_tick_run = same_tick_run.wrapping_add(1);
                    if same_tick_run > 2 {
                        let msg_ticks = ticks;
                        return DataResult::error(|| {
                            format!("More than 2 keyframes on same tick: {msg_ticks}")
                        });
                    }
                } else {
                    same_tick_run = 0;
                }
                prev_ticks = ticks;
            }
        }

        DataResult::success(keyframes)
    }

    // JAVA: net.minecraft.util.KeyframeTrack#validatePeriod
    pub fn validate_period(track: KeyframeTrack<T>, period: i32) -> DataResult<KeyframeTrack<T>> {
        for kf in track.keyframes() {
            let ticks = kf.ticks();
            if ticks < 0 || ticks > period {
                let msg_ticks = ticks;
                return DataResult::error(|| {
                    format!("Keyframe at tick {msg_ticks} must be in range [0; {period}]")
                });
            }
        }
        DataResult::success(track)
    }

    // JAVA: net.minecraft.util.KeyframeTrack#bakeSampler
    pub fn bake_sampler<F: LerpFunction<T>>(
        self,
        period: Option<i32>,
        lerp: F,
    ) -> KeyframeTrackSampler<T, F> {
        KeyframeTrackSampler::new(self, period, lerp)
    }

    pub fn builder() -> Builder<T> {
        Builder::new()
    }
}

fn is_in_order_by_ticks<T>(keyframes: &[Keyframe<T>]) -> bool {
    for i in 1..keyframes.len() {
        if keyframes[i - 1].ticks() > keyframes[i].ticks() {
            return false;
        }
    }
    true
}

/// Minimal sampler returned by `KeyframeTrack::bake_sampler`.
#[derive(Debug, Clone)]
pub struct KeyframeTrackSampler<T, F> {
    track: KeyframeTrack<T>,
    period: Option<i32>,
    lerp_fn: F,
}

impl<T, F> KeyframeTrackSampler<T, F> {
    pub fn new(track: KeyframeTrack<T>, period: Option<i32>, lerp_fn: F) -> Self {
        Self {
            track,
            period,
            lerp_fn,
        }
    }

    pub fn track(&self) -> &KeyframeTrack<T> {
        &self.track
    }

    pub fn period(&self) -> Option<i32> {
        self.period
    }

    pub fn lerp_fn(&self) -> &F {
        &self.lerp_fn
    }
}

/// Port of `net.minecraft.util.KeyframeTrack$Builder`.
#[derive(Debug)]
pub struct Builder<T> {
    keyframes: Vec<Keyframe<T>>,
    easing: EasingType,
}

impl<T> Builder<T> {
    // JAVA: net.minecraft.util.KeyframeTrack$Builder#new
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
            easing: EasingType::Linear,
        }
    }

    // JAVA: net.minecraft.util.KeyframeTrack$Builder#addKeyframe
    pub fn add_keyframe(mut self, ticks: i32, value: T) -> Self {
        self.keyframes.push(Keyframe::new(ticks, value));
        self
    }

    // JAVA: net.minecraft.util.KeyframeTrack$Builder#setEasing
    pub fn set_easing(mut self, easing: EasingType) -> Self {
        self.easing = easing;
        self
    }

    // JAVA: net.minecraft.util.KeyframeTrack$Builder#build
    pub fn build(self) -> KeyframeTrack<T> {
        let keyframes = KeyframeTrack::validate_keyframes(self.keyframes).get_or_throw();
        KeyframeTrack {
            keyframes,
            easing_type: self.easing,
        }
    }
}

impl<T> Default for Builder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_keyframes_rejects_empty() {
        let result = KeyframeTrack::<f32>::validate_keyframes(Vec::new());
        assert!(result.is_error());
        assert_eq!(result.error_message(), Some("Keyframes must not be empty"));
    }

    #[test]
    fn validate_keyframes_rejects_out_of_order_and_accepts_equal_ticks() {
        let out_of_order = vec![Keyframe::new(10, 1.0f32), Keyframe::new(5, 2.0f32)];
        let result = KeyframeTrack::validate_keyframes(out_of_order);
        assert!(result.is_error());
        assert_eq!(
            result.error_message(),
            Some("Keyframes must be ordered by ticks field")
        );

        let equal_ok = vec![Keyframe::new(5, 1.0f32), Keyframe::new(5, 2.0f32)];
        assert!(KeyframeTrack::validate_keyframes(equal_ok).is_success());
    }

    #[test]
    fn validate_keyframes_same_tick_limit_matches_java_logic() {
        // All same tick: first compares against last, so count reaches 3 -> error.
        let all_same = vec![
            Keyframe::new(5, 0i32),
            Keyframe::new(5, 1i32),
            Keyframe::new(5, 2i32),
        ];
        let err = KeyframeTrack::validate_keyframes(all_same);
        assert!(err.is_error());
        assert_eq!(
            err.error_message(),
            Some("More than 2 keyframes on same tick: 5")
        );

        // Contiguous run after a different tick: count resets, reaches 2 -> allowed.
        let contiguous_three = vec![
            Keyframe::new(0, 0i32),
            Keyframe::new(5, 1i32),
            Keyframe::new(5, 2i32),
            Keyframe::new(5, 3i32),
        ];
        assert!(KeyframeTrack::validate_keyframes(contiguous_three).is_success());

        // Four contiguous same tick: count reaches 3 -> error.
        let contiguous_four = vec![
            Keyframe::new(0, 0i32),
            Keyframe::new(5, 1i32),
            Keyframe::new(5, 2i32),
            Keyframe::new(5, 3i32),
            Keyframe::new(5, 4i32),
        ];
        assert!(KeyframeTrack::validate_keyframes(contiguous_four).is_error());
    }

    #[test]
    fn validate_period_checks_negative_and_upper_bound() {
        let track = KeyframeTrack::new(
            vec![Keyframe::new(-1, 10i32), Keyframe::new(5, 20i32)],
            EasingType::Linear,
        );
        let err = KeyframeTrack::validate_period(track, 10);
        assert!(err.is_error());
        assert_eq!(
            err.error_message(),
            Some("Keyframe at tick -1 must be in range [0; 10]")
        );

        let track2 = KeyframeTrack::new(vec![Keyframe::new(11, 1i32)], EasingType::Linear);
        assert!(KeyframeTrack::validate_period(track2, 10).is_error());

        let track3 = KeyframeTrack::new(
            vec![Keyframe::new(0, 1i32), Keyframe::new(10, 2i32)],
            EasingType::Linear,
        );
        assert!(KeyframeTrack::validate_period(track3, 10).is_success());
    }

    #[test]
    fn builder_and_float_bits_preserved() {
        let nan_bits = f64::NAN.to_bits();
        let inf_bits = f64::INFINITY.to_bits();
        let neg_inf_bits = f64::NEG_INFINITY.to_bits();

        let track = Builder::<f64>::new()
            .set_easing(EasingType::Linear)
            .add_keyframe(i32::MIN, f64::from_bits(nan_bits))
            .add_keyframe(0, f64::from_bits(inf_bits))
            .add_keyframe(i32::MAX, f64::from_bits(neg_inf_bits))
            .build();

        assert_eq!(track.easing_type(), EasingType::Linear);
        assert_eq!(track.keyframes().len(), 3);
        assert_eq!(track.keyframes()[0].ticks(), i32::MIN);
        assert_eq!(track.keyframes()[0].value().to_bits(), nan_bits);
        assert_eq!(track.keyframes()[1].value().to_bits(), inf_bits);
        assert_eq!(track.keyframes()[2].ticks(), i32::MAX);
        assert_eq!(track.keyframes()[2].value().to_bits(), neg_inf_bits);
    }

    #[test]
    fn bake_sampler_and_constructor_panics() {
        let track = KeyframeTrack::new(vec![Keyframe::new(0, 1.0f32)], EasingType::Linear);
        let sampler = track.bake_sampler(Some(20), |delta: f64, a: &f32, b: &f32| -> f32 {
            (*a as f64 + ((*b as f64 - *a as f64) * delta)) as f32
        });
        assert_eq!(sampler.period(), Some(20));
        assert_eq!(sampler.track().keyframes().len(), 1);

        let empty = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = KeyframeTrack::<i32>::new(Vec::new(), EasingType::Linear);
        }));
        assert!(empty.is_err());
    }
}
