//! JAVA: net.minecraft.util.GsonHelper
//! Self-contained port of GsonHelper JSON utility methods (minimal JsonValue model).

use std::collections::HashMap;
use std::fmt;

/// JAVA: net.minecraft.util.GsonHelper.JsonSyntaxException (mapped)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonSyntaxError(pub String);

impl fmt::Display for JsonSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for JsonSyntaxError {}

/// Minimal JSON number representation (faithful enough for GsonHelper numeric checks/conversions).
#[derive(Clone, Debug, PartialEq)]
pub enum JsonNumber {
    I64(i64),
    U64(u64),
    F64(f64),
}

/// Minimal JSON value model.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

pub type JsonObjectMap = HashMap<String, JsonValue>;

/// JAVA: net.minecraft.util.GsonHelper#getType
pub fn get_type(element: Option<&JsonValue>) -> String {
    match element {
        None => "null (missing)".to_string(),
        Some(JsonValue::Null) => "null (json)".to_string(),
        Some(JsonValue::Array(v)) => {
            let s = format!("{:?}", v);
            let abbr = abbreviate_middle(&s, "...", 10);
            format!("an array ({})", abbr)
        }
        Some(JsonValue::Object(m)) => {
            let s = format!("{:?}", m);
            let abbr = abbreviate_middle(&s, "...", 10);
            format!("an object ({})", abbr)
        }
        Some(JsonValue::Bool(b)) => {
            let s = format!("{}", b);
            let abbr = abbreviate_middle(&s, "...", 10);
            format!("a boolean ({})", abbr)
        }
        Some(JsonValue::Number(n)) => {
            let s = format!("{:?}", n);
            let abbr = abbreviate_middle(&s, "...", 10);
            format!("a number ({})", abbr)
        }
        Some(JsonValue::String(sv)) => {
            let abbr = abbreviate_middle(sv, "...", 10);
            abbr
        }
    }
}

fn abbreviate_middle(s: &str, middle: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    if max_len <= middle.chars().count() {
        return middle.chars().take(max_len).collect();
    }
    let chars: Vec<char> = s.chars().collect();
    let keep = max_len - middle.chars().count();
    let left = keep / 2;
    let right = keep - left;
    let mut out = String::new();
    for c in chars.iter().take(left) {
        out.push(*c);
    }
    out.push_str(middle);
    for c in chars.iter().skip(chars.len() - right) {
        out.push(*c);
    }
    out
}

fn is_json_primitive(v: &JsonValue) -> bool {
    matches!(
        v,
        JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_)
    )
}

fn as_primitive(v: &JsonValue) -> Option<&JsonValue> {
    if is_json_primitive(v) {
        Some(v)
    } else {
        None
    }
}

fn number_is_number(_n: &JsonNumber) -> bool {
    true
}

fn number_as_f64(n: &JsonNumber) -> f64 {
    match *n {
        JsonNumber::F64(x) => x,
        JsonNumber::I64(x) => x as f64,
        JsonNumber::U64(x) => x as f64,
    }
}

fn number_as_i64(n: &JsonNumber) -> i64 {
    match *n {
        JsonNumber::F64(x) => x as i64,
        JsonNumber::I64(x) => x,
        JsonNumber::U64(x) => x as i64,
    }
}

fn number_as_u64(n: &JsonNumber) -> u64 {
    match *n {
        JsonNumber::F64(x) => x as u64,
        JsonNumber::I64(x) => x as u64,
        JsonNumber::U64(x) => x,
    }
}

fn number_as_i32(n: &JsonNumber) -> i32 {
    match *n {
        JsonNumber::F64(x) => x as i32,
        JsonNumber::I64(x) => x as i32,
        JsonNumber::U64(x) => x as i32,
    }
}

fn number_as_i16(n: &JsonNumber) -> i16 {
    match *n {
        JsonNumber::F64(x) => x as i16,
        JsonNumber::I64(x) => x as i16,
        JsonNumber::U64(x) => x as i16,
    }
}

fn number_as_i8(n: &JsonNumber) -> i8 {
    match *n {
        JsonNumber::F64(x) => x as i8,
        JsonNumber::I64(x) => x as i8,
        JsonNumber::U64(x) => x as i8,
    }
}

fn number_as_u8(n: &JsonNumber) -> u8 {
    match *n {
        JsonNumber::F64(x) => x as u8,
        JsonNumber::I64(x) => x as u8,
        JsonNumber::U64(x) => x as u8,
    }
}

fn number_as_u16(n: &JsonNumber) -> u16 {
    match *n {
        JsonNumber::F64(x) => x as u16,
        JsonNumber::I64(x) => x as u16,
        JsonNumber::U64(x) => x as u16,
    }
}

