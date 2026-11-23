use std::fmt;

pub struct StaticCache2D<T> {
    min_x: i32,
    min_z: i32,
    size_x: i32,
    size_z: i32,
    cache: Vec<T>,
}

impl<T> StaticCache2D<T> {
    // JAVA: net.minecraft.util.StaticCache2D#create
    pub fn create<F: FnMut(i32, i32) -> T>(
        center_x: i32,
        center_z: i32,
        radius: i32,
        mut initializer: F,
    ) -> Self {
        let min_x = center_x.wrapping_sub(radius);
        let min_z = center_z.wrapping_sub(radius);
        let size = radius.wrapping_mul(2).wrapping_add(1);
        Self::new(min_x, min_z, size, size, &mut initializer)
    }

    // JAVA: net.minecraft.util.StaticCache2D#<init>
    fn new<F: FnMut(i32, i32) -> T>(
        min_x: i32,
        min_z: i32,
        size_x: i32,
        size_z: i32,
        initializer: &mut F,
    ) -> Self {
        let total = (size_x as usize).wrapping_mul(size_z as usize);
        let mut cache = Vec::with_capacity(total);
        let mut x = min_x;
        let end_x = min_x.wrapping_add(size_x);
        while x < end_x {
            let mut z = min_z;
            let end_z = min_z.wrapping_add(size_z);
            while z < end_z {
                cache.push(initializer(x, z));
                z = z.wrapping_add(1);
            }
            x = x.wrapping_add(1);
        }
        Self {
            min_x,
            min_z,
            size_x,
            size_z,
            cache,
        }
    }

    // JAVA: net.minecraft.util.StaticCache2D#forEach
    pub fn for_each<F: FnMut(&T)>(&self, mut consumer: F) {
        for item in &self.cache {
            consumer(item);
        }
    }

    // JAVA: net.minecraft.util.StaticCache2D#get
    pub fn get(&self, x: i32, z: i32) -> &T {
        if !self.contains(x, z) {
            panic!(
                "Requested out of range value ({}, {}) from {}",
                x, z, self
            );
        }
        let idx = self.get_index(x, z);
        &self.cache[idx]
    }

    // JAVA: net.minecraft.util.StaticCache2D#contains
    pub fn contains(&self, x: i32, z: i32) -> bool {
        let dx = x.wrapping_sub(self.min_x);
        let dz = z.wrapping_sub(self.min_z);
        dx >= 0 && dx < self.size_x && dz >= 0 && dz < self.size_z
    }

    // JAVA: net.minecraft.util.StaticCache2D#getIndex
    fn get_index(&self, x: i32, z: i32) -> usize {
        let dx = x.wrapping_sub(self.min_x);
        let dz = z.wrapping_sub(self.min_z);
        let idx = dx.wrapping_mul(self.size_z).wrapping_add(dz);
        idx as u32 as usize
    }
}

impl<T> fmt::Display for StaticCache2D<T> {
    // JAVA: net.minecraft.util.StaticCache2D#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StaticCache2D[{}, {}, {}, {}]",
            self.min_x,
            self.min_z,
            self.min_x.wrapping_add(self.size_x),
            self.min_z.wrapping_add(self.size_z)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_get() {
        let cache = StaticCache2D::create(10, 20, 1, |x, z| (x, z));
        assert_eq!(*cache.get(9, 19), (9, 19));
        assert_eq!(*cache.get(10, 20), (10, 20));
        assert_eq!(*cache.get(11, 21), (11, 21));
        assert!(cache.contains(9, 19));
        assert!(!cache.contains(8, 19));
        assert!(!cache.contains(12, 21));
    }

    #[test]
    fn for_each_order() {
        let cache = StaticCache2D::create(0, 0, 1, |x, z| (x, z));
        let mut out = Vec::new();
        cache.for_each(|v| out.push(*v));
        assert_eq!(
            out,
            vec![
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 0),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ]
        );
    }

    #[test]
    #[should_panic(expected = "Requested out of range value")]
    fn get_out_of_range_panics() {
        let cache = StaticCache2D::create(0, 0, 0, |x, z| (x, z));
        let _ = cache.get(1, 0);
    }

    #[test]
    fn to_string_format() {
        let cache = StaticCache2D::create(5, 6, 2, |_, _| 0i32);
        assert_eq!(cache.to_string(), "StaticCache2D[3, 4, 8, 9]");
    }

    #[test]
    fn negative_coordinates_and_indexing() {
        let cache = StaticCache2D::create(-10, -20, 1, |x, z| {
            x.wrapping_mul(100).wrapping_add(z)
        });
        assert!(cache.contains(-11, -21));
        assert!(cache.contains(-9, -19));
        assert!(!cache.contains(-12, -21));
        assert_eq!(
            *cache.get(-10, -20),
            (-10i32).wrapping_mul(100).wrapping_add(-20)
        );
    }
}
