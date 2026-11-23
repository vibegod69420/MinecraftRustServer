use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix transformations.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Null,
}

impl Dynamic {
    fn as_map_mut(&mut self) -> Option<&mut BTreeMap<String, Dynamic>> {
        match self {
            Dynamic::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn rename_field(mut self, old: &str, new: &str) -> Self {
        if let Some(m) = self.as_map_mut() {
            if let Some(v) = m.remove(old) {
                m.insert(new.to_string(), v);
            }
        }
        self
    }

    pub fn get(&self, key: &str) -> Option<Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key).cloned(),
            _ => None,
        }
    }

    pub fn set_field_if_present(mut self, key: &str, value: Option<Dynamic>) -> Self {
        if let Some(v) = value {
            if let Some(m) = self.as_map_mut() {
                m.insert(key.to_string(), v);
            }
        }
        self
    }
}

pub struct BlockEntityFurnaceBurnTimeFix {
    pub name: String,
}

impl BlockEntityFurnaceBurnTimeFix {
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityFurnaceBurnTimeFix#<init>
    pub fn new(_schema: (), name: &str) -> Self {
        Self {
            name: format!("BlockEntityFurnaceBurnTimeFix{}", name),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityFurnaceBurnTimeFix#fixBurnTime
    pub fn fix_burn_time(&self, mut d: Dynamic) -> Dynamic {
        d = d.rename_field("CookTime", "cooking_time_spent");
        d = d.rename_field("CookTimeTotal", "cooking_total_time");
        d = d.rename_field("BurnTime", "lit_time_remaining");
        let lit = d.get("lit_time_remaining");
        d.set_field_if_present("lit_total_time", lit)
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityFurnaceBurnTimeFix#fix
    pub fn fix(&self, typed: Dynamic) -> Dynamic {
        // Typed.update(DSL.remainderFinder(), this::fixBurnTime)
        self.fix_burn_time(typed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn renames_furnace_fields_and_copies_lit_time() {
        let fix = BlockEntityFurnaceBurnTimeFix::new((), "Furnace");
        let input = map(vec![
            ("CookTime", Dynamic::Int(10)),
            ("CookTimeTotal", Dynamic::Int(200)),
            ("BurnTime", Dynamic::Int(100)),
        ]);
        let out = fix.fix_burn_time(input);
        let expected = map(vec![
            ("cooking_time_spent", Dynamic::Int(10)),
            ("cooking_total_time", Dynamic::Int(200)),
            ("lit_time_remaining", Dynamic::Int(100)),
            ("lit_total_time", Dynamic::Int(100)),
        ]);
        assert_eq!(out, expected);
    }

    #[test]
    fn missing_burn_time_does_not_add_lit_total() {
        let fix = BlockEntityFurnaceBurnTimeFix::new((), "BlastFurnace");
        let input = map(vec![
            ("CookTime", Dynamic::Int(5)),
            ("CookTimeTotal", Dynamic::Int(100)),
        ]);
        let out = fix.fix(input);
        let expected = map(vec![
            ("cooking_time_spent", Dynamic::Int(5)),
            ("cooking_total_time", Dynamic::Int(100)),
        ]);
        assert_eq!(out, expected);
        assert_eq!(fix.name, "BlockEntityFurnaceBurnTimeFixBlastFurnace");
    }

    #[test]
    fn existing_lit_time_remaining_is_copied_when_no_burn_time() {
        let fix = BlockEntityFurnaceBurnTimeFix::new((), "Smoker");
        let input = map(vec![("lit_time_remaining", Dynamic::Int(42))]);
        let out = fix.fix_burn_time(input);
        let expected = map(vec![
            ("lit_time_remaining", Dynamic::Int(42)),
            ("lit_total_time", Dynamic::Int(42)),
        ]);
        assert_eq!(out, expected);
    }

    #[test]
    fn empty_map_stays_empty() {
        let fix = BlockEntityFurnaceBurnTimeFix::new((), "Furnace");
        let input = Dynamic::Map(BTreeMap::new());
        let out = fix.fix_burn_time(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn negative_and_large_values_preserved() {
        let fix = BlockEntityFurnaceBurnTimeFix::new((), "Furnace");
        let input = map(vec![
            ("CookTime", Dynamic::Int(-1)),
            ("CookTimeTotal", Dynamic::Int(i32::MAX as i64)),
            ("BurnTime", Dynamic::Int(i64::MIN / 2)),
        ]);
        let out = fix.fix_burn_time(input);
        assert_eq!(
            out.get("cooking_time_spent"),
            Some(Dynamic::Int(-1))
        );
        assert_eq!(
            out.get("cooking_total_time"),
            Some(Dynamic::Int(i32::MAX as i64))
        );
        assert_eq!(
            out.get("lit_time_remaining"),
            Some(Dynamic::Int(i64::MIN / 2))
        );
        assert_eq!(
            out.get("lit_total_time"),
            Some(Dynamic::Int(i64::MIN / 2))
        );
    }
}
