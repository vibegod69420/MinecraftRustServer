use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    Int(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

pub struct DataComponentRemainderFix<F>
where
    F: Fn(Dynamic) -> Option<Dynamic>,
{
    pub name: String,
    pub component_id: String,
    pub new_component_id: String,
    fixer: F,
}

impl<F> DataComponentRemainderFix<F>
where
    F: Fn(Dynamic) -> Option<Dynamic>,
{
    // JAVA: net.minecraft.util.datafix.fixes.DataComponentRemainderFix#DataComponentRemainderFix
    pub fn new(name: String, component_id: String, new_component_id: String, fixer: F) -> Self {
        Self {
            name,
            component_id,
            new_component_id,
            fixer,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.DataComponentRemainderFix#DataComponentRemainderFix
    pub fn new_same(name: String, component_id: String, fixer: F) -> Self {
        let new_component_id = component_id.clone();
        Self {
            name,
            component_id,
            new_component_id,
            fixer,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.DataComponentRemainderFix#fixComponent
    fn fix_component(&self, component: Dynamic) -> Option<Dynamic> {
        (self.fixer)(component)
    }

    // JAVA: net.minecraft.util.datafix.fixes.DataComponentRemainderFix#makeRule
    pub fn fix(&self, mut components: Dynamic) -> Dynamic {
        if let Dynamic::Map(ref mut map) = components {
            if let Some(existing) = map.get(&self.component_id).cloned() {
                let fixed = self.fix_component(existing);
                map.remove(&self.component_id);
                if let Some(new_val) = fixed {
                    map.insert(self.new_component_id.clone(), new_val);
                }
            }
        }
        components
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_val(s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    #[test]
    fn replaces_component_with_new_id() {
        let fix = DataComponentRemainderFix::new(
            "test".to_string(),
            "old".to_string(),
            "new".to_string(),
            |d| Some(d),
        );
        let mut m = BTreeMap::new();
        m.insert("old".to_string(), str_val("value"));
        let result = fix.fix(Dynamic::Map(m));
        let Dynamic::Map(m) = result else { panic!("expected map") };
        assert!(!m.contains_key("old"));
        assert_eq!(m.get("new"), Some(&str_val("value")));
    }

    #[test]
    fn removes_component_when_fix_returns_none() {
        let fix = DataComponentRemainderFix::new_same(
            "test".to_string(),
            "comp".to_string(),
            |_| None,
        );
        let mut m = BTreeMap::new();
        m.insert("comp".to_string(), str_val("value"));
        m.insert("keep".to_string(), Dynamic::Int(1));
        let result = fix.fix(Dynamic::Map(m));
        let Dynamic::Map(m) = result else { panic!("expected map") };
        assert!(!m.contains_key("comp"));
        assert_eq!(m.get("keep"), Some(&Dynamic::Int(1)));
    }

    #[test]
    fn missing_component_unchanged() {
        let fix = DataComponentRemainderFix::new_same(
            "test".to_string(),
            "missing".to_string(),
            |_| Some(Dynamic::Null),
        );
        let mut m = BTreeMap::new();
        m.insert("other".to_string(), Dynamic::Int(42));
        let original = Dynamic::Map(m);
        let result = fix.fix(original.clone());
        assert_eq!(result, original);
    }

    #[test]
    fn transforms_component_value_with_wrapping() {
        let fix = DataComponentRemainderFix::new(
            "test".to_string(),
            "a".to_string(),
            "b".to_string(),
            |d| match d {
                Dynamic::Int(v) => Some(Dynamic::Int(v.wrapping_add(1))),
                other => Some(other),
            },
        );
        let mut m = BTreeMap::new();
        m.insert("a".to_string(), Dynamic::Int(i64::MAX));
        let result = fix.fix(Dynamic::Map(m));
        let Dynamic::Map(m) = result else { panic!("expected map") };
        assert_eq!(m.get("b"), Some(&Dynamic::Int(i64::MIN)));
    }

    #[test]
    fn non_map_input_unchanged() {
        let fix = DataComponentRemainderFix::new_same("t".to_string(), "c".to_string(), |_| None);
        let input = Dynamic::List(vec![Dynamic::Int(-1)]);
        assert_eq!(fix.fix(input.clone()), input);
    }
}
