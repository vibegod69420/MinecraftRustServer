use crate::mth;

#[inline(always)]
fn square(x: f32) -> f32 {
    x * x
}

#[inline(always)]
fn cube(x: f32) -> f32 {
    x * x * x
}

// JAVA: net.minecraft.util.Ease#inBack
pub fn in_back(x: f32) -> f32 {
    let _c1: f32 = 1.70158;
    let _c2: f32 = 2.70158;
    square(x) * (2.70158f32 * x - 1.70158f32)
}

// JAVA: net.minecraft.util.Ease#inBounce
pub fn in_bounce(x: f32) -> f32 {
    1.0f32 - out_bounce(1.0f32 - x)
}

// JAVA: net.minecraft.util.Ease#inCubic
pub fn in_cubic(x: f32) -> f32 {
    cube(x)
}

// JAVA: net.minecraft.util.Ease#inElastic
pub fn in_elastic(x: f32) -> f32 {
    if x == 0.0f32 {
        return 0.0f32;
    }
    if x == 1.0f32 {
        return 1.0f32;
    }
    let _c1 = (std::f64::consts::PI * 2.0 / 3.0) as f32;
    (-2.0f64.powf(10.0 * (x as f64) - 10.0)
        * (((x as f64) * 10.0 - 10.75) * ((std::f64::consts::PI * 2.0 / 3.0) as f32 as f64))
            .sin()) as f32
}

// JAVA: net.minecraft.util.Ease#inExpo
pub fn in_expo(x: f32) -> f32 {
    if x == 0.0f32 {
        0.0f32
    } else {
        2.0f64.powf(10.0 * (x as f64) - 10.0) as f32
    }
}

// JAVA: net.minecraft.util.Ease#inQuart
pub fn in_quart(x: f32) -> f32 {
    square(square(x))
}

// JAVA: net.minecraft.util.Ease#inQuint
pub fn in_quint(x: f32) -> f32 {
    square(square(x)) * x
}

// JAVA: net.minecraft.util.Ease#inSine
pub fn in_sine(x: f32) -> f32 {
    1.0f32 - mth::cos((x * ((std::f64::consts::PI / 2.0) as f32)) as f64)
}

// JAVA: net.minecraft.util.Ease#inOutBounce
pub fn in_out_bounce(x: f32) -> f32 {
    if x < 0.5f32 {
        (1.0f32 - out_bounce(1.0f32 - 2.0f32 * x)) / 2.0f32
    } else {
        (1.0f32 + out_bounce(2.0f32 * x - 1.0f32)) / 2.0f32
    }
}

// JAVA: net.minecraft.util.Ease#inOutCirc
pub fn in_out_circ(x: f32) -> f32 {
    if x < 0.5f32 {
        ((1.0 - (1.0 - (2.0 * (x as f64)).powf(2.0)).sqrt()) / 2.0) as f32
    } else {
        (((1.0 - (-2.0 * (x as f64) + 2.0).powf(2.0)).sqrt() + 1.0) / 2.0) as f32
    }
}

// JAVA: net.minecraft.util.Ease#inOutCubic
pub fn in_out_cubic(x: f32) -> f32 {
    if x < 0.5f32 {
        4.0f32 * cube(x)
    } else {
        (1.0 - (-2.0 * (x as f64) + 2.0).powf(3.0) / 2.0) as f32
    }
}

// JAVA: net.minecraft.util.Ease#inOutQuad
pub fn in_out_quad(x: f32) -> f32 {
    if x < 0.5f32 {
        2.0f32 * square(x)
    } else {
        (1.0 - (-2.0 * (x as f64) + 2.0).powf(2.0) / 2.0) as f32
    }
}

// JAVA: net.minecraft.util.Ease#inOutQuart
pub fn in_out_quart(x: f32) -> f32 {
    if x < 0.5f32 {
        8.0f32 * square(square(x))
    } else {
        (1.0 - (-2.0 * (x as f64) + 2.0).powf(4.0) / 2.0) as f32
    }
}

// JAVA: net.minecraft.util.Ease#inOutQuint
pub fn in_out_quint(x: f32) -> f32 {
    if (x as f64) < 0.5 {
        16.0f32 * x * x * x * x * x
    } else {
        (1.0 - (-2.0 * (x as f64) + 2.0).powf(5.0) / 2.0) as f32
    }
}

// JAVA: net.minecraft.util.Ease#outBounce
pub fn out_bounce(x: f32) -> f32 {
    let _c1: f32 = 7.5625;
    let _c2: f32 = 2.75;
    if x < 0.36363637f32 {
        7.5625f32 * square(x)
    } else if x < 0.72727275f32 {
        7.5625f32 * square(x - 0.54545456f32) + 0.75f32
    } else if (x as f64) < 0.9090909090909091f64 {
        7.5625f32 * square(x - 0.8181818f32) + 0.9375f32
    } else {
        7.5625f32 * square(x - 0.95454544f32) + 0.984375f32
    }
}

