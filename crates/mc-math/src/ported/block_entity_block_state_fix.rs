use std::collections::BTreeMap;

/// Minimal self-contained dynamic/tag representation for the 1.21.11
/// `BlockEntityBlockStateFix` datafix.
///
/// Minecraft datafixers operate on `Dynamic<?>` / typed optics. For this
/// faithful port we model the subset of behavior needed by the fix:
/// - read/remove integer fields from a root compound-like object
/// - look up a legacy block-state tag by packed numeric id
/// - attach a transformed block-state payload to the output entity
///
/// The implementation preserves Java control flow and error behavior.
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    Int(i32),
    String(String),
    Compound(BTreeMap<String, DynamicValue>),
}

impl DynamicValue {
    /// Create an empty compound value.
    pub fn compound() -> Self {
        Self::Compound(BTreeMap::new())
    }

    /// Get a child field from a compound, if present.
    pub fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Compound(map) => map.get(key),
            _ => None,
        }
    }

    /// Read an integer field, returning `default` when absent or not an int.
    /// Mirrors `Dynamic.get(...).asInt(default)`.
    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        match self.get(key) {
            Some(Self::Int(v)) => *v,
            _ => default,
        }
    }

    /// Remove a field from a compound, returning a new value.
    /// Mirrors `Dynamic.remove(...)` by producing an updated dynamic.
    pub fn remove(&self, key: &str) -> Self {
        match self {
            Self::Compound(map) => {
                let mut next = map.clone();
                next.remove(key);
                Self::Compound(next)
            }
            _ => self.clone(),
        }
    }

    /// Insert or replace a field in a compound, returning a new value.
    pub fn set(&self, key: &str, value: Self) -> Self {
        match self {
            Self::Compound(map) => {
                let mut next = map.clone();
                next.insert(key.to_string(), value);
                Self::Compound(next)
            }
            _ => self.clone(),
        }
    }

    /// Convert to a compound map reference.
    pub fn as_compound(&self) -> Option<&BTreeMap<String, DynamicValue>> {
        match self {
            Self::Compound(map) => Some(map),
            _ => None,
        }
    }
}

/// Minimal typed wrapper equivalent to DataFixerUpper `Typed<?>` for this fix.
/// It carries the remainder dynamic payload that the Java code reads/mutates.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedBlockEntity {
    remainder: DynamicValue,
}

impl TypedBlockEntity {
    /// Create a typed block entity from its remainder dynamic.
    pub fn new(remainder: DynamicValue) -> Self {
        Self { remainder }
    }

    /// Equivalent of `Typed.get(DSL.remainderFinder())`.
    pub fn get_remainder(&self) -> &DynamicValue {
        &self.remainder
    }

    /// Equivalent of `Typed.set(DSL.remainderFinder(), value)`.
    pub fn set_remainder(mut self, remainder: DynamicValue) -> Self {
        self.remainder = remainder;
        self
    }

    /// Equivalent of setting the found `blockState` field.
    /// In this self-contained port, the field is stored in the remainder compound.
    pub fn set_block_state(self, block_state: DynamicValue) -> Self {
        let remainder = self.remainder.set("blockState", block_state);
        self.set_remainder(remainder)
    }
}

/// Minimal stand-in for `BlockStateData.getTag(...)`.
///
/// Java maps legacy numeric block ids/data to a block state dynamic/tag.
/// This trait keeps the fix self-contained while allowing callers/tests to
/// provide faithful mappings.
pub trait BlockStateData {
    fn get_tag(packed_id: i32) -> Option<DynamicValue>;
}

/// Default block state data provider used by the fix.
///
/// Returns a minimal compound with a `Name` field derived from the packed id.
/// This is sufficient for a self-contained port and deterministic tests.
/// Real integrations can use `BlockEntityBlockStateFix::fix_with`.
pub struct DefaultBlockStateData;

impl BlockStateData for DefaultBlockStateData {
    fn get_tag(packed_id: i32) -> Option<DynamicValue> {
        let mut map = BTreeMap::new();
        map.insert(
            "Name".to_string(),
            DynamicValue::String(format!("minecraft:legacy/block_{}", packed_id)),
        );
        Some(DynamicValue::Compound(map))
    }
}

