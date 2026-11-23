use std::collections::BTreeMap;
use std::fmt;

pub const SPECIAL_OBJECTIVE_MARKER: &str = "_special";

/// Minimal self-contained Identifier for datafix string conversions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    fn new(namespace: &str, path: &str) -> Result<Self, String> {
        if !Self::is_valid_namespace(namespace) || !Self::is_valid_path(path) {
            return Err("invalid identifier".to_string());
        }
        Ok(Self {
            namespace: namespace.to_string(),
            path: path.to_string(),
        })
    }

    fn with_default_namespace(path: &str) -> Result<Self, String> {
        Self::new("minecraft", path)
    }

    pub fn by_separator(name: &str, separator: char) -> Result<Self, String> {
        if separator == ':' {
            return Self::try_parse(name).ok_or_else(|| "invalid identifier".to_string());
        }
        match name.find(separator) {
            Some(i) => Self::new(&name[..i], &name[i + 1..]),
            None => Self::with_default_namespace(name),
        }
    }

    pub fn try_parse(name: &str) -> Option<Self> {
        match name.find(':') {
            Some(i) => Self::new(&name[..i], &name[i + 1..]).ok(),
            None => Self::with_default_namespace(name).ok(),
        }
    }

    fn is_valid_namespace(s: &str) -> bool {
        !s.is_empty()
            && s.chars().all(|c| {
                c == '_'
                    || c == '-'
                    || c == '.'
                    || c.is_ascii_lowercase()
                    || c.is_ascii_digit()
            })
    }

    fn is_valid_path(s: &str) -> bool {
        !s.is_empty()
            && s.chars().all(|c| {
                c == '_'
                    || c == '-'
                    || c == '.'
                    || c == '/'
                    || c.is_ascii_lowercase()
                    || c.is_ascii_digit()
            })
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Minimal dynamic value representation for DataFixerUpper hook logic.
#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    String(String),
    Map(BTreeMap<String, DynamicValue>),
}

