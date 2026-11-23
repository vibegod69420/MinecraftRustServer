use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: u32,
}

pub struct SectionPos;

impl SectionPos {
    // JAVA: net.minecraft.core.SectionPos#blockToSectionCoord(int)
    pub fn block_to_section_coord(pos: i32) -> i32 {
        pos >> 4
    }

    // JAVA: net.minecraft.core.SectionPos#sectionRelative(int)
    pub fn section_relative(pos: i32) -> i32 {
        pos & 15
    }

    // JAVA: net.minecraft.core.SectionPos#asLong(int, int, int)
    pub fn as_long_coords(x: i32, y: i32, z: i32) -> i64 {
        (((x as i64) & 4_194_303_i64) << 42)
            | ((y as i64) & 1_048_575_i64)
            | (((z as i64) & 4_194_303_i64) << 20)
    }

    // JAVA: net.minecraft.core.SectionPos#asLong(BlockPos)
    pub fn as_long(pos: &BlockPos) -> i64 {
        Self::as_long_coords(
            Self::block_to_section_coord(pos.x),
            Self::block_to_section_coord(pos.y),
            Self::block_to_section_coord(pos.z),
        )
    }
}

#[derive(Debug)]
struct LevelChunkSectionInner {
    acquire_count: i32,
    states: Vec<BlockState>,
}

impl Default for LevelChunkSectionInner {
    fn default() -> Self {
        Self {
            acquire_count: 0,
            states: vec![BlockState { id: 0 }; 4096],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LevelChunkSection {
    inner: Rc<RefCell<LevelChunkSectionInner>>,
}

impl LevelChunkSection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.acquire_count = inner.acquire_count.wrapping_add(1);
    }

    pub fn release(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.acquire_count = inner.acquire_count.wrapping_sub(1);
    }

    pub fn acquire_count(&self) -> i32 {
        self.inner.borrow().acquire_count
    }

    pub fn get_block_state(&self, x: i32, y: i32, z: i32) -> BlockState {
        let idx = (((y & 15) as usize) * 16 + ((z & 15) as usize)) * 16 + ((x & 15) as usize);
        self.inner.borrow().states[idx].clone()
    }

    pub fn set_block_state(&self, x: i32, y: i32, z: i32, state: BlockState) {
        let idx = (((y & 15) as usize) * 16 + ((z & 15) as usize)) * 16 + ((x & 15) as usize);
        self.inner.borrow_mut().states[idx] = state;
    }
}

#[derive(Debug, Clone)]
pub struct ChunkAccess {
    sections: Vec<LevelChunkSection>,
}

impl ChunkAccess {
    pub fn new(section_count: usize) -> Self {
        Self {
            sections: (0..section_count).map(|_| LevelChunkSection::new()).collect(),
        }
    }

    pub fn get_section(&self, index: i32) -> LevelChunkSection {
        self.sections[index as usize].clone()
    }
}

pub trait LevelAccessor {
    fn get_section_index(&self, y: i32) -> i32;
    fn get_sections_count(&self) -> i32;
    fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> ChunkAccess;
}

pub struct BulkSectionAccess<'a> {
    level: &'a dyn LevelAccessor,
    acquired_sections: HashMap<i64, LevelChunkSection>,
    last_section: Option<LevelChunkSection>,
    last_section_key: i64,
}

