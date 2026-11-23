pub const CAPACITY: usize = 240;

#[derive(Debug, Clone)]
pub struct LocalSampleLogger {
    pub sample: Vec<i64>,
    samples: Vec<Vec<i64>>,
    start: i32,
    current_size: i32,
}

impl LocalSampleLogger {
    // JAVA: net.minecraft.util.debugchart.LocalSampleLogger#<init>
    pub fn new(dimensions: i32) -> Self {
        Self::with_sample(dimensions, vec![0i64; dimensions as usize])
    }

    // JAVA: net.minecraft.util.debugchart.LocalSampleLogger#<init>
    pub fn with_sample(dimensions: i32, sample: Vec<i64>) -> Self {
        let dim = dimensions as usize;
        let mut samples = Vec::with_capacity(CAPACITY);
        for _ in 0..CAPACITY {
            samples.push(vec![0i64; dim]);
        }
        Self {
            sample,
            samples,
            start: 0,
            current_size: 0,
        }
    }

    // JAVA: net.minecraft.util.debugchart.LocalSampleLogger#useSample
    pub fn use_sample(&mut self) {
        let idx = self.wrap_index(self.start.wrapping_add(self.current_size));
        let row = idx as usize;
        let len = self.sample.len();
        self.samples[row][..len].copy_from_slice(&self.sample[..len]);
        if self.current_size < CAPACITY as i32 {
            self.current_size = self.current_size.wrapping_add(1);
        } else {
            self.start = self.wrap_index(self.start.wrapping_add(1));
        }
    }

    // JAVA: net.minecraft.util.debugchart.LocalSampleLogger#capacity
    pub fn capacity(&self) -> i32 {
        self.samples.len() as i32
    }

    // JAVA: net.minecraft.util.debugchart.LocalSampleLogger#size
    pub fn size(&self) -> i32 {
        self.current_size
    }

    // JAVA: net.minecraft.util.debugchart.LocalSampleLogger#get
    pub fn get(&self, index: i32) -> i64 {
        self.get_at(index, 0)
    }

    // JAVA: net.minecraft.util.debugchart.LocalSampleLogger#get
    pub fn get_at(&self, index: i32, dimension: i32) -> i64 {
        if index >= 0 && index < self.current_size {
            let row_idx = self.wrap_index(self.start.wrapping_add(index)) as usize;
            let row = &self.samples[row_idx];
            if dimension >= 0 && (dimension as usize) < row.len() {
                row[dimension as usize]
            } else {
                panic!(
                    "IndexOutOfBoundsException: {} out of bounds for dimensions {}",
                    dimension,
                    row.len()
                );
            }
        } else {
            panic!(
                "IndexOutOfBoundsException: {} out of bounds for length {}",
                index, self.current_size
            );
        }
    }

    // JAVA: net.minecraft.util.debugchart.LocalSampleLogger#wrapIndex
    fn wrap_index(&self, index: i32) -> i32 {
        index % (CAPACITY as i32)
    }

    // JAVA: net.minecraft.util.debugchart.LocalSampleLogger#reset
    pub fn reset(&mut self) {
        self.start = 0;
        self.current_size = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_log_and_get() {
        let mut logger = LocalSampleLogger::new(2);
        logger.sample[0] = 10;
        logger.sample[1] = 20;
        logger.use_sample();
        logger.sample[0] = 30;
        logger.sample[1] = 40;
        logger.use_sample();

        assert_eq!(logger.size(), 2);
        assert_eq!(logger.capacity(), 240);
        assert_eq!(logger.get(0), 10);
        assert_eq!(logger.get_at(0, 1), 20);
        assert_eq!(logger.get(1), 30);
        assert_eq!(logger.get_at(1, 1), 40);
    }

    #[test]
    fn wrap_around_at_capacity() {
        let mut logger = LocalSampleLogger::new(1);
        for i in 0..240i64 {
            logger.sample[0] = i;
            logger.use_sample();
        }
        assert_eq!(logger.size(), 240);
        assert_eq!(logger.get(0), 0);
        assert_eq!(logger.get(239), 239);

        logger.sample[0] = 999;
        logger.use_sample();
        assert_eq!(logger.size(), 240);
        assert_eq!(logger.get(0), 1);
        assert_eq!(logger.get(239), 999);

        for i in 1000..1010i64 {
            logger.sample[0] = i;
            logger.use_sample();
        }
        assert_eq!(logger.get(0), 11);
        assert_eq!(logger.get(239), 1009);
    }

    #[test]
    #[should_panic(expected = "IndexOutOfBoundsException")]
    fn get_negative_index_panics() {
        let mut logger = LocalSampleLogger::new(1);
        logger.sample[0] = 1;
        logger.use_sample();
        let _ = logger.get(-1);
    }

    #[test]
    #[should_panic(expected = "IndexOutOfBoundsException")]
    fn get_dimension_out_of_bounds_panics() {
        let mut logger = LocalSampleLogger::new(1);
        logger.sample[0] = 1;
        logger.use_sample();
        let _ = logger.get_at(0, 1);
    }

    #[test]
    fn reset_clears_state() {
        let mut logger = LocalSampleLogger::new(1);
        logger.sample[0] = 42;
        logger.use_sample();
        assert_eq!(logger.size(), 1);
        logger.reset();
        assert_eq!(logger.size(), 0);
        assert_eq!(logger.capacity(), 240);

        logger.sample[0] = 7;
        logger.use_sample();
        assert_eq!(logger.size(), 1);
        assert_eq!(logger.get(0), 7);
    }

    #[test]
    fn float_bits_preserved() {
        let mut logger = LocalSampleLogger::new(2);
        let nan_bits = f64::NAN.to_bits() as i64;
        let neg_zero_bits = (-0.0f64).to_bits() as i64;
        logger.sample[0] = nan_bits;
        logger.sample[1] = neg_zero_bits;
        logger.use_sample();

        assert_eq!(logger.get_at(0, 0), nan_bits);
        assert_eq!(logger.get_at(0, 1), neg_zero_bits);
        assert_eq!(
            f64::from_bits(logger.get_at(0, 0) as u64).to_bits(),
            f64::NAN.to_bits()
        );
        assert_eq!(
            f64::from_bits(logger.get_at(0, 1) as u64).to_bits(),
            (-0.0f64).to_bits()
        );
    }
}
