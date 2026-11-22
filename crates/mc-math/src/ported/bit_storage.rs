pub trait BitStorage {
    // JAVA: net.minecraft.util.BitStorage#getAndSet
    fn get_and_set(&mut self, index: i32, value: i32) -> i32;

    // JAVA: net.minecraft.util.BitStorage#set
    fn set(&mut self, index: i32, value: i32);

    // JAVA: net.minecraft.util.BitStorage#get
    fn get(&self, index: i32) -> i32;

    // JAVA: net.minecraft.util.BitStorage#getRaw
    fn get_raw(&self) -> &[i64];

    // JAVA: net.minecraft.util.BitStorage#getSize
    fn get_size(&self) -> i32;

    // JAVA: net.minecraft.util.BitStorage#getBits
    fn get_bits(&self) -> i32;

    // JAVA: net.minecraft.util.BitStorage#getAll
    fn get_all(&self, consumer: &mut dyn FnMut(i32));

    // JAVA: net.minecraft.util.BitStorage#unpack
    fn unpack(&self, output: &mut [i32]);

    // JAVA: net.minecraft.util.BitStorage#copy
    fn copy(&self) -> Box<dyn BitStorage>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct MockBitStorage {
        data: Vec<i32>,
        bits: i32,
        raw: Vec<i64>,
    }

    impl MockBitStorage {
        fn new(size: usize, bits: i32) -> Self {
            Self {
                data: vec![0; size],
                bits,
                raw: vec![0i64],
            }
        }
    }

    impl BitStorage for MockBitStorage {
        fn get_and_set(&mut self, index: i32, value: i32) -> i32 {
            let idx = index as usize;
            let old = self.data[idx];
            self.data[idx] = value;
            old
        }

        fn set(&mut self, index: i32, value: i32) {
            self.data[index as usize] = value;
        }

        fn get(&self, index: i32) -> i32 {
            self.data[index as usize]
        }

        fn get_raw(&self) -> &[i64] {
            &self.raw
        }

        fn get_size(&self) -> i32 {
            self.data.len() as i32
        }

        fn get_bits(&self) -> i32 {
            self.bits
        }

        fn get_all(&self, consumer: &mut dyn FnMut(i32)) {
            for &v in &self.data {
                consumer(v);
            }
        }

        fn unpack(&self, output: &mut [i32]) {
            for (i, &v) in self.data.iter().enumerate() {
                if i < output.len() {
                    output[i] = v;
                }
            }
        }

        fn copy(&self) -> Box<dyn BitStorage> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn get_set_and_get_and_set() {
        let mut s = MockBitStorage::new(4, 5);
        assert_eq!(s.get(0), 0);
        s.set(1, 7);
        assert_eq!(s.get(1), 7);
        let old = s.get_and_set(1, 3);
        assert_eq!(old, 7);
        assert_eq!(s.get(1), 3);
    }

    #[test]
    fn size_bits_and_raw() {
        let s = MockBitStorage::new(16, 4);
        assert_eq!(s.get_size(), 16);
        assert_eq!(s.get_bits(), 4);
        assert_eq!(s.get_raw(), &[0i64]);
    }

    #[test]
    fn get_all_collects_values() {
        let mut s = MockBitStorage::new(3, 2);
        s.set(0, 1);
        s.set(1, 2);
        s.set(2, 3);
        let mut out = Vec::new();
        s.get_all(&mut |v| out.push(v));
        assert_eq!(out, vec![1, 2, 3]);
    }

    #[test]
    fn unpack_fills_output() {
        let mut s = MockBitStorage::new(3, 4);
        s.set(0, 5);
        s.set(1, 6);
        s.set(2, 7);
        let mut out = [0i32; 3];
        s.unpack(&mut out);
        assert_eq!(out, [5, 6, 7]);
    }

    #[test]
    fn copy_is_independent() {
        let mut s = MockBitStorage::new(2, 3);
        s.set(0, 9);
        let mut c = s.copy();
        assert_eq!(c.get(0), 9);
        c.set(0, 1);
        assert_eq!(s.get(0), 9);
        assert_eq!(c.get(0), 1);
    }
}
