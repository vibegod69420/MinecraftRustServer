use std::collections::HashMap;
use std::fmt::Debug;

// JAVA: net.minecraft.util.ByIdMap$OutOfBoundsStrategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutOfBoundsStrategy {
    Zero,
    Wrap,
    Clamp,
}

pub struct ByIdMap;

impl ByIdMap {
    // JAVA: net.minecraft.util.ByIdMap#createMap
    fn create_map<T: Clone + Debug + 'static>(
        id_fn: &impl Fn(&T) -> i32,
        values: &[T],
    ) -> HashMap<i32, T> {
        if values.is_empty() {
            panic!("Empty value list");
        }
        let mut map: HashMap<i32, T> = HashMap::new();
        for value in values {
            let id = id_fn(value);
            if let Some(prev) = map.insert(id, value.clone()) {
                panic!(
                    "Duplicate entry on id {}: current={:?}, previous={:?}",
                    id, value, prev
                );
            }
        }
        map
    }

    // JAVA: net.minecraft.util.ByIdMap#sparse
    pub fn sparse<T: Clone + Debug + 'static>(
        id_fn: impl Fn(&T) -> i32,
        values: &[T],
        fallback: T,
    ) -> Box<dyn Fn(i32) -> T> {
        let map = Self::create_map(&id_fn, values);
        Box::new(move |id: i32| map.get(&id).cloned().unwrap_or_else(|| fallback.clone()))
    }

    // JAVA: net.minecraft.util.ByIdMap#createSortedArray
    fn create_sorted_array<T: Clone + Debug>(id_fn: &impl Fn(&T) -> i32, values: &[T]) -> Vec<T> {
        let len = values.len();
        if len == 0 {
            panic!("Empty value list");
        }

        let mut sorted: Vec<Option<T>> = vec![None; len];
        for value in values {
            let id = id_fn(value);
            if id < 0 || id >= len as i32 {
                panic!(
                    "Values are not continous, found index {} for value {:?}",
                    id, value
                );
            }
            let idx = id as usize;
            if let Some(prev) = sorted[idx].as_ref() {
                panic!(
                    "Duplicate entry on id {}: current={:?}, previous={:?}",
                    id, value, prev
                );
            }
            sorted[idx] = Some(value.clone());
        }

        for (i, slot) in sorted.iter().enumerate() {
            if slot.is_none() {
                panic!("Missing value at index: {}", i);
            }
        }

        sorted.into_iter().map(|o| o.expect("missing value")).collect()
    }

    // JAVA: net.minecraft.util.ByIdMap#continuous
    pub fn continuous<T: Clone + Debug + 'static>(
        id_fn: impl Fn(&T) -> i32,
        values: &[T],
        strategy: OutOfBoundsStrategy,
    ) -> Box<dyn Fn(i32) -> T> {
        let sorted = Self::create_sorted_array(&id_fn, values);
        let len = sorted.len() as i32;

        match strategy {
            OutOfBoundsStrategy::Zero => {
                let zero_val = sorted[0].clone();
                Box::new(move |id: i32| {
                    if id >= 0 && id < len {
                        sorted[id as usize].clone()
                    } else {
                        zero_val.clone()
                    }
                })
            }
            OutOfBoundsStrategy::Wrap => Box::new(move |id: i32| {
                let idx = positive_modulo(id, len);
                sorted[idx as usize].clone()
            }),
            OutOfBoundsStrategy::Clamp => Box::new(move |id: i32| {
                let idx = clamp_i32(id, 0, len - 1);
                sorted[idx as usize].clone()
            }),
        }
    }
}

fn positive_modulo(x: i32, y: i32) -> i32 {
    ((x % y).wrapping_add(y)) % y
}

fn clamp_i32(v: i32, min: i32, max: i32) -> i32 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Val(i32, &'static str);

    fn id_fn(v: &Val) -> i32 {
        v.0
    }

    #[test]
    fn sparse_basic_lookup_and_fallback() {
        let values = vec![Val(1, "a"), Val(5, "b"), Val(10, "c")];
        let f = ByIdMap::sparse(id_fn, &values, Val(-1, "default"));
        assert_eq!(f(1), Val(1, "a"));
        assert_eq!(f(5), Val(5, "b"));
        assert_eq!(f(10), Val(10, "c"));
        assert_eq!(f(0), Val(-1, "default"));
        assert_eq!(f(99), Val(-1, "default"));
        assert_eq!(f(-5), Val(-1, "default"));
    }

    #[test]
    fn continuous_zero_wrap_clamp() {
        let values = vec![Val(0, "z"), Val(1, "o"), Val(2, "t")];
        let fz = ByIdMap::continuous(id_fn, &values, OutOfBoundsStrategy::Zero);
        let fw = ByIdMap::continuous(id_fn, &values, OutOfBoundsStrategy::Wrap);
        let fc = ByIdMap::continuous(id_fn, &values, OutOfBoundsStrategy::Clamp);

        assert_eq!(fz(-1), Val(0, "z"));
        assert_eq!(fz(3), Val(0, "z"));
        assert_eq!(fz(1), Val(1, "o"));

        assert_eq!(fw(-1), Val(2, "t"));
        assert_eq!(fw(-4), Val(2, "t"));
        assert_eq!(fw(3), Val(0, "z"));
        assert_eq!(fw(4), Val(1, "o"));

        assert_eq!(fc(-10), Val(0, "z"));
        assert_eq!(fc(10), Val(2, "t"));
        assert_eq!(fc(1), Val(1, "o"));
    }

    #[test]
    #[should_panic(expected = "Empty value list")]
    fn sparse_empty_panics() {
        let values: Vec<Val> = vec![];
        let _ = ByIdMap::sparse(id_fn, &values, Val(0, "d"));
    }

    #[test]
    #[should_panic(expected = "Duplicate entry on id")]
    fn sparse_duplicate_panics() {
        let values = vec![Val(1, "a"), Val(1, "b")];
        let _ = ByIdMap::sparse(id_fn, &values, Val(0, "d"));
    }

    #[test]
    #[should_panic(expected = "Values are not continous")]
    fn continuous_out_of_range_panics() {
        let values = vec![Val(0, "a"), Val(5, "b")];
        let _ = ByIdMap::continuous(id_fn, &values, OutOfBoundsStrategy::Zero);
    }

    #[test]
    #[should_panic(expected = "Duplicate entry on id")]
    fn continuous_duplicate_panics() {
        let values = vec![Val(0, "a"), Val(0, "b")];
        let _ = ByIdMap::continuous(id_fn, &values, OutOfBoundsStrategy::Clamp);
    }
}
