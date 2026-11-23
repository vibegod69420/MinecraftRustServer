use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i64),
    Float(f32),
    Str(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn map() -> Self {
        Self::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn remove(mut self, key: &str) -> Self {
        if let Self::Map(map) = &mut self {
            map.remove(key);
        }
        self
    }

    pub fn set(mut self, key: &str, value: Self) -> Self {
        if let Self::Map(map) = &mut self {
            map.insert(key.to_string(), value);
        }
        self
    }

    pub fn as_float(&self, default: f32) -> f32 {
        match self {
            Self::Float(value) => *value,
            Self::Int(value) => *value as f32,
            _ => default,
        }
    }

    pub fn as_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(values) => Some(values),
            _ => None,
        }
    }

    pub fn create_float(value: f32) -> Self {
        Self::Float(value)
    }

    pub fn create_list(values: Vec<Self>) -> Self {
        Self::List(values)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.EntityEquipmentToArmorAndHandFix#fixDropChances
pub fn fix_drop_chances(mut data: Dynamic) -> Dynamic {
    let drop_chances = data
        .get("DropChances")
        .and_then(|value| value.as_list())
        .map(|values| values.iter().map(|value| value.as_float(0.0)).collect::<Vec<_>>());

    data = data.remove("DropChances");

    if let Some(mut values) = drop_chances {
        let mut index = 0usize;
        let mut next = || {
            let value = values.get(index).copied().unwrap_or(0.0);
            index = index.wrapping_add(1);
            value
        };

        let hand_drop_chance = next();

        if data.get("HandDropChances").is_none() {
            data = data.set(
                "HandDropChances",
                Dynamic::create_list(vec![
                    Dynamic::create_float(hand_drop_chance),
                    Dynamic::create_float(0.0),
                ]),
            );
        }

        if data.get("ArmorDropChances").is_none() {
            data = data.set(
                "ArmorDropChances",
                Dynamic::create_list(vec![
                    Dynamic::create_float(next()),
                    Dynamic::create_float(next()),
                    Dynamic::create_float(next()),
                    Dynamic::create_float(next()),
                ]),
            );
        }
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_drop_chances() {
        let mut data = Dynamic::map();
        data = data.set(
            "DropChances",
            Dynamic::create_list(vec![
                Dynamic::create_float(0.5),
                Dynamic::create_float(0.25),
                Dynamic::create_float(0.125),
                Dynamic::create_float(0.0625),
                Dynamic::create_float(0.03125),
            ]),
        );

        let result = fix_drop_chances(data);

        assert!(result.get("DropChances").is_none());
        assert_eq!(
            result.get("HandDropChances"),
            Some(&Dynamic::create_list(vec![
                Dynamic::create_float(0.5),
                Dynamic::create_float(0.0),
            ]))
        );
        assert_eq!(
            result.get("ArmorDropChances"),
            Some(&Dynamic::create_list(vec![
                Dynamic::create_float(0.25),
                Dynamic::create_float(0.125),
                Dynamic::create_float(0.0625),
                Dynamic::create_float(0.03125),
            ]))
        );
    }

    #[test]
    fn does_not_overwrite_existing_chances() {
        let mut data = Dynamic::map();
        data = data.set(
            "DropChances",
            Dynamic::create_list(vec![Dynamic::create_float(1.0)]),
        );
        data = data.set(
            "HandDropChances",
            Dynamic::create_list(vec![Dynamic::create_float(2.0)]),
        );
        data = data.set(
            "ArmorDropChances",
            Dynamic::create_list(vec![Dynamic::create_float(3.0)]),
        );

        let result = fix_drop_chances(data);

        assert_eq!(
            result.get("HandDropChances"),
            Some(&Dynamic::create_list(vec![Dynamic::create_float(2.0)]))
        );
        assert_eq!(
            result.get("ArmorDropChances"),
            Some(&Dynamic::create_list(vec![Dynamic::create_float(3.0)]))
        );
    }

    #[test]
    fn pads_missing_drop_chances_with_zero() {
        let mut data = Dynamic::map();
        data = data.set(
            "DropChances",
            Dynamic::create_list(vec![Dynamic::create_float(0.75)]),
        );

        let result = fix_drop_chances(data);

        assert_eq!(
            result.get("ArmorDropChances"),
            Some(&Dynamic::create_list(vec![
                Dynamic::create_float(0.0),
                Dynamic::create_float(0.0),
                Dynamic::create_float(0.0),
                Dynamic::create_float(0.0),
            ]))
        );
    }

    #[test]
    fn leaves_data_unchanged_without_drop_chances() {
        let mut data = Dynamic::map();
        data = data.set("Foo", Dynamic::Int(42));

        let result = fix_drop_chances(data.clone());

        assert_eq!(result, data);
    }

    #[test]
    fn handles_negative_and_overflow_inputs() {
        let mut data = Dynamic::map();
        data = data.set(
            "DropChances",
            Dynamic::create_list(vec![
                Dynamic::Int(i64::MIN),
                Dynamic::Float(f32::NAN),
            ]),
        );

        let result = fix_drop_chances(data);

        assert_eq!(
            result.get("HandDropChances"),
            Some(&Dynamic::create_list(vec![
                Dynamic::create_float(i64::MIN as f32),
                Dynamic::create_float(0.0),
            ]))
        );
        assert!(result
            .get("ArmorDropChances")
            .unwrap()
            .as_list()
            .unwrap()[0]
            .as_float(0.0)
            .is_nan());
    }
}
