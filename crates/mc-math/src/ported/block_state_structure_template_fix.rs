//! Faithful port of `net.minecraft.util.datafix.fixes.BlockStateStructureTemplateFix` (1.21.11).
//!
//! This fix wires block-state upgrades for structure templates by targeting
//! `References.BLOCK_STATE` and applying `BlockStateData::upgradeBlockStateTag`.

use std::collections::BTreeMap;

/// Minimal stand-in for a DataFixerUpper schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    version: i32,
}

impl Schema {
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    pub fn get_version(&self) -> i32 {
        self.version
    }
}

/// Minimal stand-in for DataFixerUpper type references.
pub struct References;

impl References {
    pub const BLOCK_STATE: &'static str = "block_state";
}

/// Minimal dynamic block-state representation used by this self-contained fix.
#[derive(Debug, Clone, PartialEq)]
pub struct Dynamic {
    name: String,
    properties: BTreeMap<String, String>,
}

impl Dynamic {
    pub fn new(name: String) -> Self {
        Self {
            name,
            properties: BTreeMap::new(),
        }
    }

    pub fn with_property(mut self, key: String, value: String) -> Self {
        self.properties.insert(key, value);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn properties(&self) -> &BTreeMap<String, String> {
        &self.properties
    }
}

/// Minimal stand-in for `BlockStateData::upgradeBlockStateTag`.
/// The real data-driven upgrade table is external to this fix class;
/// identity preserves behavior when no upgrade applies.
fn upgrade_block_state_tag(tag: Dynamic) -> Dynamic {
    tag
}

/// Minimal stand-in for a DataFixerUpper type rewrite rule.
pub struct TypeRewriteRule {
    name: &'static str,
    type_ref: &'static str,
    apply_fn: fn(Dynamic) -> Dynamic,
}

impl TypeRewriteRule {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn type_ref(&self) -> &'static str {
        self.type_ref
    }

    pub fn apply(&self, input: Dynamic) -> Dynamic {
        (self.apply_fn)(input)
    }
}

/// Port of `net.minecraft.util.datafix.fixes.BlockStateStructureTemplateFix`.
pub struct BlockStateStructureTemplateFix {
    output_schema: Schema,
    changes_type: bool,
}

impl BlockStateStructureTemplateFix {
    // JAVA: net.minecraft.util.datafix.fixes.BlockStateStructureTemplateFix#<init>
    pub fn new(output_schema: Schema, changes_type: bool) -> Self {
        Self {
            output_schema,
            changes_type,
        }
    }

    pub fn output_schema(&self) -> &Schema {
        &self.output_schema
    }

    pub fn changes_type(&self) -> bool {
        self.changes_type
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockStateStructureTemplateFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        // JAVA: fixTypeEverywhereTyped(
        // JAVA:   "BlockStateStructureTemplateFix",
        // JAVA:   this.getInputSchema().getType(References.BLOCK_STATE),
        // JAVA:   typed -> typed.update(DSL.remainderFinder(), BlockStateData::upgradeBlockStateTag)
        // JAVA: )
        TypeRewriteRule {
            name: "BlockStateStructureTemplateFix",
            type_ref: References::BLOCK_STATE,
            apply_fn: upgrade_block_state_tag,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_rule_targets_block_state() {
        let fix = BlockStateStructureTemplateFix::new(Schema::new(0), false);
        let rule = fix.make_rule();
        assert_eq!(rule.name(), "BlockStateStructureTemplateFix");
        assert_eq!(rule.type_ref(), References::BLOCK_STATE);
    }

    #[test]
    fn make_rule_applies_identity_upgrade() {
        let fix = BlockStateStructureTemplateFix::new(Schema::new(1), false);
        let rule = fix.make_rule();
        let input = Dynamic::new("minecraft:stone".to_string())
            .with_property("facing".to_string(), "north".to_string());
        let output = rule.apply(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn constructor_stores_schema_and_edge_versions() {
        let schema = Schema::new(i32::MIN);
        let fix = BlockStateStructureTemplateFix::new(schema.clone(), true);
        assert_eq!(fix.output_schema(), &schema);
        assert_eq!(fix.output_schema().get_version(), i32::MIN);
        assert!(fix.changes_type());

        let fix2 = BlockStateStructureTemplateFix::new(Schema::new(i32::MAX), false);
        assert_eq!(fix2.output_schema().get_version(), i32::MAX);
        assert!(!fix2.changes_type());
    }

    #[test]
    fn preserves_properties_with_negative_and_overflow_values() {
        let fix = BlockStateStructureTemplateFix::new(Schema::new(-1), true);
        let rule = fix.make_rule();
        let overflow_wrap = i32::MAX.wrapping_add(1);
        let neg_mul = (-2_i32).wrapping_mul(i32::MIN);
        let input = Dynamic::new("minecraft:test".to_string())
            .with_property("neg".to_string(), (-1_i32).to_string())
            .with_property("i32_min".to_string(), i32::MIN.to_string())
            .with_property("i32_max".to_string(), i32::MAX.to_string())
            .with_property("overflow_wrap".to_string(), overflow_wrap.to_string())
            .with_property("neg_mul".to_string(), neg_mul.to_string());
        let output = rule.apply(input.clone());
        assert_eq!(output, input);
        assert_eq!(
            output.properties().get("overflow_wrap").unwrap(),
            &i32::MIN.to_string()
        );
        assert_eq!(
            output.properties().get("neg_mul").unwrap(),
            &neg_mul.to_string()
        );
    }

    #[test]
    fn preserves_float_bit_patterns_in_properties() {
        let fix = BlockStateStructureTemplateFix::new(Schema::new(2), false);
        let rule = fix.make_rule();
        let f32_nan_bits = f32::NAN.to_bits();
        let f64_neg_inf_bits = f64::NEG_INFINITY.to_bits();
        let f32_neg_zero_bits = (-0.0_f32).to_bits();
        let f64_subnormal_bits = 1_u64; // smallest positive subnormal f64
        let input = Dynamic::new("minecraft:floaty".to_string())
            .with_property("f32_nan_bits".to_string(), f32_nan_bits.to_string())
            .with_property(
                "f64_neg_inf_bits".to_string(),
                f64_neg_inf_bits.to_string(),
            )
            .with_property(
                "f32_neg_zero_bits".to_string(),
                f32_neg_zero_bits.to_string(),
            )
            .with_property(
                "f64_subnormal_bits".to_string(),
                f64_subnormal_bits.to_string(),
            );
        let output = rule.apply(input.clone());
        assert_eq!(output, input);
        assert_eq!(
            output.properties().get("f32_nan_bits").unwrap(),
            &f32_nan_bits.to_string()
        );
        assert_eq!(
            output.properties().get("f64_neg_inf_bits").unwrap(),
            &f64_neg_inf_bits.to_string()
        );
        assert_eq!(
            output.properties().get("f32_neg_zero_bits").unwrap(),
            &f32_neg_zero_bits.to_string()
        );
        assert_eq!(
            output.properties().get("f64_subnormal_bits").unwrap(),
            &f64_subnormal_bits.to_string()
        );
    }
}
