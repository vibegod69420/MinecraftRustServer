use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Int(i32),
    Long(i64),
    Short(i16),
    Byte(i8),
    Float(f32),
    Double(f64),
    String(String),
    Bool(bool),
    Map(BTreeMap<String, DynValue>),
    List(Vec<DynValue>),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynValue,
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#new
    pub fn new(value: DynValue) -> Self {
        Self { value }
    }

    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get(&self, key: &str) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => Dynamic::new(m.get(key).cloned().unwrap_or(DynValue::Null)),
            _ => Dynamic::new(DynValue::Null),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#update(String, Function)
    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(&self, key: &str, f: F) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => match m.get(key) {
                Some(old) => {
                    let mut new_map = m.clone();
                    new_map.insert(key.to_string(), f(Dynamic::new(old.clone())).value);
                    Dynamic::new(DynValue::Map(new_map))
                }
                None => self.clone(),
            },
            _ => self.clone(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asStream
    pub fn as_stream(&self) -> Vec<Dynamic> {
        match &self.value {
            DynValue::List(list) => list.iter().cloned().map(Dynamic::new).collect(),
            _ => Vec::new(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createList
    pub fn create_list(values: Vec<Dynamic>) -> Dynamic {
        Dynamic::new(DynValue::List(
            values.into_iter().map(|d| d.value).collect(),
        ))
    }

    // JAVA: com.mojang.serialization.Dynamic#createIntList
    pub fn create_int_list(&self, values: Vec<i32>) -> Dynamic {
        Dynamic::new(DynValue::List(
            values.into_iter().map(DynValue::Int).collect(),
        ))
    }

    // JAVA: com.mojang.serialization.Dynamic#asNumber (int extraction helper)
    pub fn as_number_int(&self) -> Option<i32> {
        match &self.value {
            DynValue::Int(v) => Some(*v),
            DynValue::Long(v) => Some(*v as i32),
            DynValue::Short(v) => Some(i32::from(*v)),
            DynValue::Byte(v) => Some(i32::from(*v)),
            DynValue::Float(v) => Some(*v as i32),
            DynValue::Double(v) => Some(*v as i32),
            _ => None,
        }
    }
}

// JAVA: net.minecraft.util.datafix.ExtraDataFixUtils#fixBlockPos
pub fn fix_block_pos(dynamic: Dynamic) -> Dynamic {
    let x = dynamic.get("X").as_number_int();
    let y = dynamic.get("Y").as_number_int();
    let z = dynamic.get("Z").as_number_int();
    if let (Some(x), Some(y), Some(z)) = (x, y, z) {
        return dynamic.create_int_list(vec![x, y, z]);
    }
    dynamic
}

pub struct MapBannerBlockPosFormatFix;

impl MapBannerBlockPosFormatFix {
    // JAVA: net.minecraft.util.datafix.fixes.MapBannerBlockPosFormatFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.MapBannerBlockPosFormatFix#makeRule
    pub fn make_rule(&self) -> Box<dyn Fn(Dynamic) -> Dynamic> {
        Box::new(fix_map_banner_block_pos)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.MapBannerBlockPosFormatFix#makeRule (data/banners/Pos transform)
fn fix_map_banner_block_pos(data_root: Dynamic) -> Dynamic {
    data_root.update("data", |data| {
        data.update("banners", |banners| {
            let mapped: Vec<Dynamic> = banners
                .as_stream()
                .into_iter()
                .map(|banner| banner.update("Pos", fix_block_pos))
                .collect();
            if matches!(banners.value, DynValue::List(_)) {
                Dynamic::create_list(mapped)
            } else {
                banners
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn banner_with_xyz(x: i32, y: i32, z: i32) -> DynValue {
        let mut pos = BTreeMap::new();
        pos.insert("X".to_string(), DynValue::Int(x));
        pos.insert("Y".to_string(), DynValue::Int(y));
        pos.insert("Z".to_string(), DynValue::Int(z));
        let mut banner = BTreeMap::new();
        banner.insert("Pos".to_string(), DynValue::Map(pos));
        banner.insert("Color".to_string(), DynValue::String("red".to_string()));
        DynValue::Map(banner)
    }

    fn map_data_with_banners(banners: Vec<DynValue>) -> Dynamic {
        let mut data = BTreeMap::new();
        data.insert("banners".to_string(), DynValue::List(banners));
        let mut root = BTreeMap::new();
        root.insert("data".to_string(), DynValue::Map(data));
        Dynamic::new(DynValue::Map(root))
    }

    #[test]
    fn fixes_banner_pos_format() {
        let fix = MapBannerBlockPosFormatFix::new(());
        let input = map_data_with_banners(vec![banner_with_xyz(10, 64, -5)]);
        let out = fix.make_rule()(input);
        let banners = out.get("data").get("banners").as_stream();
        assert_eq!(
            banners[0].get("Pos").value,
            DynValue::List(vec![DynValue::Int(10), DynValue::Int(64), DynValue::Int(-5)])
        );
        assert_eq!(
            banners[0].get("Color").value,
            DynValue::String("red".to_string())
        );
    }

    #[test]
    fn handles_multiple_banners_and_negative_coords() {
        let fix = MapBannerBlockPosFormatFix::new(());
        let input = map_data_with_banners(vec![
            banner_with_xyz(i32::MIN, -1, i32::MAX),
            banner_with_xyz(0, 0, 0),
        ]);
        let out = fix.make_rule()(input);
        let banners = out.get("data").get("banners").as_stream();
        assert_eq!(
            banners[0].get("Pos").value,
            DynValue::List(vec![DynValue::Int(i32::MIN), DynValue::Int(-1), DynValue::Int(i32::MAX)])
        );
        assert_eq!(
            banners[1].get("Pos").value,
            DynValue::List(vec![DynValue::Int(0), DynValue::Int(0), DynValue::Int(0)])
        );
    }

    #[test]
    fn missing_or_invalid_pos_unchanged() {
        let fix = MapBannerBlockPosFormatFix::new(());
        let mut banner = BTreeMap::new();
        banner.insert("Pos".to_string(), DynValue::String("bad".to_string()));
        let input = map_data_with_banners(vec![DynValue::Map(banner.clone())]);
        let out = fix.make_rule()(input);
        let banners = out.get("data").get("banners").as_stream();
        assert_eq!(banners[0].get("Pos").value, DynValue::String("bad".to_string()));

        let mut banner2 = BTreeMap::new();
        banner2.insert("Color".to_string(), DynValue::String("blue".to_string()));
        let input2 = map_data_with_banners(vec![DynValue::Map(banner2.clone())]);
        let out2 = fix.make_rule()(input2);
        let banners2 = out2.get("data").get("banners").as_stream();
        assert_eq!(banners2[0].value, DynValue::Map(banner2));
    }

    #[test]
    fn missing_data_or_banners_returns_unchanged() {
        let fix = MapBannerBlockPosFormatFix::new(());
        let original = Dynamic::new(DynValue::Map(BTreeMap::new()));
        let out = fix.make_rule()(original.clone());
        assert_eq!(out, original);

        let mut data = BTreeMap::new();
        data.insert("scale".to_string(), DynValue::Int(1));
        let mut root = BTreeMap::new();
        root.insert("data".to_string(), DynValue::Map(data));
        let original2 = Dynamic::new(DynValue::Map(root));
        let out2 = fix.make_rule()(original2.clone());
        assert_eq!(out2, original2);
    }

    #[test]
    fn fix_block_pos_preserves_non_map_xyz() {
        let pos = {
            let mut m = BTreeMap::new();
            m.insert("X".to_string(), DynValue::Double(1.5));
            m.insert("Y".to_string(), DynValue::Long(64));
            m.insert("Z".to_string(), DynValue::Float(-2.0));
            Dynamic::new(DynValue::Map(m))
        };
        let fixed = fix_block_pos(pos);
        assert_eq!(
            fixed.value,
            DynValue::List(vec![DynValue::Int(1), DynValue::Int(64), DynValue::Int(-2)])
        );
    }
}
