use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagType(pub u8);

pub const END_TAG_TYPE: TagType = TagType(0);
pub const BYTE_TAG_TYPE: TagType = TagType(1);
pub const SHORT_TAG_TYPE: TagType = TagType(2);
pub const INT_TAG_TYPE: TagType = TagType(3);
pub const LONG_TAG_TYPE: TagType = TagType(4);
pub const FLOAT_TAG_TYPE: TagType = TagType(5);
pub const DOUBLE_TAG_TYPE: TagType = TagType(6);
pub const BYTE_ARRAY_TAG_TYPE: TagType = TagType(7);
pub const STRING_TAG_TYPE: TagType = TagType(8);
pub const LIST_TAG_TYPE: TagType = TagType(9);
pub const COMPOUND_TAG_TYPE: TagType = TagType(10);
pub const INT_ARRAY_TAG_TYPE: TagType = TagType(11);
pub const LONG_ARRAY_TAG_TYPE: TagType = TagType(12);

pub struct CompoundTag;

impl CompoundTag {
    pub const TYPE: TagType = COMPOUND_TAG_TYPE;
}

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

fn is_container_type(tag_type: TagType) -> bool {
    tag_type == LIST_TAG_TYPE || tag_type == COMPOUND_TAG_TYPE
}

/// Minimal base visitor providing depth tracking equivalent to `CollectToTag`
/// for the behavior exercised by `CollectFields`.
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

    pub fn visit_root_entry(&mut self, tag_type: TagType) -> ValueResult {
        if is_container_type(tag_type) {
            self.depth = self.depth.wrapping_add(1);
        }
        ValueResult::Continue
    }

    pub fn visit_entry(&mut self, tag_type: TagType) -> EntryResult {
        if is_container_type(tag_type) {
            self.depth = self.depth.wrapping_add(1);
        }
        EntryResult::Enter
    }

    pub fn visit_entry_named(&mut self, tag_type: TagType, _name: &str) -> EntryResult {
        if is_container_type(tag_type) {
            self.depth = self.depth.wrapping_add(1);
        }
        EntryResult::Enter
    }

    pub fn visit_container_end(&mut self) -> ValueResult {
        self.depth = self.depth.wrapping_sub(1);
        ValueResult::Continue
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSelector {
    name: String,
    tag_type: TagType,
    path: Vec<String>,
}

impl FieldSelector {
    pub fn new(name: impl Into<String>, tag_type: TagType) -> Self {
        Self {
            name: name.into(),
            tag_type,
            path: Vec::new(),
        }
    }

    pub fn with_path(name: impl Into<String>, tag_type: TagType, path: Vec<String>) -> Self {
        Self {
            name: name.into(),
            tag_type,
            path,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tag_type(&self) -> TagType {
        self.tag_type
    }

    pub fn path(&self) -> &[String] {
        &self.path
    }
}

#[derive(Debug)]
pub struct FieldTree {
    depth: i32,
    selected_fields: HashMap<String, TagType>,
    fields_to_recurse: HashMap<String, Rc<RefCell<FieldTree>>>,
}

impl FieldTree {
    pub fn create_root() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            depth: 1,
            selected_fields: HashMap::new(),
            fields_to_recurse: HashMap::new(),
        }))
    }

    pub fn depth(&self) -> i32 {
        self.depth
    }

    pub fn add_entry(&mut self, selector: &FieldSelector) {
        let path = selector.path();
        let depth_usize = if self.depth < 0 { 0usize } else { self.depth as usize };
        if depth_usize <= path.len() {
            let key_index = (self.depth.wrapping_sub(1)) as usize;
            let key = path[key_index].clone();
            let child = self
                .fields_to_recurse
                .entry(key)
                .or_insert_with(|| {
                    Rc::new(RefCell::new(FieldTree {
                        depth: self.depth.wrapping_add(1),
                        selected_fields: HashMap::new(),
                        fields_to_recurse: HashMap::new(),
                    }))
                })
                .clone();
            child.borrow_mut().add_entry(selector);
        } else {
            self.selected_fields
                .insert(selector.name().to_string(), selector.tag_type());
        }
    }

    pub fn remove_selected(&mut self, name: &str, tag_type: TagType) -> bool {
        match self.selected_fields.get(name) {
            Some(&existing) if existing == tag_type => {
                self.selected_fields.remove(name);
                true
            }
            _ => false,
        }
    }

    pub fn fields_to_recurse(&self) -> &HashMap<String, Rc<RefCell<FieldTree>>> {
        &self.fields_to_recurse
    }
}

