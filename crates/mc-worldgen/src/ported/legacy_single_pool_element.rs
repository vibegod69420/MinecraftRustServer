use std::fmt;

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

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructureTemplate {
    pub id: String,
}

impl fmt::Display for StructureTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: fmt::Display, R: fmt::Display> fmt::Display for Either<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(v) => v.fmt(f),
            Either::Right(v) => v.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructureProcessorList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    TerrainMatching,
    Rigid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LiquidSettings {
    pub ignore_waterlogging: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rotation(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePoolElementType {
    Legacy,
    Single,
    Feature,
    Empty,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockIgnoreProcessor {
    StructureBlock,
    StructureAndAir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Processor {
    BlockIgnore(BlockIgnoreProcessor),
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructurePlaceSettings {
    pub rotation: Rotation,
    pub bounding_box: BoundingBox,
    pub liquid_settings: LiquidSettings,
    pub processors: Vec<Processor>,
}

impl StructurePlaceSettings {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#popProcessor
    pub fn pop_processor(&mut self, processor: BlockIgnoreProcessor) {
        if let Some(pos) = self
            .processors
            .iter()
            .rposition(|p| matches!(p, Processor::BlockIgnore(x) if *x == processor))
        {
            self.processors.remove(pos);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#addProcessor
    pub fn add_processor(&mut self, processor: BlockIgnoreProcessor) {
        self.processors.push(Processor::BlockIgnore(processor));
    }
}

/// Minimal base representation for `SinglePoolElement` behavior needed by this port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacySinglePoolElement {
    pub template: Either<Identifier, StructureTemplate>,
    pub processors: Holder<StructureProcessorList>,
    pub projection: Projection,
    pub override_liquid_settings: Option<LiquidSettings>,
}

impl LegacySinglePoolElement {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.LegacySinglePoolElement#<init>
    pub fn new(
        template: Either<Identifier, StructureTemplate>,
        processors: Holder<StructureProcessorList>,
        projection: Projection,
        override_liquid_settings: Option<LiquidSettings>,
    ) -> Self {
        Self {
            template,
            processors,
            projection,
            override_liquid_settings,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.SinglePoolElement#getSettings
    fn super_get_settings(
        &self,
        rotation: Rotation,
        bounding_box: BoundingBox,
        liquid_settings: LiquidSettings,
        _offset: bool,
    ) -> StructurePlaceSettings {
        let mut settings = StructurePlaceSettings {
            rotation,
            bounding_box,
            liquid_settings,
            processors: Vec::new(),
        };
        settings.add_processor(BlockIgnoreProcessor::StructureBlock);
        settings
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.LegacySinglePoolElement#getSettings
    pub fn get_settings(
        &self,
        rotation: Rotation,
        bounding_box: BoundingBox,
        liquid_settings: LiquidSettings,
        offset: bool,
    ) -> StructurePlaceSettings {
        let mut settings = self.super_get_settings(rotation, bounding_box, liquid_settings, offset);
        settings.pop_processor(BlockIgnoreProcessor::StructureBlock);
        settings.add_processor(BlockIgnoreProcessor::StructureAndAir);
        settings
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.LegacySinglePoolElement#getType
    pub fn get_type(&self) -> StructurePoolElementType {
        StructurePoolElementType::Legacy
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.LegacySinglePoolElement#toString
impl fmt::Display for LegacySinglePoolElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LegacySingle[{}]", self.template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_element() -> LegacySinglePoolElement {
        LegacySinglePoolElement::new(
            Either::Left(Identifier::new("minecraft", "test")),
            Holder {
                value: StructureProcessorList,
            },
            Projection::Rigid,
            None,
        )
    }

    #[test]
    fn get_type_is_legacy() {
        let el = sample_element();
        assert_eq!(el.get_type(), StructurePoolElementType::Legacy);
    }

    #[test]
    fn get_settings_replaces_structure_block_with_structure_and_air() {
        let el = sample_element();
        let settings = el.get_settings(
            Rotation(1),
            BoundingBox {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 1,
                max_y: 2,
                max_z: 3,
            },
            LiquidSettings {
                ignore_waterlogging: false,
            },
            false,
        );
        assert_eq!(
            settings.processors,
            vec![Processor::BlockIgnore(BlockIgnoreProcessor::StructureAndAir)]
        );
        assert_eq!(settings.rotation, Rotation(1));
        assert_eq!(settings.bounding_box.max_z, 3);
    }

    #[test]
    fn to_string_uses_template() {
        let el = sample_element();
        assert_eq!(el.to_string(), "LegacySingle[minecraft:test]");

        let el_right = LegacySinglePoolElement::new(
            Either::Right(StructureTemplate {
                id: "tpl".to_string(),
            }),
            Holder {
                value: StructureProcessorList,
            },
            Projection::TerrainMatching,
            Some(LiquidSettings {
                ignore_waterlogging: true,
            }),
        );
        assert_eq!(el_right.to_string(), "LegacySingle[tpl]");
    }

    #[test]
    fn get_settings_handles_negative_and_extreme_coords() {
        let el = sample_element();
        let bb = BoundingBox {
            min_x: i32::MIN,
            min_y: -1,
            min_z: -100,
            max_x: i32::MAX,
            max_y: 0,
            max_z: 1,
        };
        let settings = el.get_settings(
            Rotation(-90),
            bb,
            LiquidSettings {
                ignore_waterlogging: true,
            },
            true,
        );
        assert_eq!(settings.bounding_box, bb);
        assert_eq!(settings.rotation, Rotation(-90));
        assert!(settings.liquid_settings.ignore_waterlogging);
        assert_eq!(
            settings.processors,
            vec![Processor::BlockIgnore(BlockIgnoreProcessor::StructureAndAir)]
        );
    }

    #[test]
    fn constructor_preserves_fields() {
        let override_ls = Some(LiquidSettings {
            ignore_waterlogging: true,
        });
        let el = LegacySinglePoolElement::new(
            Either::Left(Identifier::new("ns", "path")),
            Holder {
                value: StructureProcessorList,
            },
            Projection::TerrainMatching,
            override_ls,
        );
        assert_eq!(el.projection, Projection::TerrainMatching);
        assert_eq!(el.override_liquid_settings, override_ls);
        assert_eq!(el.template.to_string(), "ns:path");
    }
}
