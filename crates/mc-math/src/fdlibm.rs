//! Pure safe Rust ports of OpenJDK `java.lang.FdLibm` (`StrictMath` / Sun FDLIBM)
//! transcendental primitives so math behavior is identical across Windows and Linux/ARM64.

const TWO54: f64 = 1.8014398509481984e16; // 0x4350000000000000

#[inline]
fn high_word(x: f64) -> i32 {
    (x.to_bits() >> 32) as i32
}

#[inline]
fn low_word(x: f64) -> i32 {
    x.to_bits() as i32
}

#[inline]
fn set_high_word(x: f64, hi: i32) -> f64 {
    let bits = (x.to_bits() & 0x0000_0000_FFFF_FFFF) | (((hi as u32) as u64) << 32);
    f64::from_bits(bits)
}

#[inline]
fn with_words(hi: i32, lo: i32) -> f64 {
    let bits = (((hi as u32) as u64) << 32) | ((lo as u32) as u64);
    f64::from_bits(bits)
}

/// Exact Sun FDLIBM / OpenJDK `StrictMath.log(double)` (`e_log.c`).
pub fn log(x: f64) -> f64 {
    const LN2_HI: f64 = 6.93147180369123816490e-01;
    const LN2_LO: f64 = 1.90821492927058770002e-10;
    const LG1: f64 = 6.666666666666735130e-01;
    const LG2: f64 = 3.999999999940941908e-01;
    const LG3: f64 = 2.857142874366239149e-01;
    const LG4: f64 = 2.222219843214978396e-01;
    const LG5: f64 = 1.818357216161805012e-01;
    const LG6: f64 = 1.531383769920937332e-01;
    const LG7: f64 = 1.479819860511658591e-01;

    let mut hx = high_word(x);
    let lx = low_word(x);
    let mut k: i32 = 0;

    if hx < 0x0010_0000 {
        if ((hx & 0x7FFF_FFFF) | lx) == 0 {
            return -TWO54 / 0.0; // -Inf
        }
        if hx < 0 {
            return (x - x) / 0.0; // NaN
        }
        k -= 54;
        let x_scaled = x * TWO54;
        hx = high_word(x_scaled);
        return log_inner(x_scaled, hx, k, LN2_HI, LN2_LO, LG1, LG2, LG3, LG4, LG5, LG6, LG7);
    }
    if hx >= 0x7FF0_0000 {
        return x + x;
    }
    if hx == 0x3FF0_0000 && lx == 0 {
        return 0.0;
    }
    log_inner(x, hx, k, LN2_HI, LN2_LO, LG1, LG2, LG3, LG4, LG5, LG6, LG7)
}

#[allow(clippy::too_many_arguments)]
#[inline]
fn log_inner(
    x: f64,
    mut hx: i32,
    mut k: i32,
    ln2_hi: f64,
    ln2_lo: f64,
    lg1: f64,
    lg2: f64,
    lg3: f64,
    lg4: f64,
    lg5: f64,
    lg6: f64,
    lg7: f64,
) -> f64 {
    k += (hx >> 20) - 1023;
    hx &= 0x000F_FFFF;
    let i = (hx + 0x9_5F64) & 0x0010_0000;
    let x = set_high_word(x, hx | (i ^ 0x3FF0_0000));
    k += i >> 20;
    let f = x - 1.0;
    if (0x000F_FFFF & (2 + hx)) < 3 {
        if f == 0.0 {
            if k == 0 {
                return 0.0;
            }
            let dk = k as f64;
            return dk * ln2_hi + dk * ln2_lo;
        }
        let r = f * f * (0.5 - 0.33333333333333333 * f);
        if k == 0 {
            return f - r;
        }
        let dk = k as f64;
        return dk * ln2_hi - ((r - dk * ln2_lo) - f);
    }
    let s = f / (2.0 + f);
    let dk = k as f64;
    let z = s * s;
    let i_adj = hx - 0x6_147A;
    let w = z * z;
    let j = 0x6_B851 - hx;
    let t1 = w * (lg2 + w * (lg4 + w * lg6));
    let t2 = z * (lg1 + w * (lg3 + w * (lg5 + w * lg7)));
    let r = t2 + t1;
    if (i_adj | j) > 0 {
        let hfsq = 0.5 * f * f;
        if k == 0 {
            f - (hfsq - s * (hfsq + r))
        } else {
            dk * ln2_hi - ((hfsq - (s * (hfsq + r) + dk * ln2_lo)) - f)
        }
    } else if k == 0 {
        f - s * (f - r)
    } else {
        dk * ln2_hi - ((s * (f - r) - dk * ln2_lo) - f)
    }
}

