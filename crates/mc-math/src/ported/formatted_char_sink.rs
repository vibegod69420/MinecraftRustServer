//! Faithful port of `net.minecraft.util.FormattedCharSink`.
//!
//! This module provides the [`FormattedCharSink`] trait, a callback interface
//! for consuming formatted character data: an index in the current line, a
//! [`Style`], and a Unicode code point.

/// Minimal self-contained placeholder for `net.minecraft.network.chat.Style`.
///
/// The full `Style` type is outside the scope of this port. This placeholder
/// preserves the [`FormattedCharSink`] signature shape while keeping the module
/// self-contained.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Style;

/// Callback sink for formatted character iteration.
///
/// Mirrors the Java `@FunctionalInterface`
/// `net.minecraft.util.FormattedCharSink`.
///
/// Return `true` from [`FormattedCharSink::accept`] to continue iteration, or
/// `false` to stop.
pub trait FormattedCharSink {
    // JAVA: net.minecraft.util.FormattedCharSink#accept
    fn accept(&mut self, index_in_current_line: i32, style: &Style, code_point: i32) -> bool;
}

impl<F> FormattedCharSink for F
where
    F: FnMut(i32, &Style, i32) -> bool,
{
    // JAVA: net.minecraft.util.FormattedCharSink#accept
    fn accept(&mut self, index_in_current_line: i32, style: &Style, code_point: i32) -> bool {
        (self)(index_in_current_line, style, code_point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn closure_sink_receives_arguments() {
        let style = Style::default();
        let mut seen: Vec<(i32, Style, i32)> = Vec::new();
        {
            let mut sink = |index: i32, s: &Style, cp: i32| {
                seen.push((index, s.clone(), cp));
                true
            };
            assert!(sink.accept(0, &style, 'A' as i32));
            assert!(sink.accept(1, &style, 'B' as i32));
        }
        assert_eq!(
            seen,
            vec![
                (0, style.clone(), 'A' as i32),
                (1, style, 'B' as i32),
            ]
        );
    }

    #[test]
    fn struct_sink_implementation() {
        struct CountingSink {
            count: usize,
            last_index: i32,
            last_code_point: i32,
            last_style: Option<Style>,
        }

        impl FormattedCharSink for CountingSink {
            fn accept(
                &mut self,
                index_in_current_line: i32,
                style: &Style,
                code_point: i32,
            ) -> bool {
                self.count += 1;
                self.last_index = index_in_current_line;
                self.last_code_point = code_point;
                self.last_style = Some(style.clone());
                true
            }
        }

        let style = Style::default();
        let mut sink = CountingSink {
            count: 0,
            last_index: 0,
            last_code_point: 0,
            last_style: None,
        };

        assert!(FormattedCharSink::accept(&mut sink, 7, &style, 99));
        assert_eq!(sink.count, 1);
        assert_eq!(sink.last_index, 7);
        assert_eq!(sink.last_code_point, 99);
        assert_eq!(sink.last_style.as_ref(), Some(&style));
    }

    #[test]
    fn returns_false_stops_iteration() {
        let style = Style::default();
        let chars: Vec<i32> = vec!['H' as i32, 'e' as i32, 'l' as i32, 'l' as i32, 'o' as i32];
        let mut collected: Vec<i32> = Vec::new();
        {
            let mut sink = |index: i32, _s: &Style, code_point: i32| {
                if index >= 3 {
                    return false;
                }
                collected.push(code_point);
                true
            };
            for (i, &cp) in chars.iter().enumerate() {
                if !sink.accept(i as i32, &style, cp) {
                    break;
                }
            }
        }
        assert_eq!(collected, vec!['H' as i32, 'e' as i32, 'l' as i32]);
    }

    #[test]
    fn negative_and_boundary_integer_inputs() {
        let style = Style::default();
        let mut seen: Vec<(i32, i32)> = Vec::new();
        {
            let mut sink = |index: i32, _s: &Style, cp: i32| {
                seen.push((index, cp));
                true
            };
            assert!(sink.accept(i32::MIN, &style, i32::MIN));
            assert!(sink.accept(i32::MAX, &style, i32::MAX));
            assert!(sink.accept(-1, &style, -1));
            assert!(sink.accept(0, &style, 0));
            assert!(sink.accept(-123_456, &style, 0x10FFFF));
            assert!(sink.accept(i32::MIN + 1, &style, i32::MAX - 1));
        }
        assert_eq!(
            seen,
            vec![
                (i32::MIN, i32::MIN),
                (i32::MAX, i32::MAX),
                (-1, -1),
                (0, 0),
                (-123_456, 0x10FFFF),
                (i32::MIN + 1, i32::MAX - 1),
            ]
        );
    }

    #[test]
    fn float_bit_patterns_preserved_as_i32_values() {
        let style = Style::default();
        let float_bits: Vec<i32> = vec![
            f32::NAN.to_bits() as i32,
            f32::INFINITY.to_bits() as i32,
            f32::NEG_INFINITY.to_bits() as i32,
            (-0.0f32).to_bits() as i32,
            0.0f32.to_bits() as i32,
            1.0f32.to_bits() as i32,
            f32::MIN_POSITIVE.to_bits() as i32,
            f32::MAX.to_bits() as i32,
        ];
        let mut seen: Vec<(i32, i32)> = Vec::new();
        {
            let mut sink = |index: i32, _s: &Style, cp: i32| {
                seen.push((index, cp));
                true
            };
            for (i, &bits) in float_bits.iter().enumerate() {
                assert!(sink.accept(bits, &style, i as i32));
            }
        }
        for (i, &bits) in float_bits.iter().enumerate() {
            assert_eq!(seen[i].0, bits);
            assert_eq!(seen[i].1, i as i32);
        }

        // Also verify f64 bit patterns can be observed through paired i32 values
        // by passing high/low halves as index/code_point.
        let f64_values: Vec<f64> = vec![
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            -0.0f64,
            0.0f64,
            1.0f64,
            f64::MIN_POSITIVE,
            f64::MAX,
        ];
        let mut seen_f64: Vec<(i32, i32)> = Vec::new();
        {
            let mut sink = |index: i32, _s: &Style, cp: i32| {
                seen_f64.push((index, cp));
                true
            };
            for &v in &f64_values {
                let bits = v.to_bits();
                let hi = (bits >> 32) as i32;
                let lo = (bits & 0xFFFF_FFFF) as i32;
                assert!(sink.accept(hi, &style, lo));
            }
        }
        for (i, &v) in f64_values.iter().enumerate() {
            let bits = v.to_bits();
            let hi = (bits >> 32) as i32;
            let lo = (bits & 0xFFFF_FFFF) as i32;
            assert_eq!(seen_f64[i], (hi, lo));
        }
    }

    #[test]
    fn trait_object_dispatch() {
        let style = Style::default();
        let mut total: i32 = 0;
        {
            let mut sink = |index: i32, _s: &Style, _cp: i32| {
                total = total.wrapping_add(index);
                index != 2
            };
            let sink_obj: &mut dyn FormattedCharSink = &mut sink;
            assert!(sink_obj.accept(1, &style, 10));
            assert!(!sink_obj.accept(2, &style, 20));
            assert!(sink_obj.accept(3, &style, 30));
        }
        assert_eq!(total, 6);
    }

    #[test]
    fn style_reference_is_passed_through() {
        let style = Style::default();
        let style_ptr = &style as *const Style;
        let mut seen_ptr: *const Style = std::ptr::null();
        {
            let mut sink = |_i: i32, s: &Style, _cp: i32| {
                seen_ptr = s as *const Style;
                true
            };
            assert!(sink.accept(0, &style, 65));
        }
        assert_eq!(seen_ptr, style_ptr);
    }

    #[test]
    fn style_placeholder_traits() {
        let a = Style::default();
        let b = a.clone();
        assert_eq!(a, b);
        let _ = format!("{:?}", a);

        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn multiple_sinks_can_share_style() {
        let style = Style::default();
        let mut count_a: i32 = 0;
        let mut count_b: i32 = 0;
        {
            let mut sink_a = |_i: i32, _s: &Style, cp: i32| {
                count_a = count_a.wrapping_add(cp);
                true
            };
            let mut sink_b = |_i: i32, _s: &Style, cp: i32| {
                count_b = count_b.wrapping_add(cp);
                cp >= 0
            };

            assert!(sink_a.accept(0, &style, 1));
            assert!(sink_b.accept(0, &style, 1));
            assert!(sink_a.accept(1, &style, -1));
            assert!(!sink_b.accept(1, &style, -1));
        }
        assert_eq!(count_a, 0); // 1 + (-1) wrapping
        assert_eq!(count_b, 0); // 1 + (-1) wrapping
    }
}
