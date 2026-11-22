use std::io::{self, Read, Write};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueResult {
    Continue,
    Break,
}

pub trait TagVisitor {
    fn visit_end(&mut self, tag: &EndTag);
}

pub trait StreamTagVisitor {
    fn visit_end(&mut self) -> ValueResult;
}

#[derive(Debug, Default)]
pub struct StringTagVisitor {
    out: String,
}

impl StringTagVisitor {
    pub fn new() -> Self {
        Self { out: String::new() }
    }

    pub fn build(self) -> String {
        self.out
    }
}

impl TagVisitor for StringTagVisitor {
    // JAVA: net.minecraft.nbt.StringTagVisitor#visitEnd
    fn visit_end(&mut self, _tag: &EndTag) {
        self.out.push_str("END");
    }
}

impl StreamTagVisitor for StringTagVisitor {
    // JAVA: net.minecraft.nbt.StringTagVisitor#visitEnd (stream)
    fn visit_end(&mut self) -> ValueResult {
        self.out.push_str("END");
        ValueResult::Continue
    }
}

pub struct NbtAccounter {
    quota: u64,
}

impl NbtAccounter {
    pub fn new(quota: u64) -> Self {
        Self { quota }
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#accountBytes
    pub fn account_bytes(&mut self, bytes: u64) -> io::Result<()> {
        if self.quota < bytes {
            self.quota = 0;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "NbtAccounter overflow",
            ));
        }
        self.quota -= bytes;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndTag;

impl EndTag {
    pub const INSTANCE: EndTag = EndTag;
    const SELF_SIZE_IN_BYTES: i32 = 8;

    // JAVA: net.minecraft.nbt.EndTag#write
    pub fn write<W: Write>(&self, _out: &mut W) -> io::Result<()> {
        Ok(())
    }

    // JAVA: net.minecraft.nbt.EndTag#sizeInBytes
    pub fn size_in_bytes(&self) -> i32 {
        Self::SELF_SIZE_IN_BYTES
    }

    // JAVA: net.minecraft.nbt.EndTag#getId
    pub fn get_id(&self) -> u8 {
        0
    }

    // JAVA: net.minecraft.nbt.EndTag#getType
    pub fn get_type(&self) -> &'static EndTagType {
        end_tag_type()
    }

    // JAVA: net.minecraft.nbt.EndTag#copy
    pub fn copy(&self) -> EndTag {
        *self
    }

    // JAVA: net.minecraft.nbt.EndTag#accept
    pub fn accept<V: TagVisitor>(&self, visitor: &mut V) {
        visitor.visit_end(self);
    }

    // JAVA: net.minecraft.nbt.EndTag#accept
    pub fn accept_stream<V: StreamTagVisitor>(&self, visitor: &mut V) -> ValueResult {
        visitor.visit_end()
    }

    // JAVA: net.minecraft.nbt.EndTag#toString
    pub fn to_string_value(&self) -> String {
        let mut v = StringTagVisitor::new();
        TagVisitor::visit_end(&mut v, self);
        v.build()
    }
}

impl std::fmt::Display for EndTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string_value())
    }
}

pub struct EndTagType;

impl EndTagType {
    // JAVA: net.minecraft.nbt.EndTag$TYPE#load
    pub fn load<R: Read>(&self, _input: &mut R, acc: &mut NbtAccounter) -> io::Result<EndTag> {
        acc.account_bytes(8)?;
        Ok(EndTag::INSTANCE)
    }

    // JAVA: net.minecraft.nbt.EndTag$TYPE#parse
    pub fn parse<R: Read, V: StreamTagVisitor>(
        &self,
        _input: &mut R,
        visitor: &mut V,
        acc: &mut NbtAccounter,
    ) -> io::Result<ValueResult> {
        acc.account_bytes(8)?;
        Ok(visitor.visit_end())
    }

    // JAVA: net.minecraft.nbt.EndTag$TYPE#skip
    pub fn skip_with_count<R: Read>(
        &self,
        _input: &mut R,
        _count: i32,
        _acc: &mut NbtAccounter,
    ) -> io::Result<()> {
        Ok(())
    }

    // JAVA: net.minecraft.nbt.EndTag$TYPE#skip
    pub fn skip<R: Read>(&self, _input: &mut R, _acc: &mut NbtAccounter) -> io::Result<()> {
        Ok(())
    }

    // JAVA: net.minecraft.nbt.EndTag$TYPE#getName
    pub fn get_name(&self) -> &'static str {
        "END"
    }

    // JAVA: net.minecraft.nbt.EndTag$TYPE#getPrettyName
    pub fn get_pretty_name(&self) -> &'static str {
        "TAG_End"
    }
}

static END_TAG_TYPE: OnceLock<EndTagType> = OnceLock::new();

pub fn end_tag_type() -> &'static EndTagType {
    END_TAG_TYPE.get_or_init(|| EndTagType)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn instance_properties() {
        let tag = EndTag::INSTANCE;
        assert_eq!(tag.get_id(), 0);
        assert_eq!(tag.size_in_bytes(), 8);
        assert_eq!(tag.get_type().get_name(), "END");
        assert_eq!(tag.get_type().get_pretty_name(), "TAG_End");
    }

    #[test]
    fn write_is_noop_and_copy_is_same() {
        let tag = EndTag::INSTANCE;
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();
        assert!(buf.is_empty());
        let copied = tag.copy();
        assert_eq!(copied, EndTag::INSTANCE);
    }

    #[test]
    fn to_string_and_display() {
        let tag = EndTag::INSTANCE;
        assert_eq!(tag.to_string_value(), "END");
        assert_eq!(tag.to_string(), "END");
    }

    #[test]
    fn type_load_accounts_bytes() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut acc = NbtAccounter::new(8);
        let tag = end_tag_type().load(&mut input, &mut acc).unwrap();
        assert_eq!(tag, EndTag::INSTANCE);

        let mut input2 = Cursor::new(Vec::<u8>::new());
        let mut acc2 = NbtAccounter::new(7);
        assert!(end_tag_type().load(&mut input2, &mut acc2).is_err());
    }

    #[test]
    fn type_parse_and_skip() {
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut acc = NbtAccounter::new(16);
        let mut visitor = StringTagVisitor::new();
        let res = end_tag_type().parse(&mut input, &mut visitor, &mut acc).unwrap();
        assert_eq!(res, ValueResult::Continue);
        assert_eq!(visitor.build(), "END");

        end_tag_type().skip_with_count(&mut Cursor::new(Vec::<u8>::new()), 0, &mut NbtAccounter::new(0)).unwrap();
        end_tag_type().skip(&mut Cursor::new(Vec::<u8>::new()), &mut NbtAccounter::new(0)).unwrap();
    }
}