// JAVA: net.minecraft.util.Ease#outElastic
pub fn out_elastic(x: f32) -> f32 {
    let _c1 = (std::f64::consts::PI * 2.0 / 3.0) as f32;
    if x == 0.0f32 {
        0.0f32
    } else if x == 1.0f32 {
        1.0f32
    } else {
        (2.0f64.powf(-10.0 * (x as f64))
            * (((x as f64) * 10.0 - 0.75)
                * ((std::f64::consts::PI * 2.0 / 3.0) as f32 as f64))
                .sin()
            + 1.0) as f32
    }
}

// JAVA: net.minecraft.util.Ease#outExpo
pub fn out_expo(x: f32) -> f32 {
    if x == 1.0f32 {
        1.0f32
    } else {
        1.0f32 - 2.0f64.powf(-10.0 * (x as f64)) as f32
    }
}

// JAVA: net.minecraft.util.Ease#outQuad
pub fn out_quad(x: f32) -> f32 {
    1.0f32 - square(1.0f32 - x)
}

// JAVA: net.minecraft.util.Ease#outQuint
pub fn out_quint(x: f32) -> f32 {
    1.0f32 - (1.0 - (x as f64)).powf(5.0) as f32
}

// JAVA: net.minecraft.util.Ease#outSine
pub fn out_sine(x: f32) -> f32 {
    mth::sin((x * ((std::f64::consts::PI / 2.0) as f32)) as f64)
}

// JAVA: net.minecraft.util.Ease#inOutSine
pub fn in_out_sine(x: f32) -> f32 {
    -(mth::cos(((std::f64::consts::PI as f32) * x) as f64) - 1.0f32) / 2.0f32
}

// JAVA: net.minecraft.util.Ease#outBack
pub fn out_back(x: f32) -> f32 {
    let _c1: f32 = 1.70158;
    let _c2: f32 = 2.70158;
    1.0f32 + 2.70158f32 * cube(x - 1.0f32) + 1.70158f32 * square(x - 1.0f32)
}

// JAVA: net.minecraft.util.Ease#outQuart
pub fn out_quart(x: f32) -> f32 {
    1.0f32 - square(square(1.0f32 - x))
}

// JAVA: net.minecraft.util.Ease#outCubic
pub fn out_cubic(x: f32) -> f32 {
    1.0f32 - cube(1.0f32 - x)
}

// JAVA: net.minecraft.util.Ease#inOutExpo
pub fn in_out_expo(x: f32) -> f32 {
    if x < 0.5f32 {
        if x == 0.0f32 {
            0.0f32
        } else {
            (2.0f64.powf(20.0 * (x as f64) - 10.0) / 2.0) as f32
        }
    } else if x == 1.0f32 {
        1.0f32
    } else {
        ((2.0 - 2.0f64.powf(-20.0 * (x as f64) + 10.0)) / 2.0) as f32
    }
}

// JAVA: net.minecraft.util.Ease#inQuad
pub fn in_quad(x: f32) -> f32 {
    x * x
}

// JAVA: net.minecraft.util.Ease#outCirc
pub fn out_circ(x: f32) -> f32 {
    (1.0f64 - (square(x - 1.0f32) as f64)).sqrt() as f32
}

// JAVA: net.minecraft.util.Ease#inOutElastic
pub fn in_out_elastic(x: f32) -> f32 {
    let _c1 = (std::f64::consts::PI as f32) * 4.0f32 / 9.0f32;
    if x == 0.0f32 {
        return 0.0f32;
    }
    if x == 1.0f32 {
        return 1.0f32;
    }
    let sin_arg = ((20.0 * (x as f64) - 11.125) * ((std::f64::consts::PI as f32) as f64)
        * (4.0f32 as f64))
        / (9.0f32 as f64);
    let s = sin_arg.sin();
    if x < 0.5f32 {
        (-(2.0f64.powf(20.0 * (x as f64) - 10.0) * s) / 2.0) as f32
    } else {
        (2.0f64.powf(-20.0 * (x as f64) + 10.0) * s / 2.0 + 1.0) as f32
    }
}

// JAVA: net.minecraft.util.Ease#inCirc
pub fn in_circ(x: f32) -> f32 {
    (-(1.0f64 - ((x * x) as f64)).sqrt()) as f32 + 1.0f32
}

