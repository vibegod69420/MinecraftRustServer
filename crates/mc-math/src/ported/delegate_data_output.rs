use std::io;

/// Minimal mirror of `java.io.DataOutput` for self-contained delegation.
pub trait DataOutput {
    // JAVA: java.io.DataOutput#write(int)
    fn write(&mut self, b: i32) -> io::Result<()>;
    // JAVA: java.io.DataOutput#write(byte[])
    fn write_bytes(&mut self, b: &[u8]) -> io::Result<()>;
    // JAVA: java.io.DataOutput#write(byte[],int,int)
    fn write_bytes_offset(&mut self, b: &[u8], off: usize, len: usize) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeBoolean(boolean)
    fn write_boolean(&mut self, v: bool) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeByte(int)
    fn write_byte(&mut self, v: i32) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeShort(int)
    fn write_short(&mut self, v: i32) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeChar(int)
    fn write_char(&mut self, v: i32) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeInt(int)
    fn write_int(&mut self, v: i32) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeLong(long)
    fn write_long(&mut self, v: i64) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeFloat(float)
    fn write_float(&mut self, v: f32) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeDouble(double)
    fn write_double(&mut self, v: f64) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeBytes(String)
    fn write_bytes_str(&mut self, s: &str) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeChars(String)
    fn write_chars_str(&mut self, s: &str) -> io::Result<()>;
    // JAVA: java.io.DataOutput#writeUTF(String)
    fn write_utf(&mut self, s: &str) -> io::Result<()>;
}

pub struct DelegateDataOutput<D> {
    parent: D,
}

impl<D> DelegateDataOutput<D> {
    // JAVA: net.minecraft.util.DelegateDataOutput#<init>
    pub fn new(parent: D) -> Self {
        Self { parent }
    }

    pub fn into_inner(self) -> D {
        self.parent
    }

    pub fn inner(&self) -> &D {
        &self.parent
    }

    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.parent
    }
}

