use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Int(i32),
    String(String),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Null),
            _ => Dynamic::Null,
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            _ => default,
        }
    }

    pub fn map(entries: Vec<(String, Dynamic)>) -> Self {
        Dynamic::Map(entries.into_iter().collect())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleEntityRenameFix {
    pub name: String,
    pub schema: Schema,
    pub changes_type: bool,
}

impl SimpleEntityRenameFix {
    pub fn new(name: &str, schema: Schema, changes_type: bool) -> Self {
        Self {
            name: name.to_string(),
            schema,
            changes_type,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntitySkeletonSplitFix {
    pub base: SimpleEntityRenameFix,
}

impl EntitySkeletonSplitFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntitySkeletonSplitFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            base: SimpleEntityRenameFix::new("EntitySkeletonSplitFix", schema, changes_type),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntitySkeletonSplitFix#getNewNameAndTag
    pub fn get_new_name_and_tag(&self, name: String, tag: Dynamic) -> (String, Dynamic) {
        let mut name = name;
        if name == "Skeleton" {
            let skeleton_type = tag.get("SkeletonType").as_int(0);
            if skeleton_type == 1 {
                name = "WitherSkeleton".to_string();
            } else if skeleton_type == 2 {
                name = "Stray".to_string();
            }
        }
        (name, tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skeleton_tag(skeleton_type: i32) -> Dynamic {
        Dynamic::map(vec![("SkeletonType".to_string(), Dynamic::Int(skeleton_type))])
    }

    #[test]
    fn skeleton_type_1_becomes_wither_skeleton() {
        let fix = EntitySkeletonSplitFix::new(Schema, true);
        let (name, tag) = fix.get_new_name_and_tag("Skeleton".to_string(), skeleton_tag(1));
        assert_eq!(name, "WitherSkeleton");
        assert_eq!(tag, skeleton_tag(1));
    }

    #[test]
    fn skeleton_type_2_becomes_stray() {
        let fix = EntitySkeletonSplitFix::new(Schema, true);
        let (name, _) = fix.get_new_name_and_tag("Skeleton".to_string(), skeleton_tag(2));
        assert_eq!(name, "Stray");
    }

    #[test]
    fn skeleton_type_0_unchanged() {
        let fix = EntitySkeletonSplitFix::new(Schema, false);
        let (name, _) = fix.get_new_name_and_tag("Skeleton".to_string(), skeleton_tag(0));
        assert_eq!(name, "Skeleton");
    }

    #[test]
    fn missing_skeleton_type_defaults_to_zero() {
        let fix = EntitySkeletonSplitFix::new(Schema, true);
        let (name, _) = fix.get_new_name_and_tag("Skeleton".to_string(), Dynamic::map(vec![]));
        assert_eq!(name, "Skeleton");
    }

    #[test]
    fn non_skeleton_entity_unchanged() {
        let fix = EntitySkeletonSplitFix::new(Schema, true);
        let tag = skeleton_tag(1);
        let (name, out_tag) = fix.get_new_name_and_tag("Zombie".to_string(), tag.clone());
        assert_eq!(name, "Zombie");
        assert_eq!(out_tag, tag);
    }

    #[test]
    fn negative_skeleton_type_unchanged() {
        let fix = EntitySkeletonSplitFix::new(Schema, true);
        let (name, _) = fix.get_new_name_and_tag("Skeleton".to_string(), skeleton_tag(-1));
        assert_eq!(name, "Skeleton");
    }
}