fn number_as_u32(n: &JsonNumber) -> u32 {
    match *n {
        JsonNumber::F64(x) => x as u32,
        JsonNumber::I64(x) => x as u32,
        JsonNumber::U64(x) => x as u32,
    }
}

// JAVA: net.minecraft.util.GsonHelper#isStringValue
pub fn is_string_value_obj(obj: &HashMap<String, JsonValue>, member_name: &str) -> bool {
    match obj.get(member_name) {
        Some(v) => is_string_value_elem(v),
        None => false,
    }
}

// JAVA: net.minecraft.util.GsonHelper#isStringValue
pub fn is_string_value_elem(element: &JsonValue) -> bool {
    matches!(element, JsonValue::String(_))
}

// JAVA: net.minecraft.util.GsonHelper#isNumberValue
pub fn is_number_value_obj(obj: &HashMap<String, JsonValue>, member_name: &str) -> bool {
    match obj.get(member_name) {
        Some(v) => is_number_value_elem(v),
        None => false,
    }
}

// JAVA: net.minecraft.util.GsonHelper#isNumberValue
pub fn is_number_value_elem(element: &JsonValue) -> bool {
    matches!(element, JsonValue::Number(_))
}

// JAVA: net.minecraft.util.GsonHelper#isBooleanValue
pub fn is_boolean_value_obj(obj: &HashMap<String, JsonValue>, member_name: &str) -> bool {
    match obj.get(member_name) {
        Some(v) => is_boolean_value_elem(v),
        None => false,
    }
}

// JAVA: net.minecraft.util.GsonHelper#isBooleanValue
pub fn is_boolean_value_elem(element: &JsonValue) -> bool {
    matches!(element, JsonValue::Bool(_))
}

// JAVA: net.minecraft.util.GsonHelper#isArrayNode
pub fn is_array_node(obj: &HashMap<String, JsonValue>, member_name: &str) -> bool {
    match obj.get(member_name) {
        Some(JsonValue::Array(_)) => true,
        _ => false,
    }
}

// JAVA: net.minecraft.util.GsonHelper#isObjectNode
pub fn is_object_node(obj: &HashMap<String, JsonValue>, member_name: &str) -> bool {
    match obj.get(member_name) {
        Some(JsonValue::Object(_)) => true,
        _ => false,
    }
}

// JAVA: net.minecraft.util.GsonHelper#isValidPrimitive
pub fn is_valid_primitive(obj: &HashMap<String, JsonValue>, member_name: &str) -> bool {
    match obj.get(member_name) {
        Some(v) => is_json_primitive(v),
        None => false,
    }
}

