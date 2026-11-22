#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtCompound(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    pub state: BlockState,
    pub nbt: Option<NbtCompound>,
}

pub trait LevelReader {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

#[derive(Debug, Clone, Default)]
pub struct StructurePlaceSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureProcessorType {
    Rule,
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

#[derive(Debug, Clone)]
pub struct SeededRandom {
    seed: i64,
}

impl SeededRandom {
    // JAVA: net.minecraft.util.RandomSource#create(long)
    pub fn create(seed: i64) -> Self {
        // Deterministic LCG stand-in for self-contained compilation.
        // Keeps seeding/order behavior explicit for rule processing tests.
        let s = (seed ^ 0x5DEECE66Di64) & ((1i64 << 48) - 1);
        Self { seed: s }
    }

    fn next(&mut self, bits: i32) -> i32 {
        self.seed = (self
            .seed
            .wrapping_mul(0x5DEECE66Di64)
            .wrapping_add(0xBi64))
            & ((1i64 << 48) - 1);
        ((self.seed as u64) >> (48 - bits)) as i32
    }
}

impl RandomSource for SeededRandom {
    fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / ((1u32 << 24) as f32)
    }
}

// JAVA: net.minecraft.util.Mth#getSeed(int,int,int)
pub fn get_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut l = (x.wrapping_mul(3129871) as i64)
        ^ (z as i64).wrapping_mul(116129781i64)
        ^ (y as i64);
    l = l
        .wrapping_mul(l)
        .wrapping_mul(42317861i64)
        .wrapping_add(l.wrapping_mul(11i64));
    l >> 16
}

// JAVA: net.minecraft.util.Mth#getSeed(BlockPos)
pub fn get_seed_from_pos(pos: &BlockPos) -> i64 {
    get_seed(pos.x, pos.y, pos.z)
}

pub trait ProcessorRule {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.ProcessorRule#test
    fn test(
        &self,
        input_state: &BlockState,
        existing_state: &BlockState,
        rel_pos: &BlockPos,
        world_pos: &BlockPos,
        pivot: &BlockPos,
        random: &mut dyn RandomSource,
    ) -> bool;

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.ProcessorRule#getOutputState
    fn get_output_state(&self) -> BlockState;

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.ProcessorRule#getOutputTag
    fn get_output_tag(
        &self,
        random: &mut dyn RandomSource,
        existing_nbt: Option<&NbtCompound>,
    ) -> Option<NbtCompound>;
}

pub struct RuleProcessor {
    pub rules: Vec<Box<dyn ProcessorRule>>,
}

