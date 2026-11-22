use crate::TagId;

pub type TagType = TagId;

/// Faithful port of `net.minecraft.nbt.visitors.FieldSelector`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSelector {
    path: Vec<String>,
    tag_type: TagType,
    name: String,
}

impl FieldSelector {
    // JAVA: net.minecraft.nbt.visitors.FieldSelector#<init>(TagType,String)
    pub fn new(tag_type: TagType, name: impl Into<String>) -> Self {
        Self {
            path: Vec::new(),
            tag_type,
            name: name.into(),
        }
    }

    // JAVA: net.minecraft.nbt.visitors.FieldSelector#<init>(String,TagType,String)
    pub fn with_path1(p0: impl Into<String>, tag_type: TagType, name: impl Into<String>) -> Self {
        Self {
            path: vec![p0.into()],
            tag_type,
            name: name.into(),
        }
    }

    // JAVA: net.minecraft.nbt.visitors.FieldSelector#<init>(String,String,TagType,String)
    pub fn with_path2(
        p0: impl Into<String>,
        p1: impl Into<String>,
        tag_type: TagType,
        name: impl Into<String>,
    ) -> Self {
        Self {
            path: vec![p0.into(), p1.into()],
            tag_type,
            name: name.into(),
        }
    }

    // JAVA: net.minecraft.nbt.visitors.FieldSelector#path
    pub fn path(&self) -> &[String] {
        &self.path
    }

    // JAVA: net.minecraft.nbt.visitors.FieldSelector#type
    pub fn tag_type(&self) -> TagType {
        self.tag_type
    }

    // JAVA: net.minecraft.nbt.visitors.FieldSelector#name
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TagId;

    #[test]
    fn new_has_empty_path() {
        let sel = FieldSelector::new(TagId::Int, "value");
        assert!(sel.path().is_empty());
        assert_eq!(sel.tag_type(), TagId::Int);
        assert_eq!(sel.name(), "value");
    }

    #[test]
    fn with_path1_sets_single_path() {
        let sel = FieldSelector::with_path1("root", TagId::String, "name");
        assert_eq!(sel.path(), &["root".to_string()]);
        assert_eq!(sel.tag_type(), TagId::String);
        assert_eq!(sel.name(), "name");
    }

    #[test]
    fn with_path2_sets_two_path_elements() {
        let sel = FieldSelector::with_path2("a", "b", TagId::Compound, "c");
        assert_eq!(sel.path(), &["a".to_string(), "b".to_string()]);
        assert_eq!(sel.tag_type(), TagId::Compound);
        assert_eq!(sel.name(), "c");
    }

    #[test]
    fn equality_and_clone() {
        let a = FieldSelector::new(TagId::Byte, "x");
        let b = a.clone();
        assert_eq!(a, b);
        let c = FieldSelector::new(TagId::Short, "x");
        assert_ne!(a, c);
    }
}