pub struct CollectFields {
    base: CollectToTag,
    fields_to_get_count: i32,
    wanted_types: HashSet<TagType>,
    stack: Vec<Rc<RefCell<FieldTree>>>,
}

impl CollectFields {
    // JAVA: net.minecraft.nbt.visitors.CollectFields#<init>
    pub fn new(selectors: &[FieldSelector]) -> Self {
        let fields_to_get_count: i32 = selectors.len() as i32;
        let mut wanted_types: HashSet<TagType> = HashSet::new();
        let root = FieldTree::create_root();

        for selector in selectors {
            root.borrow_mut().add_entry(selector);
            wanted_types.insert(selector.tag_type());
        }

        wanted_types.insert(CompoundTag::TYPE);

        Self {
            base: CollectToTag::new(),
            fields_to_get_count,
            wanted_types,
            stack: vec![root],
        }
    }

    // JAVA: net.minecraft.nbt.visitors.CollectFields#visitRootEntry
    pub fn visit_root_entry(&mut self, tag_type: TagType) -> ValueResult {
        if tag_type != CompoundTag::TYPE {
            return ValueResult::Halt;
        }
        self.base.visit_root_entry(tag_type)
    }

    // JAVA: net.minecraft.nbt.visitors.CollectFields#visitEntry
    pub fn visit_entry(&mut self, tag_type: TagType) -> EntryResult {
        let tree_depth = self
            .stack
            .last()
            .expect("stack not empty")
            .borrow()
            .depth();
        if self.base.depth() > tree_depth {
            return self.base.visit_entry(tag_type);
        }
        if self.fields_to_get_count <= 0 {
            return EntryResult::Break;
        }
        if !self.wanted_types.contains(&tag_type) {
            return EntryResult::Skip;
        }
        self.base.visit_entry(tag_type)
    }

    // JAVA: net.minecraft.nbt.visitors.CollectFields#visitEntry
    pub fn visit_entry_named(&mut self, tag_type: TagType, name: &str) -> EntryResult {
        let tree = self.stack.last().expect("stack not empty").clone();
        let tree_depth = tree.borrow().depth();
        if self.base.depth() > tree_depth {
            return self.base.visit_entry_named(tag_type, name);
        }

        let removed = tree.borrow_mut().remove_selected(name, tag_type);
        if removed {
            self.fields_to_get_count = self.fields_to_get_count.wrapping_sub(1);
            return self.base.visit_entry_named(tag_type, name);
        }

        if tag_type == CompoundTag::TYPE {
            let recurse = tree.borrow().fields_to_recurse().get(name).cloned();
            if let Some(child) = recurse {
                self.stack.push(child);
                return self.base.visit_entry_named(tag_type, name);
            }
        }

        EntryResult::Skip
    }

    // JAVA: net.minecraft.nbt.visitors.CollectFields#visitContainerEnd
    pub fn visit_container_end(&mut self) -> ValueResult {
        let tree_depth = self
            .stack
            .last()
            .expect("stack not empty")
            .borrow()
            .depth();
        if self.base.depth() == tree_depth {
            self.stack.pop();
        }
        self.base.visit_container_end()
    }

    // JAVA: net.minecraft.nbt.visitors.CollectFields#getMissingFieldCount
    pub fn get_missing_field_count(&self) -> i32 {
        self.fields_to_get_count
    }

