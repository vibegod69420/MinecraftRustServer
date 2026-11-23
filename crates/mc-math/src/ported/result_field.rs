use std::cmp::Ordering;

// JAVA: net.minecraft.util.profiling.ResultField
#[derive(Debug, Clone)]
pub struct ResultField {
    pub percentage: f64,
    pub global_percentage: f64,
    pub count: i64,
    pub name: String,
}

impl ResultField {
    // JAVA: net.minecraft.util.profiling.ResultField#<init>
    pub fn new(name: String, percentage: f64, global_percentage: f64, count: i64) -> Self {
        Self {
            percentage,
            global_percentage,
            count,
            name,
        }
    }

    // JAVA: net.minecraft.util.profiling.ResultField#compareTo
    pub fn compare_to(&self, other: &ResultField) -> i32 {
        if other.percentage < self.percentage {
            -1
        } else if other.percentage > self.percentage {
            1
        } else {
            java_string_compare(&self.name, &other.name)
        }
    }

    // JAVA: net.minecraft.util.profiling.ResultField#getColor
    pub fn get_color(&self) -> i32 {
        let hash = java_string_hash_code(&self.name);
        (hash & 11_184_810_i32).wrapping_add(-12_303_292_i32)
    }
}

impl PartialEq for ResultField {
    fn eq(&self, other: &Self) -> bool {
        self.compare_to(other) == 0
    }
}

impl Eq for ResultField {}

impl PartialOrd for ResultField {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResultField {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.compare_to(other) {
            x if x < 0 => Ordering::Less,
            x if x > 0 => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}

fn java_string_hash_code(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(c as i32);
    }
    h
}

fn java_string_compare(a: &str, b: &str) -> i32 {
    let a_u16: Vec<u16> = a.encode_utf16().collect();
    let b_u16: Vec<u16> = b.encode_utf16().collect();
    let len = a_u16.len().min(b_u16.len());
    for i in 0..len {
        if a_u16[i] != b_u16[i] {
            return (a_u16[i] as i32).wrapping_sub(b_u16[i] as i32);
        }
    }
    (a_u16.len() as i32).wrapping_sub(b_u16.len() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_to_orders_by_percentage_desc_then_name() {
        let a = ResultField::new("b".to_string(), 50.0, 10.0, 1);
        let b = ResultField::new("a".to_string(), 50.0, 10.0, 1);
        let c = ResultField::new("c".to_string(), 60.0, 10.0, 1);
        // Higher percentage comes first (compare_to returns -1 for self when self.percentage is larger)
        assert_eq!(c.compare_to(&a), -1);
        assert_eq!(a.compare_to(&c), 1);
        // Equal percentage: compare by name
        assert_eq!(a.compare_to(&b), 1); // "b" > "a"
        assert_eq!(b.compare_to(&a), -1);
        assert_eq!(a.compare_to(&a.clone()), 0);
    }

    #[test]
    fn get_color_matches_java_formula() {
        let rf = ResultField::new("root".to_string(), 1.0, 1.0, 1);
        let hash = java_string_hash_code("root");
        let expected = (hash & 11_184_810_i32).wrapping_add(-12_303_292_i32);
        assert_eq!(rf.get_color(), expected);
    }

    #[test]
    fn java_string_hash_code_known_values() {
        assert_eq!(java_string_hash_code(""), 0);
        assert_eq!(java_string_hash_code("a"), 97);
        assert_eq!(java_string_hash_code("root"), {
            let mut h: i32 = 0;
            for c in ['r', 'o', 'o', 't'] {
                h = h.wrapping_mul(31).wrapping_add(c as i32);
            }
            h
        });
    }

    #[test]
    fn get_color_wraps_on_negative_hash() {
        // Construct a name that produces a negative hash to exercise wrapping
        let name = "\u{FFFF}\u{FFFF}\u{FFFF}";
        let rf = ResultField::new(name.to_string(), 0.0, 0.0, 0);
        let hash = java_string_hash_code(name);
        let expected = (hash & 11_184_810_i32).wrapping_add(-12_303_292_i32);
        assert_eq!(rf.get_color(), expected);
    }

    #[test]
    fn compare_to_nan_falls_back_to_name() {
        let a = ResultField::new("a".to_string(), f64::NAN, 0.0, 0);
        let b = ResultField::new("b".to_string(), 10.0, 0.0, 0);
        // NaN comparisons are false, so falls through to name comparison
        assert_eq!(a.compare_to(&b), -1); // "a" < "b"
        let c = ResultField::new("b".to_string(), f64::NAN, 0.0, 0);
        assert_eq!(a.compare_to(&c), -1);
    }

    #[test]
    fn ord_impl_matches_compare_to() {
        let a = ResultField::new("x".to_string(), 20.0, 0.0, -5);
        let b = ResultField::new("y".to_string(), 10.0, 0.0, i64::MAX);
        assert_eq!(a.cmp(&b), Ordering::Less);
        assert!(a < b);
    }
}
