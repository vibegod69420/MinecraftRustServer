/// Minimal self-contained dynamic value model for datafix porting.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i64),
    Bool(bool),
    Map(Vec<(String, Dynamic)>),
}

/// Wrapper modeling DFU `OptionalDynamic#result()`.
pub struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    pub fn result(self) -> Option<Dynamic> {
        self.0
    }
}

impl Dynamic {
    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Map(entries) => OptionalDynamic(
                entries
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone()),
            ),
            _ => OptionalDynamic(None),
        }
    }

    pub fn remove(&self, key: &str) -> Self {
        match self {
            Dynamic::Map(entries) => Dynamic::Map(
                entries
                    .iter()
                    .filter(|(k, _)| k != key)
                    .cloned()
                    .collect(),
            ),
            _ => self.clone(),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(entries) => {
                let mut new_entries = Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k == key {
                        new_entries.push((key.to_string(), value.clone()));
                        replaced = true;
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                if !replaced {
                    new_entries.push((key.to_string(), value));
                }
                Dynamic::Map(new_entries)
            }
            _ => self.clone(),
        }
    }

    pub fn empty_map(&self) -> Self {
        Dynamic::Map(Vec::new())
    }
}

/// Placeholder for DFU `Schema`.
#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct LodestoneCompassComponentFix {
    _schema: Schema,
    name: String,
    component_id: String,
    remainder_id: String,
}

impl LodestoneCompassComponentFix {
    // JAVA: net.minecraft.util.datafix.fixes.LodestoneCompassComponentFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            _schema: schema,
            name: "LodestoneCompassComponentFix".to_string(),
            component_id: "minecraft:lodestone_target".to_string(),
            remainder_id: "minecraft:lodestone_tracker".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.LodestoneCompassComponentFix#fixComponent
    pub fn fix_component(&self, dynamic: Dynamic) -> Dynamic {
        let pos = dynamic.get("pos").result();
        let dimension = dynamic.get("dimension").result();
        let mut dynamic = dynamic.remove("pos").remove("dimension");
        if let (Some(pos), Some(dimension)) = (pos, dimension) {
            let target = dynamic
                .empty_map()
                .set("pos", pos)
                .set("dimension", dimension);
            dynamic = dynamic.set("target", target);
        }
        dynamic
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn component_id(&self) -> &str {
        &self.component_id
    }

    pub fn remainder_id(&self) -> &str {
        &self.remainder_id
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
    fn fix_component_creates_target_when_pos_and_dimension_present() {
        let fix = LodestoneCompassComponentFix::new(Schema);
        let input = map(vec![
            ("pos", map(vec![("x", Dynamic::Int(1))])),
            ("dimension", Dynamic::String("minecraft:overworld".to_string())),
            ("tracked", Dynamic::Bool(true)),
        ]);

        let output = fix.fix_component(input);
        let expected = map(vec![
            ("tracked", Dynamic::Bool(true)),
            (
                "target",
                map(vec![
                    ("pos", map(vec![("x", Dynamic::Int(1))])),
                    ("dimension", Dynamic::String("minecraft:overworld".to_string())),
                ]),
            ),
        ]);
        assert_eq!(output, expected);
    }

    #[test]
    fn fix_component_removes_fields_when_pos_missing() {
        let fix = LodestoneCompassComponentFix::new(Schema);
        let input = map(vec![
            ("dimension", Dynamic::String("minecraft:the_nether".to_string())),
            ("extra", Dynamic::Int(7)),
        ]);

        let output = fix.fix_component(input);
        let expected = map(vec![("extra", Dynamic::Int(7))]);
        assert_eq!(output, expected);
    }

    #[test]
    fn fix_component_removes_fields_when_dimension_missing() {
        let fix = LodestoneCompassComponentFix::new(Schema);
        let input = map(vec![
            ("pos", map(vec![("y", Dynamic::Int(-64))])),
            ("extra", Dynamic::String("keep".to_string())),
        ]);

        let output = fix.fix_component(input);
        let expected = map(vec![("extra", Dynamic::String("keep".to_string()))]);
        assert_eq!(output, expected);
    }

    #[test]
    fn fix_component_no_target_when_both_missing() {
        let fix = LodestoneCompassComponentFix::new(Schema);
        let input = map(vec![("tracked", Dynamic::Bool(false))]);
        let output = fix.fix_component(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn constructor_sets_expected_metadata() {
        let fix = LodestoneCompassComponentFix::new(Schema);
        assert_eq!(fix.name(), "LodestoneCompassComponentFix");
        assert_eq!(fix.component_id(), "minecraft:lodestone_target");
        assert_eq!(fix.remainder_id(), "minecraft:lodestone_tracker");
    }
}
