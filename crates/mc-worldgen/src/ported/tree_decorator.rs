use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BooleanProperty {
    pub name: String,
}

impl BooleanProperty {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub block_name: String,
    pub is_air: bool,
    pub bool_properties: BTreeMap<String, bool>,
}

impl BlockState {
    pub fn air() -> Self {
        Self {
            block_name: "minecraft:air".to_string(),
            is_air: true,
            bool_properties: BTreeMap::new(),
        }
    }

    pub fn vine_default() -> Self {
        Self {
            block_name: "minecraft:vine".to_string(),
            is_air: false,
            bool_properties: BTreeMap::new(),
        }
    }

    pub fn set_value(mut self, property: &BooleanProperty, value: bool) -> Self {
        self.bool_properties.insert(property.name.clone(), value);
        self
    }

    pub fn is_air(&self) -> bool {
        self.is_air
    }
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait LevelSimulatedReader {
    fn is_state_at_position(&self, pos: &BlockPos, predicate: &dyn Fn(&BlockState) -> bool) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeDecoratorType(pub &'static str);

/// Faithful port of `net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator`.
/// The Java CODEC field is serialization metadata and is intentionally not represented here.
pub trait TreeDecorator {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator#type
    fn decorator_type(&self) -> TreeDecoratorType;

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator#place
    fn place(&self, context: &mut Context<'_>);
}

/// Faithful port of `TreeDecorator.Context`.
pub struct Context<'a> {
    level: &'a dyn LevelSimulatedReader,
    decoration_setter: Box<dyn FnMut(BlockPos, BlockState) + 'a>,
    random: &'a mut dyn RandomSource,
    logs: Vec<BlockPos>,
    leaves: Vec<BlockPos>,
    roots: Vec<BlockPos>,
}

impl<'a> Context<'a> {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#<init>
    pub fn new(
        level: &'a dyn LevelSimulatedReader,
        decoration_setter: Box<dyn FnMut(BlockPos, BlockState) + 'a>,
        random: &'a mut dyn RandomSource,
        logs: Vec<BlockPos>,
        leaves: Vec<BlockPos>,
        roots: Vec<BlockPos>,
    ) -> Self {
        let mut logs = logs;
        let mut leaves = leaves;
        let mut roots = roots;
        logs.sort_by_key(|p| p.get_y());
        leaves.sort_by_key(|p| p.get_y());
        roots.sort_by_key(|p| p.get_y());
        Self {
            level,
            decoration_setter,
            random,
            logs,
            leaves,
            roots,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#placeVine
    pub fn place_vine(&mut self, pos: BlockPos, property: &BooleanProperty) {
        self.set_block(pos, BlockState::vine_default().set_value(property, true));
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#setBlock
    pub fn set_block(&mut self, pos: BlockPos, state: BlockState) {
        (self.decoration_setter)(pos, state);
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#isAir
    pub fn is_air(&self, pos: &BlockPos) -> bool {
        self.level
            .is_state_at_position(pos, &|s: &BlockState| s.is_air())
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#checkBlock
    pub fn check_block(&self, pos: &BlockPos, predicate: &dyn Fn(&BlockState) -> bool) -> bool {
        self.level.is_state_at_position(pos, predicate)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#level
    pub fn level(&self) -> &dyn LevelSimulatedReader {
        self.level
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#random
    pub fn random(&mut self) -> &mut dyn RandomSource {
        self.random
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#logs
    pub fn logs(&self) -> &Vec<BlockPos> {
        &self.logs
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#leaves
    pub fn leaves(&self) -> &Vec<BlockPos> {
        &self.leaves
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator$Context#roots
    pub fn roots(&self) -> &Vec<BlockPos> {
        &self.roots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    struct MockLevel {
        states: HashMap<(i32, i32, i32), BlockState>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                states: HashMap::new(),
            }
        }

        fn with_state(mut self, pos: BlockPos, state: BlockState) -> Self {
            self.states.insert((pos.x, pos.y, pos.z), state);
            self
        }
    }

    impl LevelSimulatedReader for MockLevel {
        fn is_state_at_position(
            &self,
            pos: &BlockPos,
            predicate: &dyn Fn(&BlockState) -> bool,
        ) -> bool {
            let state = self
                .states
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(BlockState::air);
            predicate(&state)
        }
    }

    struct MockRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        float_idx: usize,
        int_idx: usize,
    }

    impl MockRandom {
        fn new(floats: Vec<f32>, ints: Vec<i32>) -> Self {
            Self {
                floats,
                ints,
                float_idx: 0,
                int_idx: 0,
            }
        }
    }

    impl RandomSource for MockRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.float_idx % self.floats.len().max(1)];
            self.float_idx += 1;
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.ints[self.int_idx % self.ints.len().max(1)];
            self.int_idx += 1;
            ((v % bound) + bound) % bound
        }
    }

    fn make_context<'a>(
        level: &'a dyn LevelSimulatedReader,
        random: &'a mut dyn RandomSource,
        placed: Rc<RefCell<Vec<(BlockPos, BlockState)>>>,
        logs: Vec<BlockPos>,
        leaves: Vec<BlockPos>,
        roots: Vec<BlockPos>,
    ) -> Context<'a> {
        let setter = Box::new(move |p: BlockPos, s: BlockState| {
            placed.borrow_mut().push((p, s));
        });
        Context::new(level, setter, random, logs, leaves, roots)
    }

    #[test]
    fn context_sorts_by_y_ascending_including_negatives() {
        let level = MockLevel::new();
        let mut random = MockRandom::new(vec![0.0], vec![0]);
        let placed = Rc::new(RefCell::new(Vec::new()));
        let ctx = make_context(
            &level,
            &mut random,
            placed,
            vec![
                BlockPos::new(0, 10, 0),
                BlockPos::new(0, -5, 0),
                BlockPos::new(0, -1, 0),
            ],
            vec![BlockPos::new(0, 7, 0), BlockPos::new(0, 2, 0)],
            vec![BlockPos::new(0, -3, 0), BlockPos::new(0, -10, 0)],
        );
        assert_eq!(ctx.logs().iter().map(|p| p.get_y()).collect::<Vec<_>>(), vec![-5, -1, 10]);
        assert_eq!(ctx.leaves().iter().map(|p| p.get_y()).collect::<Vec<_>>(), vec![2, 7]);
        assert_eq!(ctx.roots().iter().map(|p| p.get_y()).collect::<Vec<_>>(), vec![-10, -3]);
    }

    #[test]
    fn place_vine_sets_boolean_property_true() {
        let level = MockLevel::new();
        let mut random = MockRandom::new(vec![0.0], vec![0]);
        let placed = Rc::new(RefCell::new(Vec::new()));
        let mut ctx = make_context(&level, &mut random, placed.clone(), vec![], vec![], vec![]);
        let prop = BooleanProperty::new("up");
        ctx.place_vine(BlockPos::new(1, 2, 3), &prop);
        let placed = placed.borrow();
        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].0, BlockPos::new(1, 2, 3));
        assert_eq!(placed[0].1.block_name, "minecraft:vine");
        assert_eq!(placed[0].1.bool_properties.get("up"), Some(&true));
    }

    #[test]
    fn is_air_and_check_block_delegate_to_level() {
        let stone = BlockState {
            block_name: "minecraft:stone".to_string(),
            is_air: false,
            bool_properties: BTreeMap::new(),
        };
        let level = MockLevel::new().with_state(BlockPos::new(0, 0, 0), stone);
        let mut random = MockRandom::new(vec![0.0], vec![0]);
        let placed = Rc::new(RefCell::new(Vec::new()));
        let ctx = make_context(&level, &mut random, placed, vec![], vec![], vec![]);

        assert!(!ctx.is_air(&BlockPos::new(0, 0, 0)));
        assert!(ctx.is_air(&BlockPos::new(5, 5, 5)));
        assert!(ctx.check_block(&BlockPos::new(0, 0, 0), &|s| s.block_name == "minecraft:stone"));
        assert!(!ctx.check_block(&BlockPos::new(5, 5, 5), &|s| s.block_name == "minecraft:stone"));
    }

    #[test]
    fn set_block_calls_decoration_setter() {
        let level = MockLevel::new();
        let mut random = MockRandom::new(vec![0.0], vec![0]);
        let placed = Rc::new(RefCell::new(Vec::new()));
        let mut ctx = make_context(&level, &mut random, placed.clone(), vec![], vec![], vec![]);
        let state = BlockState {
            block_name: "minecraft:dirt".to_string(),
            is_air: false,
            bool_properties: BTreeMap::new(),
        };
        ctx.set_block(BlockPos::new(-1, -2, -3), state.clone());
        let placed = placed.borrow();
        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].0, BlockPos::new(-1, -2, -3));
        assert_eq!(placed[0].1, state);
    }

    #[test]
    fn context_preserves_input_order_for_equal_y_stable_sort() {
        let level = MockLevel::new();
        let mut random = MockRandom::new(vec![0.0], vec![0]);
        let placed = Rc::new(RefCell::new(Vec::new()));
        let logs = vec![
            BlockPos::new(1, 5, 0),
            BlockPos::new(2, 5, 0),
            BlockPos::new(3, 5, 0),
        ];
        let ctx = make_context(&level, &mut random, placed, logs, vec![], vec![]);
        let xs: Vec<i32> = ctx.logs().iter().map(|p| p.get_x()).collect();
        assert_eq!(xs, vec![1, 2, 3]);
    }
}