/// Port of `net.minecraft.util.datafix.fixes.BlockEntityBlockStateFix`.
///
/// The original class extends `NamedEntityFix` and uses DataFixerUpper schemas,
/// optic finders, and typed reads. Those infrastructure types are not ported
/// here; we preserve the method-level behavior of the fix itself.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockEntityBlockStateFix;

impl BlockEntityBlockStateFix {
    /// Creates a new fix instance.
    ///
    /// Java signature: `BlockEntityBlockStateFix(Schema outputSchema, boolean changesType)`
    /// The schema/changesType are DataFixerUpper wiring details and are not
    /// needed for this self-contained behavioral port.
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityBlockStateFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityBlockStateFix#fix
    pub fn fix(input: TypedBlockEntity) -> Result<TypedBlockEntity, String> {
        Self::fix_with::<DefaultBlockStateData>(input)
    }

    /// Testable/self-contained variant of `fix` with an injected block-state
    /// data provider. This preserves the Java control flow exactly while
    /// avoiding unported external DataFixerUpper classes.
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityBlockStateFix#fix
    pub fn fix_with<B: BlockStateData>(input: TypedBlockEntity) -> Result<TypedBlockEntity, String> {
        // Type<?> $$1 = this.getOutputSchema().getChoiceType(References.BLOCK_ENTITY, "minecraft:piston");
        // Type<?> $$2 = $$1.findFieldType("blockState");
        // OpticFinder<?> $$3 = DSL.fieldFinder("blockState", $$2);
        // Dynamic<?> $$4 = $$0.get(DSL.remainderFinder());
        let mut dynamic = input.get_remainder().clone();

        // int $$5 = $$4.get("blockId").asInt(0);
        let block_id = dynamic.get_int_or("blockId", 0);
        // $$4 = $$4.remove("blockId");
        dynamic = dynamic.remove("blockId");

        // int $$6 = $$4.get("blockData").asInt(0) & 15;
        let block_data = dynamic.get_int_or("blockData", 0) & 15;
        // $$4 = $$4.remove("blockData");
        dynamic = dynamic.remove("blockData");

        // Dynamic<?> $$7 = BlockStateData.getTag($$5 << 4 | $$6);
        let packed_id = (block_id << 4) | block_data;
        let block_state_tag = B::get_tag(packed_id)
            .ok_or_else(|| "Could not parse newly created block state tag.".to_string())?;

        // Typed<?> $$8 = $$1.pointTyped($$0.getOps()).orElseThrow(() -> new IllegalStateException("Could not create new piston block entity."));
        let output = TypedBlockEntity::new(DynamicValue::compound())
            .ok_or_create()?;

        // return $$8.set(DSL.remainderFinder(), $$4)
        //    .set($$3, $$2.readTyped($$7).result().orElseThrow(() -> new IllegalStateException("Could not parse newly created block state tag.")).getFirst());
        let parsed_block_state = read_typed_block_state(block_state_tag)?;

        Ok(output
            .set_remainder(dynamic)
            .set_block_state(parsed_block_state))
    }
}

/// Helper mirroring the Java `pointTyped(...).orElseThrow(...)` creation step.
trait PointTyped {
    fn ok_or_create(self) -> Result<TypedBlockEntity, String>;
}

impl PointTyped for TypedBlockEntity {
    fn ok_or_create(self) -> Result<TypedBlockEntity, String> {
        // In Java this can fail if the typed point cannot be created.
        // Our self-contained representation always creates it; keep the error
        // message available for faithful behavior in adapted callers.
        if false {
            return Err("Could not create new piston block entity.".to_string());
        }
        Ok(self)
    }
}

