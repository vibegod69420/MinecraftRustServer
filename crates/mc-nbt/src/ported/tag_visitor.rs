use crate::NbtTag;

/// Visitor for NBT tags.
/// Method signatures are adapted to the crate's `NbtTag` representation while preserving
/// the Java visitor entry points.
// JAVA: net.minecraft.nbt.TagVisitor
pub trait TagVisitor {
    // JAVA: net.minecraft.nbt.TagVisitor#visitString
    fn visit_string(&mut self, value: &str);

    // JAVA: net.minecraft.nbt.TagVisitor#visitByte
    fn visit_byte(&mut self, value: i8);

    // JAVA: net.minecraft.nbt.TagVisitor#visitShort
    fn visit_short(&mut self, value: i16);

    // JAVA: net.minecraft.nbt.TagVisitor#visitInt
    fn visit_int(&mut self, value: i32);

    // JAVA: net.minecraft.nbt.TagVisitor#visitLong
    fn visit_long(&mut self, value: i64);

    // JAVA: net.minecraft.nbt.TagVisitor#visitFloat
    fn visit_float(&mut self, value: f32);

    // JAVA: net.minecraft.nbt.TagVisitor#visitDouble
    fn visit_double(&mut self, value: f64);

    // JAVA: net.minecraft.nbt.TagVisitor#visitByteArray
    fn visit_byte_array(&mut self, value: &[i8]);

    // JAVA: net.minecraft.nbt.TagVisitor#visitIntArray
    fn visit_int_array(&mut self, value: &[i32]);

    // JAVA: net.minecraft.nbt.TagVisitor#visitLongArray
    fn visit_long_array(&mut self, value: &[i64]);

    // JAVA: net.minecraft.nbt.TagVisitor#visitList
    fn visit_list(&mut self, tag: &NbtTag);

    // JAVA: net.minecraft.nbt.TagVisitor#visitCompound
    fn visit_compound(&mut self, tag: &NbtTag);

    // JAVA: net.minecraft.nbt.TagVisitor#visitEnd
    fn visit_end(&mut self);
}

// JAVA: net.minecraft.nbt.TagVisitor
pub fn accept_tag<V: TagVisitor + ?Sized>(tag: &NbtTag, visitor: &mut V) {
    match tag {
        NbtTag::Byte(v) => visitor.visit_byte(*v),
        NbtTag::Short(v) => visitor.visit_short(*v),
        NbtTag::Int(v) => visitor.visit_int(*v),
        NbtTag::Long(v) => visitor.visit_long(*v),
        NbtTag::Float(v) => visitor.visit_float(*v),
        NbtTag::Double(v) => visitor.visit_double(*v),
        NbtTag::ByteArray(v) => visitor.visit_byte_array(v),
        NbtTag::String(v) => visitor.visit_string(v),
        NbtTag::List { .. } => visitor.visit_list(tag),
        NbtTag::Compound(_) => visitor.visit_compound(tag),
        NbtTag::IntArray(v) => visitor.visit_int_array(v),
        NbtTag::LongArray(v) => visitor.visit_long_array(v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NbtTag;

    #[derive(Default)]
    struct RecordingVisitor {
        events: Vec<String>,
    }

    impl TagVisitor for RecordingVisitor {
        fn visit_string(&mut self, value: &str) {
            self.events.push(format!("string:{value}"));
        }
        fn visit_byte(&mut self, value: i8) {
            self.events.push(format!("byte:{value}"));
        }
        fn visit_short(&mut self, value: i16) {
            self.events.push(format!("short:{value}"));
        }
        fn visit_int(&mut self, value: i32) {
            self.events.push(format!("int:{value}"));
        }
        fn visit_long(&mut self, value: i64) {
            self.events.push(format!("long:{value}"));
        }
        fn visit_float(&mut self, value: f32) {
            self.events.push(format!("float:{:08x}", value.to_bits()));
        }
        fn visit_double(&mut self, value: f64) {
            self.events.push(format!("double:{:016x}", value.to_bits()));
        }
        fn visit_byte_array(&mut self, value: &[i8]) {
            self.events.push(format!("byte_array:{}", value.len()));
        }
        fn visit_int_array(&mut self, value: &[i32]) {
            self.events.push(format!("int_array:{}", value.len()));
        }
        fn visit_long_array(&mut self, value: &[i64]) {
            self.events.push(format!("long_array:{}", value.len()));
        }
        fn visit_list(&mut self, tag: &NbtTag) {
            if let NbtTag::List { items, .. } = tag {
                self.events.push(format!("list:{}", items.len()));
            }
        }
        fn visit_compound(&mut self, tag: &NbtTag) {
            if let NbtTag::Compound(entries) = tag {
                self.events.push(format!("compound:{}", entries.len()));
            }
        }
        fn visit_end(&mut self) {
            self.events.push("end".to_string());
        }
    }

    #[test]
    fn visit_primitives_and_negative_values() {
        let mut v = RecordingVisitor::default();
        accept_tag(&NbtTag::Byte(-1), &mut v);
        accept_tag(&NbtTag::Short(i16::MIN), &mut v);
        accept_tag(&NbtTag::Int(-123), &mut v);
        accept_tag(&NbtTag::Long(i64::MIN), &mut v);
        assert_eq!(v.events[0], "byte:-1");
        assert_eq!(v.events[1], format!("short:{}", i16::MIN));
        assert_eq!(v.events[2], "int:-123");
        assert_eq!(v.events[3], format!("long:{}", i64::MIN));
    }

    #[test]
    fn visit_float_double_bits_including_nan_and_neg_zero() {
        let mut v = RecordingVisitor::default();
        accept_tag(&NbtTag::Float(-0.0f32), &mut v);
        accept_tag(&NbtTag::Float(f32::NAN), &mut v);
        accept_tag(&NbtTag::Double(-0.0f64), &mut v);
        accept_tag(&NbtTag::Double(f64::NAN), &mut v);
        assert_eq!(v.events[0], format!("float:{:08x}", (-0.0f32).to_bits()));
        assert_eq!(v.events[1], format!("float:{:08x}", f32::NAN.to_bits()));
        assert_eq!(v.events[2], format!("double:{:016x}", (-0.0f64).to_bits()));
        assert_eq!(v.events[3], format!("double:{:016x}", f64::NAN.to_bits()));
    }

    #[test]
    fn visit_arrays_string_list_compound_end() {
        let mut v = RecordingVisitor::default();
        accept_tag(&NbtTag::ByteArray(vec![-1, 0, 1]), &mut v);
        accept_tag(&NbtTag::IntArray(vec![i32::MIN, 0]), &mut v);
        accept_tag(&NbtTag::LongArray(vec![i64::MAX]), &mut v);
        accept_tag(&NbtTag::String("abc".to_string()), &mut v);
        accept_tag(
            &NbtTag::List {
                elem_type: 1,
                items: vec![NbtTag::Byte(1)],
            },
            &mut v,
        );
        accept_tag(&NbtTag::Compound(vec![("k".to_string(), NbtTag::Int(1))]), &mut v);
        v.visit_end();
        assert_eq!(
            v.events,
            vec![
                "byte_array:3",
                "int_array:2",
                "long_array:1",
                "string:abc",
                "list:1",
                "compound:1",
                "end",
            ]
        );
    }
}
