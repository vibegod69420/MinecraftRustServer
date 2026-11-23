use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Null,
    String(String),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    value: DynValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataResult<T>(Option<T>);

impl<T> DataResult<T> {
    pub fn result(self) -> Option<T> {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    pub fn result(&self) -> Option<Dynamic> {
        self.0.clone()
    }

    pub fn as_string(&self) -> DataResult<String> {
        match &self.0 {
            Some(d) => d.as_string(),
            None => DataResult(None),
        }
    }

    pub fn as_string_or(&self, default: &str) -> String {
        self.as_string()
            .result()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn as_stream_opt(&self) -> DataResult<Vec<Dynamic>> {
        match &self.0 {
            Some(d) => d.as_stream_opt(),
            None => DataResult(None),
        }
    }
}

impl Dynamic {
    pub fn null() -> Self {
        Dynamic {
            value: DynValue::Null,
        }
    }

    pub fn empty_map(&self) -> Self {
        Dynamic {
            value: DynValue::Map(BTreeMap::new()),
        }
    }

    pub fn string(s: &str) -> Self {
        Dynamic {
            value: DynValue::String(s.to_string()),
        }
    }

    pub fn map(entries: Vec<(&str, Dynamic)>) -> Self {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic {
            value: DynValue::Map(m),
        }
    }

    pub fn list(items: Vec<Dynamic>) -> Self {
        Dynamic {
            value: DynValue::List(items),
        }
    }

    pub fn create_string(&self, s: &str) -> Self {
        Dynamic::string(s)
    }

    pub fn create_list(&self, items: Vec<Dynamic>) -> Self {
        Dynamic::list(items)
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        match &self.value {
            DynValue::Map(m) => OptionalDynamic(m.get(key).cloned()),
            _ => OptionalDynamic(None),
        }
    }

    pub fn as_string(&self) -> DataResult<String> {
        match &self.value {
            DynValue::String(s) => DataResult(Some(s.clone())),
            _ => DataResult(None),
        }
    }

    pub fn as_stream_opt(&self) -> DataResult<Vec<Dynamic>> {
        match &self.value {
            DynValue::List(v) => DataResult(Some(v.clone())),
            _ => DataResult(None),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => {
                let mut m2 = m.clone();
                m2.insert(key.to_string(), value);
                Dynamic {
                    value: DynValue::Map(m2),
                }
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => {
                let mut m2 = m.clone();
                m2.remove(key);
                Dynamic {
                    value: DynValue::Map(m2),
                }
            }
            _ => self.clone(),
        }
    }
}

fn or_else<T>(opt: Option<T>, fallback: T) -> T {
    opt.unwrap_or(fallback)
}

#[derive(Clone, Debug)]
pub struct Schema;

#[derive(Clone, Debug)]
pub struct Type;

impl Type {
    pub fn read_typed(&self, dynamic: Dynamic) -> DataResult<(Typed, ())> {
        DataResult(Some((Typed { data: dynamic }, ())))
    }
}

#[derive(Clone, Debug)]
pub struct TypeRewriteRule;

pub struct References;

impl References {
    pub const UNTAGGED_SPAWNER: &'static str = "minecraft:untagged_spawner";
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub data: Dynamic,
}

impl Typed {
    pub fn get(&self, _finder: RemainderFinder) -> Dynamic {
        self.data.clone()
    }
}

#[derive(Clone, Debug)]
pub struct RemainderFinder;

pub struct Dsl;

impl Dsl {
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

pub struct MobSpawnerEntityIdentifiersFix {
    schema: Schema,
}

impl MobSpawnerEntityIdentifiersFix {
    // JAVA: net.minecraft.util.datafix.fixes.MobSpawnerEntityIdentifiersFix#<init>
    pub fn new(schema: Schema, _changes_type: bool) -> Self {
        Self { schema }
    }

    fn get_output_schema(&self) -> &Schema {
        &self.schema
    }

    fn get_input_schema(&self) -> &Schema {
        &self.schema
    }

    fn fix_type_everywhere_typed(
        &self,
        _name: &str,
        _in_type: Type,
        _out_type: Type,
    ) -> TypeRewriteRule {
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.MobSpawnerEntityIdentifiersFix#fix
    fn fix(&self, value: Dynamic) -> Dynamic {
        if value.get("id").as_string_or("") != "MobSpawner" {
            return value;
        }

        let mut out = value.clone();

        let entity_id = out.get("EntityId").as_string().result();
        if let Some(id) = entity_id {
            let mut spawn_data = or_else(out.get("SpawnData").result(), out.empty_map());
            let id_str = if id.is_empty() { "Pig" } else { &id };
            spawn_data = spawn_data.set("id", spawn_data.create_string(id_str));
            out = out.set("SpawnData", spawn_data);
            out = out.remove("EntityId");
        }

        let spawn_potentials = out.get("SpawnPotentials").as_stream_opt().result();
        if let Some(potentials) = spawn_potentials {
            let mapped: Vec<Dynamic> = potentials
                .into_iter()
                .map(|p| {
                    let type_id = p.get("Type").as_string().result();
                    if let Some(t) = type_id {
                        let mut entity = or_else(p.get("Properties").result(), p.empty_map());
                        entity = entity.set("id", p.create_string(&t));
                        p.set("Entity", entity).remove("Type").remove("Properties")
                    } else {
                        p
                    }
                })
                .collect();
            out = out.set("SpawnPotentials", out.create_list(mapped));
        }

        out
    }

    // JAVA: net.minecraft.util.datafix.fixes.MobSpawnerEntityIdentifiersFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let out_type = self.get_output_schema().get_type(References::UNTAGGED_SPAWNER);
        let in_type = self.get_input_schema().get_type(References::UNTAGGED_SPAWNER);
        self.fix_type_everywhere_typed("MobSpawnerEntityIdentifiersFix", in_type, out_type)
    }

    // Helper mirroring the makeRule lambda behavior for testing.
    fn apply_typed_fix(&self, typed: Typed) -> Typed {
        let out_type = self.get_output_schema().get_type(References::UNTAGGED_SPAWNER);
        let mut data = typed.get(Dsl::remainder_finder());
        data = data.set("id", data.create_string("MobSpawner"));
        let fixed = self.fix(data);
        match out_type.read_typed(fixed).result() {
            Some((first, _)) => first,
            None => typed,
        }
    }
}

impl Schema {
    pub fn get_type(&self, _reference: &str) -> Type {
        Type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fix() -> MobSpawnerEntityIdentifiersFix {
        MobSpawnerEntityIdentifiersFix::new(Schema, true)
    }

    #[test]
    fn non_mob_spawner_unchanged() {
        let fix = make_fix();
        let d = Dynamic::map(vec![
            ("id", Dynamic::string("Chest")),
            ("EntityId", Dynamic::string("Zombie")),
        ]);
        let out = fix.fix(d.clone());
        assert_eq!(out, d);
    }

    #[test]
    fn entity_id_moved_to_spawn_data() {
        let fix = make_fix();
        let d = Dynamic::map(vec![
            ("id", Dynamic::string("MobSpawner")),
            ("EntityId", Dynamic::string("Zombie")),
        ]);
        let out = fix.fix(d);
        assert!(out.get("EntityId").result().is_none());
        let spawn_data = out.get("SpawnData").result().unwrap();
        assert_eq!(spawn_data.get("id").as_string().result().unwrap(), "Zombie");
    }

    #[test]
    fn empty_entity_id_defaults_to_pig() {
        let fix = make_fix();
        let d = Dynamic::map(vec![
            ("id", Dynamic::string("MobSpawner")),
            ("EntityId", Dynamic::string("")),
        ]);
        let out = fix.fix(d);
        let spawn_data = out.get("SpawnData").result().unwrap();
        assert_eq!(spawn_data.get("id").as_string().result().unwrap(), "Pig");
    }

    #[test]
    fn spawn_potentials_migrated() {
        let fix = make_fix();
        let d = Dynamic::map(vec![
            ("id", Dynamic::string("MobSpawner")),
            (
                "SpawnPotentials",
                Dynamic::list(vec![
                    Dynamic::map(vec![
                        ("Type", Dynamic::string("Skeleton")),
                        (
                            "Properties",
                            Dynamic::map(vec![("Health", Dynamic::string("20"))]),
                        ),
                    ]),
                    Dynamic::map(vec![("Weight", Dynamic::string("1"))]),
                ]),
            ),
        ]);
        let out = fix.fix(d);
        let pots = out.get("SpawnPotentials").as_stream_opt().result().unwrap();
        let entity = pots[0].get("Entity").result().unwrap();
        assert_eq!(entity.get("id").as_string().result().unwrap(), "Skeleton");
        assert_eq!(entity.get("Health").as_string().result().unwrap(), "20");
        assert!(pots[0].get("Type").result().is_none());
        assert!(pots[0].get("Properties").result().is_none());
        assert!(pots[1].get("Entity").result().is_none());
        assert_eq!(pots[1].get("Weight").as_string().result().unwrap(), "1");
    }

    #[test]
    fn apply_typed_fix_sets_id_and_fixes_data() {
        let fix = make_fix();
        let typed = Typed {
            data: Dynamic::map(vec![
                ("id", Dynamic::string("OldSpawner")),
                ("EntityId", Dynamic::string("Creeper")),
            ]),
        };
        let out = fix.apply_typed_fix(typed);
        assert_eq!(out.data.get("id").as_string().result().unwrap(), "MobSpawner");
        let spawn_data = out.data.get("SpawnData").result().unwrap();
        assert_eq!(spawn_data.get("id").as_string().result().unwrap(), "Creeper");
    }
}
