use crate::TagId;

pub type TagType = TagId;

/// Faithful port of `net.minecraft.nbt.StreamTagVisitor.ValueResult`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueResult {
    Continue,
    Break,
    Halt,
}

/// Faithful port of `net.minecraft.nbt.StreamTagVisitor.EntryResult`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryResult {
    Enter,
    Skip,
    Break,
    Halt,
}

/// Minimal visitor trait with defaults matching `SkipAll`.
pub trait StreamTagVisitor {
    // JAVA: net.minecraft.nbt.visitors.SkipAll#visitEnd
    fn visit_end(&mut self) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(String)
    fn visit_string(&mut self, _value: &str) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(byte)
    fn visit_byte(&mut self, _value: i8) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(short)
    fn visit_short(&mut self, _value: i16) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(int)
    fn visit_int(&mut self, _value: i32) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(long)
    fn visit_long(&mut self, _value: i64) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(float)
    fn visit_float(&mut self, _value: f32) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(double)
    fn visit_double(&mut self, _value: f64) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(byte[])
    fn visit_byte_array(&mut self, _value: &[i8]) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(int[])
    fn visit_int_array(&mut self, _value: &[i32]) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visit(long[])
    fn visit_long_array(&mut self, _value: &[i64]) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visitList
    fn visit_list(&mut self, _tag_type: TagType, _len: i32) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visitElement
    fn visit_element(&mut self, _tag_type: TagType, _index: i32) -> EntryResult {
        EntryResult::Skip
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visitEntry(TagType)
    fn visit_entry(&mut self, _tag_type: TagType) -> EntryResult {
        EntryResult::Skip
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visitEntry(TagType,String)
    fn visit_entry_named(&mut self, _tag_type: TagType, _name: &str) -> EntryResult {
        EntryResult::Skip
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visitContainerEnd
    fn visit_container_end(&mut self) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.SkipAll#visitRootEntry
    fn visit_root_entry(&mut self, _tag_type: TagType) -> ValueResult {
        ValueResult::Continue
    }
}

/// Faithful port of `net.minecraft.nbt.visitors.SkipAll`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SkipAll;

pub const INSTANCE: SkipAll = SkipAll;

impl StreamTagVisitor for SkipAll {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TagId;

    #[test]
    fn skip_all_value_visits_continue() {
        let mut v = SkipAll;
        assert_eq!(v.visit_end(), ValueResult::Continue);
        assert_eq!(v.visit_string("abc"), ValueResult::Continue);
        assert_eq!(v.visit_byte(i8::MIN), ValueResult::Continue);
        assert_eq!(v.visit_short(-1), ValueResult::Continue);
        assert_eq!(v.visit_int(42), ValueResult::Continue);
        assert_eq!(v.visit_long(i64::MAX), ValueResult::Continue);
    }

    #[test]
    fn skip_all_float_and_arrays_continue() {
        let mut v = SkipAll;
        assert_eq!(v.visit_float(f32::NAN).clone(), ValueResult::Continue);
        assert_eq!(v.visit_double(f64::NEG_INFINITY), ValueResult::Continue);
        assert_eq!(v.visit_byte_array(&[1, -1]), ValueResult::Continue);
        assert_eq!(v.visit_int_array(&[i32::MIN, 0]), ValueResult::Continue);
        assert_eq!(v.visit_long_array(&[i64::MIN]), ValueResult::Continue);
    }

    #[test]
    fn skip_all_entries_skip_and_containers_continue() {
        let mut v = SkipAll;
        assert_eq!(v.visit_list(TagId::Int, 3), ValueResult::Continue);
        assert_eq!(v.visit_element(TagId::Int, 0), EntryResult::Skip);
        assert_eq!(v.visit_entry(TagId::Compound), EntryResult::Skip);
        assert_eq!(v.visit_entry_named(TagId::String, "k"), EntryResult::Skip);
        assert_eq!(v.visit_container_end(), ValueResult::Continue);
        assert_eq!(v.visit_root_entry(TagId::Compound), ValueResult::Continue);
    }

    #[test]
    fn instance_is_usable() {
        let mut v = INSTANCE;
        assert_eq!(v.visit_int(0), ValueResult::Continue);
        assert_eq!(v.visit_entry_named(TagId::Byte, "b"), EntryResult::Skip);
    }
}
