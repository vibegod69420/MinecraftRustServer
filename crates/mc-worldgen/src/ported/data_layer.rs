pub const LAYER_COUNT: usize = 16;
pub const LAYER_SIZE: usize = 128;
pub const SIZE: usize = 2048;
const NIBBLE_SIZE: usize = 4;

#[derive(Clone, Debug)]
pub struct DataLayer {
    data: Option<Vec<u8>>,
    default_value: i32,
}

impl Default for DataLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl DataLayer {
    // JAVA: net.minecraft.world.level.chunk.DataLayer#<init>
    pub fn new() -> Self {
        Self::with_default(0)
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#<init>
    pub fn with_default(default_value: i32) -> Self {
        Self {
            data: None,
            default_value,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#<init>
    pub fn from_raw(data: Vec<u8>) -> Self {
        if data.len() != SIZE {
            panic!(
                "DataLayer should be 2048 bytes not: {}",
                data.len()
            );
        }
        Self {
            data: Some(data),
            default_value: 0,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#get
    pub fn get(&self, x: i32, y: i32, z: i32) -> i32 {
        self.get_by_index(Self::get_index(x, y, z))
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#set
    pub fn set(&mut self, x: i32, y: i32, z: i32, value: i32) {
        let index = Self::get_index(x, y, z);
        self.set_by_index(index, value);
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#getIndex
    fn get_index(x: i32, y: i32, z: i32) -> i32 {
        (y << 8) | (z << 4) | x
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#get
    fn get_by_index(&self, index: i32) -> i32 {
        match &self.data {
            None => self.default_value,
            Some(data) => {
                let byte_index = Self::get_byte_index(index) as usize;
                let nibble_index = Self::get_nibble_index(index);
                ((data[byte_index] as i32) >> (4 * nibble_index)) & 15
            }
        }
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#set
    fn set_by_index(&mut self, index: i32, value: i32) {
        let byte_index = Self::get_byte_index(index) as usize;
        let nibble_index = Self::get_nibble_index(index);
        let shift = 4 * nibble_index;
        let mask = !(15_i32 << shift);
        let shifted_value = (value & 15) << shift;
        let data = self.get_data();
        let current = data[byte_index] as i32;
        data[byte_index] = ((current & mask) | shifted_value) as u8;
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#getNibbleIndex
    fn get_nibble_index(index: i32) -> i32 {
        index & 1
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#getByteIndex
    fn get_byte_index(index: i32) -> i32 {
        index >> 1
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#fill
    pub fn fill(&mut self, value: i32) {
        self.default_value = value;
        self.data = None;
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#packFilled
    fn pack_filled(value: i32) -> u8 {
        let mut b = value as u8;
        let mut i = 4i32;
        while i < 8 {
            b |= (value << i) as u8;
            i += 4;
        }
        b
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#getData
    pub fn get_data(&mut self) -> &mut [u8] {
        if self.data.is_none() {
            let mut data = vec![0u8; SIZE];
            if self.default_value != 0 {
                let filled = Self::pack_filled(self.default_value);
                data.fill(filled);
            }
            self.data = Some(data);
        }
        self.data.as_mut().unwrap().as_mut_slice()
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#copy
    pub fn copy(&self) -> Self {
        match &self.data {
            None => Self::with_default(self.default_value),
            Some(data) => Self {
                data: Some(data.clone()),
                default_value: 0,
            },
        }
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#toString
    pub fn to_string_repr(&self) -> String {
        let mut s = String::new();
        for i in 0..4096i32 {
            let val = self.get_by_index(i);
            s.push_str(&format!("{:x}", val));
            if (i & 15) == 15 {
                s.push('\n');
            }
            if (i & 0xFF) == 255 {
                s.push('\n');
            }
        }
        s
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#layerToString
    pub fn layer_to_string(&self, _layer: i32) -> String {
        let mut s = String::new();
        for i in 0..256i32 {
            let val = self.get_by_index(i);
            s.push_str(&format!("{:x}", val));
            if (i & 15) == 15 {
                s.push('\n');
            }
        }
        s
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#isDefinitelyHomogenous
    pub fn is_definitely_homogenous(&self) -> bool {
        self.data.is_none()
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#isDefinitelyFilledWith
    pub fn is_definitely_filled_with(&self, value: i32) -> bool {
        self.data.is_none() && self.default_value == value
    }

    // JAVA: net.minecraft.world.level.chunk.DataLayer#isEmpty
    pub fn is_empty(&self) -> bool {
        self.data.is_none() && self.default_value == 0
    }
}

impl std::fmt::Display for DataLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string_repr())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_round_trip() {
        let mut layer = DataLayer::new();
        layer.set(0, 0, 0, 5);
        layer.set(15, 15, 15, 15);
        layer.set(1, 2, 3, 7);
        assert_eq!(layer.get(0, 0, 0), 5);
        assert_eq!(layer.get(15, 15, 15), 15);
        assert_eq!(layer.get(1, 2, 3), 7);
        assert_eq!(layer.get(0, 0, 1), 0);
    }

    #[test]
    fn test_default_value_fill_and_lazy_init() {
        let mut layer = DataLayer::with_default(7);
        assert!(layer.is_definitely_homogenous());
        assert!(layer.is_definitely_filled_with(7));
        assert!(!layer.is_empty());
        assert_eq!(layer.get(3, 4, 5), 7);

        layer.set(3, 4, 5, 2);
        assert!(!layer.is_definitely_homogenous());
        assert_eq!(layer.get(3, 4, 5), 2);
        assert_eq!(layer.get(0, 0, 0), 7);

        layer.fill(0);
        assert!(layer.is_empty());
        assert!(layer.is_definitely_homogenous());
        assert_eq!(layer.get(3, 4, 5), 0);

        let mut filled = DataLayer::with_default(0x5);
        let data = filled.get_data();
        assert_eq!(data.len(), SIZE);
        assert!(data.iter().all(|&b| b == 0x55));
    }

    #[test]
    fn test_nibble_packing_and_value_masking() {
        let mut layer = DataLayer::new();
        // indices 0 and 1 share the same byte
        layer.set(0, 0, 0, 0xA);
        layer.set(1, 0, 0, 0xB);
        assert_eq!(layer.get(0, 0, 0), 0xA);
        assert_eq!(layer.get(1, 0, 0), 0xB);

        let data = layer.get_data();
        assert_eq!(data[0], 0xBA);

        // value masking: Java does (value & 15)
        let mut m = DataLayer::new();
        m.set(0, 0, 0, 16);
        assert_eq!(m.get(0, 0, 0), 0);
        m.set(1, 0, 0, -1);
        assert_eq!(m.get(1, 0, 0), 15);
        m.set(2, 0, 0, 255);
        assert_eq!(m.get(2, 0, 0), 15);
    }

    #[test]
    fn test_copy_semantics() {
        let mut layer = DataLayer::with_default(3);
        let copy1 = layer.copy();
        assert!(copy1.is_definitely_filled_with(3));
        assert!(copy1.is_definitely_homogenous());

        layer.set(0, 0, 0, 5);
        let mut copy2 = layer.copy();
        assert_eq!(copy2.get(0, 0, 0), 5);
        assert!(!copy2.is_definitely_homogenous());

        copy2.set(0, 0, 0, 9);
        assert_eq!(layer.get(0, 0, 0), 5);
        assert_eq!(copy2.get(0, 0, 0), 9);
    }

    #[test]
    fn test_from_raw_and_validation() {
        let data = vec![0x11u8; SIZE];
        let layer = DataLayer::from_raw(data);
        assert!(!layer.is_definitely_homogenous());
        assert_eq!(layer.get(0, 0, 0), 1);
        assert_eq!(layer.get(1, 0, 0), 1);
        assert_eq!(layer.get(15, 15, 15), 1);
    }

    #[test]
    #[should_panic(expected = "DataLayer should be 2048 bytes not:")]
    fn test_from_raw_wrong_size_panics() {
        let _ = DataLayer::from_raw(vec![0u8; 100]);
    }
}
