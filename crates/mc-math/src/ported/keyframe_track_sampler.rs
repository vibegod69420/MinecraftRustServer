#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EasingType {
    Constant,
    Linear,
}

impl EasingType {
    // JAVA: net.minecraft.util.EasingType#apply
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Self::Constant => 0.0_f32,
            Self::Linear => t,
        }
    }
}

pub trait LerpFunction<T> {
    // JAVA: net.minecraft.world.attribute.LerpFunction#apply
    fn apply(&self, alpha: f32, from: &T, to: &T) -> T;
}

impl<T, F> LerpFunction<T> for F
where
    F: Fn(f32, &T, &T) -> T,
{
    // JAVA: net.minecraft.world.attribute.LerpFunction#apply
    fn apply(&self, alpha: f32, from: &T, to: &T) -> T {
        self(alpha, from, to)
    }
}

#[derive(Clone, Debug)]
pub struct Keyframe<T> {
    ticks: i32,
    value: T,
}

impl<T> Keyframe<T> {
    // JAVA: net.minecraft.util.Keyframe#<init>
    pub fn new(ticks: i32, value: T) -> Self {
        Self { ticks, value }
    }

    // JAVA: net.minecraft.util.Keyframe#ticks
    pub fn ticks(&self) -> i32 {
        self.ticks
    }

    // JAVA: net.minecraft.util.Keyframe#value
    pub fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub struct KeyframeTrack<T> {
    easing_type: EasingType,
    keyframes: Vec<Keyframe<T>>,
}

impl<T> KeyframeTrack<T> {
    // JAVA: net.minecraft.util.KeyframeTrack#<init>
    pub fn new(easing_type: EasingType, keyframes: Vec<Keyframe<T>>) -> Self {
        Self {
            easing_type,
            keyframes,
        }
    }

    // JAVA: net.minecraft.util.KeyframeTrack#easingType
    pub fn easing_type(&self) -> EasingType {
        self.easing_type
    }

    // JAVA: net.minecraft.util.KeyframeTrack#keyframes
    pub fn keyframes(&self) -> &[Keyframe<T>] {
        &self.keyframes
    }
}

#[derive(Clone, Debug)]
pub struct Segment<T> {
    pub easing: EasingType,
    pub from_value: T,
    pub from_ticks: i32,
    pub to_value: T,
    pub to_ticks: i32,
}

impl<T: Clone> Segment<T> {
    // JAVA: net.minecraft.util.KeyframeTrackSampler$Segment#<init>
    pub fn from_keyframes(
        track: &KeyframeTrack<T>,
        from: &Keyframe<T>,
        from_ticks: i32,
        to: &Keyframe<T>,
        to_ticks: i32,
    ) -> Self {
        Self {
            easing: track.easing_type(),
            from_value: from.value().clone(),
            from_ticks,
            to_value: to.value().clone(),
            to_ticks,
        }
    }
}

pub struct KeyframeTrackSampler<T, L> {
    period_ticks: Option<i32>,
    lerp: L,
    segments: Vec<Segment<T>>,
}

impl<T: Clone, L: LerpFunction<T>> KeyframeTrackSampler<T, L> {
    // JAVA: net.minecraft.util.KeyframeTrackSampler#<init>
    pub fn new(track: &KeyframeTrack<T>, period_ticks: Option<i32>, lerp: L) -> Self {
        let segments = Self::bake_segments(track, period_ticks);
        Self {
            period_ticks,
            lerp,
            segments,
        }
    }

    // JAVA: net.minecraft.util.KeyframeTrackSampler#bakeSegments
    fn bake_segments(track: &KeyframeTrack<T>, period_ticks: Option<i32>) -> Vec<Segment<T>> {
        let keyframes = track.keyframes();
        if keyframes.is_empty() {
            return Vec::new();
        }
        if keyframes.len() == 1 {
            let value = keyframes[0].value().clone();
            return vec![Segment {
                easing: EasingType::Constant,
                from_value: value.clone(),
                from_ticks: 0,
                to_value: value,
                to_ticks: 0,
            }];
        }

        let mut segments: Vec<Segment<T>> = Vec::new();
        if let Some(period) = period_ticks {
            let first = &keyframes[0];
            let last = &keyframes[keyframes.len() - 1];
            segments.push(Segment::from_keyframes(
                track,
                last,
                last.ticks().wrapping_sub(period),
                first,
                first.ticks(),
            ));
            Self::add_segments_from_keyframes(track, keyframes, &mut segments);
            segments.push(Segment::from_keyframes(
                track,
                last,
                last.ticks(),
                first,
                first.ticks().wrapping_add(period),
            ));
        } else {
            Self::add_segments_from_keyframes(track, keyframes, &mut segments);
        }

        segments
    }

    // JAVA: net.minecraft.util.KeyframeTrackSampler#addSegmentsFromKeyframes
    fn add_segments_from_keyframes(
        track: &KeyframeTrack<T>,
        keyframes: &[Keyframe<T>],
        segments: &mut Vec<Segment<T>>,
    ) {
        let mut i: usize = 0;
        while i + 1 < keyframes.len() {
            let from = &keyframes[i];
            let to = &keyframes[i + 1];
            segments.push(Segment::from_keyframes(
                track,
                from,
                from.ticks(),
                to,
                to.ticks(),
            ));
            i += 1;
        }
    }

    // JAVA: net.minecraft.util.KeyframeTrackSampler#sample
    pub fn sample(&self, ticks: i64) -> T {
        let looped = self.loop_ticks(ticks);
        let segment = self.get_segment_at(looped);
        if looped <= segment.from_ticks as i64 {
            return segment.from_value.clone();
        }

        if looped >= segment.to_ticks as i64 {
            return segment.to_value.clone();
        }

        let delta = looped.wrapping_sub(segment.from_ticks as i64) as f32;
        let duration = segment.to_ticks.wrapping_sub(segment.from_ticks) as f32;
        let t = delta / duration;
        let alpha = segment.easing.apply(t);
        self.lerp
            .apply(alpha, &segment.from_value, &segment.to_value)
    }

