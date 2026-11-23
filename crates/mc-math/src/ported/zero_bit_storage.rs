use std::fmt;

pub const RAW: [i64; 0] = [];

pub trait BitStorage {
    fn get_and_set(&mut self, index: i32, value: i32) -> i32;
    fn set(&mut self, index: i32, value: i32);
    fn get(&self, index: i32) -> i32;
    fn get_raw(&self) -> &[i64];
    fn get_size(&self) -> i32;
    fn get_bits(&self) -> i32;
    fn get_all(&self, consumer: &mut dyn FnMut(i32));
    fn unpack(&self, output: &mut [i32]);
    fn copy_storage(&self) -> Box<dyn BitStorage>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroBitStorage {
    size: i32,
}

impl ZeroBitStorage {
    // JAVA: net.minecraft.util.ZeroBitStorage#<init>
    pub fn new(size: i32) -> Self {
        Self { size }
    }

    fn check_index(&self, index: i32) {
        if index < 0 || index >= self.size {
            panic!(
                "{} not in inclusive range [0, {}]",
                index,
                self.size.wrapping_sub(1)
            );
        }
    }

    fn check_value(value: i32) {
        if value < 0 || value > 0 {
            panic!("{} not in inclusive range [0, 0]", value);
        }
    }

    // JAVA: net.minecraft.util.ZeroBitStorage#getAndSet
    pub fn get_and_set(&mut self, index: i32, value: i32) -> i32 {
        self.check_index(index);
        Self::check_value(value);
        0
    }

    // JAVA: net.minecraft.util.ZeroBitStorage#set
    pub fn set(&mut self, index: i32, value: i32) {
        self.check_index(index);
        Self::check_value(value);
    }

    // JAVA: net.minecraft.util.ZeroBitStorage#get
    pub fn get(&self, index: i32) -> i32 {
        self.check_index(index);
        0
    }

    // JAVA: net.minecraft.util.ZeroBitStorage#getRaw
    pub fn get_raw(&self) -> &[i64] {
        &RAW
    }

    // JAVA: net.minecraft.util.ZeroBitStorage#getSize
    pub fn get_size(&self) -> i32 {
        self.size
    }

    // JAVA: net.minecraft.util.ZeroBitStorage#getBits
    pub fn get_bits(&self) -> i32 {
        0
    }

    // JAVA: net.minecraft.util.ZeroBitStorage#getAll
    pub fn get_all(&self, consumer: &mut dyn FnMut(i32)) {
        let mut i: i32 = 0;
        while i < self.size {
            consumer(0);
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.util.ZeroBitStorage#unpack
    pub fn unpack(&self, output: &mut [i32]) {
        let len = self.size.max(0) as usize;
        let limit = len.min(output.len());
        for item in &mut output[..limit] {
            *item = 0;
        }
    }

    // JAVA: net.minecraft.util.ZeroBitStorage#copy
    pub fn copy(&self) -> ZeroBitStorage {
        self.clone()
    }
}

impl BitStorage for ZeroBitStorage {
    fn get_and_set(&mut self, index: i32, value: i32) -> i32 {
        ZeroBitStorage::get_and_set(self, index, value)
    }

    fn set(&mut self, index: i32, value: i32) {
        ZeroBitStorage::set(self, index, value)
    }

    fn get(&self, index: i32) -> i32 {
        ZeroBitStorage::get(self, index)
    }

    fn get_raw(&self) -> &[i64] {
        ZeroBitStorage::get_raw(self)
    }

    fn get_size(&self) -> i32 {
        ZeroBitStorage::get_size(self)
    }

    fn get_bits(&self) -> i32 {
        ZeroBitStorage::get_bits(self)
    }

    fn get_all(&self, consumer: &mut dyn FnMut(i32)) {
        ZeroBitStorage::get_all(self, consumer)
    }

    fn unpack(&self, output: &mut [i32]) {
        ZeroBitStorage::unpack(self, output)
    }

    fn copy_storage(&self) -> Box<dyn BitStorage> {
        Box::new(self.copy())
    }
}

impl fmt::Display for ZeroBitStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ZeroBitStorage(size={})", self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_access_and_constants() {
        let mut s = ZeroBitStorage::new(5);
        assert_eq!(s.get_size(), 5);
        assert_eq!(s.get_bits(), 0);
        assert_eq!(s.get(0), 0);
        assert_eq!(s.get(4), 0);
        assert_eq!(s.get_and_set(2, 0), 0);
        s.set(3, 0);
        assert!(s.get_raw().is_empty());
        assert_eq!(RAW.len(), 0);
    }

    #[test]
    fn get_all_and_unpack() {
        let s = ZeroBitStorage::new(4);
        let mut seen = Vec::new();
        s.get_all(&mut |v| seen.push(v));
        assert_eq!(seen, vec![0, 0, 0, 0]);

        let mut out = [1, 2, 3, 4, 5];
        s.unpack(&mut out);
        assert_eq!(out, [0, 0, 0, 0, 5]);
    }

    #[test]
    #[should_panic]
    fn get_rejects_negative_index() {
        let s = ZeroBitStorage::new(3);
        let _ = s.get(-1);
    }

    #[test]
    #[should_panic]
    fn set_rejects_nonzero_value() {
        let mut s = ZeroBitStorage::new(3);
        s.set(1, 1);
    }

    #[test]
    #[should_panic]
    fn get_and_set_rejects_index_at_size() {
        let mut s = ZeroBitStorage::new(3);
        let _ = s.get_and_set(3, 0);
    }

    #[test]
    fn copy_returns_equivalent_storage() {
        let s = ZeroBitStorage::new(2);
        let c = s.copy();
        assert_eq!(s, c);
        let boxed = s.copy_storage();
        assert_eq!(boxed.get_size(), 2);
        assert_eq!(boxed.get_bits(), 0);
    }
}