/// Kernel `__kernel_sin(x, y, iy)` (`k_sin.c`) for `|x| <= pi/4`.
fn kernel_sin(x: f64, y: f64, iy: i32) -> f64 {
    const S1: f64 = -1.66666666666666324348e-01;
    const S2: f64 = 8.33333333332248946124e-03;
    const S3: f64 = -1.98412698298579493134e-04;
    const S4: f64 = 2.75573137070700676789e-06;
    const S5: f64 = -2.50507602534068634195e-08;
    const S6: f64 = 1.58969099521155010221e-10;

    let ix = high_word(x) & 0x7FFF_FFFF;
    if ix < 0x3E40_0000 {
        if (x as i32) == 0 {
            return x;
        }
    }
    let z = x * x;
    let v = z * x;
    let r = S2 + z * (S3 + z * (S4 + z * (S5 + z * S6)));
    if iy == 0 {
        x + v * (S1 + z * r)
    } else {
        x - ((z * (0.5 * y - v * r) - y) - v * S1)
    }
}

/// Kernel `__kernel_cos(x, y)` (`k_cos.c`) for `|x| <= pi/4`.
fn kernel_cos(x: f64, y: f64) -> f64 {
    const C1: f64 = 4.16666666666666019037e-02;
    const C2: f64 = -1.38888888888741095749e-03;
    const C3: f64 = 2.48015872894767294178e-05;
    const C4: f64 = -2.75573143513906633035e-07;
    const C5: f64 = 2.08757232129817482790e-09;
    const C6: f64 = -1.13596475577881948265e-11;

    let ix = high_word(x) & 0x7FFF_FFFF;
    if ix < 0x3E40_0000 {
        if (x as i32) == 0 {
            return 1.0;
        }
    }
    let z = x * x;
    let r = z * (C1 + z * (C2 + z * (C3 + z * (C4 + z * (C5 + z * C6)))));
    if ix < 0x3FD3_3333 {
        1.0 - (0.5 * z - (z * r - x * y))
    } else {
        let qx = if ix > 0x3FE9_0000 {
            0.28125
        } else {
            with_words(ix - 0x0020_0000, 0)
        };
        let hz = 0.5 * z - qx;
        let a = 1.0 - qx;
        a - (hz - (z * r - x * y))
    }
}

/// Exact Sun FDLIBM / OpenJDK `StrictMath.asin(double)` (`e_asin.c`) for `|x| <= 1`.
pub fn asin(x: f64) -> f64 {
    const PIO2_HI: f64 = 1.57079632679489655800e+00;
    const PIO2_LO: f64 = 6.12323399573676603587e-17;
    const PIO4_HI: f64 = 7.85398163397448278999e-01;
    const P_S0: f64 = 1.66666666666666657415e-01;
    const P_S1: f64 = -3.25565818622400915405e-01;
    const P_S2: f64 = 2.01212532134862925881e-01;
    const P_S3: f64 = -4.00555345006794114027e-02;
    const P_S4: f64 = 7.91534994289814532176e-04;
    const P_S5: f64 = 3.47933107596021167570e-05;
    const Q_S1: f64 = -2.40339491173441421878e+00;
    const Q_S2: f64 = 2.02094576023350569471e+00;
    const Q_S3: f64 = -6.88283971605453293030e-01;
    const Q_S4: f64 = 7.70381505559019352791e-02;

    let hx = high_word(x);
    let ix = hx & 0x7FFF_FFFF;
    if ix >= 0x3FF0_0000 {
        if ((ix - 0x3FF0_0000) | low_word(x)) == 0 {
            return x * PIO2_HI + x * PIO2_LO;
        }
        return (x - x) / (x - x);
    } else if ix < 0x3FE0_0000 {
        if ix < 0x3E40_0000 && (TWO54 + x) > 1.0 {
            return x;
        }
        let t = x * x;
        let p = t * (P_S0 + t * (P_S1 + t * (P_S2 + t * (P_S3 + t * (P_S4 + t * P_S5)))));
        let q = 1.0 + t * (Q_S1 + t * (Q_S2 + t * (Q_S3 + t * Q_S4)));
        let w = p / q;
        return x + x * w;
    }

    let w = 1.0 - x.abs();
    let t = w * 0.5;
    let p = t * (P_S0 + t * (P_S1 + t * (P_S2 + t * (P_S3 + t * (P_S4 + t * P_S5)))));
    let q = 1.0 + t * (Q_S1 + t * (Q_S2 + t * (Q_S3 + t * Q_S4)));
    let s = t.sqrt();
    if ix >= 0x3FEF_3333 {
        let r = p / q;
        let t2 = PIO2_HI - (2.0 * (s + s * r) - PIO2_LO);
        return if hx > 0 { t2 } else { -t2 };
    }
    let w2 = set_high_word(s, high_word(s) & -1_i32); // preserve exact bitmask below
    let w_trunc = with_words(high_word(s), 0);
    let _ = w2;
    let c = (t - w_trunc * w_trunc) / (s + w_trunc);
    let r = p / q;
    let p2 = 2.0 * s * r - (PIO2_LO - 2.0 * c);
    let q2 = PIO4_HI - 2.0 * w_trunc;
    let t2 = PIO4_HI - (p2 - q2);
    if hx > 0 { t2 } else { -t2 }
}

