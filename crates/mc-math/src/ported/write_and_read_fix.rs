#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#getType
    pub fn get_type(&self, reference: &TypeReference) -> TypeReference {
        reference.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeReference(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeRewriteRule {
    pub name: String,
    pub type_name: String,
    pub input_version: i32,
    pub output_version: i32,
}

pub struct WriteAndReadFix {
    pub output_schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub type_reference: TypeReference,
}

impl WriteAndReadFix {
    // JAVA: net.minecraft.util.datafix.fixes.WriteAndReadFix#<init>
    pub fn new(output_schema: Schema, name: String, type_reference: TypeReference) -> Self {
        Self {
            output_schema,
            changes_type: true,
            name,
            type_reference,
        }
    }

    // JAVA: com.mojang.datafixers.DataFix#getInputSchema
    pub fn get_input_schema(&self) -> Schema {
        Schema {
            version: self.output_schema.version.wrapping_sub(1),
        }
    }

    // JAVA: com.mojang.datafixers.DataFix#getOutputSchema
    pub fn get_output_schema(&self) -> &Schema {
        &self.output_schema
    }

    // JAVA: com.mojang.datafixers.DataFix#writeAndRead
    pub fn write_and_read(
        &self,
        name: &str,
        _input_type: TypeReference,
        _output_type: TypeReference,
    ) -> TypeRewriteRule {
        TypeRewriteRule {
            name: name.to_string(),
            type_name: self.type_reference.0.clone(),
            input_version: self.get_input_schema().version,
            output_version: self.get_output_schema().version,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.WriteAndReadFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let input_type = self
            .get_input_schema()
            .get_type(&self.type_reference);
        let output_type = self.get_output_schema().get_type(&self.type_reference);
        self.write_and_read(&self.name, input_type, output_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_sets_changes_type_true() {
        let fix = WriteAndReadFix::new(
            Schema { version: 10 },
            "TestFix".to_string(),
            TypeReference("entity".to_string()),
        );
        assert!(fix.changes_type);
        assert_eq!(fix.name, "TestFix");
        assert_eq!(fix.type_reference, TypeReference("entity".to_string()));
    }

    #[test]
    fn make_rule_carries_name_type_and_versions() {
        let fix = WriteAndReadFix::new(
            Schema { version: 5 },
            "MyFix".to_string(),
            TypeReference("block".to_string()),
        );
        let rule = fix.make_rule();
        assert_eq!(rule.name, "MyFix");
        assert_eq!(rule.type_name, "block");
        assert_eq!(rule.input_version, 4);
        assert_eq!(rule.output_version, 5);
    }

    #[test]
    fn negative_version_wraps_on_input_schema() {
        let fix = WriteAndReadFix::new(
            Schema { version: i32::MIN },
            "Neg".to_string(),
            TypeReference("t".to_string()),
        );
        let rule = fix.make_rule();
        assert_eq!(rule.output_version, i32::MIN);
        assert_eq!(rule.input_version, i32::MAX); // wrapping_sub(1)
    }

    #[test]
    fn empty_name_and_type_edge_case() {
        let fix = WriteAndReadFix::new(
            Schema { version: 0 },
            String::new(),
            TypeReference(String::new()),
        );
        let rule = fix.make_rule();
        assert_eq!(rule.name, "");
        assert_eq!(rule.type_name, "");
        assert_eq!(rule.input_version, -1);
        assert_eq!(rule.output_version, 0);
    }

    #[test]
    fn float_bits_encoded_as_version_are_preserved() {
        let bits = f32::NEG_INFINITY.to_bits() as i32;
        let fix = WriteAndReadFix::new(
            Schema { version: bits },
            "FloatBits".to_string(),
            TypeReference("x".to_string()),
        );
        let rule = fix.make_rule();
        assert_eq!(rule.output_version, bits);
        assert_eq!(rule.input_version, bits.wrapping_sub(1));
        assert_eq!(f32::from_bits(rule.output_version as u32), f32::NEG_INFINITY);
    }
}