impl<D: DataOutput> DataOutput for DelegateDataOutput<D> {
    // JAVA: net.minecraft.util.DelegateDataOutput#write(int)
    fn write(&mut self, b: i32) -> io::Result<()> {
        self.parent.write(b)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#write(byte[])
    fn write_bytes(&mut self, b: &[u8]) -> io::Result<()> {
        self.parent.write_bytes(b)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#write(byte[],int,int)
    fn write_bytes_offset(&mut self, b: &[u8], off: usize, len: usize) -> io::Result<()> {
        self.parent.write_bytes_offset(b, off, len)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeBoolean(boolean)
    fn write_boolean(&mut self, v: bool) -> io::Result<()> {
        self.parent.write_boolean(v)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeByte(int)
    fn write_byte(&mut self, v: i32) -> io::Result<()> {
        self.parent.write_byte(v)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeShort(int)
    fn write_short(&mut self, v: i32) -> io::Result<()> {
        self.parent.write_short(v)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeChar(int)
    fn write_char(&mut self, v: i32) -> io::Result<()> {
        self.parent.write_char(v)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeInt(int)
    fn write_int(&mut self, v: i32) -> io::Result<()> {
        self.parent.write_int(v)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeLong(long)
    fn write_long(&mut self, v: i64) -> io::Result<()> {
        self.parent.write_long(v)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeFloat(float)
    fn write_float(&mut self, v: f32) -> io::Result<()> {
        self.parent.write_float(v)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeDouble(double)
    fn write_double(&mut self, v: f64) -> io::Result<()> {
        self.parent.write_double(v)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeBytes(String)
    fn write_bytes_str(&mut self, s: &str) -> io::Result<()> {
        self.parent.write_bytes_str(s)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeChars(String)
    fn write_chars_str(&mut self, s: &str) -> io::Result<()> {
        self.parent.write_chars_str(s)
    }

    // JAVA: net.minecraft.util.DelegateDataOutput#writeUTF(String)
    fn write_utf(&mut self, s: &str) -> io::Result<()> {
        self.parent.write_utf(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[derive(Default)]
    struct MockOut {
        log: Vec<String>,
    }

    impl DataOutput for MockOut {
        fn write(&mut self, b: i32) -> io::Result<()> { self.log.push(format!("w:{b}")); Ok(()) }
        fn write_bytes(&mut self, b: &[u8]) -> io::Result<()> { self.log.push(format!("wb:{}", b.len())); Ok(()) }
        fn write_bytes_offset(&mut self, b: &[u8], off: usize, len: usize) -> io::Result<()> {
            self.log.push(format!("wbo:{}:{}:{}", b.len(), off, len)); Ok(())
        }
        fn write_boolean(&mut self, v: bool) -> io::Result<()> { self.log.push(format!("bool:{v}")); Ok(()) }
        fn write_byte(&mut self, v: i32) -> io::Result<()> { self.log.push(format!("byte:{v}")); Ok(()) }
        fn write_short(&mut self, v: i32) -> io::Result<()> { self.log.push(format!("short:{v}")); Ok(()) }
        fn write_char(&mut self, v: i32) -> io::Result<()> { self.log.push(format!("char:{v}")); Ok(()) }
        fn write_int(&mut self, v: i32) -> io::Result<()> { self.log.push(format!("int:{v}")); Ok(()) }
        fn write_long(&mut self, v: i64) -> io::Result<()> { self.log.push(format!("long:{v}")); Ok(()) }
        fn write_float(&mut self, v: f32) -> io::Result<()> { self.log.push(format!("float:{}", v.to_bits())); Ok(()) }
        fn write_double(&mut self, v: f64) -> io::Result<()> { self.log.push(format!("double:{}", v.to_bits())); Ok(()) }
        fn write_bytes_str(&mut self, s: &str) -> io::Result<()> { self.log.push(format!("bytes:{s}")); Ok(()) }
        fn write_chars_str(&mut self, s: &str) -> io::Result<()> { self.log.push(format!("chars:{s}")); Ok(()) }
        fn write_utf(&mut self, s: &str) -> io::Result<()> { self.log.push(format!("utf:{s}")); Ok(()) }
    }

    #[test]
    fn delegates_all_calls() {
        let mut d = DelegateDataOutput::new(MockOut::default());
        d.write(255).unwrap();
        d.write_bytes(&[1, 2, 3]).unwrap();
        d.write_bytes_offset(&[1, 2, 3, 4], 1, 2).unwrap();
        d.write_boolean(true).unwrap();
        d.write_byte(-1).unwrap();
        d.write_short(-32768).unwrap();
        d.write_char(65).unwrap();
        d.write_int(i32::MIN).unwrap();
        d.write_long(i64::MIN).unwrap();
        d.write_float(f32::NAN).unwrap();
        d.write_double(f64::NEG_INFINITY).unwrap();
        d.write_bytes_str("ab").unwrap();
        d.write_chars_str("cd").unwrap();
        d.write_utf("ef").unwrap();

        let log = &d.inner().log;
        assert_eq!(log[0], "w:255");
        assert_eq!(log[1], "wb:3");
        assert_eq!(log[2], "wbo:4:1:2");
        assert_eq!(log[3], "bool:true");
        assert_eq!(log[4], "byte:-1");
        assert_eq!(log[5], "short:-32768");
        assert_eq!(log[6], "char:65");
        assert_eq!(log[7], format!("int:{}", i32::MIN));
        assert_eq!(log[8], format!("long:{}", i64::MIN));
        assert_eq!(log[9], format!("float:{}", f32::NAN.to_bits()));
        assert_eq!(log[10], format!("double:{}", f64::NEG_INFINITY.to_bits()));
        assert_eq!(log[11], "bytes:ab");
        assert_eq!(log[12], "chars:cd");
        assert_eq!(log[13], "utf:ef");
    }

    #[test]
    fn into_inner_returns_parent() {
        let mut d = DelegateDataOutput::new(MockOut::default());
        d.write_int(42).unwrap();
        let inner = d.into_inner();
        assert_eq!(inner.log, vec!["int:42"]);
    }

    #[test]
    fn propagates_error() {
        struct ErrOut;
        impl DataOutput for ErrOut {
            fn write(&mut self, _: i32) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_bytes(&mut self, _: &[u8]) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_bytes_offset(&mut self, _: &[u8], _: usize, _: usize) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_boolean(&mut self, _: bool) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_byte(&mut self, _: i32) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_short(&mut self, _: i32) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_char(&mut self, _: i32) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_int(&mut self, _: i32) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_long(&mut self, _: i64) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_float(&mut self, _: f32) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_double(&mut self, _: f64) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_bytes_str(&mut self, _: &str) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_chars_str(&mut self, _: &str) -> io::Result<()> { Err(io::Error::other("boom")) }
            fn write_utf(&mut self, _: &str) -> io::Result<()> { Err(io::Error::other("boom")) }
        }
        let mut d = DelegateDataOutput::new(ErrOut);
        assert!(d.write_int(1).is_err());
        assert!(d.write_utf("x").is_err());
    }
}
