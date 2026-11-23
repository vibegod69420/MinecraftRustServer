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

    // JAVA: com.mojang.serialization.Dynamic#asInt
    pub fn as_int(self, default: i32) -> i32 {
        match self.0 {
            Some(Dynamic::Int(v)) => v,
            Some(Dynamic::Byte(v)) => v as i32,
            Some(Dynamic::Long(v)) => v as i32,
            Some(Dynamic::String(s)) => s.parse::<i32>().unwrap_or(default),
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

    // JAVA: com.mojang.serialization.Dynamic#createInt
    pub fn create_int(&self, v: i32) -> Dynamic {
        Dynamic::Int(v)
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

pub struct EntityZombieSplitFix;

impl EntityZombieSplitFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityZombieSplitFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityZombieSplitFix#fix
    pub fn fix(&self, name: String, data: Dynamic) -> (String, Dynamic) {
        if name != "Zombie" {
            return (name, data);
        }

        let zombie_type = data.get("ZombieType").as_int(0);
        let (new_name, mut new_data) = match zombie_type {
            1 | 2 | 3 | 4 | 5 => {
                let converted =
                    self.change_schema_to_zombie_villager(data.clone(), zombie_type.wrapping_sub(1));
                ("ZombieVillager".to_string(), converted)
            }
            6 => ("Husk".to_string(), data),
            _ => ("Zombie".to_string(), data),
        };

        new_data = new_data.remove("ZombieType");
        (new_name, new_data)
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityZombieSplitFix#changeSchemaToZombieVillager
    fn change_schema_to_zombie_villager(&self, data: Dynamic, profession: i32) -> Dynamic {
        let prof_val = data.create_int(profession);
        data.set("Profession", prof_val)
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
    fn non_zombie_unchanged() {
        let fix = EntityZombieSplitFix::new(());
        let data = map(vec![("id".into(), Dynamic::String("Skeleton".into()))]);
        let (name, out) = fix.fix("Skeleton".to_string(), data.clone());
        assert_eq!(name, "Skeleton");
        assert_eq!(out, data);
    }

    #[test]
    fn zombie_default_type_stays_zombie() {
        let fix = EntityZombieSplitFix::new(());
        let data = map(vec![
            ("ZombieType".into(), Dynamic::Int(0)),
            ("Health".into(), Dynamic::Int(20)),
        ]);
        let (name, out) = fix.fix("Zombie".to_string(), data);
        assert_eq!(name, "Zombie");
        assert_eq!(out.get("ZombieType").result(), None);
        assert_eq!(out.get("Health").as_int(0), 20);
    }

    #[test]
    fn zombie_villager_conversion_profession_offset() {
        let fix = EntityZombieSplitFix::new(());
        for zt in 1..=5 {
            let data = map(vec![("ZombieType".into(), Dynamic::Int(zt))]);
            let (name, out) = fix.fix("Zombie".to_string(), data);
            assert_eq!(name, "ZombieVillager");
            assert_eq!(out.get("Profession").as_int(-1), zt - 1);
            assert_eq!(out.get("ZombieType").result(), None);
        }
    }

    #[test]
    fn zombie_type_6_becomes_husk() {
        let fix = EntityZombieSplitFix::new(());
        let data = map(vec![("ZombieType".into(), Dynamic::Int(6))]);
        let (name, out) = fix.fix("Zombie".to_string(), data);
        assert_eq!(name, "Husk");
        assert_eq!(out.get("ZombieType").result(), None);
        assert_eq!(out.get("Profession").result(), None);
    }

    #[test]
    fn missing_or_negative_or_overflow_zombie_type_defaults() {
        let fix = EntityZombieSplitFix::new(());
        let (name, out) = fix.fix("Zombie".to_string(), map(vec![]));
        assert_eq!(name, "Zombie");
        assert_eq!(out.get("ZombieType").result(), None);

        for zt in [i32::MIN, -100, -1, 7, 100, i32::MAX] {
            let data = map(vec![("ZombieType".into(), Dynamic::Int(zt))]);
            let (name, out) = fix.fix("Zombie".to_string(), data);
            assert_eq!(name, "Zombie", "zt={zt}");
            assert_eq!(out.get("ZombieType").result(), None);
        }
    }

    #[test]
    fn preserves_float_bits() {
        let fix = EntityZombieSplitFix::new(());
        let bits = f32::NAN.to_bits();
        let data = map(vec![
            ("ZombieType".into(), Dynamic::Int(6)),
            ("Rot".into(), Dynamic::Float(f32::from_bits(bits))),
        ]);
        let (name, out) = fix.fix("Zombie".to_string(), data);
        assert_eq!(name, "Husk");
        match out.get("Rot").result() {
            Some(Dynamic::Float(f)) => assert_eq!(f.to_bits(), bits),
            other => panic!("expected float, got {:?}", other),
        }
    }
}
