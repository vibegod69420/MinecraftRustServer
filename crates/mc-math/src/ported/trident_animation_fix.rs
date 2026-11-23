use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i32),
    FloatBits(u32),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Map(m) => OptionalDynamic(m.get(key).cloned()),
            _ => OptionalDynamic(None),
        }
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, key: &str, f: F) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                if let Some(current) = m.remove(key) {
                    let updated = f(current);
                    m.insert(key.to_string(), updated);
                }
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    pub fn as_string_result(&self) -> DataResult<String> {
        match self {
            Dynamic::String(s) => DataResult::success(s.clone()),
            _ => DataResult::error(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    pub fn as_string(&self, default: &str) -> String {
        match &self.0 {
            Some(Dynamic::String(s)) => s.clone(),
            _ => default.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataResult<T>(Option<T>);

impl<T> DataResult<T> {
    pub fn success(v: T) -> Self {
        Self(Some(v))
    }

    pub fn error() -> Self {
        Self(None)
    }

    pub fn result(self) -> Option<T> {
        self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema;

pub struct TridentAnimationFix {
    _schema: Schema,
}

impl TridentAnimationFix {
    // JAVA: net.minecraft.util.datafix.fixes.TridentAnimationFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.TridentAnimationFix#fixComponent
    pub fn fix_component(&self, component: Dynamic) -> Dynamic {
        component.update("animation", |anim| {
            let anim_str = anim.as_string_result().result().unwrap_or_default();
            if anim_str == "spear" {
                anim.create_string("trident")
            } else {
                anim
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component_with_animation(anim: &str) -> Dynamic {
        let mut m = BTreeMap::new();
        m.insert("animation".to_string(), Dynamic::String(anim.to_string()));
        Dynamic::Map(m)
    }

    #[test]
    fn spear_becomes_trident() {
        let fix = TridentAnimationFix::new(Schema);
        let out = fix.fix_component(component_with_animation("spear"));
        assert_eq!(out.get("animation").as_string(""), "trident");
    }

    #[test]
    fn trident_unchanged() {
        let fix = TridentAnimationFix::new(Schema);
        let input = component_with_animation("trident");
        let out = fix.fix_component(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn other_animation_unchanged() {
        let fix = TridentAnimationFix::new(Schema);
        let input = component_with_animation("eat");
        let out = fix.fix_component(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_animation_no_insertion() {
        let fix = TridentAnimationFix::new(Schema);
        let input = Dynamic::empty_map();
        let out = fix.fix_component(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn float_bits_and_negative_int_preserved() {
        let fix = TridentAnimationFix::new(Schema);
        let mut m = BTreeMap::new();
        m.insert("animation".to_string(), Dynamic::String("spear".to_string()));
        m.insert(
            "speed".to_string(),
            Dynamic::FloatBits(f32::NEG_INFINITY.to_bits()),
        );
        m.insert("damage".to_string(), Dynamic::Int(i32::MIN));
        let out = fix.fix_component(Dynamic::Map(m));
        assert_eq!(out.get("animation").as_string(""), "trident");
        if let Dynamic::Map(ref map) = out {
            match map.get("speed") {
                Some(Dynamic::FloatBits(b)) => assert_eq!(f32::from_bits(*b), f32::NEG_INFINITY),
                other => panic!("{:?}", other),
            }
            assert_eq!(map.get("damage"), Some(&Dynamic::Int(i32::MIN)));
        } else {
            panic!("expected map");
        }
    }
}