    pub fn depth(&self) -> i32 {
        self.base.depth()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_wanted_fields_and_skips_unwanted() {
        let mut cf = CollectFields::new(&[FieldSelector::new("name", STRING_TAG_TYPE)]);
        assert_eq!(
            cf.visit_root_entry(CompoundTag::TYPE),
            ValueResult::Continue
        );
        assert_eq!(cf.get_missing_field_count(), 1);
        assert_eq!(
            cf.visit_entry_named(INT_TAG_TYPE, "id"),
            EntryResult::Skip
        );
        assert_eq!(
            cf.visit_entry_named(STRING_TAG_TYPE, "name"),
            EntryResult::Enter
        );
        assert_eq!(cf.get_missing_field_count(), 0);
    }

    #[test]
    fn breaks_when_no_fields_remaining_and_halts_non_compound_root() {
        let mut cf = CollectFields::new(&[FieldSelector::new("x", INT_TAG_TYPE)]);
        assert_eq!(cf.visit_root_entry(LIST_TAG_TYPE), ValueResult::Halt);
        assert_eq!(cf.get_missing_field_count(), 1);

        let mut cf = CollectFields::new(&[FieldSelector::new("x", INT_TAG_TYPE)]);
        assert_eq!(
            cf.visit_root_entry(CompoundTag::TYPE),
            ValueResult::Continue
        );
        assert_eq!(
            cf.visit_entry_named(INT_TAG_TYPE, "x"),
            EntryResult::Enter
        );
        assert_eq!(cf.get_missing_field_count(), 0);
        assert_eq!(cf.visit_entry(INT_TAG_TYPE), EntryResult::Break);
    }

    #[test]
    fn recurses_into_nested_compound_and_pops_at_matching_depth() {
        let sel = FieldSelector::with_path("id", STRING_TAG_TYPE, vec!["tag".to_string()]);
        let mut cf = CollectFields::new(&[sel]);
        assert_eq!(
            cf.visit_root_entry(CompoundTag::TYPE),
            ValueResult::Continue
        );
        assert_eq!(cf.depth(), 1);

        assert_eq!(
            cf.visit_entry_named(CompoundTag::TYPE, "tag"),
            EntryResult::Enter
        );
        assert_eq!(cf.depth(), 2);
        assert_eq!(
            cf.visit_entry_named(STRING_TAG_TYPE, "id"),
            EntryResult::Enter
        );
        assert_eq!(cf.get_missing_field_count(), 0);

        assert_eq!(cf.visit_container_end(), ValueResult::Continue);
        assert_eq!(cf.depth(), 1);
        assert_eq!(cf.visit_container_end(), ValueResult::Continue);
        assert_eq!(cf.depth(), 0);
    }

    #[test]
    fn handles_negative_count_and_wrapping_integer_semantics() {
        let mut cf = CollectFields::new(&[FieldSelector::new("a", INT_TAG_TYPE)]);
        cf.fields_to_get_count = -1;
        assert_eq!(cf.get_missing_field_count(), -1);
        assert_eq!(
            cf.visit_root_entry(CompoundTag::TYPE),
            ValueResult::Continue
        );
        assert_eq!(cf.visit_entry(INT_TAG_TYPE), EntryResult::Break);

        let prev = i32::MIN;
        let next = prev.wrapping_sub(1);
        assert_eq!(next, i32::MAX);

        let mut cf2 = CollectFields::new(&[FieldSelector::new("a", INT_TAG_TYPE)]);
        cf2.fields_to_get_count = i32::MIN;
        assert_eq!(cf2.visit_entry(INT_TAG_TYPE), EntryResult::Break);
    }

    #[test]
    fn distinguishes_float_and_double_types_and_preserves_float_bits() {
        let nan_bits: u32 = f32::NAN.to_bits();
        assert_eq!(f32::from_bits(nan_bits).to_bits(), nan_bits);
        let neg_zero_bits: u64 = (-0.0f64).to_bits();
        assert_eq!(f64::from_bits(neg_zero_bits).to_bits(), neg_zero_bits);
        assert_ne!(neg_zero_bits, 0.0f64.to_bits());

        let mut cf = CollectFields::new(&[
            FieldSelector::new("f", FLOAT_TAG_TYPE),
            FieldSelector::new("d", DOUBLE_TAG_TYPE),
        ]);
        assert_eq!(
            cf.visit_root_entry(CompoundTag::TYPE),
            ValueResult::Continue
        );
        assert_eq!(
            cf.visit_entry_named(DOUBLE_TAG_TYPE, "f"),
            EntryResult::Skip
        );
        assert_eq!(
            cf.visit_entry_named(FLOAT_TAG_TYPE, "f"),
            EntryResult::Enter
        );
        assert_eq!(
            cf.visit_entry_named(DOUBLE_TAG_TYPE, "d"),
            EntryResult::Enter
        );
        assert_eq!(cf.get_missing_field_count(), 0);
    }

    #[test]
    fn skips_unwanted_compound_and_enters_wanted_compound_type() {
        let mut cf = CollectFields::new(&[FieldSelector::new("payload", COMPOUND_TAG_TYPE)]);
        assert_eq!(
            cf.visit_root_entry(CompoundTag::TYPE),
            ValueResult::Continue
        );
        // "other" compound is not selected and not in recurse tree: skip
        assert_eq!(
            cf.visit_entry_named(CompoundTag::TYPE, "other"),
            EntryResult::Skip
        );
        // selected compound field should be entered
        assert_eq!(
            cf.visit_entry_named(CompoundTag::TYPE, "payload"),
            EntryResult::Enter
        );
        assert_eq!(cf.get_missing_field_count(), 0);
    }
}
