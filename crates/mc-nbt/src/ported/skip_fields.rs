use std::collections::{HashMap, VecDeque};

use crate::TagId;

pub type TagType = TagId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueResult {
    Continue,
    Break,
    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryResult {
    Enter,
    Skip,
    Break,
    Halt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSelector {
    pub path: Vec<String>,
    pub tag_type: TagType,
    pub name: String,
}

impl FieldSelector {
    // JAVA: net.minecraft.nbt.visitors.FieldSelector#<init>(TagType,String)
    pub fn new(tag_type: TagType, name: impl Into<String>) -> Self {
        Self { path: Vec::new(), tag_type, name: name.into() }
    }

    // JAVA: net.minecraft.nbt.visitors.FieldSelector#<init>(String,TagType,String)
    pub fn with_path1(p0: impl Into<String>, tag_type: TagType, name: impl Into<String>) -> Self {
        Self { path: vec![p0.into()], tag_type, name: name.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldTree {
    pub depth: i32,
    pub selected_fields: HashMap<String, TagType>,
    pub fields_to_recurse: HashMap<String, FieldTree>,
}

impl FieldTree {
    // JAVA: net.minecraft.nbt.visitors.FieldTree#createRoot
    pub fn create_root() -> Self {
        Self { depth: 1, selected_fields: HashMap::new(), fields_to_recurse: HashMap::new() }
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
                .or_insert_with(|| FieldTree {
                    depth: child_depth,
                    selected_fields: HashMap::new(),
                    fields_to_recurse: HashMap::new(),
                })
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

/// Minimal self-contained base stub for `CollectToTag`.
#[derive(Debug, Default)]
pub struct CollectToTag {
    depth: i32,
}

impl CollectToTag {
    pub fn new() -> Self {
        Self { depth: 0 }
    }

    pub fn depth(&self) -> i32 {
        self.depth
    }

    pub fn visit_root_entry(&mut self, _tag_type: TagType) -> ValueResult {
        self.depth = self.depth.wrapping_add(1);
        ValueResult::Continue
    }

    pub fn visit_entry_named(&mut self, _tag_type: TagType, _name: &str) -> EntryResult {
        EntryResult::Enter
    }

    pub fn visit_container_end(&mut self) -> ValueResult {
        self.depth = self.depth.wrapping_sub(1);
        ValueResult::Continue
    }
}

/// Faithful port of `net.minecraft.nbt.visitors.SkipFields`.
#[derive(Debug)]
pub struct SkipFields {
    base: CollectToTag,
    stack: VecDeque<FieldTree>,
}

impl SkipFields {
    // JAVA: net.minecraft.nbt.visitors.SkipFields#<init>
    pub fn new(selectors: Vec<FieldSelector>) -> Self {
        let mut root = FieldTree::create_root();
        for sel in selectors {
            root.add_entry(sel);
        }
        let mut stack = VecDeque::new();
        stack.push_front(root);
        Self { base: CollectToTag::new(), stack }
    }

    pub fn depth(&self) -> i32 {
        self.base.depth()
    }

    pub fn base_mut(&mut self) -> &mut CollectToTag {
        &mut self.base
    }

    // JAVA: net.minecraft.nbt.visitors.SkipFields#visitEntry
    pub fn visit_entry(&mut self, tag_type: TagType, name: &str) -> EntryResult {
        let (is_selected, child) = {
            let tree = self.stack.front().expect("stack should not be empty");
            let is_selected = tree.is_selected(tag_type, name);
            let child = if !is_selected && tag_type == TagId::Compound {
                tree.fields_to_recurse.get(name).cloned()
            } else {
                None
            };
            (is_selected, child)
        };
        if is_selected {
            return EntryResult::Skip;
        }
        if let Some(child) = child {
            self.stack.push_front(child);
        }
        self.base.visit_entry_named(tag_type, name)
    }

    // JAVA: net.minecraft.nbt.visitors.SkipFields#visitContainerEnd
    pub fn visit_container_end(&mut self) -> ValueResult {
        if self.depth() == self.stack.front().expect("stack should not be empty").depth {
            self.stack.pop_front();
        }
        self.base.visit_container_end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TagId;

    #[test]
    fn skips_selected_field() {
        let mut sf = SkipFields::new(vec![FieldSelector::new(TagId::Int, "skipMe")]);
        sf.base_mut().visit_root_entry(TagId::Compound);
        assert_eq!(sf.visit_entry(TagId::Int, "skipMe"), EntryResult::Skip);
        assert_eq!(sf.visit_entry(TagId::Int, "keepMe"), EntryResult::Enter);
    }

    #[test]
    fn pushes_recurse_tree_for_compound_path() {
        let mut sf = SkipFields::new(vec![FieldSelector::with_path1("nested", TagId::String, "name")]);
        sf.base_mut().visit_root_entry(TagId::Compound);
        assert_eq!(sf.visit_entry(TagId::Compound, "nested"), EntryResult::Enter);
        assert_eq!(sf.visit_entry(TagId::String, "name"), EntryResult::Skip);
    }

    #[test]
    fn does_not_push_for_non_matching_compound() {
        let mut sf = SkipFields::new(vec![FieldSelector::with_path1("nested", TagId::String, "name")]);
        sf.base_mut().visit_root_entry(TagId::Compound);
        assert_eq!(sf.visit_entry(TagId::Compound, "other"), EntryResult::Enter);
        // "name" at root level is not selected by this selector
        assert_eq!(sf.visit_entry(TagId::String, "name"), EntryResult::Enter);
    }

    #[test]
    fn pops_stack_on_container_end_at_matching_depth() {
        let mut sf = SkipFields::new(vec![FieldSelector::with_path1("nested", TagId::Int, "x")]);
        sf.base_mut().visit_root_entry(TagId::Compound); // depth becomes 1
        assert_eq!(sf.visit_entry(TagId::Compound, "nested"), EntryResult::Enter);
        // simulate entering nested compound in base
        sf.base_mut().visit_root_entry(TagId::Compound); // depth becomes 2
        assert_eq!(sf.depth(), 2);
        assert_eq!(sf.visit_container_end(), ValueResult::Continue);
        assert_eq!(sf.depth(), 1);
        // stack should have popped back to root
        assert_eq!(sf.visit_entry(TagId::Int, "x"), EntryResult::Enter);
    }
}
