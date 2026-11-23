#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum References {
    BlockEntity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type {
    pub reference: References,
    pub name: String,
}

impl Schema {
    pub fn get_choice_type(&self, reference: References, name: &str) -> Type {
        Type {
            reference,
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeRewriteRule {
    pub name: String,
    pub input_type: Type,
    pub output_type: Type,
}

pub const DECORATED_POT_ID: &str = "minecraft:decorated_pot";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecoratedPotFieldRenameFix {
    pub output_schema: Schema,
    pub changes_type: bool,
}

impl DecoratedPotFieldRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.DecoratedPotFieldRenameFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            output_schema: schema,
            changes_type: true,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.DecoratedPotFieldRenameFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let input_type = self
            .get_input_schema()
            .get_choice_type(References::BlockEntity, DECORATED_POT_ID);
        let output_type = self
            .get_output_schema()
            .get_choice_type(References::BlockEntity, DECORATED_POT_ID);
        self.convert_unchecked("DecoratedPotFieldRenameFix", input_type, output_type)
    }

    fn get_input_schema(&self) -> &Schema {
        &self.output_schema
    }

    fn get_output_schema(&self) -> &Schema {
        &self.output_schema
    }

    fn convert_unchecked(&self, name: &str, input_type: Type, output_type: Type) -> TypeRewriteRule {
        TypeRewriteRule {
            name: name.to_string(),
            input_type,
            output_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_rule_has_correct_name() {
        let fix = DecoratedPotFieldRenameFix::new(Schema);
        let rule = fix.make_rule();
        assert_eq!(rule.name, "DecoratedPotFieldRenameFix");
    }

    #[test]
    fn make_rule_uses_decorated_pot_choice_type() {
        let fix = DecoratedPotFieldRenameFix::new(Schema);
        let rule = fix.make_rule();
        let expected = Type {
            reference: References::BlockEntity,
            name: DECORATED_POT_ID.to_string(),
        };
        assert_eq!(rule.input_type, expected);
        assert_eq!(rule.output_type, expected);
    }

    #[test]
    fn constructor_sets_changes_type_true() {
        let fix = DecoratedPotFieldRenameFix::new(Schema);
        assert!(fix.changes_type);
    }

    #[test]
    fn make_rule_input_and_output_are_same_type() {
        let fix = DecoratedPotFieldRenameFix::new(Schema);
        let rule = fix.make_rule();
        assert_eq!(rule.input_type, rule.output_type);
    }
}
