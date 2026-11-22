use std::fmt;

pub const DEFAULT_NBT_QUOTA: i32 = 2_097_152;
pub const UNCOMPRESSED_NBT_QUOTA: i32 = 104_857_600;
const MAX_STACK_DEPTH: i32 = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtAccounterException {
    message: String,
}

impl NbtAccounterException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for NbtAccounterException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NbtAccounterException {}

#[derive(Debug, Clone)]
pub struct NbtAccounter {
    quota: i64,
    usage: i64,
    max_depth: i32,
    depth: i32,
}

impl NbtAccounter {
    // JAVA: net.minecraft.nbt.NbtAccounter#<init>
    pub fn new(quota: i64, max_depth: i32) -> Self {
        Self {
            quota,
            usage: 0,
            max_depth,
            depth: 0,
        }
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#create
    pub fn create(quota: i64) -> Self {
        Self::new(quota, MAX_STACK_DEPTH)
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#defaultQuota
    pub fn default_quota() -> Self {
        Self::new(DEFAULT_NBT_QUOTA as i64, MAX_STACK_DEPTH)
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#uncompressedQuota
    pub fn uncompressed_quota() -> Self {
        Self::new(UNCOMPRESSED_NBT_QUOTA as i64, MAX_STACK_DEPTH)
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#unlimitedHeap
    pub fn unlimited_heap() -> Self {
        Self::new(i64::MAX, MAX_STACK_DEPTH)
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#accountBytes(long,long)
    pub fn account_bytes_mul(&mut self, count: i64, size: i64) -> Result<(), NbtAccounterException> {
        self.account_bytes(count.wrapping_mul(size))
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#accountBytes(long)
    pub fn account_bytes(&mut self, bytes: i64) -> Result<(), NbtAccounterException> {
        if bytes < 0 {
            return Err(NbtAccounterException::new(format!(
                "Tried to account NBT tag with negative size: {}",
                bytes
            )));
        }
        let next = self.usage.wrapping_add(bytes);
        if next > self.quota {
            return Err(NbtAccounterException::new(format!(
                "Tried to read NBT tag that was too big; tried to allocate: {} + {} bytes where max allowed: {}",
                self.usage, bytes, self.quota
            )));
        }
        self.usage = next;
        Ok(())
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#pushDepth
    pub fn push_depth(&mut self) -> Result<(), NbtAccounterException> {
        if self.depth >= self.max_depth {
            return Err(NbtAccounterException::new(format!(
                "Tried to read NBT tag with too high complexity, depth > {}",
                self.max_depth
            )));
        }
        self.depth = self.depth.wrapping_add(1);
        Ok(())
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#popDepth
    pub fn pop_depth(&mut self) -> Result<(), NbtAccounterException> {
        if self.depth <= 0 {
            return Err(NbtAccounterException::new(
                "NBT-Accounter tried to pop stack-depth at top-level".to_string(),
            ));
        }
        self.depth = self.depth.wrapping_sub(1);
        Ok(())
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#getUsage
    pub fn get_usage(&self) -> i64 {
        self.usage
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#getDepth
    pub fn get_depth(&self) -> i32 {
        self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_quotas() {
        let d = NbtAccounter::default_quota();
        assert_eq!(d.quota, DEFAULT_NBT_QUOTA as i64);
        assert_eq!(d.max_depth, 512);
        let u = NbtAccounter::uncompressed_quota();
        assert_eq!(u.quota, UNCOMPRESSED_NBT_QUOTA as i64);
        let uh = NbtAccounter::unlimited_heap();
        assert_eq!(uh.quota, i64::MAX);
    }

    #[test]
    fn account_bytes_and_negative() {
        let mut a = NbtAccounter::create(16);
        a.account_bytes(8).unwrap();
        a.account_bytes(8).unwrap();
        assert_eq!(a.get_usage(), 16);
        let err = a.account_bytes(1).unwrap_err();
        assert!(err.to_string().contains("too big"));
        let neg = a.account_bytes(-1).unwrap_err();
        assert!(neg.to_string().contains("negative size"));
    }

    #[test]
    fn account_bytes_mul() {
        let mut a = NbtAccounter::create(100);
        a.account_bytes_mul(10, 3).unwrap();
        assert_eq!(a.get_usage(), 30);
        let err = a.account_bytes_mul(10, 8).unwrap_err();
        assert!(err.to_string().contains("too big"));
    }

    #[test]
    fn depth_push_pop_and_limits() {
        let mut a = NbtAccounter::new(100, 2);
        a.push_depth().unwrap();
        a.push_depth().unwrap();
        assert_eq!(a.get_depth(), 2);
        let err = a.push_depth().unwrap_err();
        assert!(err.to_string().contains("too high complexity"));
        a.pop_depth().unwrap();
        a.pop_depth().unwrap();
        assert_eq!(a.get_depth(), 0);
        let err2 = a.pop_depth().unwrap_err();
        assert!(err2.to_string().contains("top-level"));
    }
}
