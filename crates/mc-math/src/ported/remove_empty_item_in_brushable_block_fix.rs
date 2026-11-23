/// Minimal self-contained dynamic value for datafix porting.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Int(i32),
    Map(Vec<(String, Dynamic)>),
    Other,
}

struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    fn result(self) -> Option<Dynamic> {
        self.0
    }

    fn as_string(&self, default: &str) -> String {
        match &self.0 {
            Some(Dynamic::String(s)) => s.clone(),
            _ => default.to_string(),
        }
    }

    fn as_int(&self, default: i32) -> i32 {
        match &self.0 {
            Some(Dynamic::Int(n)) => *n,
            _ => default,
        }
    }
}

impl Dynamic {
    fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Self::Map(entries) => OptionalDynamic(
                entries
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone()),
            ),
            _ => OptionalDynamic(None),
        }
    }

    fn remove(self, key: &str) -> Self {
        match self {
            Self::Map(mut entries) => {
                entries.retain(|(k, _)| k != key);
                Self::Map(entries)
            }
            other => other,
        }
    }
}

/// Minimal stub for DFU Schema.
pub struct Schema;

pub struct RemoveEmptyItemInBrushableBlockFix {
    _schema: Schema,
}

impl RemoveEmptyItemInBrushableBlockFix {
    // JAVA: net.minecraft.util.datafix.fixes.RemoveEmptyItemInBrushableBlockFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.RemoveEmptyItemInBrushableBlockFix#fix
    pub fn fix(&self, value: Dynamic) -> Dynamic {
        let item = value.get("item").result();
        if let Some(item_dyn) = item {
            if Self::is_empty_stack(&item_dyn) {
                return value.remove("item");
            }
        }
        value
    }

    // JAVA: net.minecraft.util.datafix.fixes.RemoveEmptyItemInBrushableBlockFix#isEmptyStack
    fn is_empty_stack(value: &Dynamic) -> bool {
        let id = ensure_namespaced(&value.get("id").as_string("minecraft:air"));
        let count: i32 = value.get("count").as_int(0);
        id == "minecraft:air" || count == 0
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#ensureNamespaced
fn ensure_namespaced(s: &str) -> String {
    if s.contains(':') {
        s.to_string()
    } else {
        format!("minecraft:{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn removes_air_item() {
        let fix = RemoveEmptyItemInBrushableBlockFix::new(Schema);
        let input = map(vec![
            (
                "item",
                map(vec![
                    ("id", Dynamic::String("minecraft:air".to_string())),
                    ("count", Dynamic::Int(1)),
                ]),
            ),
            ("LootTable", Dynamic::Other),
        ]);
        let out = fix.fix(input);
        assert_eq!(out, map(vec![("LootTable", Dynamic::Other)]));
    }

    #[test]
    fn removes_zero_count_item() {
        let fix = RemoveEmptyItemInBrushableBlockFix::new(Schema);
        let input = map(vec![(
            "item",
            map(vec![
                ("id", Dynamic::String("minecraft:diamond".to_string())),
                ("count", Dynamic::Int(0)),
            ]),
        )]);
        let out = fix.fix(input);
        assert_eq!(out, map(vec![]));
    }

    #[test]
    fn keeps_non_empty_item() {
        let fix = RemoveEmptyItemInBrushableBlockFix::new(Schema);
        let item = map(vec![
            ("id", Dynamic::String("minecraft:diamond".to_string())),
            ("count", Dynamic::Int(1)),
        ]);
        let input = map(vec![("item", item.clone())]);
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_item_is_noop() {
        let fix = RemoveEmptyItemInBrushableBlockFix::new(Schema);
        let input = map(vec![("LootTable", Dynamic::Other)]);
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn normalizes_un_namespaced_id_and_defaults() {
        assert_eq!(ensure_namespaced("stone"), "minecraft:stone");
        assert_eq!(ensure_namespaced("minecraft:air"), "minecraft:air");

        let fix = RemoveEmptyItemInBrushableBlockFix::new(Schema);
        // missing id defaults to minecraft:air, missing count defaults to 0 -> empty
        let input = map(vec![("item", map(vec![]))]);
        let out = fix.fix(input);
        assert_eq!(out, map(vec![]));
    }
}
