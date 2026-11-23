#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Map(Vec<(String, Dynamic)>),
    Empty,
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(entries) => entries
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or(Dynamic::Empty),
            _ => Dynamic::Empty,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut entries) => {
                if let Some(entry) = entries.iter_mut().find(|(k, _)| k == key) {
                    entry.1 = value;
                } else {
                    entries.push((key.to_string(), value));
                }
                Dynamic::Map(entries)
            }
            other => other,
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct References;
impl References {
    pub const CHUNK: &'static str = "minecraft:chunk";
}

pub struct DSL;
impl DSL {
    // JAVA: com.mojang.datafixers.DSL#fieldFinder
    pub fn field_finder(name: &str, _type: Type) -> OpticFinder {
        OpticFinder {
            name: name.to_string(),
        }
    }

    // JAVA: com.mojang.datafixers.DSL#remainderFinder
    pub fn remainder_finder() -> OpticFinder {
        OpticFinder {
            name: "remainder".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Type;

impl Type {
    // JAVA: com.mojang.datafixers.types.Type#findFieldType
    pub fn find_field_type(&self, _name: &str) -> Type {
        Type
    }
}

#[derive(Clone, Debug)]
pub struct OpticFinder {
    name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    value: Dynamic,
}

impl Typed {
    pub fn new(value: Dynamic) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &Dynamic {
        &self.value
    }

    pub fn into_value(self) -> Dynamic {
        self.value
    }

    // JAVA: com.mojang.datafixers.Typed#get
    pub fn get(&self, finder: &OpticFinder) -> Dynamic {
        if finder.name == "remainder" {
            self.value.clone()
        } else {
            self.value.get(&finder.name)
        }
    }

    // JAVA: com.mojang.datafixers.Typed#set
    pub fn set(mut self, finder: &OpticFinder, value: Dynamic) -> Self {
        if finder.name == "remainder" {
            self.value = value;
        } else {
            self.value = self.value.set(&finder.name, value);
        }
        self
    }

    // JAVA: com.mojang.datafixers.Typed#updateTyped
    pub fn update_typed<F: FnOnce(Typed) -> Typed>(mut self, finder: &OpticFinder, f: F) -> Self {
        let inner = Typed::new(self.get(finder));
        let updated = f(inner);
        self = self.set(finder, updated.into_value());
        self
    }
}

pub struct DataFix {
    schema: Schema,
    changes_type: bool,
}

impl DataFix {
    // JAVA: com.mojang.datafixers.DataFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self { schema, changes_type }
    }

    // JAVA: com.mojang.datafixers.DataFix#getInputSchema
    pub fn get_input_schema(&self) -> &Schema {
        &self.schema
    }

    // JAVA: com.mojang.datafixers.DataFix#getOutputSchema
    pub fn get_output_schema(&self) -> &Schema {
        &self.schema
    }

    pub fn changes_type(&self) -> bool {
        self.changes_type
    }
}

pub struct TypeRewriteRule;

pub struct ChunkStatusFix {
    base: DataFix,
}

impl ChunkStatusFix {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkStatusFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            base: DataFix::new(schema, changes_type),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkStatusFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule
    }

    /// Applies the chunk status fix logic, mirroring the rule body.
    // JAVA: net.minecraft.util.datafix.fixes.ChunkStatusFix#makeRule
    pub fn apply_to_chunk(&self, chunk: Typed) -> Typed {
        let chunk_type = self.base.get_input_schema().get_type(References::CHUNK);
        let level_type = chunk_type.find_field_type("Level");
        let level_finder = DSL::field_finder("Level", level_type);
        let _out_type = self.base.get_output_schema().get_type(References::CHUNK);

        chunk.update_typed(&level_finder, |level_typed| {
            let mut level_dyn = level_typed.get(&DSL::remainder_finder());
            let status = level_dyn.get("Status").as_string("empty");
            if status == "postprocessed" {
                let new_status = level_dyn.create_string("fullchunk");
                level_dyn = level_dyn.set("Status", new_status);
            }
            level_typed.set(&DSL::remainder_finder(), level_dyn)
        })
    }

    fn get_type(&self, _reference: &str) -> Type {
        Type
    }
}

trait SchemaExt {
    fn get_type(&self, _reference: &str) -> Type;
}
impl SchemaExt for Schema {
    fn get_type(&self, _reference: &str) -> Type {
        Type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    fn chunk_with_status(status: &str) -> Typed {
        Typed::new(map(vec![(
            "Level",
            map(vec![("Status", Dynamic::String(status.to_string()))]),
        )]))
    }

    fn get_status(chunk: &Dynamic) -> String {
        chunk.get("Level").get("Status").as_string("missing")
    }

    #[test]
    fn rewrites_postprocessed_to_fullchunk() {
        let fix = ChunkStatusFix::new(Schema, true);
        let out = fix.apply_to_chunk(chunk_with_status("postprocessed")).into_value();
        assert_eq!(get_status(&out), "fullchunk");
    }

    #[test]
    fn preserves_other_statuses() {
        let fix = ChunkStatusFix::new(Schema, true);
        for status in ["empty", "fullchunk", "minecraft:full", "structure_starts"] {
            let out = fix.apply_to_chunk(chunk_with_status(status)).into_value();
            assert_eq!(get_status(&out), status);
        }
    }

    #[test]
    fn defaults_missing_status_to_empty() {
        let fix = ChunkStatusFix::new(Schema, true);
        let chunk = Typed::new(map(vec![("Level", map(vec![]))]));
        let out = fix.apply_to_chunk(chunk).into_value();
        // Missing status defaults to "empty", not "postprocessed", so unchanged
        assert_eq!(out.get("Level").get("Status"), Dynamic::Empty);
    }

    #[test]
    fn make_rule_and_changes_type() {
        let fix = ChunkStatusFix::new(Schema, true);
        let _rule = fix.make_rule();
        assert!(fix.base.changes_type());
        let fix2 = ChunkStatusFix::new(Schema, false);
        assert!(!fix2.base.changes_type());
    }
}
