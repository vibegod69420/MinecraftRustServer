use std::marker::PhantomData;

/// Stub for `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// Stub for `net.minecraft.core.BlockPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Stub for `net.minecraft.world.entity.Entity`.
pub trait Entity {}

/// Stub for `net.minecraft.util.debug.DebugSubscription<T>`.
#[derive(Debug)]
pub struct DebugSubscription<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> DebugSubscription<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for DebugSubscription<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Faithful port of `net.minecraft.util.debug.DebugValueAccess.EventVisitor<T>`.
pub trait EventVisitor<T> {
    // JAVA: net.minecraft.util.debug.DebugValueAccess$EventVisitor#accept
    fn accept(&mut self, value: T, a: i32, b: i32);
}

/// Faithful port of `net.minecraft.util.debug.DebugValueAccess`.
///
/// External Minecraft types are represented by local stubs/generics so this
/// module remains self-contained.
pub trait DebugValueAccess {
    type Entity: Entity;

    // JAVA: net.minecraft.util.debug.DebugValueAccess#forEachChunk
    fn for_each_chunk<T>(
        &self,
        subscription: &DebugSubscription<T>,
        visitor: &mut dyn FnMut(ChunkPos, &T),
    );

    // JAVA: net.minecraft.util.debug.DebugValueAccess#getChunkValue
    fn get_chunk_value<T>(
        &self,
        subscription: &DebugSubscription<T>,
        pos: ChunkPos,
    ) -> Option<T>;

    // JAVA: net.minecraft.util.debug.DebugValueAccess#forEachBlock
    fn for_each_block<T>(
        &self,
        subscription: &DebugSubscription<T>,
        visitor: &mut dyn FnMut(BlockPos, &T),
    );

    // JAVA: net.minecraft.util.debug.DebugValueAccess#getBlockValue
    fn get_block_value<T>(
        &self,
        subscription: &DebugSubscription<T>,
        pos: BlockPos,
    ) -> Option<T>;

    // JAVA: net.minecraft.util.debug.DebugValueAccess#forEachEntity
    fn for_each_entity<T>(
        &self,
        subscription: &DebugSubscription<T>,
        visitor: &mut dyn FnMut(&Self::Entity, &T),
    );

    // JAVA: net.minecraft.util.debug.DebugValueAccess#getEntityValue
    fn get_entity_value<T>(
        &self,
        subscription: &DebugSubscription<T>,
        entity: &Self::Entity,
    ) -> Option<T>;

    // JAVA: net.minecraft.util.debug.DebugValueAccess#forEachEvent
    fn for_each_event<T>(
        &self,
        subscription: &DebugSubscription<T>,
        visitor: &mut dyn EventVisitor<T>,
    );
}

#[cfg(test)]
mod tests {
    use super::{
        BlockPos, ChunkPos, DebugSubscription, DebugValueAccess, Entity, EventVisitor,
    };

    #[derive(Debug)]
    struct TestEntity(i32);
    impl Entity for TestEntity {}

    struct EmptyAccess;
    impl DebugValueAccess for EmptyAccess {
        type Entity = TestEntity;

        fn for_each_chunk<T>(
            &self,
            _subscription: &DebugSubscription<T>,
            _visitor: &mut dyn FnMut(ChunkPos, &T),
        ) {
        }

        fn get_chunk_value<T>(
            &self,
            _subscription: &DebugSubscription<T>,
            _pos: ChunkPos,
        ) -> Option<T> {
            None
        }

        fn for_each_block<T>(
            &self,
            _subscription: &DebugSubscription<T>,
            _visitor: &mut dyn FnMut(BlockPos, &T),
        ) {
        }

        fn get_block_value<T>(
            &self,
            _subscription: &DebugSubscription<T>,
            _pos: BlockPos,
        ) -> Option<T> {
            None
        }

        fn for_each_entity<T>(
            &self,
            _subscription: &DebugSubscription<T>,
            _visitor: &mut dyn FnMut(&Self::Entity, &T),
        ) {
        }

        fn get_entity_value<T>(
            &self,
            _subscription: &DebugSubscription<T>,
            _entity: &Self::Entity,
        ) -> Option<T> {
            None
        }

        fn for_each_event<T>(
            &self,
            _subscription: &DebugSubscription<T>,
            _visitor: &mut dyn EventVisitor<T>,
        ) {
        }
    }

    struct RecordingVisitor {
        events: Vec<(i32, i32, i32)>,
    }

    impl EventVisitor<i32> for RecordingVisitor {
        fn accept(&mut self, value: i32, a: i32, b: i32) {
            self.events.push((value, a, b));
        }
    }

    #[test]
    fn empty_access_returns_none() {
        let access = EmptyAccess;
        let sub = DebugSubscription::<i32>::new();
        let entity = TestEntity(7);
        assert_eq!(access.get_chunk_value(&sub, ChunkPos { x: 1, z: -2 }), None);
        assert_eq!(
            access.get_block_value(&sub, BlockPos { x: -1, y: 64, z: 2 }),
            None
        );
        assert_eq!(access.get_entity_value(&sub, &entity), None);
    }

    #[test]
    fn for_each_methods_accept_empty_visitors() {
        let access = EmptyAccess;
        let sub = DebugSubscription::<u8>::default();
        let mut chunk_count = 0;
        let mut block_count = 0;
        let mut entity_count = 0;
        access.for_each_chunk(&sub, &mut |_p, _v| chunk_count += 1);
        access.for_each_block(&sub, &mut |_p, _v| block_count += 1);
        access.for_each_entity(&sub, &mut |_e, _v| entity_count += 1);
        assert_eq!(chunk_count, 0);
        assert_eq!(block_count, 0);
        assert_eq!(entity_count, 0);
    }

    #[test]
    fn event_visitor_trait_is_implementable() {
        let mut visitor = RecordingVisitor { events: Vec::new() };
        visitor.accept(42, -1, i32::MAX);
        visitor.accept(i32::MIN, 0, 1);
        assert_eq!(visitor.events, vec![(42, -1, i32::MAX), (i32::MIN, 0, 1)]);
    }

    #[test]
    fn stub_positions_handle_negative_and_extreme_values() {
        let chunk = ChunkPos { x: i32::MIN, z: i32::MAX };
        let block = BlockPos { x: -1, y: i32::MIN, z: i32::MAX };
        assert_eq!(chunk.x, i32::MIN);
        assert_eq!(chunk.z, i32::MAX);
        assert_eq!(block.x, -1);
        assert_eq!(block.y, i32::MIN);
        assert_eq!(block.z, i32::MAX);
    }
}