impl DynamicValue {
    pub fn get(&self, key: &str) -> Option<&DynamicValue> {
        match self {
            DynamicValue::Map(m) => m.get(key),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            DynamicValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn set(&self, key: &str, value: DynamicValue) -> DynamicValue {
        match self {
            DynamicValue::Map(m) => {
                let mut new_m = m.clone();
                new_m.insert(key.to_string(), value);
                DynamicValue::Map(new_m)
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> DynamicValue {
        match self {
            DynamicValue::Map(m) => {
                let mut new_m = m.clone();
                new_m.remove(key);
                DynamicValue::Map(new_m)
            }
            _ => self.clone(),
        }
    }

    pub fn string(s: &str) -> DynamicValue {
        DynamicValue::String(s.to_string())
    }

    pub fn map(entries: Vec<(String, DynamicValue)>) -> DynamicValue {
        DynamicValue::Map(entries.into_iter().collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CriterionIdType {
    ItemName,
    BlockName,
    EntityName,
    NamespacedString,
    PlainString,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatCategoryType {
    ItemName,
    BlockName,
    EntityName,
    NamespacedString,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredTypes {
    pub stats_categories: Vec<(String, StatCategoryType)>,
    pub criterion_types: BTreeMap<String, CriterionIdType>,
}

pub struct V1451_6Schema {
    pub version_key: i32,
}

impl V1451_6Schema {
    // JAVA: net.minecraft.util.datafix.schemas.V1451_6#<init>
    pub fn new(version_key: i32) -> Self {
        Self { version_key }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1451_6#registerTypes
    pub fn register_types(&self) -> RegisteredTypes {
        RegisteredTypes {
            stats_categories: vec![
                ("minecraft:mined".to_string(), StatCategoryType::BlockName),
                (
                    "minecraft:crafted".to_string(),
                    StatCategoryType::ItemName,
                ),
                ("minecraft:used".to_string(), StatCategoryType::ItemName),
                ("minecraft:broken".to_string(), StatCategoryType::ItemName),
                (
                    "minecraft:picked_up".to_string(),
                    StatCategoryType::ItemName,
                ),
                ("minecraft:dropped".to_string(), StatCategoryType::ItemName),
                (
                    "minecraft:killed".to_string(),
                    StatCategoryType::EntityName,
                ),
                (
                    "minecraft:killed_by".to_string(),
                    StatCategoryType::EntityName,
                ),
                (
                    "minecraft:custom".to_string(),
                    StatCategoryType::NamespacedString,
                ),
            ],
            criterion_types: create_criterion_types(),
        }
    }
}

// JAVA: net.minecraft.util.datafix.schemas.V1451_6#packNamespacedWithDot
pub fn pack_namespaced_with_dot(name: &str) -> String {
    match Identifier::try_parse(name) {
        Some(id) => format!("{}.{}", id.namespace, id.path),
        None => name.to_string(),
    }
}

// JAVA: net.minecraft.util.datafix.schemas.V1451_6#UNPACK_OBJECTIVE_ID
pub fn unpack_objective_id(value: &DynamicValue) -> DynamicValue {
    let criteria_name = value.get("CriteriaName").and_then(|v| v.as_string());
    let result = criteria_name.map(|name| match name.find(':') {
        None => (SPECIAL_OBJECTIVE_MARKER.to_string(), name.to_string()),
        Some(idx) => {
            let type_part = &name[..idx];
            let id_part = &name[idx + 1..];
            match (
                Identifier::by_separator(type_part, '.'),
                Identifier::by_separator(id_part, '.'),
            ) {
                (Ok(t), Ok(i)) => (t.to_string(), i.to_string()),
                _ => (SPECIAL_OBJECTIVE_MARKER.to_string(), name.to_string()),
            }
        }
    });

    match result {
        Some((type_str, id_str)) => {
            let criteria_type = DynamicValue::map(vec![
                ("type".to_string(), DynamicValue::string(&type_str)),
                ("id".to_string(), DynamicValue::string(&id_str)),
            ]);
            value.set("CriteriaType", criteria_type)
        }
        None => value.clone(),
    }
}

// JAVA: net.minecraft.util.datafix.schemas.V1451_6#REPACK_OBJECTIVE_ID
pub fn repack_objective_id(value: &DynamicValue) -> DynamicValue {
    let criteria_type_opt = value.get("CriteriaType").and_then(|ct| {
        let type_str = ct.get("type").and_then(|v| v.as_string());
        let id_str = ct.get("id").and_then(|v| v.as_string());
        match (type_str, id_str) {
            (Some(t), Some(id)) => {
                if t == SPECIAL_OBJECTIVE_MARKER {
                    Some(DynamicValue::string(id))
                } else {
                    let packed = format!(
                        "{}:{}",
                        pack_namespaced_with_dot(t),
                        pack_namespaced_with_dot(id)
                    );
                    Some(DynamicValue::string(&packed))
                }
            }
            _ => None,
        }
    });

    match criteria_type_opt {
        Some(criteria_name) => value.set("CriteriaName", criteria_name).remove("CriteriaType"),
        None => value.clone(),
    }
}

// JAVA: net.minecraft.util.datafix.schemas.V1451_6#createCriterionTypes
pub fn create_criterion_types() -> BTreeMap<String, CriterionIdType> {
    let mut map = BTreeMap::new();
    map.insert(
        "minecraft:mined".to_string(),
        CriterionIdType::BlockName,
    );
    map.insert(
        "minecraft:crafted".to_string(),
        CriterionIdType::ItemName,
    );
    map.insert("minecraft:used".to_string(), CriterionIdType::ItemName);
    map.insert(
        "minecraft:broken".to_string(),
        CriterionIdType::ItemName,
    );
    map.insert(
        "minecraft:picked_up".to_string(),
        CriterionIdType::ItemName,
    );
    map.insert(
        "minecraft:dropped".to_string(),
        CriterionIdType::ItemName,
    );
    map.insert(
        "minecraft:killed".to_string(),
        CriterionIdType::EntityName,
    );
    map.insert(
        "minecraft:killed_by".to_string(),
        CriterionIdType::EntityName,
    );
    map.insert(
        "minecraft:custom".to_string(),
        CriterionIdType::NamespacedString,
    );
    map.insert(
        SPECIAL_OBJECTIVE_MARKER.to_string(),
        CriterionIdType::PlainString,
    );
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_namespaced_with_dot_cases() {
        assert_eq!(
            pack_namespaced_with_dot("minecraft:stone"),
            "minecraft.stone"
        );
        assert_eq!(
            pack_namespaced_with_dot("foo:bar/baz"),
            "foo.bar/baz"
        );
        // Invalid identifier returns original string.
        assert_eq!(
            pack_namespaced_with_dot("Invalid:Name"),
            "Invalid:Name"
        );
        assert_eq!(pack_namespaced_with_dot("plain_path"), "minecraft.plain_path");
    }

    #[test]
    fn unpack_objective_id_dotted_format() {
        let input = DynamicValue::map(vec![(
            "CriteriaName".to_string(),
            DynamicValue::string("minecraft.crafted:minecraft.diamond"),
        )]);
        let out = unpack_objective_id(&input);
        let ct = out.get("CriteriaType").unwrap();
        assert_eq!(
            ct.get("type").unwrap().as_string(),
            Some("minecraft:crafted")
        );
        assert_eq!(
            ct.get("id").unwrap().as_string(),
            Some("minecraft:diamond")
        );
    }

    #[test]
    fn unpack_objective_id_no_colon_or_invalid_becomes_special() {
        let no_colon = DynamicValue::map(vec![(
            "CriteriaName".to_string(),
            DynamicValue::string("dummy"),
        )]);
        let out = unpack_objective_id(&no_colon);
        let ct = out.get("CriteriaType").unwrap();
        assert_eq!(ct.get("type").unwrap().as_string(), Some("_special"));
        assert_eq!(ct.get("id").unwrap().as_string(), Some("dummy"));

        // Invalid identifier chars trigger exception fallback to _special.
        let invalid = DynamicValue::map(vec![(
            "CriteriaName".to_string(),
            DynamicValue::string("Bad.Name:also.Bad"),
        )]);
        let out_invalid = unpack_objective_id(&invalid);
        let ct_invalid = out_invalid.get("CriteriaType").unwrap();
        assert_eq!(
            ct_invalid.get("type").unwrap().as_string(),
            Some("_special")
        );
        assert_eq!(
            ct_invalid.get("id").unwrap().as_string(),
            Some("Bad.Name:also.Bad")
        );
    }

    #[test]
    fn repack_objective_id_normal_and_special() {
        let input = DynamicValue::map(vec![(
            "CriteriaType".to_string(),
            DynamicValue::map(vec![
                ("type".to_string(), DynamicValue::string("minecraft:crafted")),
                (
                    "id".to_string(),
                    DynamicValue::string("minecraft:diamond"),
                ),
            ]),
        )]);
        let out = repack_objective_id(&input);
        assert_eq!(
            out.get("CriteriaName").unwrap().as_string(),
            Some("minecraft.crafted:minecraft.diamond")
        );
        assert!(out.get("CriteriaType").is_none());

        let special = DynamicValue::map(vec![(
            "CriteriaType".to_string(),
            DynamicValue::map(vec![
                ("type".to_string(), DynamicValue::string("_special")),
                ("id".to_string(), DynamicValue::string("dummy")),
            ]),
        )]);
        let out2 = repack_objective_id(&special);
        assert_eq!(out2.get("CriteriaName").unwrap().as_string(), Some("dummy"));
        assert!(out2.get("CriteriaType").is_none());
    }

    #[test]
    fn hooks_preserve_on_missing_fields_and_round_trip() {
        let empty = DynamicValue::map(vec![]);
        assert_eq!(unpack_objective_id(&empty), empty);

        let missing_id = DynamicValue::map(vec![(
            "CriteriaType".to_string(),
            DynamicValue::map(vec![("type".to_string(), DynamicValue::string("minecraft:custom"))]),
        )]);
        assert_eq!(repack_objective_id(&missing_id), missing_id);

        let original = DynamicValue::map(vec![
            (
                "CriteriaName".to_string(),
                DynamicValue::string("minecraft.used:minecraft.apple"),
            ),
            ("DisplayName".to_string(), DynamicValue::string("test")),
        ]);
        let unpacked = unpack_objective_id(&original);
        let repacked = repack_objective_id(&unpacked);
        assert_eq!(
            repacked.get("CriteriaName").unwrap().as_string(),
            Some("minecraft.used:minecraft.apple")
        );
        assert_eq!(
            repacked.get("DisplayName").unwrap().as_string(),
            Some("test")
        );
        assert!(repacked.get("CriteriaType").is_none());
    }

    #[test]
    fn create_criterion_types_and_register_types_match_schema() {
        let types = create_criterion_types();
        assert_eq!(types.len(), 10);
        assert_eq!(
            types.get("minecraft:mined"),
            Some(&CriterionIdType::BlockName)
        );
        assert_eq!(
            types.get("minecraft:crafted"),
            Some(&CriterionIdType::ItemName)
        );
        assert_eq!(
            types.get("minecraft:killed"),
            Some(&CriterionIdType::EntityName)
        );
        assert_eq!(
            types.get("minecraft:custom"),
            Some(&CriterionIdType::NamespacedString)
        );
        assert_eq!(
            types.get("_special"),
            Some(&CriterionIdType::PlainString)
        );

        let schema = V1451_6Schema::new(1451);
        let registered = schema.register_types();
        assert_eq!(registered.stats_categories.len(), 9);
        assert_eq!(registered.criterion_types, types);
    }
}