/// Helper mirroring `$$2.readTyped($$7).result().orElseThrow(...)`.
fn read_typed_block_state(tag: DynamicValue) -> Result<DynamicValue, String> {
    // Java reads/parses the dynamic into the blockState type. For this
    // self-contained port, a compound tag is accepted as a parsed block state.
    match tag {
        DynamicValue::Compound(_) => Ok(tag),
        _ => Err("Could not parse newly created block state tag.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(entries: &[(&str, DynamicValue)]) -> DynamicValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert((*k).to_string(), v.clone());
        }
        DynamicValue::Compound(map)
    }

    struct TestBlockStateData;

    impl BlockStateData for TestBlockStateData {
        fn get_tag(packed_id: i32) -> Option<DynamicValue> {
            Some(compound(&[
                ("Name", DynamicValue::String("minecraft:piston_head".to_string())),
                ("PackedId", DynamicValue::Int(packed_id)),
            ]))
        }
    }

    struct MissingBlockStateData;

    impl BlockStateData for MissingBlockStateData {
        fn get_tag(_packed_id: i32) -> Option<DynamicValue> {
            None
        }
    }

    #[test]
    fn fix_moves_legacy_id_and_data_into_block_state() {
        let input = TypedBlockEntity::new(compound(&[
            ("blockId", DynamicValue::Int(5)),
            ("blockData", DynamicValue::Int(3)),
            ("facing", DynamicValue::Int(1)),
        ]));

        let output = BlockEntityBlockStateFix::fix_with::<TestBlockStateData>(input).unwrap();
        let remainder = output.get_remainder();

        assert_eq!(remainder.get("blockId"), None);
        assert_eq!(remainder.get("blockData"), None);
        assert_eq!(remainder.get_int_or("facing", -1), 1);

        let block_state = remainder.get("blockState").expect("blockState should be set");
        assert_eq!(
            block_state.get("Name"),
            Some(&DynamicValue::String("minecraft:piston_head".to_string()))
        );
        // (5 << 4) | (3 & 15) = 83
        assert_eq!(block_state.get_int_or("PackedId", -1), 83);
    }

    #[test]
    fn fix_masks_block_data_to_low_nibble() {
        let input = TypedBlockEntity::new(compound(&[
            ("blockId", DynamicValue::Int(1)),
            ("blockData", DynamicValue::Int(0xFF)),
        ]));

        let output = BlockEntityBlockStateFix::fix_with::<TestBlockStateData>(input).unwrap();
        let block_state = output.get_remainder().get("blockState").unwrap();
        // (1 << 4) | (255 & 15) = 16 | 15 = 31
        assert_eq!(block_state.get_int_or("PackedId", -1), 31);
    }

    #[test]
    fn fix_defaults_missing_fields_to_zero() {
        let input = TypedBlockEntity::new(compound(&[("keep", DynamicValue::Int(7))]));

        let output = BlockEntityBlockStateFix::fix_with::<TestBlockStateData>(input).unwrap();
        let remainder = output.get_remainder();
        assert_eq!(remainder.get_int_or("keep", -1), 7);
        let block_state = remainder.get("blockState").unwrap();
        // (0 << 4) | (0 & 15) = 0
        assert_eq!(block_state.get_int_or("PackedId", -1), 0);
    }

    #[test]
    fn fix_handles_negative_block_data_with_java_mask() {
        let input = TypedBlockEntity::new(compound(&[
            ("blockId", DynamicValue::Int(2)),
            ("blockData", DynamicValue::Int(-1)),
        ]));

        let output = BlockEntityBlockStateFix::fix_with::<TestBlockStateData>(input).unwrap();
        let block_state = output.get_remainder().get("blockState").unwrap();
        // Java: -1 & 15 == 15
        // (2 << 4) | 15 = 47
        assert_eq!(block_state.get_int_or("PackedId", -1), 47);
    }

    #[test]
    fn fix_errors_when_block_state_tag_missing() {
        let input = TypedBlockEntity::new(compound(&[
            ("blockId", DynamicValue::Int(10)),
            ("blockData", DynamicValue::Int(1)),
        ]));

        let err = BlockEntityBlockStateFix::fix_with::<MissingBlockStateData>(input).unwrap_err();
        assert_eq!(err, "Could not parse newly created block state tag.");
    }

    #[test]
    fn read_typed_block_state_rejects_non_compound() {
        let err = read_typed_block_state(DynamicValue::Int(1)).unwrap_err();
        assert_eq!(err, "Could not parse newly created block state tag.");
    }
}
