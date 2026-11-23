use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    Int(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

pub struct VillagerTradeFix;

impl VillagerTradeFix {
    // JAVA: net.minecraft.util.datafix.fixes.VillagerTradeFix#VillagerTradeFix
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.VillagerTradeFix#updateItemStack
    fn update_item_stack(&self, mut item_stack: Dynamic) -> Dynamic {
        if let Dynamic::Map(ref mut map) = item_stack {
            if let Some(Dynamic::String(id)) = map.get_mut("id") {
                if id == "minecraft:carved_pumpkin" {
                    *id = "minecraft:pumpkin".to_string();
                }
            }
        }
        item_stack
    }

    // JAVA: net.minecraft.util.datafix.fixes.VillagerTradeFix#makeRule
    pub fn fix(&self, mut trade: Dynamic) -> Dynamic {
        if let Dynamic::Map(ref mut map) = trade {
            for key in ["buy", "buyB", "sell"] {
                if let Some(value) = map.remove(key) {
                    let fixed = self.update_item_stack(value);
                    map.insert(key.to_string(), fixed);
                }
            }
        }
        trade
    }
}

impl Default for VillagerTradeFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str) -> Dynamic {
        let mut m = BTreeMap::new();
        m.insert("id".to_string(), Dynamic::String(id.to_string()));
        m.insert("Count".to_string(), Dynamic::Int(1));
        Dynamic::Map(m)
    }

    #[test]
    fn carved_pumpkin_becomes_pumpkin() {
        let fix = VillagerTradeFix::new();
        let mut trade = BTreeMap::new();
        trade.insert("buy".to_string(), item("minecraft:carved_pumpkin"));
        trade.insert("sell".to_string(), item("minecraft:emerald"));
        let result = fix.fix(Dynamic::Map(trade));
        let Dynamic::Map(m) = result else { panic!("expected map") };
        assert_eq!(m.get("buy"), Some(&item("minecraft:pumpkin")));
        assert_eq!(m.get("sell"), Some(&item("minecraft:emerald")));
    }

    #[test]
    fn buyb_is_fixed() {
        let fix = VillagerTradeFix::new();
        let mut trade = BTreeMap::new();
        trade.insert("buyB".to_string(), item("minecraft:carved_pumpkin"));
        let result = fix.fix(Dynamic::Map(trade));
        let Dynamic::Map(m) = result else { panic!("expected map") };
        assert_eq!(m.get("buyB"), Some(&item("minecraft:pumpkin")));
    }

    #[test]
    fn non_carved_pumpkin_unchanged() {
        let fix = VillagerTradeFix::new();
        let mut trade = BTreeMap::new();
        trade.insert("buy".to_string(), item("minecraft:apple"));
        let result = fix.fix(Dynamic::Map(trade));
        let Dynamic::Map(m) = result else { panic!("expected map") };
        assert_eq!(m.get("buy"), Some(&item("minecraft:apple")));
    }

    #[test]
    fn missing_fields_no_panic() {
        let fix = VillagerTradeFix::new();
        let result = fix.fix(Dynamic::Map(BTreeMap::new()));
        assert_eq!(result, Dynamic::Map(BTreeMap::new()));
    }

    #[test]
    fn non_map_input_unchanged() {
        let fix = VillagerTradeFix::new();
        let input = Dynamic::String("nope".to_string());
        assert_eq!(fix.fix(input.clone()), input);
    }
}
