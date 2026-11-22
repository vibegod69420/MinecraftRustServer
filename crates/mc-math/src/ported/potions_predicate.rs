/// Minimal self-contained stand-in for `net.minecraft.world.item.alchemy.Potion`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Potion(pub &'static str);

/// Minimal self-contained stand-in for `net.minecraft.core.Holder`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Holder<T>(pub T);

/// Minimal self-contained stand-in for `net.minecraft.core.HolderSet`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T: PartialEq> {
    pub entries: Vec<Holder<T>>,
}

impl<T: PartialEq> HolderSet<T> {
    pub fn new(entries: Vec<Holder<T>>) -> Self {
        Self { entries }
    }

    pub fn contains(&self, holder: &Holder<T>) -> bool {
        self.entries.iter().any(|e| e == holder)
    }
}

/// Minimal self-contained stand-in for `net.minecraft.world.item.alchemy.PotionContents`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PotionContents {
    potion: Option<Holder<Potion>>,
}

impl PotionContents {
    pub fn new(potion: Option<Holder<Potion>>) -> Self {
        Self { potion }
    }

    // JAVA: net.minecraft.world.item.alchemy.PotionContents#potion
    pub fn potion(&self) -> Option<&Holder<Potion>> {
        self.potion.as_ref()
    }
}

/// Minimal self-contained stand-in for `net.minecraft.core.component.DataComponentType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataComponentType(pub &'static str);

/// Minimal self-contained stand-in for
/// `net.minecraft.core.component.predicates.DataComponentPredicate`.
pub trait DataComponentPredicate {
    fn matches_contents(&self, contents: &PotionContents) -> bool;
}

/// Port of `net.minecraft.core.component.predicates.PotionsPredicate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionsPredicate {
    pub potions: HolderSet<Potion>,
}

impl PotionsPredicate {
    pub fn new(potions: HolderSet<Potion>) -> Self {
        Self { potions }
    }

    // JAVA: net.minecraft.core.component.predicates.PotionsPredicate#componentType
    pub fn component_type(&self) -> DataComponentType {
        DataComponentType("minecraft:potion_contents")
    }

    // JAVA: net.minecraft.core.component.predicates.PotionsPredicate#matches
    pub fn matches(&self, contents: &PotionContents) -> bool {
        match contents.potion() {
            Some(holder) => self.potions.contains(holder),
            None => false,
        }
    }

    // JAVA: net.minecraft.core.component.predicates.PotionsPredicate#potions
    pub fn potions(potions: HolderSet<Potion>) -> Self {
        Self::new(potions)
    }
}

impl DataComponentPredicate for PotionsPredicate {
    fn matches_contents(&self, contents: &PotionContents) -> bool {
        self.matches(contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holder(id: &'static str) -> Holder<Potion> {
        Holder(Potion(id))
    }

    #[test]
    fn empty_potion_contents_does_not_match() {
        let pred = PotionsPredicate::new(HolderSet::new(vec![holder("healing")]));
        assert!(!pred.matches(&PotionContents::new(None)));
    }

    #[test]
    fn matching_potion_matches() {
        let pred = PotionsPredicate::new(HolderSet::new(vec![holder("healing"), holder("swiftness")]));
        assert!(pred.matches(&PotionContents::new(Some(holder("healing")))));
        assert!(pred.matches(&PotionContents::new(Some(holder("swiftness")))));
    }

    #[test]
    fn non_matching_potion_does_not_match() {
        let pred = PotionsPredicate::new(HolderSet::new(vec![holder("healing")]));
        assert!(!pred.matches(&PotionContents::new(Some(holder("harming")))));
    }

    #[test]
    fn empty_holder_set_never_matches() {
        let pred = PotionsPredicate::potions(HolderSet::new(vec![]));
        assert!(!pred.matches(&PotionContents::new(None)));
        assert!(!pred.matches(&PotionContents::new(Some(holder("healing")))));
    }

    #[test]
    fn component_type_is_potion_contents() {
        let pred = PotionsPredicate::new(HolderSet::new(vec![]));
        assert_eq!(
            pred.component_type(),
            DataComponentType("minecraft:potion_contents")
        );
    }
}
