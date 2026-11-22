use std::collections::HashSet;
use std::marker::PhantomData;

pub type LongSet = HashSet<i64>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SavedDataType<T> {
    pub name: String,
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct StructureFeatureIndexSavedData {
    dirty: bool,
    all: LongSet,
    remaining: LongSet,
}

impl StructureFeatureIndexSavedData {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData#<init>(LongSet, LongSet)
    fn new_with_sets(all: LongSet, remaining: LongSet) -> Self {
        Self {
            dirty: false,
            all,
            remaining,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData#<init>()
    pub fn new() -> Self {
        Self::new_with_sets(HashSet::new(), HashSet::new())
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData#type
    pub fn type_(name: &str) -> SavedDataType<Self> {
        SavedDataType {
            name: name.to_string(),
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData#addIndex
    pub fn add_index(&mut self, index: i64) {
        self.all.insert(index);
        self.remaining.insert(index);
        self.dirty = true;
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData#hasStartIndex
    pub fn has_start_index(&self, index: i64) -> bool {
        self.all.contains(&index)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData#hasUnhandledIndex
    pub fn has_unhandled_index(&self, index: i64) -> bool {
        self.remaining.contains(&index)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData#removeIndex
    pub fn remove_index(&mut self, index: i64) {
        if self.remaining.remove(&index) {
            self.dirty = true;
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData#getAll
    pub fn get_all(&self) -> &LongSet {
        &self.all
    }

    // JAVA: net.minecraft.world.level.saveddata.SavedData#setDirty
    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    // JAVA: net.minecraft.world.level.saveddata.SavedData#isDirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl Default for StructureFeatureIndexSavedData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty_and_clean() {
        let d = StructureFeatureIndexSavedData::new();
        assert!(d.get_all().is_empty());
        assert!(!d.has_start_index(0));
        assert!(!d.has_unhandled_index(0));
        assert!(!d.is_dirty());
    }

    #[test]
    fn add_index_marks_dirty_and_present_in_both_sets() {
        let mut d = StructureFeatureIndexSavedData::new();
        d.add_index(42);
        assert!(d.has_start_index(42));
        assert!(d.has_unhandled_index(42));
        assert!(d.is_dirty());
        assert_eq!(d.get_all().len(), 1);
    }

    #[test]
    fn remove_index_only_affects_remaining_and_sets_dirty() {
        let mut d = StructureFeatureIndexSavedData::new();
        d.add_index(10);
        d.dirty = false;
        d.remove_index(10);
        assert!(d.has_start_index(10));
        assert!(!d.has_unhandled_index(10));
        assert!(d.is_dirty());
    }

    #[test]
    fn remove_missing_does_not_set_dirty() {
        let mut d = StructureFeatureIndexSavedData::new();
        d.remove_index(999);
        assert!(!d.is_dirty());
    }

    #[test]
    fn handles_negative_and_extreme_i64_indices() {
        let mut d = StructureFeatureIndexSavedData::new();
        d.add_index(-1);
        d.add_index(i64::MIN);
        d.add_index(i64::MAX);
        assert!(d.has_start_index(-1));
        assert!(d.has_start_index(i64::MIN));
        assert!(d.has_unhandled_index(i64::MAX));
        assert_eq!(d.get_all().len(), 3);
    }

    #[test]
    fn type_factory_preserves_name() {
        let t = StructureFeatureIndexSavedData::type_("structure_refs");
        assert_eq!(t.name, "structure_refs");
    }
}
