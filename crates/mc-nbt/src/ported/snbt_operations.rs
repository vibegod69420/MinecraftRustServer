// === FILE: snbt_operations.rs ===
//! Faithful port of `net.minecraft.nbt.SnbtOperations`.

use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::sync::OnceLock;

pub const BUILTIN_TRUE: &str = "true";
pub const BUILTIN_FALSE: &str = "false";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelayedParseException {
    pub message_key: &'static str,
}

pub const ERROR_EXPECTED_STRING_UUID: DelayedParseException = DelayedParseException {
    message_key: "snbt.parser.expected_string_uuid",
};

pub const ERROR_EXPECTED_NUMBER_OR_BOOLEAN: DelayedParseException = DelayedParseException {
    message_key: "snbt.parser.expected_number_or_boolean",
};

/// Minimal self-contained stand-in for `ParseState<StringReader>` used by SNBT operations.
#[derive(Debug, Clone, Default)]
pub struct ParseState {
    mark_pos: usize,
    errors: Vec<(usize, DelayedParseException)>,
}

impl ParseState {
    pub fn new(mark_pos: usize) -> Self {
        Self {
            mark_pos,
            errors: Vec::new(),
        }
    }

    pub fn mark(&self) -> usize {
        self.mark_pos
    }

    pub fn store_error(&mut self, mark: usize, error: DelayedParseException) {
        self.errors.push((mark, error));
    }

    pub fn errors(&self) -> &[(usize, DelayedParseException)] {
        &self.errors
    }
}

/// Minimal self-contained stand-in for Mojang `DynamicOps<T>`.
pub trait DynamicOps<T> {
    fn get_boolean_value(&self, value: &T) -> Option<bool>;
    fn get_number_value_f64(&self, value: &T) -> Option<f64>;
    fn get_string_value(&self, value: &T) -> Option<String>;
    fn create_boolean(&self, value: bool) -> T;
    fn create_int_list(&self, values: Vec<i32>) -> T;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuiltinKey {
    pub id: String,
    pub arg_count: i32,
}

impl fmt::Display for BuiltinKey {
    // JAVA: net.minecraft.nbt.SnbtOperations#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.id, self.arg_count)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinOperationKind {
    Bool,
    Uuid,
}

// JAVA: net.minecraft.nbt.SnbtOperations#convert
fn bool_convert<T, Ops: DynamicOps<T>>(ops: &Ops, value: &T) -> Option<bool> {
    if let Some(b) = ops.get_boolean_value(value) {
        return Some(b);
    }
    if let Some(n) = ops.get_number_value_f64(value) {
        return Some(n != 0.0);
    }
    None
}

// JAVA: net.minecraft.nbt.SnbtOperations#run
fn bool_run<T, Ops: DynamicOps<T>>(ops: &Ops, args: &[T], state: &mut ParseState) -> Option<T> {
    let arg = args.first()?;
    match bool_convert(ops, arg) {
        Some(b) => Some(ops.create_boolean(b)),
        None => {
            let mark = state.mark();
            state.store_error(mark, ERROR_EXPECTED_NUMBER_OR_BOOLEAN.clone());
            None
        }
    }
}

fn uuid_to_int_array(most: i64, least: i64) -> [i32; 4] {
    [
        most.wrapping_shr(32) as i32,
        most as i32,
        least.wrapping_shr(32) as i32,
        least as i32,
    ]
}

fn parse_uuid(s: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return None;
    }
    let (p0, p1, p2, p3, p4) = (parts[0], parts[1], parts[2], parts[3], parts[4]);
    if p0.len() != 8 || p1.len() != 4 || p2.len() != 4 || p3.len() != 4 || p4.len() != 12 {
        return None;
    }

    let most_hi = u64::from_str_radix(p0, 16).ok()?;
    let most_mid = u64::from_str_radix(p1, 16).ok()?;
    let most_lo = u64::from_str_radix(p2, 16).ok()?;
    let least_hi = u64::from_str_radix(p3, 16).ok()?;
    let least_lo = u64::from_str_radix(p4, 16).ok()?;

    let most = ((most_hi << 32) | (most_mid << 16) | most_lo) as i64;
    let least = ((least_hi << 48) | least_lo) as i64;
    Some((most, least))
}

