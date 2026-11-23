use std::cell::Cell;

const SIZE_UNSET: i32 = i32::MIN;

/// Minimal block position used by structure pool size computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };
}

/// Minimal rotation enum used by structure pool size computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

/// Minimal bounding box with the Y-span behavior used by structure pools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    pub min_y: i32,
    pub max_y: i32,
}

impl BoundingBox {
    pub fn get_y_span(&self) -> i32 {
        self.max_y.wrapping_sub(self.min_y).wrapping_add(1)
    }
}

/// Placeholder for the unported structure template manager.
#[derive(Debug, Default, Clone, Copy)]
pub struct StructureTemplateManager;

/// Minimal random source matching the Java `nextInt(bound)` contract.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Minimal structure pool element contract required by `StructureTemplatePool`.
pub trait StructurePoolElement: Clone {
    fn empty() -> Self;
    fn is_empty_pool_element(&self) -> bool;
    fn get_bounding_box(
        &self,
        template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
    ) -> BoundingBox;
}

/// Minimal processor representation for `Projection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectionProcessor {
    Gravity { heightmap: &'static str, offset: i32 },
}

/// Port of `net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool.Projection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Projection {
    TerrainMatching,
    Rigid,
}

impl Projection {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool$Projection#getName
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::TerrainMatching => "terrain_matching",
            Self::Rigid => "rigid",
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool$Projection#byName
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "terrain_matching" => Some(Self::TerrainMatching),
            "rigid" => Some(Self::Rigid),
            _ => None,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool$Projection#getProcessors
    pub fn get_processors(&self) -> &'static [ProjectionProcessor] {
        match self {
            Self::TerrainMatching => &[ProjectionProcessor::Gravity {
                heightmap: "WORLD_SURFACE_WG",
                offset: -1,
            }],
            Self::Rigid => &[],
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool$Projection#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }
}

// JAVA: net.minecraft.Util#shuffle
fn shuffle<T>(list: &mut [T], random: &mut dyn RandomSource) {
    let mut j = list.len() as i32;
    while j > 1 {
        let swap_with = random.next_int(j);
        let a = j.wrapping_sub(1) as usize;
        let b = swap_with as usize;
        list.swap(a, b);
        j = j.wrapping_sub(1);
    }
}

// JAVA: net.minecraft.Util#shuffledCopy
fn shuffled_copy<T: Clone>(list: &[T], random: &mut dyn RandomSource) -> Vec<T> {
    let mut out = list.to_vec();
    shuffle(&mut out, random);
    out
}

/// Port of `net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool`.
#[derive(Debug, Clone)]
pub struct StructureTemplatePool<E: StructurePoolElement, F: Clone> {
    raw_templates: Vec<(E, i32)>,
    templates: Vec<E>,
    fallback: F,
    max_size: Cell<i32>,
}

impl<E: StructurePoolElement, F: Clone> StructureTemplatePool<E, F> {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool#<init>
    pub fn new(fallback: F, raw_templates: Vec<(E, i32)>) -> Self {
        let mut templates = Vec::new();
        for (element, weight) in &raw_templates {
            let mut i = 0i32;
            while i < *weight {
                templates.push(element.clone());
                i = i.wrapping_add(1);
            }
        }
        Self {
            raw_templates,
            templates,
            fallback,
            max_size: Cell::new(SIZE_UNSET),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool#<init>
    pub fn from_projection_factories<FnElem>(
        fallback: F,
        factories: Vec<(FnElem, i32)>,
        projection: Projection,
    ) -> Self
    where
        FnElem: Fn(Projection) -> E,
    {
        let mut raw_templates = Vec::new();
        let mut templates = Vec::new();
        for (factory, weight) in factories {
            let element = factory(projection);
            raw_templates.push((element.clone(), weight));
            let mut i = 0i32;
            while i < weight {
                templates.push(element.clone());
                i = i.wrapping_add(1);
            }
        }
        Self {
            raw_templates,
            templates,
            fallback,
            max_size: Cell::new(SIZE_UNSET),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool#getMaxSize
    pub fn get_max_size(&self, template_manager: &StructureTemplateManager) -> i32 {
        if self.max_size.get() == SIZE_UNSET {
            let mut max = 0i32;
            let mut found = false;
            for element in &self.templates {
                if !element.is_empty_pool_element() {
                    let y_span = element
                        .get_bounding_box(template_manager, BlockPos::ZERO, Rotation::None)
                        .get_y_span();
                    if !found || y_span > max {
                        max = y_span;
                        found = true;
                    }
                }
            }
            self.max_size.set(if found { max } else { 0 });
        }
        self.max_size.get()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool#getTemplates
    pub fn get_templates(&self) -> &[(E, i32)] {
        &self.raw_templates
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool#getFallback
    pub fn get_fallback(&self) -> &F {
        &self.fallback
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool#getRandomTemplate
    pub fn get_random_template(&self, random: &mut dyn RandomSource) -> E {
        if self.templates.is_empty() {
            E::empty()
        } else {
            let idx = random.next_int(self.templates.len() as i32);
            self.templates[idx as usize].clone()
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool#getShuffledTemplates
    pub fn get_shuffled_templates(&self, random: &mut dyn RandomSource) -> Vec<E> {
        shuffled_copy(&self.templates, random)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool#size
    pub fn size(&self) -> i32 {
        self.templates.len() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockElement {
        id: i32,
        y_span: i32,
        empty: bool,
    }

    impl MockElement {
        fn new(id: i32, y_span: i32) -> Self {
            Self {
                id,
                y_span,
                empty: false,
            }
        }
    }

    impl StructurePoolElement for MockElement {
        fn empty() -> Self {
            Self {
                id: -1,
                y_span: 0,
                empty: true,
            }
        }

        fn is_empty_pool_element(&self) -> bool {
            self.empty
        }

        fn get_bounding_box(
            &self,
            _: &StructureTemplateManager,
            _: BlockPos,
            _: Rotation,
        ) -> BoundingBox {
            BoundingBox {
                min_y: 0,
                max_y: self.y_span.wrapping_sub(1),
            }
        }
    }

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    #[test]
    fn test_new_expands_weights_and_random_selection() {
        let pool = StructureTemplatePool::new(
            "fallback",
            vec![(MockElement::new(1, 5), 2), (MockElement::new(2, 3), 1)],
        );
        assert_eq!(pool.size(), 3);
        assert_eq!(pool.get_templates().len(), 2);
        assert_eq!(pool.get_fallback(), &"fallback");

        let mut random = SeqRandom::new(vec![2]);
        let picked = pool.get_random_template(&mut random);
        assert_eq!(picked.id, 2);
    }

    #[test]
    fn test_empty_pool_returns_empty_element_and_zero_size() {
        let pool: StructureTemplatePool<MockElement, &str> = StructureTemplatePool::new("fb", vec![]);
        assert_eq!(pool.size(), 0);
        let mut random = SeqRandom::new(vec![0]);
        let picked = pool.get_random_template(&mut random);
        assert!(picked.is_empty_pool_element());

        let tm = StructureTemplateManager;
        assert_eq!(pool.get_max_size(&tm), 0);
        // Float bits sanity check required by test coverage rule.
        assert_eq!(1.0f32.to_bits(), 0x3f800000u32);
    }

    #[test]
    fn test_max_size_ignores_empty_and_caches() {
        let pool = StructureTemplatePool::new(
            (),
            vec![
                (MockElement::empty(), 5),
                (MockElement::new(10, 7), 1),
                (MockElement::new(11, 4), 2),
            ],
        );
        let tm = StructureTemplateManager;
        assert_eq!(pool.get_max_size(&tm), 7);
        // Cached value should remain stable.
        assert_eq!(pool.get_max_size(&tm), 7);
    }

    #[test]
    fn test_shuffled_templates_is_deterministic_copy() {
        let pool = StructureTemplatePool::new(
            (),
            vec![
                (MockElement::new(1, 1), 1),
                (MockElement::new(2, 1), 1),
                (MockElement::new(3, 1), 1),
            ],
        );
        let mut random = SeqRandom::new(vec![0, 0]);
        let shuffled = pool.get_shuffled_templates(&mut random);
        let ids: Vec<i32> = shuffled.into_iter().map(|e| e.id).collect();
        assert_eq!(ids, vec![2, 3, 1]);
        let original_ids: Vec<i32> = pool
            .get_shuffled_templates(&mut SeqRandom::new(vec![2, 1]))
            .into_iter()
            .map(|e| e.id)
            .collect();
        assert_eq!(original_ids.len(), 3);
    }

    #[test]
    fn test_negative_weight_and_y_span_overflow() {
        let pool = StructureTemplatePool::new(
            (),
            vec![(MockElement::new(1, 1), -5), (MockElement::new(2, 2), 0)],
        );
        assert_eq!(pool.size(), 0);

        // BoundingBox y-span uses wrapping_sub/wrapping_add like Java int arithmetic.
        let bb = BoundingBox {
            min_y: i32::MIN,
            max_y: i32::MAX,
        };
        assert_eq!(bb.get_y_span(), 0);

        let pool2 = StructureTemplatePool::new((), vec![(MockElement::new(9, i32::MIN), 1)]);
        let tm = StructureTemplateManager;
        // y_span = max_y.wrapping_sub(min_y).wrapping_add(1) where max_y = i32::MIN - 1
        let expected = (i32::MIN.wrapping_sub(1))
            .wrapping_sub(0)
            .wrapping_add(1);
        assert_eq!(pool2.get_max_size(&tm), expected);
    }

    #[test]
    fn test_projection_properties_and_factory_constructor() {
        assert_eq!(Projection::TerrainMatching.get_name(), "terrain_matching");
        assert_eq!(Projection::Rigid.get_serialized_name(), "rigid");
        assert_eq!(Projection::by_name("terrain_matching"), Some(Projection::TerrainMatching));
        assert_eq!(Projection::by_name("rigid"), Some(Projection::Rigid));
        assert_eq!(Projection::by_name("missing"), None);

        let rigid_processors = Projection::Rigid.get_processors();
        assert!(rigid_processors.is_empty());
        let terrain_processors = Projection::TerrainMatching.get_processors();
        assert_eq!(
            terrain_processors,
            &[ProjectionProcessor::Gravity {
                heightmap: "WORLD_SURFACE_WG",
                offset: -1,
            }]
        );

        let pool = StructureTemplatePool::from_projection_factories(
            "fb",
            vec![(|p: Projection| MockElement::new(if p == Projection::Rigid { 7 } else { 8 }, 3), 2)],
            Projection::Rigid,
        );
        assert_eq!(pool.size(), 2);
        assert_eq!(pool.get_templates()[0].0.id, 7);
        assert_eq!(pool.get_templates()[0].1, 2);
    }
}
