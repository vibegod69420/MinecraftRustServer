use std::f64::consts::PI;
use std::fmt::Write;

// JAVA: net.minecraft.world.level.levelgen.synth.NoiseUtils#biasTowardsExtreme(double,double)
pub fn bias_towards_extreme(value: f64, bias: f64) -> f64 {
    value + (PI * value).sin() * bias / PI
}

// JAVA: net.minecraft.world.level.levelgen.synth.NoiseUtils#parityNoiseOctaveConfigString(StringBuilder,double,double,double,byte[])
pub fn parity_noise_octave_config_string_bytes(
    builder: &mut String,
    xo: f64,
    yo: f64,
    zo: f64,
    p: &[i8],
) {
    let xo_f = xo as f32;
    let yo_f = yo as f32;
    let zo_f = zo as f32;
    let p0 = p[0] as i32;
    let p255 = p[255] as i32;
    write!(
        builder,
        "xo={:.3}, yo={:.3}, zo={:.3}, p0={}, p255={}",
        xo_f, yo_f, zo_f, p0, p255
    )
    .unwrap();
}

// JAVA: net.minecraft.world.level.levelgen.synth.NoiseUtils#parityNoiseOctaveConfigString(StringBuilder,double,double,double,int[])
pub fn parity_noise_octave_config_string_ints(
    builder: &mut String,
    xo: f64,
    yo: f64,
    zo: f64,
    p: &[i32],
) {
    let xo_f = xo as f32;
    let yo_f = yo as f32;
    let zo_f = zo as f32;
    let p0 = p[0];
    let p255 = p[255];
    write!(
        builder,
        "xo={:.3}, yo={:.3}, zo={:.3}, p0={}, p255={}",
        xo_f, yo_f, zo_f, p0, p255
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bias_towards_extreme_zero_bias() {
        let v = bias_towards_extreme(0.5, 0.0);
        assert_eq!(v.to_bits(), 0.5f64.to_bits());
    }

    #[test]
    fn bias_towards_extreme_midpoint() {
        // sin(PI * 0.5) = 1.0, so result = 0.5 + 1.0 * 0.2 / PI
        let v = bias_towards_extreme(0.5, 0.2);
        let expected = 0.5 + 1.0f64 * 0.2 / PI;
        assert_eq!(v.to_bits(), expected.to_bits());
    }

    #[test]
    fn bias_towards_extreme_negative_input() {
        let v = bias_towards_extreme(-0.25, 0.5);
        let expected = -0.25 + (PI * -0.25).sin() * 0.5 / PI;
        assert_eq!(v.to_bits(), expected.to_bits());
    }

    #[test]
    fn parity_string_bytes_basic() {
        let mut s = String::new();
        let mut p = vec![0i8; 256];
        p[0] = 7;
        p[255] = -3;
        parity_noise_octave_config_string_bytes(&mut s, 1.25, -2.5, 3.0, &p);
        assert_eq!(s, "xo=1.250, yo=-2.500, zo=3.000, p0=7, p255=-3");
    }

    #[test]
    fn parity_string_ints_basic() {
        let mut s = String::new();
        let mut p = vec![0i32; 256];
        p[0] = 42;
        p[255] = -123456;
        parity_noise_octave_config_string_ints(&mut s, 0.0, 1.0, -1.0, &p);
        assert_eq!(s, "xo=0.000, yo=1.000, zo=-1.000, p0=42, p255=-123456");
    }

    #[test]
    fn parity_string_appends_and_handles_negative_inputs() {
        let mut s = String::from("prefix:");
        let mut pb = vec![0i8; 256];
        pb[0] = -1;
        pb[255] = i8::MIN;
        parity_noise_octave_config_string_bytes(&mut s, -1.5, -0.0, 3.0, &pb);

        let mut s2 = String::from("prefix:");
        let mut pi = vec![0i32; 256];
        pi[0] = -1;
        pi[255] = i32::MIN;
        parity_noise_octave_config_string_ints(&mut s2, -1.5, -0.0, 3.0, &pi);

        assert_eq!(
            s,
            "prefix:xo=-1.500, yo=-0.000, zo=3.000, p0=-1, p255=-128"
        );
        assert_eq!(
            s2,
            "prefix:xo=-1.500, yo=-0.000, zo=3.000, p0=-1, p255=-2147483648"
        );
    }
}
