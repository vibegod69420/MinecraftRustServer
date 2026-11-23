/// Minimal identifier representation for self-contained configuration port.
/// JAVA: net.minecraft.resources.Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

/// Stub for structure processor list; codec/type infrastructure is not ported.
#[derive(Debug, Clone)]
pub struct StructureProcessorList;

/// Minimal holder wrapper for self-contained configuration port.
/// JAVA: net.minecraft.core.Holder
#[derive(Debug, Clone)]
pub struct Holder<T> {
    pub value: T,
}

impl<T> Holder<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

pub trait FeatureConfiguration {}

/// Port of FossilFeatureConfiguration.
/// Note: The Java CODEC field is omitted because Mojang serialization codec
/// infrastructure is not available in the declared dependencies.
#[derive(Debug, Clone)]
pub struct FossilFeatureConfiguration {
    pub fossil_structures: Vec<Identifier>,
    pub overlay_structures: Vec<Identifier>,
    pub fossil_processors: Holder<StructureProcessorList>,
    pub overlay_processors: Holder<StructureProcessorList>,
    pub max_empty_corners_allowed: i32,
}

impl FeatureConfiguration for FossilFeatureConfiguration {}

impl FossilFeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.FossilFeatureConfiguration#<init>
    pub fn new(
        fossil_structures: Vec<Identifier>,
        overlay_structures: Vec<Identifier>,
        fossil_processors: Holder<StructureProcessorList>,
        overlay_processors: Holder<StructureProcessorList>,
        max_empty_corners_allowed: i32,
    ) -> Self {
        if fossil_structures.is_empty() {
            panic!("Fossil structure lists need at least one entry");
        }
        if fossil_structures.len() != overlay_structures.len() {
            panic!("Fossil structure lists must be equal lengths");
        }
        Self {
            fossil_structures,
            overlay_structures,
            fossil_processors,
            overlay_processors,
            max_empty_corners_allowed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(path: &str) -> Identifier {
        Identifier::new("minecraft", path)
    }

    fn dummy_processors() -> Holder<StructureProcessorList> {
        Holder::new(StructureProcessorList)
    }

    #[test]
    fn valid_construction_stores_fields() {
        let fossils = vec![id("fossil1"), id("fossil2")];
        let overlays = vec![id("overlay1"), id("overlay2")];
        let cfg = FossilFeatureConfiguration::new(
            fossils.clone(),
            overlays.clone(),
            dummy_processors(),
            dummy_processors(),
            4,
        );
        assert_eq!(cfg.fossil_structures, fossils);
        assert_eq!(cfg.overlay_structures, overlays);
        assert_eq!(cfg.max_empty_corners_allowed, 4);
    }

    #[test]
    #[should_panic(expected = "Fossil structure lists need at least one entry")]
    fn empty_fossil_structures_panics() {
        let _ = FossilFeatureConfiguration::new(
            vec![],
            vec![],
            dummy_processors(),
            dummy_processors(),
            0,
        );
    }

    #[test]
    #[should_panic(expected = "Fossil structure lists must be equal lengths")]
    fn mismatched_lengths_panics() {
        let _ = FossilFeatureConfiguration::new(
            vec![id("f1"), id("f2")],
            vec![id("o1")],
            dummy_processors(),
            dummy_processors(),
            0,
        );
    }

    #[test]
    fn allows_boundary_max_empty_corners() {
        let cfg = FossilFeatureConfiguration::new(
            vec![id("f")],
            vec![id("o")],
            dummy_processors(),
            dummy_processors(),
            7,
        );
        assert_eq!(cfg.max_empty_corners_allowed, 7);
        let cfg0 = FossilFeatureConfiguration::new(
            vec![id("f")],
            vec![id("o")],
            dummy_processors(),
            dummy_processors(),
            0,
        );
        assert_eq!(cfg0.max_empty_corners_allowed, 0);
    }
}