/// Exact Sun FDLIBM / OpenJDK `StrictMath.sin(double)` for `[0, 2*pi]` (`Mth.SIN` LUT).
pub fn sin_small(x: f64) -> f64 {
    const PIO2_1: f64 = 1.57079632673412561417e+00;
    const PIO2_1T: f64 = 6.07710050650619224932e-11;
    const PIO2_2: f64 = 6.07710050630396597660e-11;
    const PIO2_2T: f64 = 2.02226624879595063154e-21;
    const PIO2_3: f64 = 2.02226624871116645580e-21;
    const PIO2_3T: f64 = 8.47842766036889956997e-32;
    const INVPIO2: f64 = 6.36619772367581382433e-01;
    const PIO4: f64 = 7.85398163397448278999e-01;

    if x <= PIO4 {
        return kernel_sin(x, 0.0, 0);
    }
    let fn_val = (x * INVPIO2 + 0.5).floor();
    let n = fn_val as i32;
    let j = (high_word(x) >> 20) & 0x7FF;
    let mut r = x - fn_val * PIO2_1;
    let mut w = fn_val * PIO2_1T;
    let mut y0 = r - w;
    let i = j - ((high_word(y0) >> 20) & 0x7FF);
    if i > 16 {
        let t = r;
        w = fn_val * PIO2_2;
        r = t - w;
        w = fn_val * PIO2_2T - ((t - r) - w);
        y0 = r - w;
        let i2 = j - ((high_word(y0) >> 20) & 0x7FF);
        if i2 > 49 {
            let t2 = r;
            w = fn_val * PIO2_3;
            r = t2 - w;
            w = fn_val * PIO2_3T - ((t2 - r) - w);
            y0 = r - w;
        }
    }
    let y1 = (r - y0) - w;
    match n & 3 {
        0 => kernel_sin(y0, y1, 1),
        1 => kernel_cos(y0, y1),
        2 => -kernel_sin(y0, y1, 1),
        _ => -kernel_cos(y0, y1),
    }
}

/// Exact Sun FDLIBM / OpenJDK `StrictMath.cos(double)` for `|x| <= pi/2` (`Mth.COS_TAB`).
pub fn cos_small(x: f64) -> f64 {
    const PIO2_1: f64 = 1.57079632673412561417e+00;
    const PIO2_1T: f64 = 6.07710050650619224932e-11;
    const PIO2_2: f64 = 6.07710050630396597660e-11;
    const PIO2_2T: f64 = 2.02226624879595063154e-21;
    const PIO4: f64 = 7.85398163397448278999e-01;

    let ax = x.abs();
    if ax <= PIO4 {
        return kernel_cos(ax, 0.0);
    }
    let j = (high_word(ax) >> 20) & 0x7FF;
    let mut r = ax - PIO2_1;
    let mut w = PIO2_1T;
    let mut y0 = r - w;
    let i = j - ((high_word(y0) >> 20) & 0x7FF);
    if i > 16 {
        let t = r;
        w = PIO2_2;
        r = t - w;
        w = PIO2_2T - ((t - r) - w);
        y0 = r - w;
    }
    let y1 = (r - y0) - w;
    -kernel_sin(y0, y1, 1)
}
