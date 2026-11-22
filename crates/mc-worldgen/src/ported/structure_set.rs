use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    pub name: String,
    _marker: PhantomData<T>,
}

impl<T> Holder<T> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Structure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructurePlacement {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureSelectionEntry {
    pub structure: Holder<Structure>,
    pub weight: i32,
}

impl StructureSelectionEntry {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructureSet$StructureSelectionEntry#<init>
    pub fn new(structure: Holder<Structure>, weight: i32) -> Self {
        Self { structure, weight }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureSet {
    pub structures: Vec<StructureSelectionEntry>,
    pub placement: StructurePlacement,
}

impl StructureSet {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructureSet#<init>(List, StructurePlacement)
    pub fn new(structures: Vec<StructureSelectionEntry>, placement: StructurePlacement) -> Self {
        Self {
            structures,
            placement,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureSet#<init>(Holder, StructurePlacement)
    pub fn from_single(structure: Holder<Structure>, placement: StructurePlacement) -> Self {
        Self::new(vec![StructureSelectionEntry::new(structure, 1)], placement)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureSet#entry(Holder, int)
    pub fn entry_weighted(structure: Holder<Structure>, weight: i32) -> StructureSelectionEntry {
        StructureSelectionEntry::new(structure, weight)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructureSet#entry(Holder)
    pub fn entry(structure: Holder<Structure>) -> StructureSelectionEntry {
        StructureSelectionEntry::new(structure, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holder(name: &str) -> Holder<Structure> {
        Holder::new(name)
    }

    fn placement(id: &str) -> StructurePlacement {
        StructurePlacement { id: id.to_string() }
    }

    #[test]
    fn from_single_creates_one_entry_with_weight_one() {
        let s = StructureSet::from_single(holder("village"), placement("p1"));
        assert_eq!(s.structures.len(), 1);
        assert_eq!(s.structures[0].weight, 1);
        assert_eq!(s.structures[0].structure, holder("village"));
        assert_eq!(s.placement, placement("p1"));
    }

    #[test]
    fn entry_helpers_match_java_behavior() {
        let e1 = StructureSet::entry(holder("a"));
        assert_eq!(e1.weight, 1);
        assert_eq!(e1.structure, holder("a"));

        let e2 = StructureSet::entry_weighted(holder("b"), 5);
        assert_eq!(e2.weight, 5);
        assert_eq!(e2.structure, holder("b"));
    }

    #[test]
    fn canonical_constructor_preserves_entries_and_placement() {
        let entries = vec![
            StructureSet::entry_weighted(holder("a"), 2),
            StructureSet::entry(holder("b")),
        ];
        let s = StructureSet::new(entries, placement("p"));
        assert_eq!(s.structures.len(), 2);
        assert_eq!(s.structures[0].weight, 2);
        assert_eq!(s.structures[1].weight, 1);
        assert_eq!(s.placement, placement("p"));
    }

    #[test]
    fn entry_stores_negative_and_zero_weight() {
        let e_neg = StructureSelectionEntry::new(holder("x"), -1);
        let e_zero = StructureSelectionEntry::new(holder("y"), 0);
        let e_max = StructureSelectionEntry::new(holder("z"), i32::MAX);
        assert_eq!(e_neg.weight, -1);
        assert_eq!(e_zero.weight, 0);
        assert_eq!(e_max.weight, i32::MAX);
    }
}
