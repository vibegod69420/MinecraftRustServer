use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#offset(int,int,int)
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(
            self.x.wrapping_add(dx),
            self.y.wrapping_add(dy),
            self.z.wrapping_add(dz),
        )
    }

    // JAVA: net.minecraft.core.BlockPos#immutable()
    pub fn immutable(&self) -> Self {
        *self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    Bedrock,
    EndGateway,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EndGatewayConfiguration {
    pub exit: Option<BlockPos>,
    pub exit_exact: bool,
}

impl EndGatewayConfiguration {
    pub fn get_exit(&self) -> Option<BlockPos> {
        self.exit
    }

    pub fn is_exit_exact(&self) -> bool {
        self.exit_exact
    }
}

pub trait WorldGenLevel {
    fn set_block(&mut self, pos: BlockPos, kind: BlockKind, flags: i32) -> bool;
    fn try_set_end_gateway_exit(&mut self, gateway_pos: BlockPos, exit: BlockPos, exact: bool);
}

// JAVA: net.minecraft.core.BlockPos#betweenClosed(net.minecraft.core.BlockPos,net.minecraft.core.BlockPos)
pub fn between_closed(from: BlockPos, to: BlockPos) -> Vec<BlockPos> {
    let min_x = from.x.min(to.x);
    let max_x = from.x.max(to.x);
    let min_y = from.y.min(to.y);
    let max_y = from.y.max(to.y);
    let min_z = from.z.min(to.z);
    let max_z = from.z.max(to.z);

    let mut out = Vec::new();
    let mut y = min_y;
    while y <= max_y {
        let mut z = min_z;
        while z <= max_z {
            let mut x = min_x;
            while x <= max_x {
                out.push(BlockPos::new(x, y, z));
                if x == max_x {
                    break;
                }
                x = x.wrapping_add(1);
            }
            if z == max_z {
                break;
            }
            z = z.wrapping_add(1);
        }
        if y == max_y {
            break;
        }
        y = y.wrapping_add(1);
    }
    out
}

pub struct EndGatewayFeature;

impl EndGatewayFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.EndGatewayFeature#<init>(com.mojang.serialization.Codec)
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.EndGatewayFeature#place(net.minecraft.world.level.levelgen.feature.FeaturePlaceContext)
    pub fn place<L: WorldGenLevel + ?Sized>(
        &self,
        level: &mut L,
        origin: BlockPos,
        config: &EndGatewayConfiguration,
    ) -> bool {
        let from = origin.offset(-1, -2, -1);
        let to = origin.offset(1, 2, 1);
        for pos in between_closed(from, to) {
            let is_x = pos.x == origin.x;
            let is_y = pos.y == origin.y;
            let is_z = pos.z == origin.z;
            let is_top_or_bottom = (pos.y.wrapping_sub(origin.y)).wrapping_abs() == 2;
            if is_x && is_y && is_z {
                let immutable_pos = pos.immutable();
                level.set_block(immutable_pos, BlockKind::EndGateway, 2);
                if let Some(exit) = config.get_exit() {
                    level.try_set_end_gateway_exit(immutable_pos, exit, config.is_exit_exact());
                }
            } else if is_y {
                level.set_block(pos, BlockKind::Air, 2);
            } else if is_top_or_bottom && is_x && is_z {
                level.set_block(pos, BlockKind::Bedrock, 2);
            } else if (is_x || is_z) && !is_top_or_bottom {
                level.set_block(pos, BlockKind::Bedrock, 2);
            } else {
                level.set_block(pos, BlockKind::Air, 2);
            }
        }

        true
    }
}

impl Default for EndGatewayFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockWorld {
        blocks: HashMap<(i32, i32, i32), BlockKind>,
        gateway_exits: HashMap<(i32, i32, i32), (BlockPos, bool)>,
    }

    impl MockWorld {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
                gateway_exits: HashMap::new(),
            }
        }

        fn kind_at(&self, pos: BlockPos) -> BlockKind {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(BlockKind::Air)
        }
    }

    impl WorldGenLevel for MockWorld {
        fn set_block(&mut self, pos: BlockPos, kind: BlockKind, _flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), kind);
            true
        }

        fn try_set_end_gateway_exit(&mut self, gateway_pos: BlockPos, exit: BlockPos, exact: bool) {
            self.gateway_exits
                .insert((gateway_pos.x, gateway_pos.y, gateway_pos.z), (exit, exact));
        }
    }

    #[test]
    fn places_gateway_structure() {
        let mut world = MockWorld::new();
        let origin = BlockPos::new(10, 70, -5);
        let config = EndGatewayConfiguration {
            exit: None,
            exit_exact: false,
        };
        assert!(EndGatewayFeature::new().place(&mut world, origin, &config));

        assert_eq!(world.kind_at(origin), BlockKind::EndGateway);
        // top and bottom center bedrock
        assert_eq!(world.kind_at(origin.offset(0, -2, 0)), BlockKind::Bedrock);
        assert_eq!(world.kind_at(origin.offset(0, 2, 0)), BlockKind::Bedrock);
        // middle layer cross is air at center y except gateway already placed
        assert_eq!(world.kind_at(origin.offset(1, 0, 0)), BlockKind::Air);
        // y=-1 cross bedrock
        assert_eq!(world.kind_at(origin.offset(1, -1, 0)), BlockKind::Bedrock);
        assert_eq!(world.kind_at(origin.offset(0, -1, 1)), BlockKind::Bedrock);
        // corners on middle layers are air
        assert_eq!(world.kind_at(origin.offset(1, -1, 1)), BlockKind::Air);
        assert_eq!(world.kind_at(origin.offset(-1, 1, -1)), BlockKind::Air);
    }

    #[test]
    fn sets_exit_when_configured() {
        let mut world = MockWorld::new();
        let origin = BlockPos::new(0, 64, 0);
        let exit = BlockPos::new(100, 50, 100);
        let config = EndGatewayConfiguration {
            exit: Some(exit),
            exit_exact: true,
        };
        assert!(EndGatewayFeature::new().place(&mut world, origin, &config));
        assert_eq!(
            world.gateway_exits.get(&(0, 64, 0)).copied(),
            Some((exit, true))
        );
    }

    #[test]
    fn no_exit_when_not_configured() {
        let mut world = MockWorld::new();
        let origin = BlockPos::new(0, 64, 0);
        let config = EndGatewayConfiguration {
            exit: None,
            exit_exact: false,
        };
        assert!(EndGatewayFeature::new().place(&mut world, origin, &config));
        assert!(world.gateway_exits.is_empty());
    }

    #[test]
    fn between_closed_includes_all_positions() {
        let positions = between_closed(BlockPos::new(0, 0, 0), BlockPos::new(1, 1, 1));
        assert_eq!(positions.len(), 8);
        assert_eq!(positions[0], BlockPos::new(0, 0, 0));
        assert!(positions.contains(&BlockPos::new(1, 1, 1)));
    }

    #[test]
    fn structure_has_expected_bedrock_count() {
        let mut world = MockWorld::new();
        let origin = BlockPos::new(0, 0, 0);
        let config = EndGatewayConfiguration {
            exit: None,
            exit_exact: false,
        };
        EndGatewayFeature::new().place(&mut world, origin, &config);
        let bedrock_count = world
            .blocks
            .values()
            .filter(|k| **k == BlockKind::Bedrock)
            .count();
        let gateway_count = world
            .blocks
            .values()
            .filter(|k| **k == BlockKind::EndGateway)
            .count();
        assert_eq!(bedrock_count, 12);
        assert_eq!(gateway_count, 1);
    }
}
