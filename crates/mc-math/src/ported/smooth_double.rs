/// Java-compatible `Math.signum` for `f64`.
/// Java returns +0.0 for +0.0 and -0.0 for -0.0; Rust's `f64::signum` returns +/-1.0 for zeros.
fn java_signum_f64(x: f64) -> f64 {
    if x == 0.0 {
        x
    } else if x.is_nan() {
        f64::NAN
    } else if x > 0.0 {
        1.0
    } else {
        -1.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct SmoothDouble {
    target_value: f64,
    remaining_value: f64,
    last_amount: f64,
}

impl SmoothDouble {
    // JAVA: net.minecraft.util.SmoothDouble#getNewDeltaValue
    pub fn get_new_delta_value(&mut self, input: f64, speed: f64) -> f64 {
        self.target_value += input;
        let mut delta = self.target_value - self.remaining_value;
        let smoothed = crate::mth::lerp_f64(0.5, self.last_amount, delta);
        let sign = java_signum_f64(delta);
        if sign * delta > sign * self.last_amount {
            delta = smoothed;
        }

        self.last_amount = smoothed;
        self.remaining_value += delta * speed;
        delta * speed
    }

    // JAVA: net.minecraft.util.SmoothDouble#reset
    pub fn reset(&mut self) {
        self.target_value = 0.0;
        self.remaining_value = 0.0;
        self.last_amount = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::SmoothDouble;

    #[test]
    fn reset_clears_state() {
        let mut s = SmoothDouble::default();
        let _ = s.get_new_delta_value(10.0, 0.5);
        s.reset();
        let out = s.get_new_delta_value(0.0, 1.0);
        assert_eq!(out, 0.0);
        assert_eq!(s.target_value, 0.0);
        assert_eq!(s.remaining_value, 0.0);
        assert_eq!(s.last_amount, 0.0);
    }

    #[test]
    fn basic_smoothing_step() {
        let mut s = SmoothDouble::default();
        let out = s.get_new_delta_value(10.0, 0.5);
        // target=10, delta=10, smoothed=lerp(0.5,0,10)=5, sign=1, 10>0 => delta=5
        // last_amount=5, remaining += 2.5, return 2.5
        assert_eq!(out, 2.5);
        assert_eq!(s.last_amount, 5.0);
        assert_eq!(s.remaining_value, 2.5);
    }

    #[test]
    fn negative_input() {
        let mut s = SmoothDouble::default();
        let out = s.get_new_delta_value(-8.0, 0.25);
        // target=-8, delta=-8, smoothed=lerp(0.5,0,-8)=-4, sign=-1
        // sign*delta = 8, sign*last_amount = 0, 8>0 => delta=-4
        // remaining += -1.0, return -1.0
        assert_eq!(out, -1.0);
        assert_eq!(s.last_amount, -4.0);
    }

    #[test]
    fn zero_delta_signum_preserves_java_behavior() {
        let mut s = SmoothDouble::default();
        let out = s.get_new_delta_value(0.0, 1.0);
        // delta=0, smoothed=0, signum(0.0)=0.0, 0*0 > 0*0 is false
        // last_amount=0, remaining += 0, return 0
        assert_eq!(out, 0.0);
        assert_eq!(s.last_amount.to_bits(), 0.0f64.to_bits());
    }

    #[test]
    fn repeated_updates_converge_toward_target() {
        let mut s = SmoothDouble::default();
        let mut last = 0.0;
        for _ in 0..20 {
            last = s.get_new_delta_value(1.0, 1.0);
        }
        // With constant input 1.0 and speed 1.0, output should settle near 1.0
        assert!((last - 1.0).abs() < 1e-6, "last={last}");
    }

    #[test]
    fn float_bits_stable_for_known_case() {
        let mut s = SmoothDouble::default();
        let out = s.get_new_delta_value(1.0, 0.5);
        // delta=1, smoothed=0.5, sign=1, 1>0 => delta=0.5, return 0.25
        assert_eq!(out.to_bits(), 0.25f64.to_bits());
    }
}