// JAVA: net.minecraft.nbt.SnbtOperations#run
fn uuid_run<T, Ops: DynamicOps<T>>(ops: &Ops, args: &[T], state: &mut ParseState) -> Option<T> {
    let arg = args.first()?;
    let s = match ops.get_string_value(arg) {
        Some(s) => s,
        None => {
            let mark = state.mark();
            state.store_error(mark, ERROR_EXPECTED_STRING_UUID.clone());
            return None;
        }
    };

    let (most, least) = match parse_uuid(&s) {
        Some(v) => v,
        None => {
            let mark = state.mark();
            state.store_error(mark, ERROR_EXPECTED_STRING_UUID.clone());
            return None;
        }
    };

    let ints = uuid_to_int_array(most, least);
    Some(ops.create_int_list(ints.to_vec()))
}

impl BuiltinOperationKind {
    pub fn run<T, Ops: DynamicOps<T>>(
        &self,
        ops: &Ops,
        args: &[T],
        state: &mut ParseState,
    ) -> Option<T> {
        match self {
            BuiltinOperationKind::Bool => bool_run(ops, args, state),
            BuiltinOperationKind::Uuid => uuid_run(ops, args, state),
        }
    }
}

pub fn builtin_operations() -> &'static HashMap<BuiltinKey, BuiltinOperationKind> {
    static OPS: OnceLock<HashMap<BuiltinKey, BuiltinOperationKind>> = OnceLock::new();
    OPS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(
            BuiltinKey {
                id: "bool".to_string(),
                arg_count: 1,
            },
            BuiltinOperationKind::Bool,
        );
        m.insert(
            BuiltinKey {
                id: "uuid".to_string(),
                arg_count: 1,
            },
            BuiltinOperationKind::Uuid,
        );
        m
    })
}