// JAVA: net.minecraft.util.Ease#inOutBack
pub fn in_out_back(x: f32) -> f32 {
    let _c1: f32 = 1.70158;
    let _c2: f32 = 2.5949094;
    if x < 0.5f32 {
        4.0f32 * x * x * (7.189819f32 * x - 2.5949094f32) / 2.0f32
    } else {
        let t = 2.0f32 * x - 2.0f32;
        (t * t * (3.5949094f32 * t + 2.5949094f32) + 2.0f32) / 2.0f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_zero_one() {
        let funcs: &[(&str, fn(f32) -> f32)] = &[
            ("inBack", in_back),
            ("inBounce", in_bounce),
            ("inCubic", in_cubic),
            ("inElastic", in_elastic),
            ("inExpo", in_expo),
            ("inQuart", in_quart),
            ("inQuint", in_quint),
            ("inSine", in_sine),
            ("inOutBounce", in_out_bounce),
            ("inOutCirc", in_out_circ),
            ("inOutCubic", in_out_cubic),
            ("inOutQuad", in_out_quad),
            ("inOutQuart", in_out_quart),
            ("inOutQuint", in_out_quint),
            ("outBounce", out_bounce),
            ("outElastic", out_elastic),
            ("outExpo", out_expo),
            ("outQuad", out_quad),
            ("outQuint", out_quint),
            ("outSine", out_sine),
            ("inOutSine", in_out_sine),
            ("outBack", out_back),
            ("outQuart", out_quart),
            ("outCubic", out_cubic),
            ("inOutExpo", in_out_expo),
            ("inQuad", in_quad),
            ("outCirc", out_circ),
            ("inOutElastic", in_out_elastic),
            ("inCirc", in_circ),
            ("inOutBack", in_out_back),
        ];
        for (name, f) in funcs {
            let v0 = f(0.0f32);
            let v1 = f(1.0f32);
            assert!(
                (v0 - 0.0f32).abs() < 1e-4,
                "{name}(0) = {v0}, expected ~0.0"
            );
            assert!(
                (v1 - 1.0f32).abs() < 1e-4,
                "{name}(1) = {v1}, expected ~1.0"
            );
        }
    }

    #[test]
    fn exact_float_bits_simple_cases() {
        assert_eq!(in_quad(0.5f32).to_bits(), 0.25f32.to_bits());
        assert_eq!(in_cubic(0.5f32).to_bits(), 0.125f32.to_bits());
        assert_eq!(in_quart(0.5f32).to_bits(), 0.0625f32.to_bits());
        assert_eq!(in_quad(-2.0f32).to_bits(), 4.0f32.to_bits());
        assert_eq!(in_elastic(0.0f32).to_bits(), 0.0f32.to_bits());
        assert_eq!(in_elastic(1.0f32).to_bits(), 1.0f32.to_bits());
        assert_eq!(out_elastic(0.0f32).to_bits(), 0.0f32.to_bits());
        assert_eq!(out_elastic(1.0f32).to_bits(), 1.0f32.to_bits());
        assert_eq!(in_out_elastic(0.0f32).to_bits(), 0.0f32.to_bits());
        assert_eq!(in_out_elastic(1.0f32).to_bits(), 1.0f32.to_bits());
    }

    #[test]
    fn out_bounce_thresholds_and_shape() {
        assert_eq!(out_bounce(0.0f32).to_bits(), 0.0f32.to_bits());
        assert!((out_bounce(1.0f32) - 1.0f32).abs() < 1e-6);
        // First segment: x < 0.36363637F
        let v1 = out_bounce(0.2f32);
        assert!(v1 > 0.0f32 && v1 < 1.0f32);
        // Second segment
        let v2 = out_bounce(0.5f32);
        assert!(v2 > 0.5f32 && v2 < 1.0f32);
        // Third segment (double threshold boundary)
        let v3 = out_bounce(0.85f32);
        assert!(v3 > 0.9f32 && v3 <= 1.0f32);
        // Fourth segment
        let v4 = out_bounce(0.95f32);
        assert!(v4 > 0.9f32 && v4 <= 1.1f32);
    }

    #[test]
    fn negative_inputs_and_large_values_no_panic() {
        let funcs: &[fn(f32) -> f32] = &[
            in_back, in_bounce, in_cubic, in_elastic, in_expo, in_quart, in_quint,
            in_sine, in_out_bounce, in_out_circ, in_out_cubic, in_out_quad, in_out_quart,
            in_out_quint, out_bounce, out_elastic, out_expo, out_quad, out_quint, out_sine,
            in_out_sine, out_back, out_quart, out_cubic, in_out_expo, in_quad, out_circ,
            in_out_elastic, in_circ, in_out_back,
        ];
        for &f in funcs {
            let v_neg = f(-1.0f32);
            let v_big = f(2.0f32);
            // Should not panic; NaN/inf are possible for out-of-domain inputs
            assert!(v_neg.is_nan() || v_neg.is_finite() || v_neg.is_infinite());
            assert!(v_big.is_nan() || v_big.is_finite() || v_big.is_infinite());
        }
    }

    #[test]
    fn in_out_sine_midpoint_and_symmetry() {
        let mid = in_out_sine(0.5f32);
        assert!((mid - 0.5f32).abs() < 1e-6, "mid={mid}");
        // inOutSine should be roughly symmetric around midpoint
        let a = in_out_sine(0.25f32);
        let b = in_out_sine(0.75f32);
        assert!((a + b - 1.0f32).abs() < 1e-5, "a={a}, b={b}");
    }
}
