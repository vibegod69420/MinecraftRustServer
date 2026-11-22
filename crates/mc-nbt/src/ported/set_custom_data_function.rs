use crate::NbtTag;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemFunctionType {
    SetCustomData,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ItemStack {
    pub custom_data: Option<NbtTag>,
}

impl ItemStack {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetCustomDataFunction {
    pub conditions: Vec<LootItemCondition>,
    pub tag: NbtTag,
}

impl SetCustomDataFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomDataFunction#<init>
    fn new(conditions: Vec<LootItemCondition>, tag: NbtTag) -> Self {
        Self { conditions, tag }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomDataFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::SetCustomData
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomDataFunction#run
    pub fn run(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
        update_custom_data(&mut stack, |data| {
            merge_compound(data, &self.tag);
        });
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomDataFunction#setCustomData
    #[deprecated(note = "Mirrors Java @Deprecated; use builder pipeline in full implementation")]
    pub fn set_custom_data(
        tag: NbtTag,
    ) -> impl FnOnce(Vec<LootItemCondition>) -> SetCustomDataFunction {
        move |conditions| SetCustomDataFunction::new(conditions, tag)
    }
}

/// Faithful model of `CustomData.update(DataComponents.CUSTOM_DATA, stack, consumer)`.
fn update_custom_data<F: FnOnce(&mut NbtTag)>(stack: &mut ItemStack, f: F) {
    let mut data = match stack.custom_data.take() {
        Some(NbtTag::Compound(entries)) => NbtTag::Compound(entries),
        _ => NbtTag::Compound(Vec::new()),
    };
    f(&mut data);
    stack.custom_data = Some(data);
}

/// Faithful model of `CompoundTag#merge(CompoundTag)`.
/// Recursively merges nested compounds; replaces non-compound values and type-mismatched keys.
fn merge_compound(target: &mut NbtTag, source: &NbtTag) {
    let source_entries = match source {
        NbtTag::Compound(entries) => entries.clone(),
        _ => return,
    };
    let target_entries = match target {
        NbtTag::Compound(entries) => entries,
        _ => return,
    };

    for (key, value) in source_entries {
        let existing_idx = target_entries.iter().position(|(k, _)| *k == key);
        let source_is_compound = matches!(value, NbtTag::Compound(_));

        if source_is_compound {
            if let Some(idx) = existing_idx {
                if matches!(target_entries[idx].1, NbtTag::Compound(_)) {
                    merge_compound(&mut target_entries[idx].1, &value);
                    continue;
                }
            }
        }

        match existing_idx {
            Some(idx) => target_entries[idx].1 = value,
            None => target_entries.push((key, value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NbtTag;

    fn compound(entries: Vec<(&str, NbtTag)>) -> NbtTag {
        NbtTag::Compound(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn run_merges_custom_data_into_empty_stack() {
        let tag = compound(vec![("a", NbtTag::Int(1)), ("b", NbtTag::String("x".into()))]);
        let func = SetCustomDataFunction::new(vec![], tag.clone());
        let out = func.run(ItemStack::new(), &LootContext);
        assert_eq!(out.custom_data, Some(tag));
        assert_eq!(func.get_type(), LootItemFunctionType::SetCustomData);
    }

    #[test]
    fn run_overwrites_non_compound_and_preserves_existing() {
        let mut stack = ItemStack::new();
        stack.custom_data = Some(compound(vec![
            ("a", NbtTag::Int(1)),
            ("keep", NbtTag::Byte(-1)),
        ]));
        let func = SetCustomDataFunction::new(vec![], compound(vec![("a", NbtTag::Int(2))]));
        let out = func.run(stack, &LootContext);
        let data = out.custom_data.unwrap();
        assert_eq!(data.compound_get("a"), Some(&NbtTag::Int(2)));
        assert_eq!(data.compound_get("keep"), Some(&NbtTag::Byte(-1)));
    }

    #[test]
    fn merge_recursive_compound_and_type_mismatch() {
        let mut stack = ItemStack::new();
        stack.custom_data = Some(compound(vec![
            ("nested", compound(vec![("x", NbtTag::Int(1)), ("y", NbtTag::Int(2))])),
            ("mismatch", NbtTag::Int(9)),
        ]));
        let func = SetCustomDataFunction::new(
            vec![],
            compound(vec![
                ("nested", compound(vec![("y", NbtTag::Int(20)), ("z", NbtTag::Int(3))])),
                ("mismatch", compound(vec![("inner", NbtTag::Byte(1))])),
            ]),
        );
        let out = func.run(stack, &LootContext);
        let data = out.custom_data.unwrap();
        let nested = data.compound_get("nested").unwrap();
        assert_eq!(nested.compound_get("x"), Some(&NbtTag::Int(1)));
        assert_eq!(nested.compound_get("y"), Some(&NbtTag::Int(20)));
        assert_eq!(nested.compound_get("z"), Some(&NbtTag::Int(3)));
        let mismatch = data.compound_get("mismatch").unwrap();
        assert_eq!(mismatch.compound_get("inner"), Some(&NbtTag::Byte(1)));
    }

    #[test]
    fn run_preserves_float_bits_and_negative_values() {
        let tag = compound(vec![
            ("f", NbtTag::Float(f32::from_bits(0xFFC0_0001))), // NaN payload
            ("d", NbtTag::Double(f64::from_bits(0x8000_0000_0000_0000))), // -0.0
            ("neg", NbtTag::Long(i64::MIN)),
        ]);
        let func = SetCustomDataFunction::new(vec![], tag.clone());
        let out = func.run(ItemStack::new(), &LootContext);
        let data = out.custom_data.unwrap();
        match data.compound_get("f").unwrap() {
            NbtTag::Float(v) => assert_eq!(v.to_bits(), 0xFFC0_0001),
            _ => panic!("expected float"),
        }
        match data.compound_get("d").unwrap() {
            NbtTag::Double(v) => assert_eq!(v.to_bits(), 0x8000_0000_0000_0000),
            _ => panic!("expected double"),
        }
        assert_eq!(data.compound_get("neg"), Some(&NbtTag::Long(i64::MIN)));
    }

    #[test]
    #[allow(deprecated)]
    fn set_custom_data_builder_creates_function() {
        let tag = compound(vec![("k", NbtTag::Short(7))]);
        let builder = SetCustomDataFunction::set_custom_data(tag.clone());
        let func = builder(vec![LootItemCondition]);
        assert_eq!(func.conditions.len(), 1);
        assert_eq!(func.tag, tag);
    }
}
