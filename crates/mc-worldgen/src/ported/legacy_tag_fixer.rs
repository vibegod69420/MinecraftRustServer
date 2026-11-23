/// Minimal placeholder for `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

/// Minimal placeholder for `net.minecraft.nbt.CompoundTag`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundTag {
    pub marker: i32,
}

impl CompoundTag {
    pub fn new(marker: i32) -> Self {
        Self { marker }
    }
}

/// Port of `net.minecraft.world.level.chunk.storage.LegacyTagFixer`.
pub trait LegacyTagFixer {
    // JAVA: net.minecraft.world.level.chunk.storage.LegacyTagFixer#applyFix
    fn apply_fix(&self, tag: CompoundTag) -> CompoundTag;

    // JAVA: net.minecraft.world.level.chunk.storage.LegacyTagFixer#markChunkDone
    fn mark_chunk_done(&self, _pos: ChunkPos) {}

    // JAVA: net.minecraft.world.level.chunk.storage.LegacyTagFixer#targetDataVersion
    fn target_data_version(&self) -> i32 {
        -1
    }
}

/// Identity fixer corresponding to `LegacyTagFixer.EMPTY` supplier result.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyLegacyTagFixer;

impl LegacyTagFixer for EmptyLegacyTagFixer {
    // JAVA: net.minecraft.world.level.chunk.storage.LegacyTagFixer#applyFix
    fn apply_fix(&self, tag: CompoundTag) -> CompoundTag {
        tag
    }
}

/// Returns the identity fixer, analogous to `LegacyTagFixer.EMPTY.get()`.
pub fn empty_legacy_tag_fixer() -> EmptyLegacyTagFixer {
    EmptyLegacyTagFixer
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct IncrementFixer {
        marked: Cell<i32>,
    }

    impl LegacyTagFixer for IncrementFixer {
        fn apply_fix(&self, mut tag: CompoundTag) -> CompoundTag {
            tag.marker = tag.marker.wrapping_add(1);
            tag
        }

        fn mark_chunk_done(&self, pos: ChunkPos) {
            self.marked.set(self.marked.get().wrapping_add(pos.x).wrapping_add(pos.z));
        }

        fn target_data_version(&self) -> i32 {
            42
        }
    }

    #[test]
    fn empty_fixer_returns_tag_unchanged() {
        let fixer = empty_legacy_tag_fixer();
        let tag = CompoundTag::new(123);
        let out = fixer.apply_fix(tag.clone());
        assert_eq!(out, tag);
    }

    #[test]
    fn empty_fixer_default_methods() {
        let fixer = EmptyLegacyTagFixer;
        assert_eq!(fixer.target_data_version(), -1);
        // Default no-op should not panic.
        fixer.mark_chunk_done(ChunkPos::new(i32::MIN, i32::MAX));
    }

    #[test]
    fn custom_fixer_applies_fix_and_tracks_chunks() {
        let fixer = IncrementFixer {
            marked: Cell::new(0),
        };
        let out = fixer.apply_fix(CompoundTag::new(10));
        assert_eq!(out.marker, 11);
        assert_eq!(fixer.target_data_version(), 42);
        fixer.mark_chunk_done(ChunkPos::new(2, -5));
        fixer.mark_chunk_done(ChunkPos::new(3, 1));
        assert_eq!(fixer.marked.get(), 1); // 2 + (-5) + 3 + 1 = 1
    }

    #[test]
    fn apply_fix_handles_integer_edge_values() {
        let fixer = IncrementFixer {
            marked: Cell::new(0),
        };
        let out = fixer.apply_fix(CompoundTag::new(i32::MAX));
        assert_eq!(out.marker, i32::MIN); // wrapping_add(1)
    }

    #[test]
    fn empty_fixer_preserves_negative_marker() {
        let fixer = empty_legacy_tag_fixer();
        let tag = CompoundTag::new(-999);
        assert_eq!(fixer.apply_fix(tag.clone()), tag);
    }
}
