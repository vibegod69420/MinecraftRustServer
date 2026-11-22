/// Mirrors `net.minecraft.nbt.StreamTagVisitor`.
pub trait StreamTagVisitor {
    // JAVA: net.minecraft.nbt.StreamTagVisitor#visitEnd
    fn visit_end(&mut self) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(String)
    fn visit_string(&mut self, value: &str) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(byte)
    fn visit_byte(&mut self, value: i8) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(short)
    fn visit_short(&mut self, value: i16) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(int)
    fn visit_int(&mut self, value: i32) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(long)
    fn visit_long(&mut self, value: i64) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(float)
    fn visit_float(&mut self, value: f32) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(double)
    fn visit_double(&mut self, value: f64) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(byte[])
    fn visit_byte_array(&mut self, value: &[i8]) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(int[])
    fn visit_int_array(&mut self, value: &[i32]) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit(long[])
    fn visit_long_array(&mut self, value: &[i64]) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visitList
    fn visit_list(&mut self, tag_type: u8, size: i32) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visitEntry(TagType)
    fn visit_entry(&mut self, tag_type: u8) -> EntryResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visitEntry(TagType,String)
    fn visit_entry_named(&mut self, tag_type: u8, name: &str) -> EntryResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visitElement
    fn visit_element(&mut self, tag_type: u8, index: i32) -> EntryResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visitContainerEnd
    fn visit_container_end(&mut self) -> ValueResult;

    // JAVA: net.minecraft.nbt.StreamTagVisitor#visitRootEntry
    fn visit_root_entry(&mut self, tag_type: u8) -> ValueResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryResult {
    Enter,
    Skip,
    Break,
    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueResult {
    Continue,
    Break,
    Halt,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyVisitor;

    impl StreamTagVisitor for DummyVisitor {
        fn visit_end(&mut self) -> ValueResult { ValueResult::Continue }
        fn visit_string(&mut self, _value: &str) -> ValueResult { ValueResult::Continue }
        fn visit_byte(&mut self, _value: i8) -> ValueResult { ValueResult::Continue }
        fn visit_short(&mut self, _value: i16) -> ValueResult { ValueResult::Continue }
        fn visit_int(&mut self, _value: i32) -> ValueResult { ValueResult::Continue }
        fn visit_long(&mut self, _value: i64) -> ValueResult { ValueResult::Continue }
        fn visit_float(&mut self, _value: f32) -> ValueResult { ValueResult::Continue }
        fn visit_double(&mut self, _value: f64) -> ValueResult { ValueResult::Continue }
        fn visit_byte_array(&mut self, _value: &[i8]) -> ValueResult { ValueResult::Continue }
        fn visit_int_array(&mut self, _value: &[i32]) -> ValueResult { ValueResult::Continue }
        fn visit_long_array(&mut self, _value: &[i64]) -> ValueResult { ValueResult::Continue }
        fn visit_list(&mut self, _tag_type: u8, _size: i32) -> ValueResult { ValueResult::Continue }
        fn visit_entry(&mut self, _tag_type: u8) -> EntryResult { EntryResult::Enter }
        fn visit_entry_named(&mut self, _tag_type: u8, _name: &str) -> EntryResult { EntryResult::Skip }
        fn visit_element(&mut self, _tag_type: u8, _index: i32) -> EntryResult { EntryResult::Break }
        fn visit_container_end(&mut self) -> ValueResult { ValueResult::Halt }
        fn visit_root_entry(&mut self, _tag_type: u8) -> ValueResult { ValueResult::Continue }
    }

    #[test]
    fn enum_variants_are_distinct() {
        assert_ne!(EntryResult::Enter, EntryResult::Skip);
        assert_ne!(EntryResult::Break, EntryResult::Halt);
        assert_ne!(ValueResult::Continue, ValueResult::Break);
        assert_ne!(ValueResult::Break, ValueResult::Halt);
    }

    #[test]
    fn visitor_dispatch_works() {
        let mut v = DummyVisitor;
        assert_eq!(v.visit_end(), ValueResult::Continue);
        assert_eq!(v.visit_string("abc"), ValueResult::Continue);
        assert_eq!(v.visit_byte(-1), ValueResult::Continue);
        assert_eq!(v.visit_entry(10), EntryResult::Enter);
        assert_eq!(v.visit_entry_named(1, "k"), EntryResult::Skip);
        assert_eq!(v.visit_element(3, -5), EntryResult::Break);
        assert_eq!(v.visit_container_end(), ValueResult::Halt);
    }

    #[test]
    fn array_and_numeric_visits() {
        let mut v = DummyVisitor;
        assert_eq!(v.visit_byte_array(&[1, -2]), ValueResult::Continue);
        assert_eq!(v.visit_int_array(&[i32::MIN, i32::MAX]), ValueResult::Continue);
        assert_eq!(v.visit_long_array(&[0, -1]), ValueResult::Continue);
        assert_eq!(v.visit_float(f32::NAN), ValueResult::Continue);
        assert_eq!(v.visit_double(f64::NEG_INFINITY), ValueResult::Continue);
    }
}
