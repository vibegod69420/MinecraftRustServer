use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

pub type DataResult<T> = Result<T, String>;

pub const HEX_COLOR_PREFIX: &str = "#";
pub const MAX_PROPERTY_NAME_LENGTH: usize = 64;
pub const MAX_PROPERTY_VALUE_LENGTH: usize = 32767;
pub const MAX_PROPERTY_SIGNATURE_LENGTH: usize = 1024;
pub const MAX_PROPERTIES: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagOrElementLocation {
    pub id: String,
    pub tag: bool,
}

impl TagOrElementLocation {
    // JAVA: net.minecraft.util.ExtraCodecs.TagOrElementLocation#decoratedId
    pub fn decorated_id(&self) -> String {
        if self.tag {
            format!("#{}", self.id)
        } else {
            self.id.clone()
        }
    }
}

impl fmt::Display for TagOrElementLocation {
    // JAVA: net.minecraft.util.ExtraCodecs.TagOrElementLocation#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.decorated_id())
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#hexColor
pub fn parse_hex_color(input: &str, digits: u32) -> DataResult<i32> {
    if !input.starts_with(HEX_COLOR_PREFIX) {
        return Err("Hex color must begin with #".to_string());
    }
    let prefix_len = HEX_COLOR_PREFIX.len();
    let value_len = input.len().saturating_sub(prefix_len);
    let expected = digits as usize;
    if value_len != expected {
        return Err(format!(
            "Hex color is wrong size, expected {} digits but got {}",
            digits, value_len
        ));
    }
    let hex_part = &input[prefix_len..];
    let max_value: u64 = if digits.wrapping_mul(4) >= 64 {
        u64::MAX
    } else {
        (1u64 << (digits.wrapping_mul(4) as u32)) - 1u64
    };
    match u64::from_str_radix(hex_part, 16) {
        Ok(parsed) if parsed <= max_value => Ok(parsed as i32),
        Ok(_) => Err(format!("Color value out of range: {}", input)),
        Err(_) => Err(format!("Invalid color value: {}", input)),
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#hexColor
pub fn format_hex_color(value: i32, digits: u32) -> String {
    let digits_usize = digits as usize;
    format!("{}{:0width$x}", HEX_COLOR_PREFIX, value as u32, width = digits_usize)
}

// JAVA: net.minecraft.util.ExtraCodecs#intRangeWithMessage
pub fn int_range_with_message<F>(min: i32, max: i32, message: F, value: i32) -> DataResult<i32>
where
    F: Fn(i32) -> String,
{
    if value >= min && value <= max {
        Ok(value)
    } else {
        Err(message(value))
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#intRange
pub fn int_range(min: i32, max: i32, value: i32) -> DataResult<i32> {
    int_range_with_message(min, max, |v| {
        format!("Value must be within range [{};{}]: {}", min, max, v)
    }, value)
}

// JAVA: net.minecraft.util.ExtraCodecs#longRangeWithMessage
pub fn long_range_with_message<F>(min: i64, max: i64, message: F, value: i64) -> DataResult<i64>
where
    F: Fn(i64) -> String,
{
    if value >= min && value <= max {
        Ok(value)
    } else {
        Err(message(value))
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#longRange
pub fn long_range(min: i64, max: i64, value: i64) -> DataResult<i64> {
    long_range_with_message(min, max, |v| {
        format!("Value must be within range [{};{}]: {}", min, max, v)
    }, value)
}

// JAVA: net.minecraft.util.ExtraCodecs#floatRangeMinInclusiveWithMessage
pub fn float_range_min_inclusive_with_message<F>(
    min: f32,
    max: f32,
    message: F,
    value: f32,
) -> DataResult<f32>
where
    F: Fn(f32) -> String,
{
    // Java Float.compareTo: -0.0 < 0.0, NaN == NaN and NaN > all other floats
    let min_ok = value.is_nan() || min.is_nan() || value >= min;
    let min_ok = if value.is_nan() && min.is_nan() {
        true
    } else if value.is_nan() {
        true
    } else if min.is_nan() {
        false
    } else {
        value.total_cmp(&min) != std::cmp::Ordering::Less
    };
    let max_ok = if value.is_nan() && max.is_nan() {
        true
    } else if value.is_nan() {
        true
    } else if max.is_nan() {
        false
    } else {
        value.total_cmp(&max) != std::cmp::Ordering::Greater
    };
    if min_ok && max_ok {
        Ok(value)
    } else {
        Err(message(value))
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#floatRangeMinExclusiveWithMessage
pub fn float_range_min_exclusive_with_message<F>(
    min: f32,
    max: f32,
    message: F,
    value: f32,
) -> DataResult<f32>
where
    F: Fn(f32) -> String,
{
    let min_ok = if value.is_nan() && min.is_nan() {
        false
    } else if value.is_nan() {
        true
    } else if min.is_nan() {
        false
    } else {
        value.total_cmp(&min) == std::cmp::Ordering::Greater
    };
    let max_ok = if value.is_nan() && max.is_nan() {
        true
    } else if value.is_nan() {
        true
    } else if max.is_nan() {
        false
    } else {
        value.total_cmp(&max) != std::cmp::Ordering::Greater
    };
    if min_ok && max_ok {
        Ok(value)
    } else {
        Err(message(value))
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#floatRange
pub fn float_range(min: f32, max: f32, value: f32) -> DataResult<f32> {
    float_range_min_inclusive_with_message(min, max, |v| {
        format!("Value must be within range [{};{}]: {}", min, max, v)
    }, value)
}

// JAVA: net.minecraft.util.ExtraCodecs#toOptionalLong
pub fn to_optional_long(value: Option<i64>) -> Option<i64> {
    value
}

// JAVA: net.minecraft.util.ExtraCodecs#fromOptionalLong
pub fn from_optional_long(value: Option<i64>) -> Option<i64> {
    value
}

// JAVA: net.minecraft.util.ExtraCodecs#idResolverCodec
pub fn id_resolver_from_int<E: Clone + fmt::Debug>(
    id_to_value: impl Fn(i32) -> Option<E>,
    value_to_id: impl Fn(&E) -> i32,
    unknown_id: i32,
    value: &E,
) -> DataResult<i32> {
    let id = value_to_id(value);
    if id == unknown_id {
        Err(format!("Element with unknown id: {:?}", value))
    } else {
        Ok(id)
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#idResolverCodec
pub fn id_resolver_to_value<E: Clone + fmt::Debug>(
    id_to_value: impl Fn(i32) -> Option<E>,
    id: i32,
) -> DataResult<E> {
    match id_to_value(id) {
        Some(v) => Ok(v),
        None => Err(format!("Unknown element id: {}", id)),
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#ensureHomogenous
pub fn ensure_homogenous<E: fmt::Debug, T: PartialEq + fmt::Debug, F: Fn(&E) -> T>(
    items: &[E],
    type_fn: F,
) -> DataResult<()> {
    let mut iter = items.iter();
    if let Some(first) = iter.next() {
        let first_type = type_fn(first);
        for item in iter {
            let item_type = type_fn(item);
            if item_type != first_type {
                return Err(format!(
                    "Mixed type list: element {:?} had type {:?}, but list is of type {:?}",
                    item, item_type, first_type
                ));
            }
        }
    }
    Ok(())
}

pub struct LateBoundIdMapper<I, V> {
    id_to_value: HashMap<I, V>,
    value_to_id: HashMap<V, I>,
}

impl<I, V> LateBoundIdMapper<I, V>
where
    I: Clone + Eq + Hash + fmt::Debug,
    V: Clone + Eq + Hash + fmt::Debug,
{
    // JAVA: net.minecraft.util.ExtraCodecs.LateBoundIdMapper#LateBoundIdMapper
    pub fn new() -> Self {
        Self {
            id_to_value: HashMap::new(),
            value_to_id: HashMap::new(),
        }
    }

    // JAVA: net.minecraft.util.ExtraCodecs.LateBoundIdMapper#put
    pub fn put(&mut self, id: I, value: V) -> &mut Self {
        self.value_to_id.insert(value.clone(), id.clone());
        self.id_to_value.insert(id, value);
        self
    }

    // JAVA: net.minecraft.util.ExtraCodecs.LateBoundIdMapper#codec
    pub fn get_by_id(&self, id: &I) -> Option<&V> {
        self.id_to_value.get(id)
    }

    // JAVA: net.minecraft.util.ExtraCodecs.LateBoundIdMapper#codec
    pub fn get_id_by_value(&self, value: &V) -> Option<&I> {
        self.value_to_id.get(value)
    }

    // JAVA: net.minecraft.util.ExtraCodecs.LateBoundIdMapper#values
    pub fn values(&self) -> Vec<V> {
        self.id_to_value.values().cloned().collect()
    }
}

impl<I, V> Default for LateBoundIdMapper<I, V>
where
    I: Clone + Eq + Hash + fmt::Debug,
    V: Clone + Eq + Hash + fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#orElsePartial
pub fn or_else_partial<T: Clone>(default: T, result: DataResult<T>) -> DataResult<T> {
    match result {
        Ok(v) => Ok(v),
        Err(err) => Err(format!("({} -> using default)", err)),
    }
}

// JAVA: net.minecraft.util.ExtraCodecs#orElsePartial
pub fn or_else_partial_value<T: Clone>(default: T, result: DataResult<T>) -> T {
    match result {
        Ok(v) => v,
        Err(_) => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_color_parse_and_format() {
        assert_eq!(parse_hex_color("#FF00AA", 6).unwrap(), 0xFF00AA_i32);
        assert_eq!(parse_hex_color("#ffffffff", 8).unwrap(), -1_i32);
        assert_eq!(format_hex_color(0x00FF00AA_u32 as i32, 8), "#00ff00aa");
        assert_eq!(format_hex_color(-1, 8), "#ffffffff");

        let err = parse_hex_color("FF00AA", 6).unwrap_err();
        assert!(err.contains("must begin with #"));
        let err = parse_hex_color("#FFF", 6).unwrap_err();
        assert!(err.contains("wrong size"));
        let err = parse_hex_color("#ZZZZZZ", 6).unwrap_err();
        assert!(err.contains("Invalid color value"));
    }

    #[test]
    fn int_and_long_ranges() {
        assert_eq!(int_range(0, 10, 0).unwrap(), 0);
        assert_eq!(int_range(0, 10, 10).unwrap(), 10);
        assert!(int_range(0, 10, -1).is_err());
        assert!(int_range(0, 10, 11).is_err());

        let min_i32 = i32::MIN;
        let max_i32 = i32::MAX;
        assert_eq!(int_range(min_i32, max_i32, min_i32).unwrap(), min_i32);
        assert_eq!(long_range(-5, 5, -5).unwrap(), -5);
        assert!(long_range(-5, 5, 6).is_err());
    }

    #[test]
    fn float_ranges_match_java_compare_semantics() {
        assert_eq!(float_range(0.0, 1.0, 0.0).unwrap(), 0.0);
        assert_eq!(float_range(-0.0, 1.0, 0.0).unwrap(), 0.0);
        assert!(float_range(0.0, 1.0, -0.1).is_err());

        let nan = f32::NAN;
        // Java Float.compareTo: NaN is equal to NaN and greater than any other float
        assert!(float_range_min_inclusive_with_message(0.0, f32::MAX, |_| "e".into(), nan).is_ok());
        assert!(float_range_min_exclusive_with_message(0.0, f32::MAX, |_| "e".into(), nan).is_ok());
        assert!(float_range_min_inclusive_with_message(nan, nan, |_| "e".into(), nan).is_ok());
    }

    #[test]
    fn id_resolver_and_late_bound_mapper() {
        let values = [10_i32, 20, 30];
        let id_to_value = |id: i32| values.get(id as usize).copied();
        let value_to_id = |v: &i32| values.iter().position(|x| x == v).map(|i| i as i32).unwrap_or(-1);

        assert_eq!(id_resolver_to_value(id_to_value, 1).unwrap(), 20);
        assert!(id_resolver_to_value(id_to_value, 5).is_err());
        assert_eq!(id_resolver_from_int(id_to_value, value_to_id, -1, &30).unwrap(), 2);
        assert!(id_resolver_from_int(id_to_value, |_| -1, -1, &30).is_err());

        let mut mapper = LateBoundIdMapper::new();
        mapper.put(1_i32, "one".to_string()).put(2, "two".to_string());
        assert_eq!(mapper.get_by_id(&1), Some(&"one".to_string()));
        assert_eq!(mapper.get_id_by_value(&"two".to_string()), Some(&2));
        assert_eq!(mapper.values().len(), 2);
    }

    #[test]
    fn ensure_homogenous_and_tag_location() {
        #[derive(Debug)]
        enum Item { A(i32), B(i32) }
        let ok = vec![Item::A(1), Item::A(2)];
        assert!(ensure_homogenous(&ok, |i| std::mem::discriminant(i)).is_ok());
        let bad = vec![Item::A(1), Item::B(2)];
        let err = ensure_homogenous(&bad, |i| std::mem::discriminant(i)).unwrap_err();
        assert!(err.contains("Mixed type list"));

        let tag = TagOrElementLocation { id: "minecraft:stone".to_string(), tag: true };
        let elem = TagOrElementLocation { id: "minecraft:dirt".to_string(), tag: false };
        assert_eq!(tag.decorated_id(), "#minecraft:stone");
        assert_eq!(tag.to_string(), "#minecraft:stone");
        assert_eq!(elem.decorated_id(), "minecraft:dirt");
    }

    #[test]
    fn or_else_partial_and_optional_long_helpers() {
        let ok: DataResult<i32> = Ok(5);
        let err: DataResult<i32> = Err("bad".to_string());
        assert_eq!(or_else_partial(0, ok).unwrap(), 5);
        let wrapped = or_else_partial(0, err.clone()).unwrap_err();
        assert_eq!(wrapped, "(bad -> using default)");
        assert_eq!(or_else_partial_value(42, err), 42);

        assert_eq!(to_optional_long(Some(7)), Some(7));
        assert_eq!(to_optional_long(None), None);
        assert_eq!(from_optional_long(Some(-1)), Some(-1));
    }
}
