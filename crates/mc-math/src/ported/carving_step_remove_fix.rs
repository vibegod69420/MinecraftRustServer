use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Missing,
    Long(i64),
    Bytes(Vec<i8>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Missing),
            _ => Dynamic::Missing,
        }
    }

    pub fn result(&self) -> Option<Dynamic> {
        match self {
            Dynamic::Missing => None,
            _ => Some(self.clone()),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                out.insert(key.to_string(), value);
                Dynamic::Map(out)
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                out.remove(key);
                Dynamic::Map(out)
            }
            _ => self.clone(),
        }
    }
}

pub struct CarvingStepRemoveFix;

impl CarvingStepRemoveFix {
    // JAVA: net.minecraft.util.datafix.fixes.CarvingStepRemoveFix#CarvingStepRemoveFix
    pub fn new(_output_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.CarvingStepRemoveFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic {
        Self::fix_chunk
    }

    // JAVA: net.minecraft.util.datafix.fixes.CarvingStepRemoveFix#fixChunk
    pub fn fix_chunk(chunk: Dynamic) -> Dynamic {
        let mut out = chunk;
        if let Some(carving_masks) = out.get("CarvingMasks").result() {
            if let Some(air_mask) = carving_masks.get("AIR").result() {
                out = out.set("carving_mask", air_mask);
            }
        }
        out.remove("CarvingMasks")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: &[(&str, Dynamic)]) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        Dynamic::Map(m)
    }

    #[test]
    fn moves_air_mask_and_removes_carving_masks() {
        let air = Dynamic::Bytes(vec![1_i8, 2, -1]);
        let masks = map(&[
            ("AIR", air.clone()),
            ("LIQUID", Dynamic::Bytes(vec![4_i8])),
        ]);
        let input = map(&[("CarvingMasks", masks), ("xPos", Dynamic::Long(7))]);
        let out = CarvingStepRemoveFix::fix_chunk(input);
        assert_eq!(out.get("carving_mask"), air);
        assert_eq!(out.get("CarvingMasks").result(), None);
        assert_eq!(out.get("xPos"), Dynamic::Long(7));
    }

    #[test]
    fn removes_carving_masks_without_air() {
        let masks = map(&[("LIQUID", Dynamic::Bytes(vec![4_i8]))]);
        let input = map(&[("CarvingMasks", masks)]);
        let out = CarvingStepRemoveFix::fix_chunk(input);
        assert_eq!(out.get("carving_mask").result(), None);
        assert_eq!(out.get("CarvingMasks").result(), None);
    }

    #[test]
    fn no_carving_masks_keeps_other_fields() {
        let input = map(&[("xPos", Dynamic::Long(-5))]);
        let out = CarvingStepRemoveFix::fix_chunk(input.clone());
        assert_eq!(out.get("xPos"), Dynamic::Long(-5));
        assert_eq!(out.get("CarvingMasks").result(), None);
    }

    #[test]
    fn make_rule_applies_fix_chunk() {
        let fix = CarvingStepRemoveFix::new(());
        let air = Dynamic::Bytes(vec![0_i8]);
        let input = map(&[("CarvingMasks", map(&[("AIR", air.clone())]))]);
        let out = fix.make_rule()(input);
        assert_eq!(out.get("carving_mask"), air);
    }
}
