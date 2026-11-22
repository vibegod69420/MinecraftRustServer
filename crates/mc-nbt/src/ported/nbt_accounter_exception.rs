use std::fmt;

/// Base NBT exception type, kept self-contained for this module.
// JAVA: net.minecraft.nbt.NbtException
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtException {
    message: String,
}

impl NbtException {
    // JAVA: net.minecraft.nbt.NbtException#<init>
    pub fn new(message: String) -> Self {
        Self { message }
    }

    // JAVA: net.minecraft.nbt.NbtException#getMessage
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for NbtException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NbtException {}

// JAVA: net.minecraft.nbt.NbtAccounterException
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtAccounterException {
    inner: NbtException,
}

impl NbtAccounterException {
    // JAVA: net.minecraft.nbt.NbtAccounterException#<init>
    pub fn new(message: String) -> Self {
        Self {
            inner: NbtException::new(message),
        }
    }

    // JAVA: net.minecraft.nbt.NbtAccounterException#getMessage
    pub fn message(&self) -> &str {
        self.inner.message()
    }

    // JAVA: net.minecraft.nbt.NbtAccounterException
    pub fn as_nbt_exception(&self) -> &NbtException {
        &self.inner
    }
}

impl fmt::Display for NbtAccounterException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl std::error::Error for NbtAccounterException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl From<NbtAccounterException> for NbtException {
    fn from(value: NbtAccounterException) -> Self {
        value.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn new_and_message() {
        let e = NbtAccounterException::new("limit exceeded".to_string());
        assert_eq!(e.message(), "limit exceeded");
        assert_eq!(e.to_string(), "limit exceeded");
    }

    #[test]
    fn as_nbt_exception_and_from() {
        let e = NbtAccounterException::new("oops".to_string());
        assert_eq!(e.as_nbt_exception().message(), "oops");
        let base: NbtException = e.clone().into();
        assert_eq!(base.message(), "oops");
    }

    #[test]
    fn source_is_nbt_exception() {
        let e = NbtAccounterException::new("depth".to_string());
        let src = e.source().expect("source");
        assert_eq!(src.to_string(), "depth");
    }

    #[test]
    fn empty_and_negative_style_messages() {
        let empty = NbtAccounterException::new(String::new());
        assert_eq!(empty.message(), "");
        let neg = NbtAccounterException::new("-1 bytes".to_string());
        assert_eq!(neg.message(), "-1 bytes");
    }
}