// JAVA: net.minecraft.util.GsonHelper#isValidNode
pub fn is_valid_node(obj: Option<&HashMap<String, JsonValue>>, member_name: &str) -> bool {
    match obj {
        None => false,
        Some(m) => m.get(member_name).is_some(),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getNonNull
pub fn get_non_null<'a>(
    obj: &'a HashMap<String, JsonValue>,
    member_name: &str,
) -> Result<&'a JsonValue, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(JsonValue::Null) | None => Err(JsonSyntaxError(format!("Missing field {}", member_name))),
        Some(v) => Ok(v),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToString
pub fn convert_to_string(element: &JsonValue, member_name: &str) -> Result<String, JsonSyntaxError> {
    match element {
        JsonValue::String(s) => Ok(s.clone()),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a string, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsString
pub fn get_as_string(
    obj: &HashMap<String, JsonValue>,
    member_name: &str,
) -> Result<String, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_string(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a string",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsString (overload with default)
pub fn get_as_string_or(
    obj: &HashMap<String, JsonValue>,
    member_name: &str,
    default: Option<String>,
) -> Result<Option<String>, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => Ok(Some(convert_to_string(v, member_name)?)),
        None => Ok(default),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToBoolean
pub fn convert_to_boolean(element: &JsonValue, member_name: &str) -> Result<bool, JsonSyntaxError> {
    match element {
        JsonValue::Bool(b) => Ok(*b),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a Boolean, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsBoolean
pub fn get_as_boolean(obj: &HashMap<String, JsonValue>, member_name: &str) -> Result<bool, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_boolean(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a Boolean",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsBoolean (with default)
pub fn get_as_boolean_or(
    obj: &HashMap<String, JsonValue>,
    member_name: &str,
    default: bool,
) -> Result<bool, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_boolean(v, member_name),
        None => Ok(default),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToDouble
pub fn convert_to_double(element: &JsonValue, member_name: &str) -> Result<f64, JsonSyntaxError> {
    match element {
        JsonValue::Number(n) => Ok(number_as_f64(n)),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a Double, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsDouble
pub fn get_as_double(obj: &HashMap<String, JsonValue>, member_name: &str) -> Result<f64, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_double(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a Double",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsDouble (with default)
pub fn get_as_double_or(
    obj: &HashMap<String, JsonValue>,
    member_name: &str,
    default: f64,
) -> Result<f64, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_double(v, member_name),
        None => Ok(default),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToFloat
pub fn convert_to_float(element: &JsonValue, member_name: &str) -> Result<f32, JsonSyntaxError> {
    match element {
        JsonValue::Number(n) => Ok(number_as_f64(n) as f32),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a Float, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsFloat
pub fn get_as_float(obj: &HashMap<String, JsonValue>, member_name: &str) -> Result<f32, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_float(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a Float",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsFloat (with default)
pub fn get_as_float_or(
    obj: &HashMap<String, JsonValue>,
    member_name: &str,
    default: f32,
) -> Result<f32, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_float(v, member_name),
        None => Ok(default),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToLong
pub fn convert_to_long(element: &JsonValue, member_name: &str) -> Result<i64, JsonSyntaxError> {
    match element {
        JsonValue::Number(n) => Ok(number_as_i64(n)),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a Long, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsLong
pub fn get_as_long(obj: &HashMap<String, JsonValue>, member_name: &str) -> Result<i64, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_long(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a Long",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsLong (with default)
pub fn get_as_long_or(
    obj: &HashMap<String, JsonValue>,
    member_name: &str,
    default: i64,
) -> Result<i64, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_long(v, member_name),
        None => Ok(default),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToInt
pub fn convert_to_int(element: &JsonValue, member_name: &str) -> Result<i32, JsonSyntaxError> {
    match element {
        JsonValue::Number(n) => Ok(number_as_i32(n)),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a Int, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsInt
pub fn get_as_int(obj: &HashMap<String, JsonValue>, member_name: &str) -> Result<i32, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_int(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a Int",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsInt (with default)
pub fn get_as_int_or(
    obj: &HashMap<String, JsonValue>,
    member_name: &str,
    default: i32,
) -> Result<i32, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_int(v, member_name),
        None => Ok(default),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToByte
pub fn convert_to_byte(element: &JsonValue, member_name: &str) -> Result<i8, JsonSyntaxError> {
    match element {
        JsonValue::Number(n) => Ok(number_as_i8(n)),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a Byte, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsByte
pub fn get_as_byte(obj: &HashMap<String, JsonValue>, member_name: &str) -> Result<i8, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_byte(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a Byte",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsByte (with default)
pub fn get_as_byte_or(
    obj: &HashMap<String, JsonValue>,
    member_name: &str,
    default: i8,
) -> Result<i8, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_byte(v, member_name),
        None => Ok(default),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToShort
pub fn convert_to_short(element: &JsonValue, member_name: &str) -> Result<i16, JsonSyntaxError> {
    match element {
        JsonValue::Number(n) => Ok(number_as_i16(n)),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a Short, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsShort
pub fn get_as_short(obj: &HashMap<String, JsonValue>, member_name: &str) -> Result<i16, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_short(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a Short",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsShort (with default)
pub fn get_as_short_or(
    obj: &HashMap<String, JsonValue>,
    member_name: &str,
    default: i16,
) -> Result<i16, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_short(v, member_name),
        None => Ok(default),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToJsonObject
pub fn convert_to_json_object<'a>(
    element: &'a JsonValue,
    member_name: &str,
) -> Result<&'a HashMap<String, JsonValue>, JsonSyntaxError> {
    match element {
        JsonValue::Object(m) => Ok(m),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a JsonObject, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsJsonObject
pub fn get_as_json_object<'a>(
    obj: &'a HashMap<String, JsonValue>,
    member_name: &str,
) -> Result<&'a HashMap<String, JsonValue>, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_json_object(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a JsonObject",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsJsonObject (with default)
pub fn get_as_json_object_or<'a>(
    obj: &'a HashMap<String, JsonValue>,
    member_name: &str,
    default: Option<&'a HashMap<String, JsonValue>>,
) -> Result<Option<&'a HashMap<String, JsonValue>>, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => Ok(Some(convert_to_json_object(v, member_name)?)),
        None => Ok(default),
    }
}

// JAVA: net.minecraft.util.GsonHelper#convertToJsonArray
pub fn convert_to_json_array<'a>(
    element: &'a JsonValue,
    member_name: &str,
) -> Result<&'a Vec<JsonValue>, JsonSyntaxError> {
    match element {
        JsonValue::Array(a) => Ok(a),
        _ => Err(JsonSyntaxError(format!(
            "Expected {} to be a JsonArray, was {}",
            member_name,
            get_type(Some(element))
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsJsonArray
pub fn get_as_json_array<'a>(
    obj: &'a HashMap<String, JsonValue>,
    member_name: &str,
) -> Result<&'a Vec<JsonValue>, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => convert_to_json_array(v, member_name),
        None => Err(JsonSyntaxError(format!(
            "Missing {}, expected to find a JsonArray",
            member_name
        ))),
    }
}

// JAVA: net.minecraft.util.GsonHelper#getAsJsonArray (with default)
pub fn get_as_json_array_or<'a>(
    obj: &'a HashMap<String, JsonValue>,
    member_name: &str,
    default: Option<&'a Vec<JsonValue>>,
) -> Result<Option<&'a Vec<JsonValue>>, JsonSyntaxError> {
    match obj.get(member_name) {
        Some(v) => Ok(Some(convert_to_json_array(v, member_name)?)),
        None => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(pairs: Vec<(&str, JsonValue)>) -> HashMap<String, JsonValue> {
        pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
    }

    #[test]
    fn test_is_valid_node_and_primitives() {
        let m = obj(vec![
            ("s", JsonValue::String("hi".to_string())),
            ("n", JsonValue::Number(JsonNumber::I64(1))),
            ("b", JsonValue::Bool(true)),
            ("arr", JsonValue::Array(vec![])),
        ]);
        assert!(is_valid_node(Some(&m), "s"));
        assert!(!is_valid_node(Some(&m), "nope"));
        assert!(!is_valid_node(None, "s"));
        assert!(is_string_value_obj(&m, "s"));
        assert!(!is_string_value_obj(&m, "n"));
        assert!(is_number_value_obj(&m, "n"));
        assert!(is_boolean_value_obj(&m, "b"));
        assert!(is_array_node(&m, "arr"));
        assert!(!is_object_node(&m, "arr"));
    }

    #[test]
    fn test_get_as_string_and_defaults() {
        let m = obj(vec![("name", JsonValue::String("Steve".to_string()))]);
        assert_eq!(get_as_string(&m, "name").unwrap(), "Steve");
        let err = get_as_string(&m, "missing").unwrap_err();
        assert!(err.0.contains("Missing"));
        assert_eq!(
            get_as_string_or(&m, "missing", Some("def".to_string())).unwrap(),
            Some("def".to_string())
        );
    }

    #[test]
    fn test_convert_numbers_and_errors() {
        let m = obj(vec![
            ("i", JsonValue::Number(JsonNumber::I64(-5))),
            ("f", JsonValue::Number(JsonNumber::F64(1.5))),
        ]);
        assert_eq!(get_as_int(&m, "i").unwrap(), -5);
        assert_eq!(get_as_long(&m, "i").unwrap(), -5);
        let fv = get_as_float(&m, "f").unwrap();
        assert!((fv - 1.5).abs() < 1e-6);
        let dv = get_as_double(&m, "f").unwrap();
        assert!((dv - 1.5).abs() < 1e-12);
        // wrong type
        let bad = obj(vec![("s", JsonValue::String("x".to_string()))]);
        assert!(get_as_int(&bad, "s").is_err());
    }

    #[test]
    fn test_objects_arrays_conversion() {
        let inner = obj(vec![("k", JsonValue::Number(JsonNumber::I64(7)))]);
        let m = obj(vec![
            ("o", JsonValue::Object(inner.clone())),
            ("a", JsonValue::Array(vec![JsonValue::Bool(true)])),
        ]);
        let oref = get_as_json_object(&m, "o").unwrap();
        assert!(oref.contains_key("k"));
        let aref = get_as_json_array(&m, "a").unwrap();
        assert_eq!(aref.len(), 1);
        assert!(get_as_json_object(&m, "a").is_err());
    }

    #[test]
    fn test_get_non_null_and_type() {
        let m = obj(vec![("x", JsonValue::Null), ("y", JsonValue::String("z".to_string()))]);
        assert!(get_non_null(&m, "x").is_err());
        assert!(get_non_null(&m, "no").is_err());
        assert_eq!(
            convert_to_string(get_non_null(&m, "y").unwrap(), "y").unwrap(),
            "z"
        );
        assert!(get_type(None).contains("missing"));
        assert!(get_type(Some(&JsonValue::Null)).contains("null"));
    }
}
