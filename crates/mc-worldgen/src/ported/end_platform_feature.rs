use std::collections::HashMap;

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn from_pos(pos: BlockPos) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            z: pos.z,
        }
    }

    pub fn set(&mut self, pos: BlockPos) -> &mut Self {
        self.x = pos.x;
        self.y = pos.y;
        self.z = pos.z;
        self
    }

    pub fn move_offset(&mut self, dx: i32, dy: i32, dz: i32) -> &mut Self {
        self.x = self.x.wrapping_add(dx);
        self.y = self.y.wrapping_add(dy);
        self.z = self.z.wrapping_add(dz);
        self
    }

    pub fn immutable(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Obsidian,
    Other(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
}

impl BlockState {
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    pub fn is(&self, block: Block) -> bool {
        self.block == block
    }
}

pub trait ServerLevelAccessor {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn destroy_block(&mut self, pos: BlockPos, drop: bool);
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32);
}

pub struct NoneFeatureConfiguration;

pub struct FeaturePlaceContext<'a, L: ?Sized> {
    level: &'a mut L,
    origin: BlockPos,
    _config: NoneFeatureConfiguration,
}

impl<'a, L: ?Sized> FeaturePlaceContext<'a, L> {
    pub fn new(level: &'a mut L, origin: BlockPos) -> Self {
        Self {
            level,
            origin,
            _config: NoneFeatureConfiguration,
        }
    }

    pub fn level(&mut self) -> &mut L {
        self.level
    }

    pub fn origin(&self) -> BlockPos {
        self.origin
    }
}

pub struct EndPlatformFeature;

impl EndPlatformFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.EndPlatformFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.EndPlatformFeature#place
    pub fn place<L: ServerLevelAccessor + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L>,
    ) -> bool {
        let origin = ctx.origin();
        create_end_platform(ctx.level(), origin, false);
        true
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.EndPlatformFeature#createEndPlatform
pub fn create_end_platform<L: ServerLevelAccessor + ?Sized>(
    level: &mut L,
    origin: BlockPos,
    destroy: bool,
) {
    let mut mutable_pos = MutableBlockPos::from_pos(origin);

    let mut z_off: i32 = -2;
    while z_off <= 2 {
        let mut x_off: i32 = -2;
        while x_off <= 2 {
            let mut y_off: i32 = -1;
            while y_off < 3 {
                let target_pos = mutable_pos
                    .set(origin)
                    .move_offset(x_off, y_off, z_off)
                    .immutable();
                let target_block = if y_off == -1 {
                    Block::Obsidian
                } else {
                    Block::Air
                };
                if !level.get_block_state(target_pos).is(target_block) {
                    if destroy {
                        level.destroy_block(target_pos, true);
                    }
                    level.set_block(target_pos, BlockState::new(target_block), 3);
                }
                y_off = y_off.wrapping_add(1);
            }
            x_off = x_off.wrapping_add(1);
        }
        z_off = z_off.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
        destroyed: Vec<(BlockPos, bool)>,
        set_calls: Vec<(BlockPos, BlockState, i32)>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
                destroyed: Vec::new(),
                set_calls: Vec::new(),
            }
        }
    }

    impl ServerLevelAccessor for MockLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            *self
                .blocks
                .get(&(pos.x, pos.y, pos.z))
                .unwrap_or(&BlockState::new(Block::Air))
        }

        fn destroy_block(&mut self, pos: BlockPos, drop: bool) {
            self.destroyed.push((pos, drop));
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) {
            self.set_calls.push((pos, state, flags));
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }
    }

    #[test]
    fn creates_platform_on_empty_level() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(0, 100, 0);
        create_end_platform(&mut level, origin, false);

        let obsidian_count = level
            .set_calls
            .iter()
            .filter(|(_, s, _)| s.block == Block::Obsidian)
            .count();
        let air_count = level
            .set_calls
            .iter()
            .filter(|(_, s, _)| s.block == Block::Air)
            .count();
        assert_eq!(obsidian_count, 25);
        assert_eq!(air_count, 0);
        assert!(level.destroyed.is_empty());

        for dx in -2..=2 {
            for dz in -2..=2 {
                assert_eq!(
                    level.get_block_state(BlockPos::new(dx, 99, dz)).block,
                    Block::Obsidian
                );
                assert_eq!(
                    level.get_block_state(BlockPos::new(dx, 100, dz)).block,
                    Block::Air
                );
            }
        }
    }

    #[test]
    fn skips_when_block_already_matches() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(0, 64, 0);
        for dx in -2..=2 {
            for dz in -2..=2 {
                level
                    .blocks
                    .insert((dx, 63, dz), BlockState::new(Block::Obsidian));
            }
        }
        create_end_platform(&mut level, origin, false);
        assert!(level.set_calls.is_empty());
        assert!(level.destroyed.is_empty());
    }

    #[test]
    fn destroy_flag_destroys_replaced_blocks() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(0, 64, 0);
        level
            .blocks
            .insert((1, 63, 1), BlockState::new(Block::Other(1)));
        level
            .blocks
            .insert((0, 64, 0), BlockState::new(Block::Other(2)));

        create_end_platform(&mut level, origin, true);

        assert!(level
            .destroyed
            .iter()
            .any(|(p, drop)| *p == BlockPos::new(1, 63, 1) && *drop));
        assert!(level
            .destroyed
            .iter()
            .any(|(p, drop)| *p == BlockPos::new(0, 64, 0) && *drop));
    }

    #[test]
    fn negative_origin_coordinates() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(-10, -5, -20);
        create_end_platform(&mut level, origin, false);

        assert_eq!(
            level.get_block_state(BlockPos::new(-12, -6, -22)).block,
            Block::Obsidian
        );
        assert_eq!(
            level.get_block_state(BlockPos::new(-8, -4, -18)).block,
            Block::Air
        );
    }

    #[test]
    fn place_returns_true_and_creates_platform() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(0, 50, 0);
        let mut ctx = FeaturePlaceContext::new(&mut level, origin);
        let feature = EndPlatformFeature::new();
        assert!(feature.place(&mut ctx));
        assert_eq!(
            ctx.level()
                .get_block_state(BlockPos::new(0, 49, 0))
                .block,
            Block::Obsidian
        );
    }

    #[test]
    fn wrapping_coordinates_near_i32_edge() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(i32::MAX, 0, i32::MIN);
        create_end_platform(&mut level, origin, false);
        let wrapped_x = i32::MAX.wrapping_add(2);
        let wrapped_z = i32::MIN.wrapping_add(-2);
        assert_eq!(
            level
                .get_block_state(BlockPos::new(wrapped_x, -1, wrapped_z))
                .block,
            Block::Obsidian
        );
    }
}