impl RuleProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RuleProcessor#<init>
    pub fn new(rules: Vec<Box<dyn ProcessorRule>>) -> Self {
        Self { rules }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RuleProcessor#processBlock
    pub fn process_block(
        &self,
        level: &dyn LevelReader,
        _pos1: &BlockPos,
        pos2: &BlockPos,
        info3: &StructureBlockInfo,
        info4: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let mut random = SeededRandom::create(get_seed_from_pos(&info4.pos));
        let existing_state = level.get_block_state(&info4.pos);

        for rule in &self.rules {
            if rule.test(
                &info4.state,
                &existing_state,
                &info3.pos,
                &info4.pos,
                pos2,
                &mut random,
            ) {
                return Some(StructureBlockInfo {
                    pos: info4.pos,
                    state: rule.get_output_state(),
                    nbt: rule.get_output_tag(&mut random, info4.nbt.as_ref()),
                });
            }
        }

        Some(info4.clone())
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RuleProcessor#getType
    pub fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::Rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        state: BlockState,
    }

    impl LevelReader for MockLevel {
        fn get_block_state(&self, _pos: &BlockPos) -> BlockState {
            self.state.clone()
        }
    }

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    fn bs(id: u32) -> BlockState {
        BlockState { id }
    }

    fn info(p: BlockPos, id: u32, nbt: Option<NbtCompound>) -> StructureBlockInfo {
        StructureBlockInfo {
            pos: p,
            state: bs(id),
            nbt,
        }
    }

    struct FixedRule {
        matches: bool,
        output_id: u32,
    }

    impl ProcessorRule for FixedRule {
        fn test(
            &self,
            _input_state: &BlockState,
            _existing_state: &BlockState,
            _rel_pos: &BlockPos,
            _world_pos: &BlockPos,
            _pivot: &BlockPos,
            _random: &mut dyn RandomSource,
        ) -> bool {
            self.matches
        }

        fn get_output_state(&self) -> BlockState {
            bs(self.output_id)
        }

        fn get_output_tag(
            &self,
            _random: &mut dyn RandomSource,
            existing_nbt: Option<&NbtCompound>,
        ) -> Option<NbtCompound> {
            existing_nbt.cloned()
        }
    }

    struct RandomCaptureRule {
        expected_first_float_bits: u32,
    }

    impl ProcessorRule for RandomCaptureRule {
        fn test(
            &self,
            _input_state: &BlockState,
            _existing_state: &BlockState,
            _rel_pos: &BlockPos,
            _world_pos: &BlockPos,
            _pivot: &BlockPos,
            random: &mut dyn RandomSource,
        ) -> bool {
            let f = random.next_float();
            f.to_bits() == self.expected_first_float_bits
        }

        fn get_output_state(&self) -> BlockState {
            bs(77)
        }

        fn get_output_tag(
            &self,
            _random: &mut dyn RandomSource,
            _existing_nbt: Option<&NbtCompound>,
        ) -> Option<NbtCompound> {
            None
        }
    }

    struct ArgCheckRule;

    impl ProcessorRule for ArgCheckRule {
        fn test(
            &self,
            input_state: &BlockState,
            existing_state: &BlockState,
            rel_pos: &BlockPos,
            world_pos: &BlockPos,
            pivot: &BlockPos,
            _random: &mut dyn RandomSource,
        ) -> bool {
            assert_eq!(input_state.id, 10);
            assert_eq!(existing_state.id, 20);
            assert_eq!(*rel_pos, pos(1, 2, 3));
            assert_eq!(*world_pos, pos(4, 5, 6));
            assert_eq!(*pivot, pos(7, 8, 9));
            true
        }

        fn get_output_state(&self) -> BlockState {
            bs(30)
        }

        fn get_output_tag(
            &self,
            _random: &mut dyn RandomSource,
            existing_nbt: Option<&NbtCompound>,
        ) -> Option<NbtCompound> {
            existing_nbt.cloned()
        }
    }

    #[test]
    fn no_rules_returns_original() {
        let proc = RuleProcessor::new(vec![]);
        let level = MockLevel { state: bs(1) };
        let i4 = info(pos(1, 2, 3), 10, None);
        let i3 = info(pos(0, 0, 0), 0, None);
        let r = proc.process_block(
            &level,
            &pos(0, 0, 0),
            &pos(0, 0, 0),
            &i3,
            &i4,
            &StructurePlaceSettings,
        );
        assert_eq!(r, Some(i4));
    }

    #[test]
    fn matching_rule_transforms_and_preserves_pos() {
        let proc = RuleProcessor::new(vec![Box::new(FixedRule {
            matches: true,
            output_id: 99,
        })]);
        let level = MockLevel { state: bs(1) };
        let i4 = info(pos(5, 6, 7), 10, Some(NbtCompound(vec![9, 8])));
        let i3 = info(pos(0, 0, 0), 0, None);
        let r = proc
            .process_block(
                &level,
                &pos(0, 0, 0),
                &pos(0, 0, 0),
                &i3,
                &i4,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(r.pos, pos(5, 6, 7));
        assert_eq!(r.state, bs(99));
        assert_eq!(r.nbt, Some(NbtCompound(vec![9, 8])));
    }

    #[test]
    fn first_matching_rule_wins() {
        let proc = RuleProcessor::new(vec![
            Box::new(FixedRule {
                matches: false,
                output_id: 1,
            }),
            Box::new(FixedRule {
                matches: true,
                output_id: 2,
            }),
            Box::new(FixedRule {
                matches: true,
                output_id: 3,
            }),
        ]);
        let level = MockLevel { state: bs(0) };
        let i4 = info(pos(1, 1, 1), 10, None);
        let i3 = info(pos(0, 0, 0), 0, None);
        let r = proc
            .process_block(
                &level,
                &pos(0, 0, 0),
                &pos(0, 0, 0),
                &i3,
                &i4,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(r.state, bs(2));
    }

    #[test]
    fn random_seeded_from_info4_position_and_args_passed() {
        let p = pos(10, -20, 30);
        let seed = get_seed_from_pos(&p);
        let mut rng = SeededRandom::create(seed);
        let expected = rng.next_float().to_bits();

        let proc = RuleProcessor::new(vec![Box::new(RandomCaptureRule {
            expected_first_float_bits: expected,
        })]);
        let level = MockLevel { state: bs(0) };
        let i4 = info(p, 5, None);
        let i3 = info(pos(0, 0, 0), 0, None);
        let r = proc
            .process_block(
                &level,
                &pos(0, 0, 0),
                &pos(0, 0, 0),
                &i3,
                &i4,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(r.state, bs(77));

        let proc2 = RuleProcessor::new(vec![Box::new(ArgCheckRule)]);
        let level2 = MockLevel { state: bs(20) };
        let i3b = info(pos(1, 2, 3), 0, None);
        let i4b = info(pos(4, 5, 6), 10, None);
        let r2 = proc2
            .process_block(
                &level2,
                &pos(0, 0, 0),
                &pos(7, 8, 9),
                &i3b,
                &i4b,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(r2.state, bs(30));
    }

    #[test]
    fn get_seed_negative_and_overflow_deterministic() {
        let s1 = get_seed(i32::MIN, -1, i32::MAX);
        let s2 = get_seed(i32::MIN, -1, i32::MAX);
        assert_eq!(s1, s2);
        let s3 = get_seed(i32::MAX, i32::MIN, -12345);
        let s4 = get_seed(i32::MAX, i32::MIN, -12345);
        assert_eq!(s3, s4);
        assert_ne!(s1, s3);
    }

    #[test]
    fn get_type_is_rule() {
        let proc = RuleProcessor::new(vec![]);
        assert_eq!(proc.get_type(), StructureProcessorType::Rule);
    }
}
