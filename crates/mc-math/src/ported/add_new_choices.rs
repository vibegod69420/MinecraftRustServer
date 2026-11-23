use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeReference {
    type_name: String,
}

impl TypeReference {
    pub fn new(type_name: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

#[derive(Debug, Clone)]
pub struct TaggedChoiceType<K: Clone + PartialEq + Display> {
    key_type_name: String,
    types: Vec<K>,
}

impl<K: Clone + PartialEq + Display> TaggedChoiceType<K> {
    pub fn new(key_type_name: &str, types: Vec<K>) -> Self {
        Self {
            key_type_name: key_type_name.to_string(),
            types,
        }
    }

    pub fn get_key_type(&self) -> &str {
        &self.key_type_name
    }

    pub fn has_type(&self, key: &K) -> bool {
        self.types.iter().any(|t| t == key)
    }
}

pub struct AddNewChoices {
    name: String,
    type_ref: TypeReference,
}

impl AddNewChoices {
    // JAVA: net.minecraft.util.datafix.fixes.AddNewChoices#<init>
    pub fn new(name: &str, type_ref: TypeReference) -> Self {
        Self {
            name: name.to_string(),
            type_ref,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.AddNewChoices#cap
    pub fn cap<K: Clone + PartialEq + Display>(
        &self,
        input: &TaggedChoiceType<K>,
        output: &TaggedChoiceType<K>,
        key: &K,
    ) -> Result<K, String> {
        if input.get_key_type() != output.get_key_type() {
            return Err("Could not inject: key type is not the same".to_string());
        }
        if !output.has_type(key) {
            return Err(format!(
                "{}: Unknown type {} in '{}'",
                self.name,
                key,
                self.type_ref.type_name()
            ));
        }
        Ok(key.clone())
    }

    // JAVA: net.minecraft.util.datafix.fixes.AddNewChoices#makeRule
    pub fn make_rule<'a, K: Clone + PartialEq + Display + 'a>(
        &'a self,
        input: &TaggedChoiceType<K>,
        output: &'a TaggedChoiceType<K>,
    ) -> Result<impl Fn(&K) -> Result<K, String> + 'a, String> {
        if input.get_key_type() != output.get_key_type() {
            return Err("Could not inject: key type is not the same".to_string());
        }
        let name = self.name.clone();
        let type_name = self.type_ref.type_name().to_string();
        Ok(move |key: &K| {
            if !output.has_type(key) {
                return Err(format!(
                    "{}: Unknown type {} in '{}'",
                    name, key, type_name
                ));
            }
            Ok(key.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_known_type() {
        let fix = AddNewChoices::new("test", TypeReference::new("entity"));
        let input = TaggedChoiceType::new("string", vec!["a".to_string(), "b".to_string()]);
        let output = TaggedChoiceType::new(
            "string",
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        let result = fix.cap(&input, &output, &"a".to_string());
        assert_eq!(result.unwrap(), "a".to_string());
    }

    #[test]
    fn test_cap_unknown_type() {
        let fix = AddNewChoices::new("MyFix", TypeReference::new("entity"));
        let input = TaggedChoiceType::new("string", vec!["a".to_string()]);
        let output = TaggedChoiceType::new("string", vec!["b".to_string()]);
        let err = fix.cap(&input, &output, &"a".to_string()).unwrap_err();
        assert_eq!(err, "MyFix: Unknown type a in 'entity'");
    }

    #[test]
    fn test_cap_key_type_mismatch() {
        let fix = AddNewChoices::new("test", TypeReference::new("entity"));
        let input = TaggedChoiceType::new("string", vec!["a".to_string()]);
        let output = TaggedChoiceType::new("int", vec!["a".to_string()]);
        let err = fix.cap(&input, &output, &"a".to_string()).unwrap_err();
        assert_eq!(err, "Could not inject: key type is not the same");
    }

    #[test]
    fn test_make_rule_rejects_key_type_mismatch() {
        let fix = AddNewChoices::new("test", TypeReference::new("block"));
        let input = TaggedChoiceType::new("string", vec!["a".to_string()]);
        let output = TaggedChoiceType::new("int", vec!["a".to_string()]);
        let result = fix.make_rule(&input, &output);
        assert!(result.is_err());
    }

    #[test]
    fn test_make_rule_validates_elements() {
        let fix = AddNewChoices::new("MyFix", TypeReference::new("entity"));
        let input = TaggedChoiceType::new("string", vec!["a".to_string()]);
        let output =
            TaggedChoiceType::new("string", vec!["a".to_string(), "b".to_string()]);
        let rule = fix.make_rule(&input, &output).ok().expect("rule should be created");
        assert_eq!(rule(&"a".to_string()).unwrap(), "a".to_string());
        let err = rule(&"z".to_string()).unwrap_err();
        assert_eq!(err, "MyFix: Unknown type z in 'entity'");
    }
}
