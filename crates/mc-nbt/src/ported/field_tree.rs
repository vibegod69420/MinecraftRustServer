use std::collections::HashMap;

use crate::TagId;

pub type TagType = TagId;

/// Minimal self-contained selector shape used by `FieldTree::add_entry`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSelector {
    pub path: Vec<String>,
    pub tag_type: TagType,
    pub name: String,
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
}

/// Faithful port of `net.minecraft.nbt.visitors.FieldTree`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldTree {
    pub depth: i32,
    pub selected_fields: HashMap<String, TagType>,
    pub fields_to_recurse: HashMap<String, FieldTree>,
}

impl FieldTree {
    fn new_at_depth(depth: i32) -> Self {
        Self {
            depth,
            selected_fields: HashMap::new(),
            fields_to_recurse: HashMap::new(),
        }
    }

    // JAVA: net.minecraft.nbt.visitors.FieldTree#createRoot
    pub fn create_root() -> Self {
        Self::new_at_depth(1)
    }

    // JAVA: net.minecraft.nbt.visitors.FieldTree#addEntry
    pub fn add_entry(&mut self, selector: FieldSelector) {
        let path_len = selector.path.len() as i32;
        if self.depth <= path_len {
            let idx = (self.depth - 1) as usize;
            let key = selector.path[idx].clone();
            let child_depth = self.depth.wrapping_add(1);
            self.fields_to_recurse
                .entry(key)
                .or_insert_with(|| FieldTree::new_at_depth(child_depth))
                .add_entry(selector);
        } else {
            self.selected_fields.insert(selector.name, selector.tag_type);
        }
    }

    // JAVA: net.minecraft.nbt.visitors.FieldTree#isSelected
    pub fn is_selected(&self, tag_type: TagType, name: &str) -> bool {
        self.selected_fields.get(name) == Some(&tag_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TagId;

    #[test]
    fn create_root_depth_is_one() {
        let root = FieldTree::create_root();
        assert_eq!(root.depth, 1);
        assert!(root.selected_fields.is_empty());
        assert!(root.fields_to_recurse.is_empty());
    }

    #[test]
    fn add_entry_selects_at_root() {
        let mut root = FieldTree::create_root();
        root.add_entry(FieldSelector::new(TagId::Int, "x"));
        assert!(root.is_selected(TagId::Int, "x"));
        assert!(!root.is_selected(TagId::Long, "x"));
        assert!(!root.is_selected(TagId::Int, "y"));
    }

    #[test]
    fn add_entry_with_path_creates_recurse_tree() {
        let mut root = FieldTree::create_root();
        root.add_entry(FieldSelector::with_path1("child", TagId::String, "name"));
        assert!(!root.is_selected(TagId::String, "name"));
        let child = root.fields_to_recurse.get("child").expect("child tree");
        assert_eq!(child.depth, 2);
        assert!(child.is_selected(TagId::String, "name"));
    }

    #[test]
    fn add_entry_deep_path() {
        let mut root = FieldTree::create_root();
        root.add_entry(FieldSelector {
            path: vec!["a".into(), "b".into()],
            tag_type: TagId::Byte,
            name: "v".into(),
        });
        let a = root.fields_to_recurse.get("a").unwrap();
        let b = a.fields_to_recurse.get("b").unwrap();
        assert_eq!(b.depth, 3);
        assert!(b.is_selected(TagId::Byte, "v"));
    }

    #[test]
    fn is_selected_requires_exact_type() {
        let mut root = FieldTree::create_root();
        root.add_entry(FieldSelector::new(TagId::Float, "f"));
        assert!(root.is_selected(TagId::Float, "f"));
        assert!(!root.is_selected(TagId::Double, "f"));
    }
}
