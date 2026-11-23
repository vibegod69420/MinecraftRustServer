#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Int(i32),
    Float(f32),
    String(String),
    Map(Vec<(Dynamic, Dynamic)>),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#asNumber
    pub fn as_number_int(&self) -> Option<i32> {
        match self {
            Dynamic::Int(v) => Some(*v),
            Dynamic::Float(v) => Some(*v as i32),
            _ => None,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createInt
    pub fn create_int(&self, value: i32) -> Dynamic {
        Dynamic::Int(value)
    }

    // JAVA: com.mojang.serialization.Dynamic#update(String, Function)
    pub fn update<F>(self, key: &str, mut fixer: F) -> Dynamic
    where
        F: FnMut(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(entries) => {
                let mut out = Vec::with_capacity(entries.len());
                for (k, v) in entries {
                    let is_match = matches!(&k, Dynamic::String(s) if s == key);
                    if is_match {
                        out.push((k, fixer(v)));
                    } else {
                        out.push((k, v));
                    }
                }
                Dynamic::Map(out)
            }
            other => other,
        }
    }
}

pub struct StructureReferenceCountFix;

impl StructureReferenceCountFix {
    // JAVA: net.minecraft.util.datafix.fixes.StructureReferenceCountFix#<init>
    pub fn new(_schema: Schema, _changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructureReferenceCountFix#makeRule
    pub fn make_rule(&self, dynamic: Dynamic) -> Dynamic {
        Self::set_count_to_at_least_one(dynamic)
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructureReferenceCountFix#setCountToAtLeastOne
    fn set_count_to_at_least_one(dynamic: Dynamic) -> Dynamic {
        dynamic.update("references", |v| {
            let value = match v.as_number_int() {
                Some(i) if i > 0 => i,
                _ => 1,
            };
            v.create_int(value)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(value: &str) -> Dynamic {
        Dynamic::String(value.to_owned())
    }

    #[test]
    fn positive_reference_kept() {
        let fix = StructureReferenceCountFix::new(Schema, false);
        let input = Dynamic::Map(vec![(s("references"), Dynamic::Int(5))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("references"), Dynamic::Int(5))]));
    }

    #[test]
    fn zero_reference_becomes_one() {
        let fix = StructureReferenceCountFix::new(Schema, false);
        let input = Dynamic::Map(vec![(s("references"), Dynamic::Int(0))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("references"), Dynamic::Int(1))]));
    }

    #[test]
    fn negative_reference_becomes_one() {
        let fix = StructureReferenceCountFix::new(Schema, false);
        let input = Dynamic::Map(vec![(s("references"), Dynamic::Int(-3))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("references"), Dynamic::Int(1))]));
    }

    #[test]
    fn missing_reference_unchanged() {
        let fix = StructureReferenceCountFix::new(Schema, false);
        let input = Dynamic::Map(vec![(s("name"), s("test"))]);
        let out = fix.make_rule(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn non_numeric_reference_becomes_one() {
        let fix = StructureReferenceCountFix::new(Schema, false);
        let input = Dynamic::Map(vec![(s("references"), s("abc"))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("references"), Dynamic::Int(1))]));
    }

    #[test]
    fn float_reference_truncated_to_int() {
        let fix = StructureReferenceCountFix::new(Schema, false);
        let input = Dynamic::Map(vec![(s("references"), Dynamic::Float(2.7))]);
        let out = fix.make_rule(input);
        assert_eq!(out, Dynamic::Map(vec![(s("references"), Dynamic::Int(2))]));
    }
}