    // JAVA: net.minecraft.util.KeyframeTrackSampler#getSegmentAt
    fn get_segment_at(&self, ticks: i64) -> &Segment<T> {
        for segment in &self.segments {
            if ticks < segment.to_ticks as i64 {
                return segment;
            }
        }
        self.segments
            .last()
            .expect("keyframe track sampler has no segments")
    }

    // JAVA: net.minecraft.util.KeyframeTrackSampler#loopTicks
    fn loop_ticks(&self, ticks: i64) -> i64 {
        match self.period_ticks {
            Some(period) => {
                let y64 = period as i64;
                let mut r = (ticks % y64) as i32;
                if (r ^ period) < 0 && r != 0 {
                    r = r.wrapping_add(period);
                }
                r as i64
            }
            None => ticks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lerp_f32(alpha: f32, from: &f32, to: &f32) -> f32 {
        *from + alpha * (*to - *from)
    }

    #[test]
    fn single_keyframe_returns_constant_value() {
        let track = KeyframeTrack::new(EasingType::Linear, vec![Keyframe::new(5, 42.0f32)]);
        let sampler = KeyframeTrackSampler::new(&track, Some(10), |_: f32, a: &f32, _: &f32| *a);
        assert_eq!(sampler.sample(i64::MIN).to_bits(), 42.0f32.to_bits());
        assert_eq!(sampler.sample(0).to_bits(), 42.0f32.to_bits());
        assert_eq!(sampler.sample(i64::MAX).to_bits(), 42.0f32.to_bits());
    }

    #[test]
    fn linear_interpolation_and_boundaries() {
        let track = KeyframeTrack::new(
            EasingType::Linear,
            vec![Keyframe::new(0, 0.0f32), Keyframe::new(10, 10.0f32)],
        );
        let sampler = KeyframeTrackSampler::new(&track, None, lerp_f32);
        assert_eq!(sampler.sample(-1).to_bits(), 0.0f32.to_bits());
        assert_eq!(sampler.sample(0).to_bits(), 0.0f32.to_bits());
        assert_eq!(sampler.sample(5).to_bits(), 5.0f32.to_bits());
        assert_eq!(sampler.sample(10).to_bits(), 10.0f32.to_bits());
        assert_eq!(sampler.sample(11).to_bits(), 10.0f32.to_bits());
    }

    #[test]
    fn periodic_looping_and_negative_ticks() {
        let track = KeyframeTrack::new(
            EasingType::Linear,
            vec![Keyframe::new(0, 0.0f32), Keyframe::new(10, 20.0f32)],
        );
        let sampler = KeyframeTrackSampler::new(&track, Some(10), lerp_f32);
        assert_eq!(sampler.sample(0).to_bits(), 0.0f32.to_bits());
        assert_eq!(sampler.sample(10).to_bits(), 0.0f32.to_bits());
        assert_eq!(sampler.sample(15).to_bits(), 10.0f32.to_bits());

        let alpha = 9.0f32 / 10.0f32;
        let expected = lerp_f32(alpha, &0.0f32, &20.0f32);
        assert_eq!(sampler.sample(-1).to_bits(), expected.to_bits());
        assert_eq!(sampler.sample(-10).to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn constant_easing_holds_start_value() {
        let track = KeyframeTrack::new(
            EasingType::Constant,
            vec![Keyframe::new(0, 1.5f32), Keyframe::new(10, 9.5f32)],
        );
        let sampler = KeyframeTrackSampler::new(&track, None, lerp_f32);
        assert_eq!(sampler.sample(0).to_bits(), 1.5f32.to_bits());
        assert_eq!(sampler.sample(3).to_bits(), 1.5f32.to_bits());
        assert_eq!(sampler.sample(9).to_bits(), 1.5f32.to_bits());
        assert_eq!(sampler.sample(10).to_bits(), 9.5f32.to_bits());
    }

    #[test]
    fn float_bits_precision() {
        let track = KeyframeTrack::new(
            EasingType::Linear,
            vec![Keyframe::new(0, 0.0f32), Keyframe::new(8, 1.0f32)],
        );
        let sampler = KeyframeTrackSampler::new(&track, None, lerp_f32);
        let v = sampler.sample(1);
        let expected_t = 1.0f32 / 8.0f32;
        let expected = lerp_f32(expected_t, &0.0f32, &1.0f32);
        assert_eq!(v.to_bits(), expected.to_bits());
        assert_eq!(v.to_bits(), 0.125f32.to_bits());
    }

    #[test]
    fn wrapping_bake_and_extreme_ticks() {
        let track = KeyframeTrack::new(
            EasingType::Linear,
            vec![
                Keyframe::new(i32::MAX - 2, 0.0f32),
                Keyframe::new(i32::MAX, 2.0f32),
            ],
        );
        let sampler = KeyframeTrackSampler::new(&track, Some(5), lerp_f32);
        let v = sampler.sample(i32::MAX as i64);
        assert!(v.is_finite());

        let track2 = KeyframeTrack::new(
            EasingType::Linear,
            vec![Keyframe::new(0, 1.0f32), Keyframe::new(10, 3.0f32)],
        );
        let sampler2 = KeyframeTrackSampler::new(&track2, Some(10), lerp_f32);
        assert!(sampler2.sample(i64::MIN).is_finite());
        assert!(sampler2.sample(i64::MAX).is_finite());
    }
}
