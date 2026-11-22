use serde::{Deserialize, Serialize};

/// Faithful port of `net.minecraft.util.Keyframe<T>`.
///
/// The original Java class also exposes a Mojang `Codec<Keyframe<T>>` factory.
/// This Rust port uses `serde` derives for serialization instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe<T> {
    ticks: i32,
    value: T,
}

impl<T> Keyframe<T> {
    // JAVA: net.minecraft.util.Keyframe#<init>
    pub fn new(ticks: i32, value: T) -> Self {
        Self { ticks, value }
    }

    // JAVA: net.minecraft.util.Keyframe#ticks
    pub fn ticks(&self) -> i32 {
        self.ticks
    }

    // JAVA: net.minecraft.util.Keyframe#value
    pub fn value(&self) -> &T {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_accessors() {
        let k = Keyframe::new(10, 2.5f32);
        assert_eq!(k.ticks(), 10);
        assert_eq!(*k.value(), 2.5f32);
    }

    #[test]
    fn zero_and_negative_ticks_are_preserved() {
        let k0 = Keyframe::new(0, "zero");
        assert_eq!(k0.ticks(), 0);
        let kn = Keyframe::new(-7, "neg");
        assert_eq!(kn.ticks(), -7);
        assert_eq!(*kn.value(), "neg");
    }

    #[test]
    fn clone_and_equality() {
        let a = Keyframe::new(3, 42i32);
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn serde_roundtrip() {
        let k = Keyframe::new(7, String::from("hello"));
        let json = serde_json::to_string(&k).unwrap();
        let back: Keyframe<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(k, back);
    }

    #[test]
    fn serde_field_names_match_record_components() {
        let k = Keyframe::new(5, 1i32);
        let json = serde_json::to_value(&k).unwrap();
        assert_eq!(json.get("ticks").and_then(|v| v.as_i64()), Some(5));
        assert_eq!(json.get("value").and_then(|v| v.as_i64()), Some(1));
    }
}
