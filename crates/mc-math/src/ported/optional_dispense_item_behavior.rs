//! Faithful port of `net.minecraft.core.dispenser.OptionalDispenseItemBehavior` (1.21.11).
//!
//! This module is self-contained and provides minimal local stubs for the dispenser
//! block-source/level interfaces used by the class.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

pub trait Level {
    fn level_event(&mut self, event_id: i32, pos: BlockPos, data: i32);
}

pub trait BlockSource {
    type L: Level;
    fn level(&mut self) -> &mut Self::L;
    fn pos(&self) -> BlockPos;
}

/// Base stub for `DefaultDispenseItemBehavior` inheritance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DefaultDispenseItemBehavior;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionalDispenseItemBehavior {
    success: bool,
}

impl Default for OptionalDispenseItemBehavior {
    fn default() -> Self {
        Self { success: true }
    }
}

impl OptionalDispenseItemBehavior {
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: net.minecraft.core.dispenser.OptionalDispenseItemBehavior#isSuccess
    pub fn is_success(&self) -> bool {
        self.success
    }

    // JAVA: net.minecraft.core.dispenser.OptionalDispenseItemBehavior#setSuccess
    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }

    // JAVA: net.minecraft.core.dispenser.OptionalDispenseItemBehavior#playSound
    pub fn play_sound<S: BlockSource>(&self, source: &mut S) {
        let event_id = if self.is_success() { 1000 } else { 1001 };
        let pos = source.pos();
        source.level().level_event(event_id, pos, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct MockLevel {
        events: Vec<(i32, BlockPos, i32)>,
    }

    impl Level for MockLevel {
        fn level_event(&mut self, event_id: i32, pos: BlockPos, data: i32) {
            self.events.push((event_id, pos, data));
        }
    }

    struct MockSource {
        level: MockLevel,
        pos: BlockPos,
    }

    impl MockSource {
        fn new(pos: BlockPos) -> Self {
            Self {
                level: MockLevel::default(),
                pos,
            }
        }
    }

    impl BlockSource for MockSource {
        type L = MockLevel;
        fn level(&mut self) -> &mut Self::L {
            &mut self.level
        }
        fn pos(&self) -> BlockPos {
            self.pos
        }
    }

    #[test]
    fn default_success_is_true() {
        let behavior = OptionalDispenseItemBehavior::new();
        assert!(behavior.is_success());
    }

    #[test]
    fn set_success_updates_state() {
        let mut behavior = OptionalDispenseItemBehavior::new();
        behavior.set_success(false);
        assert!(!behavior.is_success());
        behavior.set_success(true);
        assert!(behavior.is_success());
    }

    #[test]
    fn play_sound_success_emits_1000() {
        let behavior = OptionalDispenseItemBehavior::new();
        let mut source = MockSource::new(BlockPos::new(1, 64, -2));
        behavior.play_sound(&mut source);
        assert_eq!(source.level.events, vec![(1000, BlockPos::new(1, 64, -2), 0)]);
    }

    #[test]
    fn play_sound_failure_emits_1001() {
        let mut behavior = OptionalDispenseItemBehavior::new();
        behavior.set_success(false);
        let mut source = MockSource::new(BlockPos::new(-10, 0, 999));
        behavior.play_sound(&mut source);
        assert_eq!(
            source.level.events,
            vec![(1001, BlockPos::new(-10, 0, 999), 0)]
        );
    }

    #[test]
    fn play_sound_uses_current_pos() {
        let behavior = OptionalDispenseItemBehavior::new();
        let mut source = MockSource::new(BlockPos::new(i32::MIN, i32::MAX, -1));
        behavior.play_sound(&mut source);
        assert_eq!(
            source.level.events[0].1,
            BlockPos::new(i32::MIN, i32::MAX, -1)
        );
    }
}
