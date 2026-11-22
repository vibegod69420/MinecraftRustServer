use std::any::Any;
use std::marker::PhantomData;

/// Minimal self-contained stub for `net.minecraft.core.component.DataComponentType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataComponentType<T: ?Sized + 'static> {
    pub id: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized + 'static> DataComponentType<T> {
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

/// Minimal self-contained map abstraction for component storage.
pub trait DataComponentMap {
    fn get_raw(&self, type_id: u32) -> Option<&dyn Any>;
    fn has(&self, type_id: u32) -> bool;
    fn iter_values<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Any> + 'a>;
}

/// Port of `net.minecraft.core.component.DataComponentHolder`.
/// In Java this extends `DataComponentGetter`; behavior is preserved here via default methods.
pub trait DataComponentHolder {
    type Components: DataComponentMap;

    // JAVA: net.minecraft.core.component.DataComponentHolder#getComponents
    fn components(&self) -> &Self::Components;

    // JAVA: net.minecraft.core.component.DataComponentHolder#get
    fn get<T: 'static>(&self, component_type: &DataComponentType<T>) -> Option<&T> {
        self.components()
            .get_raw(component_type.id)?
            .downcast_ref::<T>()
    }

    // JAVA: net.minecraft.core.component.DataComponentHolder#getAllOfType
    fn get_all_of_type<T: 'static>(&self) -> Vec<&T> {
        self.components()
            .iter_values()
            .filter_map(|value| value.downcast_ref::<T>())
            .collect()
    }

    // JAVA: net.minecraft.core.component.DataComponentHolder#getOrDefault
    fn get_or_default<'a, T: 'static>(
        &'a self,
        component_type: &DataComponentType<T>,
        default: &'a T,
    ) -> &'a T {
        match self.get(component_type) {
            Some(value) => value,
            None => default,
        }
    }

    // JAVA: net.minecraft.core.component.DataComponentHolder#has
    fn has<T: ?Sized + 'static>(&self, component_type: &DataComponentType<T>) -> bool {
        self.components().has(component_type.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleMap {
        entries: Vec<(u32, Box<dyn Any>)>,
    }

    impl DataComponentMap for SimpleMap {
        fn get_raw(&self, type_id: u32) -> Option<&dyn Any> {
            self.entries
                .iter()
                .find(|(id, _)| *id == type_id)
                .map(|(_, value)| value.as_ref())
        }

        fn has(&self, type_id: u32) -> bool {
            self.entries.iter().any(|(id, _)| *id == type_id)
        }

        fn iter_values<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Any> + 'a> {
            Box::new(self.entries.iter().map(|(_, value)| value.as_ref()))
        }
    }

    struct SimpleHolder {
        map: SimpleMap,
    }

    impl DataComponentHolder for SimpleHolder {
        type Components = SimpleMap;

        fn components(&self) -> &Self::Components {
            &self.map
        }
    }

    #[test]
    fn has_get_and_default_delegate_to_components() {
        let holder = SimpleHolder {
            map: SimpleMap {
                entries: vec![(1, Box::new(123_i32))],
            },
        };
        let present = DataComponentType::<i32>::new(1);
        let missing = DataComponentType::<i32>::new(2);
        let default = -5_i32;

        assert!(holder.has(&present));
        assert!(!holder.has(&missing));
        assert_eq!(holder.get(&present), Some(&123_i32));
        assert_eq!(holder.get(&missing), None);
        assert_eq!(holder.get_or_default(&missing, &default), &-5_i32);
    }

    #[test]
    fn get_all_of_type_filters_matching_values() {
        let holder = SimpleHolder {
            map: SimpleMap {
                entries: vec![
                    (1, Box::new(10_i32)),
                    (2, Box::new(20_i32)),
                    (3, Box::new(1.5_f32)),
                    (4, Box::new(-3_i32)),
                ],
            },
        };
        let ints = holder.get_all_of_type::<i32>();
        assert_eq!(ints.len(), 3);
        assert!(ints.contains(&&10_i32));
        assert!(ints.contains(&&20_i32));
        assert!(ints.contains(&&-3_i32));
    }

    #[test]
    fn handles_negative_and_overflow_integer_values() {
        let overflow_val = i32::MAX.wrapping_add(1);
        let holder = SimpleHolder {
            map: SimpleMap {
                entries: vec![
                    (1, Box::new(i32::MIN)),
                    (2, Box::new(i32::MAX)),
                    (3, Box::new(overflow_val)),
                ],
            },
        };
        let min_ty = DataComponentType::<i32>::new(1);
        let max_ty = DataComponentType::<i32>::new(2);
        let ovf_ty = DataComponentType::<i32>::new(3);
        assert_eq!(holder.get(&min_ty), Some(&i32::MIN));
        assert_eq!(holder.get(&max_ty), Some(&i32::MAX));
        assert_eq!(holder.get(&ovf_ty), Some(&i32::MIN));
    }

    #[test]
    fn handles_float_bits_including_nan_and_neg_zero() {
        let nan_bits = 0xFFC0_0001_u32;
        let neg_zero_bits = 0x8000_0000_u32;
        let holder = SimpleHolder {
            map: SimpleMap {
                entries: vec![
                    (1, Box::new(f32::from_bits(nan_bits))),
                    (2, Box::new(f32::from_bits(neg_zero_bits))),
                ],
            },
        };
        let nan_ty = DataComponentType::<f32>::new(1);
        let nz_ty = DataComponentType::<f32>::new(2);
        assert_eq!(holder.get(&nan_ty).unwrap().to_bits(), nan_bits);
        assert_eq!(holder.get(&nz_ty).unwrap().to_bits(), neg_zero_bits);
    }
}
