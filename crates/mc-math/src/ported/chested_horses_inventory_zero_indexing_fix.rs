//! Faithful port of `net.minecraft.util.datafix.fixes.ChestedHorsesInventoryZeroIndexingFix`.
//!
//! Java 1.21.11 source:
//! - adjusts item stack `Slot` bytes in chest-horse-like entity inventories by subtracting 2,
//!   clamped/defaulted through Java's `Dynamic#asInt(2)` semantics.

/// Minimal dynamic representation used by this self-contained port.
///
/// Java's `Dynamic<T>` is a DataFixerUpper abstraction. For this fix we only need:
/// - reading an integer from a `Slot` field with Java-like default `2`
/// - creating a byte value for the updated `Slot`
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Bool(bool),
    Compound(Vec<(String, DynamicValue)>),
    List(Vec<DynamicValue>),
    Other,
}

impl DynamicValue {
    // JAVA: net.minecraft.util.datafix.fixes.ChestedHorsesInventoryZeroIndexingFix#makeRule
    // (supporting behavior: Dynamic#asInt fallback/default semantics are represented here)
    fn as_int(&self, default: i32) -> i32 {
        match self {
            DynamicValue::Byte(v) => i32::from(*v),
            DynamicValue::Short(v) => i32::from(*v),
            DynamicValue::Int(v) => *v,
            DynamicValue::Long(v) => *v as i32,
            DynamicValue::Float(v) => *v as i32,
            DynamicValue::Double(v) => *v as i32,
            DynamicValue::Bool(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            DynamicValue::String(s) => s.parse::<i32>().unwrap_or(default),
            _ => default,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChestedHorsesInventoryZeroIndexingFix#horseLikeInventoryIndexingFixer
    // (supporting behavior: Dynamic#createByte)
    fn create_byte(value: i8) -> Self {
        DynamicValue::Byte(value)
    }

    fn as_compound_mut(&mut self) -> Option<&mut Vec<(String, DynamicValue)>> {
        match self {
            DynamicValue::Compound(entries) => Some(entries),
            _ => None,
        }
    }

    fn as_compound(&self) -> Option<&[(String, DynamicValue)]> {
        match self {
            DynamicValue::Compound(entries) => Some(entries.as_slice()),
            _ => None,
        }
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut DynamicValue> {
        self.as_compound_mut()?
            .iter_mut()
            .find_map(|(k, v)| if k == key { Some(v) } else { None })
    }

    fn get(&self, key: &str) -> Option<&DynamicValue> {
        self.as_compound()?
            .iter()
            .find_map(|(k, v)| if k == key { Some(v) } else { None })
    }
}

/// Minimal schema/type placeholders mirroring DataFixerUpper wiring.
/// This fix does not depend on schema content in Java beyond type registration.
#[derive(Debug, Clone, Default)]
pub struct Schema;

/// DataFixerUpper reference constants relevant to this fix.
pub struct References;

impl References {
    pub const ITEM_STACK: &'static str = "minecraft:item_stack";
    pub const ENTITY: &'static str = "minecraft:entity";
}

/// Minimal optic/type placeholders to keep the port self-contained.
#[derive(Debug, Clone, Default)]
pub struct OpticFinder;

#[derive(Debug, Clone, Default)]
pub struct TypeRewriteRule;

/// Faithful port of `ChestedHorsesInventoryZeroIndexingFix`.
#[derive(Debug, Clone)]
pub struct ChestedHorsesInventoryZeroIndexingFix {
    _input_schema: Schema,
}

impl ChestedHorsesInventoryZeroIndexingFix {
    // JAVA: net.minecraft.util.datafix.fixes.ChestedHorsesInventoryZeroIndexingFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            _input_schema: schema,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChestedHorsesInventoryZeroIndexingFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        // Java builds a TypeRewriteRule sequence over four horse-like entity choices.
        // In this self-contained port, rule construction is represented as a no-op marker;
        // the behavioral fix is exposed via `fix_entity_dynamic`.
        let _item_stack_finder = OpticFinder;
        let _entity_type = ();
        let _rule = self.horse_like_inventory_indexing_fixer(
            &_item_stack_finder,
            &_entity_type,
            "minecraft:llama",
        );
        let _rule = self.horse_like_inventory_indexing_fixer(
            &_item_stack_finder,
            &_entity_type,
            "minecraft:trader_llama",
        );
        let _rule = self.horse_like_inventory_indexing_fixer(
            &_item_stack_finder,
            &_entity_type,
            "minecraft:mule",
        );
        let _rule = self.horse_like_inventory_indexing_fixer(
            &_item_stack_finder,
            &_entity_type,
            "minecraft:donkey",
        );
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChestedHorsesInventoryZeroIndexingFix#horseLikeInventoryIndexingFixer
    fn horse_like_inventory_indexing_fixer(
        &self,
        _item_stack_finder: &OpticFinder,
        _entity_type: &(),
        entity_id: &str,
    ) -> TypeRewriteRule {
        // Java:
        // - finds named entity choice `entity_id`
        // - finds "Items" field
        // - for each item stack dynamic, updates "Slot": byte(asInt(2) - 2)
        let _ = entity_id;
        TypeRewriteRule
    }

    /// Self-contained behavioral entry point corresponding to applying this fix to a single
    /// entity dynamic value.
    ///
    /// Java applies this through DFU typed rewriting for specific entity choices. This helper
    /// preserves the core slot transformation semantics for those entity ids.
    // JAVA: net.minecraft.util.datafix.fixes.ChestedHorsesInventoryZeroIndexingFix#horseLikeInventoryIndexingFixer
    pub fn fix_entity_dynamic(&self, entity_id: &str, entity: &mut DynamicValue) {
        if !matches!(
            entity_id,
            "minecraft:llama"
                | "minecraft:trader_llama"
                | "minecraft:mule"
                | "minecraft:donkey"
        ) {
            return;
        }

        let Some(items) = entity.get_mut("Items") else {
            return;
        };
        let DynamicValue::List(items_list) = items else {
            return;
        };

        for item in items_list.iter_mut() {
            Self::fix_item_stack_slot(item);
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChestedHorsesInventoryZeroIndexingFix#horseLikeInventoryIndexingFixer
    fn fix_item_stack_slot(item_stack: &mut DynamicValue) {
        let Some(slot) = item_stack.get_mut("Slot") else {
            return;
        };
        let slot_int = slot.as_int(2);
        let new_slot = (slot_int.wrapping_sub(2)) as i8;
        *slot = DynamicValue::create_byte(new_slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(entries: Vec<(&str, DynamicValue)>) -> DynamicValue {
        DynamicValue::Compound(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn item(slot: DynamicValue) -> DynamicValue {
        compound(vec![("Slot", slot), ("id", DynamicValue::String("minecraft:apple".to_string()))])
    }

    fn entity_with_items(items: Vec<DynamicValue>) -> DynamicValue {
        compound(vec![("Items", DynamicValue::List(items))])
    }

    #[test]
    fn fixes_known_chested_horse_slots_by_subtracting_two() {
        let fix = ChestedHorsesInventoryZeroIndexingFix::new(Schema);
        let mut entity = entity_with_items(vec![
            item(DynamicValue::Byte(2)),
            item(DynamicValue::Int(5)),
            item(DynamicValue::Short(3)),
        ]);

        fix.fix_entity_dynamic("minecraft:llama", &mut entity);

        let items = match entity.get("Items").unwrap() {
            DynamicValue::List(v) => v,
            _ => panic!("expected list"),
        };
        assert_eq!(items[0].get("Slot"), Some(&DynamicValue::Byte(0)));
        assert_eq!(items[1].get("Slot"), Some(&DynamicValue::Byte(3)));
        assert_eq!(items[2].get("Slot"), Some(&DynamicValue::Byte(1)));
    }

    #[test]
    fn uses_java_default_slot_value_two_when_missing_or_unparseable() {
        let fix = ChestedHorsesInventoryZeroIndexingFix::new(Schema);
        let mut entity = entity_with_items(vec![
            compound(vec![("id", DynamicValue::String("minecraft:apple".to_string()))]),
            item(DynamicValue::String("not-a-number".to_string())),
            item(DynamicValue::Other),
        ]);

        fix.fix_entity_dynamic("minecraft:donkey", &mut entity);

        let items = match entity.get("Items").unwrap() {
            DynamicValue::List(v) => v,
            _ => panic!("expected list"),
        };
        // Missing Slot: unchanged because Java updates existing Slot field only.
        assert_eq!(items[0].get("Slot"), None);
        // Unparseable/non-numeric: asInt(2) default => 2-2 = 0
        assert_eq!(items[1].get("Slot"), Some(&DynamicValue::Byte(0)));
        assert_eq!(items[2].get("Slot"), Some(&DynamicValue::Byte(0)));
    }

    #[test]
    fn ignores_unrelated_entity_ids() {
        let fix = ChestedHorsesInventoryZeroIndexingFix::new(Schema);
        let mut entity = entity_with_items(vec![item(DynamicValue::Byte(4))]);
        let original = entity.clone();

        fix.fix_entity_dynamic("minecraft:horse", &mut entity);

        assert_eq!(entity, original);
    }

    #[test]
    fn handles_negative_and_wrapping_slot_values_with_java_int_then_byte_cast() {
        let fix = ChestedHorsesInventoryZeroIndexingFix::new(Schema);
        let mut entity = entity_with_items(vec![
            item(DynamicValue::Int(-1)),
            item(DynamicValue::Int(i32::MIN)),
            item(DynamicValue::Long(130)),
        ]);

        fix.fix_entity_dynamic("minecraft:mule", &mut entity);

        let items = match entity.get("Items").unwrap() {
            DynamicValue::List(v) => v,
            _ => panic!("expected list"),
        };
        // -1 - 2 = -3
        assert_eq!(items[0].get("Slot"), Some(&DynamicValue::Byte(-3)));
        // i32::MIN wrapping_sub 2 keeps Java int wrap, then cast to i8 in Java byte storage.
        let expected_min = (i32::MIN.wrapping_sub(2)) as i8;
        assert_eq!(items[1].get("Slot"), Some(&DynamicValue::Byte(expected_min)));
        // 130 - 2 = 128, cast to i8 in Rust/Java byte narrowing => -128
        assert_eq!(items[2].get("Slot"), Some(&DynamicValue::Byte(-128)));
    }

    #[test]
    fn no_items_field_or_non_list_is_noop() {
        let fix = ChestedHorsesInventoryZeroIndexingFix::new(Schema);

        let mut no_items = compound(vec![("id", DynamicValue::String("minecraft:llama".to_string()))]);
        let no_items_original = no_items.clone();
        fix.fix_entity_dynamic("minecraft:llama", &mut no_items);
        assert_eq!(no_items, no_items_original);

        let mut items_not_list = compound(vec![("Items", DynamicValue::Int(1))]);
        let items_not_list_original = items_not_list.clone();
        fix.fix_entity_dynamic("minecraft:trader_llama", &mut items_not_list);
        assert_eq!(items_not_list, items_not_list_original);
    }

    #[test]
    fn applies_to_all_four_target_entity_types() {
        let fix = ChestedHorsesInventoryZeroIndexingFix::new(Schema);
        for id in [
            "minecraft:llama",
            "minecraft:trader_llama",
            "minecraft:mule",
            "minecraft:donkey",
        ] {
            let mut entity = entity_with_items(vec![item(DynamicValue::Byte(7))]);
            fix.fix_entity_dynamic(id, &mut entity);
            let items = match entity.get("Items").unwrap() {
                DynamicValue::List(v) => v,
                _ => panic!("expected list"),
            };
            assert_eq!(items[0].get("Slot"), Some(&DynamicValue::Byte(5)), "failed for {id}");
        }
    }
}