impl<'a> BulkSectionAccess<'a> {
    // JAVA: net.minecraft.world.level.chunk.BulkSectionAccess#<init>(LevelAccessor)
    pub fn new(level: &'a dyn LevelAccessor) -> Self {
        Self {
            level,
            acquired_sections: HashMap::new(),
            last_section: None,
            last_section_key: 0,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.BulkSectionAccess#getSection(BlockPos)
    pub fn get_section(&mut self, pos: &BlockPos) -> Option<LevelChunkSection> {
        let section_index = self.level.get_section_index(pos.get_y());
        if section_index >= 0 && section_index < self.level.get_sections_count() {
            let key = SectionPos::as_long(pos);
            let needs_lookup = match &self.last_section {
                None => true,
                Some(_) => self.last_section_key != key,
            };
            if needs_lookup {
                let level = self.level;
                let acquired = &mut self.acquired_sections;
                let section = acquired
                    .entry(key)
                    .or_insert_with(|| {
                        let chunk = level.get_chunk(
                            SectionPos::block_to_section_coord(pos.get_x()),
                            SectionPos::block_to_section_coord(pos.get_z()),
                        );
                        let s = chunk.get_section(section_index);
                        s.acquire();
                        s
                    })
                    .clone();
                self.last_section = Some(section);
                self.last_section_key = key;
            }
            self.last_section.clone()
        } else {
            None
        }
    }

    // JAVA: net.minecraft.world.level.chunk.BulkSectionAccess#getBlockState(BlockPos)
    pub fn get_block_state(&mut self, pos: &BlockPos) -> BlockState {
        let section = self.get_section(pos);
        match section {
            None => BlockState { id: 0 }, // AIR
            Some(s) => {
                let x = SectionPos::section_relative(pos.get_x());
                let y = SectionPos::section_relative(pos.get_y());
                let z = SectionPos::section_relative(pos.get_z());
                s.get_block_state(x, y, z)
            }
        }
    }

    // JAVA: net.minecraft.world.level.chunk.BulkSectionAccess#close()
    pub fn close(&mut self) {
        for section in self.acquired_sections.values() {
            section.release();
        }
        self.acquired_sections.clear();
        self.last_section = None;
    }
}

impl Drop for BulkSectionAccess<'_> {
    fn drop(&mut self) {
        if !self.acquired_sections.is_empty() {
            self.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        sections_count: i32,
        min_section: i32,
        chunks: HashMap<(i32, i32), ChunkAccess>,
    }

    impl MockLevel {
        fn new(min_y: i32, height: i32) -> Self {
            Self {
                sections_count: height / 16,
                min_section: min_y / 16,
                chunks: HashMap::new(),
            }
        }
    }

    impl LevelAccessor for MockLevel {
        fn get_section_index(&self, y: i32) -> i32 {
            (y >> 4) - self.min_section
        }

        fn get_sections_count(&self) -> i32 {
            self.sections_count
        }

        fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> ChunkAccess {
            self.chunks
                .get(&(chunk_x, chunk_z))
                .cloned()
                .unwrap_or_else(|| ChunkAccess::new(self.sections_count as usize))
        }
    }

    #[test]
    fn test_out_of_range_returns_air() {
        let level = MockLevel::new(-64, 384);
        let mut access = BulkSectionAccess::new(&level);
        let state = access.get_block_state(&BlockPos::new(0, -1000, 0));
        assert_eq!(state, BlockState { id: 0 });
        let state2 = access.get_block_state(&BlockPos::new(0, 1000, 0));
        assert_eq!(state2, BlockState { id: 0 });
    }

    #[test]
    fn test_caches_section_acquire_once() {
        let mut level = MockLevel::new(0, 256);
        let chunk = ChunkAccess::new(16);
        chunk.get_section(0).set_block_state(1, 2, 3, BlockState { id: 42 });
        level.chunks.insert((0, 0), chunk.clone());
        let mut access = BulkSectionAccess::new(&level);
        let pos = BlockPos::new(1, 2, 3);
        assert_eq!(access.get_block_state(&pos).id, 42);
        assert_eq!(access.get_block_state(&pos).id, 42);
        let sec = access.get_section(&pos).unwrap();
        assert_eq!(sec.acquire_count(), 1);
    }

    #[test]
    fn test_negative_coords() {
        let mut level = MockLevel::new(-64, 384);
        let chunk = ChunkAccess::new(24);
        chunk.get_section(0).set_block_state(0, 0, 0, BlockState { id: 7 });
        level.chunks.insert((-1, -1), chunk);
        let mut access = BulkSectionAccess::new(&level);
        let pos = BlockPos::new(-16, -64, -16);
        assert_eq!(access.get_block_state(&pos).id, 7);
    }

    #[test]
    fn test_close_releases_sections() {
        let mut level = MockLevel::new(0, 256);
        let chunk = ChunkAccess::new(16);
        level.chunks.insert((0, 0), chunk.clone());
        let mut access = BulkSectionAccess::new(&level);
        let _ = access.get_section(&BlockPos::new(0, 0, 0));
        assert_eq!(chunk.get_section(0).acquire_count(), 1);
        access.close();
        assert_eq!(chunk.get_section(0).acquire_count(), 0);
    }

    #[test]
    fn test_multiple_sections_acquired_and_released() {
        let mut level = MockLevel::new(0, 256);
        let chunk = ChunkAccess::new(16);
        level.chunks.insert((0, 0), chunk.clone());
        {
            let mut access = BulkSectionAccess::new(&level);
            let _ = access.get_section(&BlockPos::new(0, 0, 0));
            let _ = access.get_section(&BlockPos::new(0, 16, 0));
            assert_eq!(chunk.get_section(0).acquire_count(), 1);
            assert_eq!(chunk.get_section(1).acquire_count(), 1);
        }
        assert_eq!(chunk.get_section(0).acquire_count(), 0);
        assert_eq!(chunk.get_section(1).acquire_count(), 0);
    }

    #[test]
    fn test_section_pos_helpers_negative_and_edges() {
        assert_eq!(SectionPos::section_relative(-1), 15);
        assert_eq!(SectionPos::section_relative(16), 0);
        assert_eq!(SectionPos::section_relative(15), 15);
        assert_eq!(SectionPos::block_to_section_coord(-1), -1);
        assert_eq!(SectionPos::block_to_section_coord(-16), -1);
        assert_eq!(SectionPos::block_to_section_coord(0), 0);
        let p = BlockPos::new(-1, -1, -1);
        let key = SectionPos::as_long(&p);
        let key2 = SectionPos::as_long_coords(-1, -1, -1);
        assert_eq!(key, key2);
    }
}