// JAVA: net.minecraft.nbt.SnbtOperations#possibleValues
pub fn builtin_ids_possible_values(_state: &ParseState) -> Vec<String> {
    let mut keys: BTreeSet<String> = BTreeSet::new();
    keys.insert(BUILTIN_FALSE.to_string());
    keys.insert(BUILTIN_TRUE.to_string());
    for k in builtin_operations().keys() {
        keys.insert(k.id.clone());
    }
    keys.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestVal {
        Bool(bool),
        Int(i32),
        Long(i64),
        Float(f32),
        Double(f64),
        String(String),
        IntList(Vec<i32>),
        Other,
    }

    struct MockOps;

    impl DynamicOps<TestVal> for MockOps {
        fn get_boolean_value(&self, value: &TestVal) -> Option<bool> {
            match value {
                TestVal::Bool(b) => Some(*b),
                _ => None,
            }
        }

        fn get_number_value_f64(&self, value: &TestVal) -> Option<f64> {
            match value {
                TestVal::Int(v) => Some(f64::from(*v)),
                TestVal::Long(v) => Some(*v as f64),
                TestVal::Float(v) => Some(f64::from(*v)),
                TestVal::Double(v) => Some(*v),
                _ => None,
            }
        }

        fn get_string_value(&self, value: &TestVal) -> Option<String> {
            match value {
                TestVal::String(s) => Some(s.clone()),
                _ => None,
            }
        }

        fn create_boolean(&self, value: bool) -> TestVal {
            TestVal::Bool(value)
        }

        fn create_int_list(&self, values: Vec<i32>) -> TestVal {
            TestVal::IntList(values)
        }
    }

    #[test]
    fn bool_builtin_boolean_and_negative_numbers() {
        let ops = MockOps;
        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Bool.run(&ops, &[TestVal::Bool(true)], &mut state);
        assert_eq!(r, Some(TestVal::Bool(true)));
        assert!(state.errors().is_empty());

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Bool.run(&ops, &[TestVal::Int(0)], &mut state);
        assert_eq!(r, Some(TestVal::Bool(false)));

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Bool.run(&ops, &[TestVal::Int(-5)], &mut state);
        assert_eq!(r, Some(TestVal::Bool(true)));

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Bool.run(&ops, &[TestVal::Long(i64::MIN)], &mut state);
        assert_eq!(r, Some(TestVal::Bool(true)));
    }

    #[test]
    fn bool_builtin_float_bits_edge_cases() {
        let ops = MockOps;

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Bool.run(&ops, &[TestVal::Double(f64::NAN)], &mut state);
        assert_eq!(r, Some(TestVal::Bool(true))); // NaN != 0.0

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Bool.run(&ops, &[TestVal::Double(-0.0_f64)], &mut state);
        assert_eq!(r, Some(TestVal::Bool(false))); // -0.0 == 0.0

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Bool.run(
            &ops,
            &[TestVal::Float(f32::NEG_INFINITY)],
            &mut state,
        );
        assert_eq!(r, Some(TestVal::Bool(true)));

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Bool.run(&ops, &[TestVal::Float(-0.0_f32)], &mut state);
        assert_eq!(r, Some(TestVal::Bool(false)));
    }

    #[test]
    fn bool_builtin_error_on_invalid_type() {
        let ops = MockOps;
        let mut state = ParseState::new(42);
        let r = BuiltinOperationKind::Bool.run(
            &ops,
            &[TestVal::String("true".to_string())],
            &mut state,
        );
        assert_eq!(r, None);
        assert_eq!(state.errors().len(), 1);
        assert_eq!(state.errors()[0].0, 42);
        assert_eq!(state.errors()[0].1, ERROR_EXPECTED_NUMBER_OR_BOOLEAN);

        let mut state = ParseState::new(7);
        let r = BuiltinOperationKind::Bool.run(&ops, &[TestVal::Other], &mut state);
        assert_eq!(r, None);
        assert_eq!(state.errors()[0].1, ERROR_EXPECTED_NUMBER_OR_BOOLEAN);
    }

    #[test]
    fn uuid_builtin_valid_and_negative_int_overflow() {
        let ops = MockOps;

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Uuid.run(
            &ops,
            &[TestVal::String(
                "ffffffff-ffff-ffff-ffff-ffffffffffff".to_string(),
            )],
            &mut state,
        );
        assert_eq!(r, Some(TestVal::IntList(vec![-1, -1, -1, -1])));

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Uuid.run(
            &ops,
            &[TestVal::String(
                "80000000-0000-0000-8000-000000000000".to_string(),
            )],
            &mut state,
        );
        assert_eq!(r, Some(TestVal::IntList(vec![i32::MIN, 0, i32::MIN, 0])));

        let mut state = ParseState::new(0);
        let r = BuiltinOperationKind::Uuid.run(
            &ops,
            &[TestVal::String(
                "00000001-0002-0003-0004-000000000005".to_string(),
            )],
            &mut state,
        );
        assert_eq!(
            r,
            Some(TestVal::IntList(vec![1, 0x0002_0003, 0x0004_0000, 5]))
        );
    }

    #[test]
    fn uuid_builtin_invalid_input_reports_error() {
        let ops = MockOps;

        let mut state = ParseState::new(7);
        let r = BuiltinOperationKind::Uuid.run(&ops, &[TestVal::Int(1)], &mut state);
        assert_eq!(r, None);
        assert_eq!(state.errors().len(), 1);
        assert_eq!(state.errors()[0].0, 7);
        assert_eq!(state.errors()[0].1, ERROR_EXPECTED_STRING_UUID);

        let mut state = ParseState::new(8);
        let r = BuiltinOperationKind::Uuid.run(
            &ops,
            &[TestVal::String("not-a-uuid".to_string())],
            &mut state,
        );
        assert_eq!(r, None);
        assert_eq!(state.errors().len(), 1);
        assert_eq!(state.errors()[0].1, ERROR_EXPECTED_STRING_UUID);
    }

    #[test]
    fn builtin_key_display_operations_and_possible_values() {
        let key = BuiltinKey {
            id: "bool".to_string(),
            arg_count: 1,
        };
        assert_eq!(key.to_string(), "bool/1");

        let ops = builtin_operations();
        assert_eq!(
            ops.get(&BuiltinKey {
                id: "bool".to_string(),
                arg_count: 1
            }),
            Some(&BuiltinOperationKind::Bool)
        );
        assert_eq!(
            ops.get(&BuiltinKey {
                id: "uuid".to_string(),
                arg_count: 1
            }),
            Some(&BuiltinOperationKind::Uuid)
        );

        let state = ParseState::new(0);
        let ids = builtin_ids_possible_values(&state);
        assert!(ids.contains(&"true".to_string()));
        assert!(ids.contains(&"false".to_string()));
        assert!(ids.contains(&"bool".to_string()));
        assert!(ids.contains(&"uuid".to_string()));
    }
}
