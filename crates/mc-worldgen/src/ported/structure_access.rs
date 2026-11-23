use std::collections::{HashMap, HashSet};

/// Minimal self-contained placeholder for `net.minecraft.world.level.levelgen.structure.Structure`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Structure {
    pub id: String,
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.levelgen.structure.StructureStart`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructureStart {
    pub valid: bool,
}

/// Self-contained replacement for `it.unimi.dsi.fastutil.longs.LongSet`.
pub type LongSet = HashSet<i64>;

/// Port of `net.minecraft.world.level.chunk.StructureAccess`.
pub trait StructureAccess {
    // JAVA: net.minecraft.world.level.chunk.StructureAccess#getStartForStructure
    fn get_start_for_structure(&self, structure: &Structure) -> Option<&StructureStart>;

    // JAVA: net.minecraft.world.level.chunk.StructureAccess#setStartForStructure
    fn set_start_for_structure(&mut self, structure: Structure, start: StructureStart);

    // JAVA: net.minecraft.world.level.chunk.StructureAccess#getReferencesForStructure
    fn get_references_for_structure(&self, structure: &Structure) -> LongSet;

    // JAVA: net.minecraft.world.level.chunk.StructureAccess#addReferenceForStructure
    fn add_reference_for_structure(&mut self, structure: Structure, reference: i64);

    // JAVA: net.minecraft.world.level.chunk.StructureAccess#getAllReferences
    fn get_all_references(&self) -> &HashMap<Structure, LongSet>;

    // JAVA: net.minecraft.world.level.chunk.StructureAccess#setAllReferences
    fn set_all_references(&mut self, references: HashMap<Structure, LongSet>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockStructureAccess {
        starts: HashMap<Structure, StructureStart>,
        references: HashMap<Structure, LongSet>,
    }

    impl StructureAccess for MockStructureAccess {
        fn get_start_for_structure(&self, structure: &Structure) -> Option<&StructureStart> {
            self.starts.get(structure)
        }

        fn set_start_for_structure(&mut self, structure: Structure, start: StructureStart) {
            self.starts.insert(structure, start);
        }

        fn get_references_for_structure(&self, structure: &Structure) -> LongSet {
            self.references.get(structure).cloned().unwrap_or_default()
        }

        fn add_reference_for_structure(&mut self, structure: Structure, reference: i64) {
            self.references.entry(structure).or_default().insert(reference);
        }

        fn get_all_references(&self) -> &HashMap<Structure, LongSet> {
            &self.references
        }

        fn set_all_references(&mut self, references: HashMap<Structure, LongSet>) {
            self.references = references;
        }
    }

    fn structure(id: &str) -> Structure {
        Structure {
            id: id.to_owned(),
        }
    }

    #[test]
    fn start_for_structure_round_trip() {
        let mut access = MockStructureAccess::default();
        let st = structure("village");
        assert!(access.get_start_for_structure(&st).is_none());
        access.set_start_for_structure(st.clone(), StructureStart { valid: true });
        assert_eq!(
            access.get_start_for_structure(&st),
            Some(&StructureStart { valid: true })
        );
    }

    #[test]
    fn references_add_and_get_with_extreme_values() {
        let mut access = MockStructureAccess::default();
        let st = structure("fortress");
        access.add_reference_for_structure(st.clone(), 42);
        access.add_reference_for_structure(st.clone(), -1);
        access.add_reference_for_structure(st.clone(), i64::MIN);
        access.add_reference_for_structure(st.clone(), i64::MAX);
        let refs = access.get_references_for_structure(&st);
        assert!(refs.contains(&42));
        assert!(refs.contains(&-1));
        assert!(refs.contains(&i64::MIN));
        assert!(refs.contains(&i64::MAX));
        assert_eq!(refs.len(), 4);
    }

    #[test]
    fn missing_structure_returns_empty_references() {
        let access = MockStructureAccess::default();
        let refs = access.get_references_for_structure(&structure("missing"));
        assert!(refs.is_empty());
    }

    #[test]
    fn set_and_get_all_references() {
        let mut access = MockStructureAccess::default();
        let mut map = HashMap::new();
        let mut set = LongSet::new();
        set.insert(i64::MAX);
        set.insert(0);
        set.insert(-123_456_789_i64);
        map.insert(structure("monument"), set);
        access.set_all_references(map);

        let all = access.get_all_references();
        assert_eq!(all.len(), 1);
        let refs = all.get(&structure("monument")).unwrap();
        assert!(refs.contains(&i64::MAX));
        assert!(refs.contains(&0));
        assert!(refs.contains(&-123_456_789_i64));
    }

    #[test]
    fn overwrite_start_replaces_value() {
        let mut access = MockStructureAccess::default();
        let st = structure("temple");
        access.set_start_for_structure(st.clone(), StructureStart { valid: false });
        access.set_start_for_structure(st.clone(), StructureStart { valid: true });
        assert!(access.get_start_for_structure(&st).unwrap().valid);
    }
}
