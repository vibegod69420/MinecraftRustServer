use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

pub trait RandomSource {
    fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32;
}

pub trait IntProvider {
    fn get_min_value(&self) -> i32;
    fn get_max_value(&self) -> i32;
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerProperty {
    name: String,
    possible_values: Vec<i32>,
}

impl IntegerProperty {
    pub fn new(name: impl Into<String>, possible_values: Vec<i32>) -> Self {
        Self {
            name: name.into(),
            possible_values,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_possible_values(&self) -> &[i32] {
        &self.possible_values
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Property {
    Integer(IntegerProperty),
    Other { name: String },
}

impl Property {
    pub fn get_name(&self) -> &str {
        match self {
            Property::Integer(p) => p.get_name(),
            Property::Other { name } => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    properties: Vec<(Property, i32)>,
}

impl BlockState {
    pub fn new(properties: Vec<(Property, i32)>) -> Self {
        Self { properties }
    }

    pub fn has_property(&self, property: &IntegerProperty) -> bool {
        self.properties.iter().any(|(p, _)| match p {
            Property::Integer(ip) => ip == property,
            _ => false,
        })
    }

    pub fn get_properties(&self) -> impl Iterator<Item = &Property> {
        self.properties.iter().map(|(p, _)| p)
    }

    pub fn get_value(&self, property: &IntegerProperty) -> Option<i32> {
        self.properties.iter().find_map(|(p, v)| match p {
            Property::Integer(ip) if ip == property => Some(*v),
            _ => None,
        })
    }

    pub fn set_value(&self, property: &IntegerProperty, value: i32) -> Self {
        let mut result = self.clone();
        for (p, v) in &mut result.properties {
            if let Property::Integer(ip) = p {
                if ip == property {
                    *v = value;
                    return result;
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStateProviderType {
    RandomizedIntStateProvider,
}

pub trait BlockStateProvider {
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState;
    fn get_type(&self) -> BlockStateProviderType;
}

pub struct RandomizedIntStateProvider<S, V> {
    source: S,
    property_name: String,
    property: RefCell<Option<IntegerProperty>>,
    values: V,
}

impl<S: BlockStateProvider, V: IntProvider> RandomizedIntStateProvider<S, V> {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider#<init>(BlockStateProvider, IntegerProperty, IntProvider)
    pub fn new(source: S, property: IntegerProperty, values: V) -> Self {
        let possible_values = property.get_possible_values();
        let min = values.get_min_value();
        let max = values.get_max_value();
        let mut i = min;
        while i <= max {
            if !possible_values.contains(&i) {
                panic!(
                    "Property value out of range: {}: {}",
                    property.get_name(),
                    i
                );
            }
            if i == max {
                break;
            }
            i = i.wrapping_add(1);
        }

        let property_name = property.get_name().to_string();
        Self {
            source,
            property_name,
            property: RefCell::new(Some(property)),
            values,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider#<init>(BlockStateProvider, String, IntProvider)
    pub fn new_with_name(source: S, property_name: impl Into<String>, values: V) -> Self {
        Self {
            source,
            property_name: property_name.into(),
            property: RefCell::new(None),
            values,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider#type
    pub fn get_type(&self) -> BlockStateProviderType {
        BlockStateProviderType::RandomizedIntStateProvider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider#getState
    pub fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState {
        let state = self.source.get_state(random, pos);

        let needs_resolve = {
            let cache = self.property.borrow();
            match cache.as_ref() {
                None => true,
                Some(p) => !state.has_property(p),
            }
        };

        if needs_resolve {
            match find_property(&state, &self.property_name) {
                None => return state,
                Some(p) => {
                    *self.property.borrow_mut() = Some(p);
                }
            }
        }

        let value = self.values.sample(random);
        let cache = self.property.borrow();
        let prop = cache
            .as_ref()
            .expect("property should be resolved before setValue");
        state.set_value(prop, value)
    }
}

impl<S: BlockStateProvider, V: IntProvider> BlockStateProvider for RandomizedIntStateProvider<S, V> {
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState {
        RandomizedIntStateProvider::get_state(self, random, pos)
    }

    fn get_type(&self) -> BlockStateProviderType {
        RandomizedIntStateProvider::get_type(self)
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider#findProperty
fn find_property(state: &BlockState, name: &str) -> Option<IntegerProperty> {
    state
        .get_properties()
        .filter(|p| p.get_name() == name)
        .filter_map(|p| match p {
            Property::Integer(ip) => Some(ip.clone()),
            _ => None,
        })
        .next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct FixedRandom(i32);

    impl RandomSource for FixedRandom {
        fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32 {
            self.0.clamp(min, max)
        }
    }

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32 {
            let v = if self.idx < self.values.len() {
                let v = self.values[self.idx];
                self.idx += 1;
                v
            } else {
                min
            };
            v.clamp(min, max)
        }
    }

    #[derive(Clone)]
    struct UniformIntProvider {
        min: i32,
        max: i32,
    }

    impl UniformIntProvider {
        fn new(min: i32, max: i32) -> Self {
            Self { min, max }
        }
    }

    impl IntProvider for UniformIntProvider {
        fn get_min_value(&self) -> i32 {
            self.min
        }

        fn get_max_value(&self) -> i32 {
            self.max
        }

        fn sample(&self, random: &mut dyn RandomSource) -> i32 {
            random.next_int_between_inclusive(self.min, self.max)
        }
    }

    struct FixedStateProvider {
        state: BlockState,
    }

    impl FixedStateProvider {
        fn new(state: BlockState) -> Self {
            Self { state }
        }
    }

    impl BlockStateProvider for FixedStateProvider {
        fn get_state(&self, _random: &mut dyn RandomSource, _pos: BlockPos) -> BlockState {
            self.state.clone()
        }

        fn get_type(&self) -> BlockStateProviderType {
            BlockStateProviderType::RandomizedIntStateProvider
        }
    }

    fn age_state(value: i32) -> (IntegerProperty, BlockState) {
        let prop = IntegerProperty::new("age", vec![0, 1, 2, 3]);
        let state = BlockState::new(vec![(Property::Integer(prop.clone()), value)]);
        (prop, state)
    }

    #[test]
    fn get_state_sets_sampled_integer_value() {
        let (prop, state) = age_state(0);
        let source = FixedStateProvider::new(state);
        let values = UniformIntProvider::new(0, 3);
        let provider = RandomizedIntStateProvider::new(source, prop.clone(), values);

        let mut random = FixedRandom(2);
        let result = provider.get_state(&mut random, BlockPos::new(1, -2, 3));
        assert_eq!(result.get_value(&prop), Some(2));
    }

    #[test]
    #[should_panic(expected = "Property value out of range: age: 4")]
    fn constructor_rejects_out_of_range_values() {
        let (prop, state) = age_state(0);
        let source = FixedStateProvider::new(state);
        let values = UniformIntProvider::new(0, 5);
        let _ = RandomizedIntStateProvider::new(source, prop, values);
    }

    #[test]
    fn missing_property_returns_original_state_without_sampling() {
        let prop = IntegerProperty::new("age", vec![0, 1, 2, 3]);
        let other = IntegerProperty::new("level", vec![0, 1]);
        let state = BlockState::new(vec![(Property::Integer(other), 1)]);
        let source = FixedStateProvider::new(state.clone());
        let values = UniformIntProvider::new(0, 3);
        let provider = RandomizedIntStateProvider::new_with_name(source, "age", values);

        let mut random = SeqRandom::new(vec![2]);
        let result = provider.get_state(&mut random, BlockPos::new(0, 0, 0));
        assert_eq!(result, state);
        assert_eq!(random.idx, 0, "random should not be consumed when property is missing");
        // keep prop used to avoid warning in some contexts and document intent
        assert_eq!(prop.get_name(), "age");
    }

    #[test]
    fn lazy_resolution_and_cache_across_calls() {
        let (prop, state) = age_state(0);
        let source = FixedStateProvider::new(state);
        let values = UniformIntProvider::new(0, 3);
        let provider = RandomizedIntStateProvider::new_with_name(source, "age", values);

        let mut random = SeqRandom::new(vec![1, 3]);
        let r1 = provider.get_state(&mut random, BlockPos::new(0, 0, 0));
        let r2 = provider.get_state(&mut random, BlockPos::new(0, 0, 0));
        assert_eq!(r1.get_value(&prop), Some(1));
        assert_eq!(r2.get_value(&prop), Some(3));
    }

    #[test]
    fn negative_values_and_re_resolution_when_property_absent() {
        let prop = IntegerProperty::new("offset", vec![-2, -1, 0, 1]);
        let state_with = BlockState::new(vec![(Property::Integer(prop.clone()), 0)]);
        let state_without = BlockState::new(vec![(Property::Other {
            name: "offset".to_string(),
        }, 7)]);

        struct SwitchingSource {
            states: Vec<BlockState>,
            idx: Cell<usize>,
        }

        impl BlockStateProvider for SwitchingSource {
            fn get_state(&self, _random: &mut dyn RandomSource, _pos: BlockPos) -> BlockState {
                let i = self.idx.get();
                self.idx.set(i + 1);
                self.states[i].clone()
            }

            fn get_type(&self) -> BlockStateProviderType {
                BlockStateProviderType::RandomizedIntStateProvider
            }
        }

        let source = SwitchingSource {
            states: vec![state_with.clone(), state_without.clone()],
            idx: Cell::new(0),
        };
        let values = UniformIntProvider::new(-2, 1);
        let provider = RandomizedIntStateProvider::new_with_name(source, "offset", values);

        let mut random = SeqRandom::new(vec![-1, 0]);
        let r1 = provider.get_state(&mut random, BlockPos::new(-10, -20, -30));
        assert_eq!(r1.get_value(&prop), Some(-1));

        let r2 = provider.get_state(&mut random, BlockPos::new(0, 0, 0));
        assert_eq!(r2, state_without);
        assert_eq!(
            random.idx, 1,
            "random should not be consumed when resolved property is not an IntegerProperty"
        );
    }

    #[test]
    fn constructor_validation_handles_i32_boundaries() {
        let prop_min = IntegerProperty::new("v", vec![i32::MIN, i32::MIN + 1]);
        let state_min = BlockState::new(vec![(Property::Integer(prop_min.clone()), i32::MIN)]);
        let provider_min = RandomizedIntStateProvider::new(
            FixedStateProvider::new(state_min),
            prop_min.clone(),
            UniformIntProvider::new(i32::MIN, i32::MIN + 1),
        );
        let mut random = FixedRandom(i32::MIN);
        let result_min = provider_min.get_state(&mut random, BlockPos::new(0, 0, 0));
        assert_eq!(result_min.get_value(&prop_min), Some(i32::MIN));

        let prop_max = IntegerProperty::new("v", vec![i32::MAX - 1, i32::MAX]);
        let state_max = BlockState::new(vec![(Property::Integer(prop_max.clone()), i32::MAX)]);
        let provider_max = RandomizedIntStateProvider::new(
            FixedStateProvider::new(state_max),
            prop_max.clone(),
            UniformIntProvider::new(i32::MAX - 1, i32::MAX),
        );
        let mut random = FixedRandom(i32::MAX);
        let result_max = provider_max.get_state(&mut random, BlockPos::new(0, 0, 0));
        assert_eq!(result_max.get_value(&prop_max), Some(i32::MAX));
    }
}
