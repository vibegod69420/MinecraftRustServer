use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Byte(i8),
    Bool(bool),
    Float(f32),
    Double(f64),
    Null,
}

pub struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    // JAVA: com.mojang.serialization.OptionalDynamic#result
    pub fn result(self) -> Option<Dynamic> {
        self.0
    }

    // JAVA: com.mojang.serialization.Dynamic#asBoolean
    pub fn as_boolean(self, default: bool) -> bool {
        match self.0 {
            Some(Dynamic::Bool(v)) => v,
            Some(Dynamic::Byte(v)) => v != 0,
            Some(Dynamic::Int(v)) => v != 0,
            Some(Dynamic::Long(v)) => v != 0,
            _ => default,
        }
    }
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Map(m) => OptionalDynamic(m.get(key).cloned()),
            _ => OptionalDynamic(None),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createByte
    pub fn create_byte(&self, v: i8) -> Dynamic {
        Dynamic::Byte(v)
    }

    // JAVA: com.mojang.serialization.Dynamic#set
    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut new_map = m.clone();
                new_map.insert(key.to_string(), value);
                Dynamic::Map(new_map)
            }
            _ => self.clone(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#remove
    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut new_map = m.clone();
                new_map.remove(key);
                Dynamic::Map(new_map)
            }
            _ => self.clone(),
        }
    }
}

pub struct AbstractArrowPickupFix;

impl AbstractArrowPickupFix {
    // JAVA: net.minecraft.util.datafix.fixes.AbstractArrowPickupFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.AbstractArrowPickupFix#updateProjectiles
    pub fn update_projectiles(&self, name: &str, data: Dynamic) -> Dynamic {
        let data = self.update_entity(name, data, "minecraft:arrow", Self::update_pickup);
        let data = self.update_entity(name, data, "minecraft:spectral_arrow", Self::update_pickup);
        self.update_entity(name, data, "minecraft:trident", Self::update_pickup)
    }

    // JAVA: net.minecraft.util.datafix.fixes.AbstractArrowPickupFix#updatePickup
    pub fn update_pickup(d: Dynamic) -> Dynamic {
        if d.get("pickup").result().is_some() {
            return d;
        }

        let player = d.get("player").as_boolean(true);
        let pickup_byte = d.create_byte(if player { 1i8 } else { 0i8 });
        d.set("pickup", pickup_byte).remove("player")
    }

    // JAVA: net.minecraft.util.datafix.fixes.AbstractArrowPickupFix#updateEntity
    fn update_entity(
        &self,
        name: &str,
        data: Dynamic,
        target: &str,
        f: fn(Dynamic) -> Dynamic,
    ) -> Dynamic {
        if name == target {
            f(data)
        } else {
            data
        }
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
    fn arrow_player_true_sets_pickup_1() {
        let fix = AbstractArrowPickupFix::new(());
        let data = map(vec![("player".into(), Dynamic::Bool(true))]);
        let out = fix.update_projectiles("minecraft:arrow", data);
        assert_eq!(out.get("pickup").result(), Some(Dynamic::Byte(1)));
        assert_eq!(out.get("player").result(), None);
    }

    #[test]
    fn arrow_player_false_sets_pickup_0() {
        let fix = AbstractArrowPickupFix::new(());
        let data = map(vec![("player".into(), Dynamic::Bool(false))]);
        let out = fix.update_projectiles("minecraft:spectral_arrow", data);
        assert_eq!(out.get("pickup").result(), Some(Dynamic::Byte(0)));
        assert_eq!(out.get("player").result(), None);
    }

    #[test]
    fn missing_player_defaults_to_true() {
        let fix = AbstractArrowPickupFix::new(());
        let out = fix.update_projectiles("minecraft:trident", map(vec![]));
        assert_eq!(out.get("pickup").result(), Some(Dynamic::Byte(1)));
    }

    #[test]
    fn existing_pickup_is_preserved() {
        let fix = AbstractArrowPickupFix::new(());
        let data = map(vec![
            ("pickup".into(), Dynamic::Int(2)),
            ("player".into(), Dynamic::Bool(false)),
        ]);
        let out = fix.update_projectiles("minecraft:arrow", data.clone());
        assert_eq!(out, data);
    }

    #[test]
    fn non_projectile_unchanged() {
        let fix = AbstractArrowPickupFix::new(());
        let data = map(vec![("player".into(), Dynamic::Bool(false))]);
        let out = fix.update_projectiles("minecraft:zombie", data.clone());
        assert_eq!(out, data);
    }

    #[test]
    fn preserves_float_bits_and_negative_values() {
        let fix = AbstractArrowPickupFix::new(());
        let bits = f64::NAN.to_bits();
        let data = map(vec![
            ("player".into(), Dynamic::Byte(0)),
            ("damage".into(), Dynamic::Double(f64::from_bits(bits))),
            ("owner".into(), Dynamic::Int(i32::MIN)),
        ]);
        let out = fix.update_projectiles("minecraft:arrow", data);
        assert_eq!(out.get("pickup").result(), Some(Dynamic::Byte(0)));
        assert_eq!(out.get("owner").result(), Some(Dynamic::Int(i32::MIN)));
        match out.get("damage").result() {
            Some(Dynamic::Double(f)) => assert_eq!(f.to_bits(), bits),
            other => panic!("{:?}", other),
        }
    }
}
