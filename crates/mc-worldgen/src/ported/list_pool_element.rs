use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Projection {
    TerrainMatching,
    Rigid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn get_x(self) -> i32 {
        self.x
    }

    #[inline]
    pub const fn get_y(self) -> i32 {
        self.y
    }

    #[inline]
    pub const fn get_z(self) -> i32 {
        self.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    #[inline]
    pub const fn new(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    pub fn encapsulating_boxes(
        boxes: impl IntoIterator<Item = BoundingBox>,
    ) -> Option<BoundingBox> {
        let mut iter = boxes.into_iter();
        let first = iter.next()?;
        let mut min_x = first.min_x;
        let mut min_y = first.min_y;
        let mut min_z = first.min_z;
        let mut max_x = first.max_x;
        let mut max_y = first.max_y;
        let mut max_z = first.max_z;
        for b in iter {
            min_x = min_x.min(b.min_x);
            min_y = min_y.min(b.min_y);
            min_z = min_z.min(b.min_z);
            max_x = max_x.max(b.max_x);
            max_y = max_y.max(b.max_y);
            max_z = max_z.max(b.max_z);
        }
        Some(BoundingBox::new(min_x, min_y, min_z, max_x, max_y, max_z))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LiquidSettings {
    ApplyWaterlogging,
    IgnoreWaterlogging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructurePoolElementType {
    List,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JigsawBlockInfo;

pub trait StructureTemplateManager: fmt::Debug {}
pub trait WorldGenLevel: fmt::Debug {}
pub trait StructureManager: fmt::Debug {}
pub trait ChunkGenerator: fmt::Debug {}
pub trait RandomSource: fmt::Debug {}

pub trait StructurePoolElement: fmt::Debug + fmt::Display {
    fn get_size(
        &self,
        template_manager: &dyn StructureTemplateManager,
        rotation: Rotation,
    ) -> Vec3i;

    fn get_shuffled_jigsaw_blocks(
        &self,
        template_manager: &dyn StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
        random: &mut dyn RandomSource,
    ) -> Vec<JigsawBlockInfo>;

    fn get_bounding_box(
        &self,
        template_manager: &dyn StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
    ) -> BoundingBox;

    fn place(
        &self,
        template_manager: &dyn StructureTemplateManager,
        level: &mut dyn WorldGenLevel,
        structure_manager: &dyn StructureManager,
        chunk_generator: &dyn ChunkGenerator,
        pos: BlockPos,
        pivot: BlockPos,
        rotation: Rotation,
        bounding_box: &BoundingBox,
        random: &mut dyn RandomSource,
        liquid_settings: LiquidSettings,
        keep_jigsaws: bool,
    ) -> bool;

    fn get_type(&self) -> StructurePoolElementType;

    fn set_projection(&mut self, projection: Projection);

    fn projection(&self) -> Projection;

    fn is_empty_pool_element(&self) -> bool {
        false
    }
}

pub struct ListPoolElement {
    projection: Projection,
    elements: Vec<Box<dyn StructurePoolElement>>,
}

impl fmt::Debug for ListPoolElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListPoolElement")
            .field("projection", &self.projection)
            .field("elements", &self.elements)
            .finish()
    }
}

impl ListPoolElement {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#<init>
    pub fn new(elements: Vec<Box<dyn StructurePoolElement>>, projection: Projection) -> Self {
        if elements.is_empty() {
            panic!("Elements are empty");
        }
        let mut this = Self {
            projection,
            elements,
        };
        this.set_projection_on_each_element(projection);
        this
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#getElements
    pub fn get_elements(&self) -> &[Box<dyn StructurePoolElement>] {
        &self.elements
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#setProjectionOnEachElement
    fn set_projection_on_each_element(&mut self, projection: Projection) {
        for element in &mut self.elements {
            element.set_projection(projection);
        }
    }
}

impl StructurePoolElement for ListPoolElement {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#getSize
    fn get_size(
        &self,
        template_manager: &dyn StructureTemplateManager,
        rotation: Rotation,
    ) -> Vec3i {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut z: i32 = 0;

        for element in &self.elements {
            let size = element.get_size(template_manager, rotation);
            x = x.max(size.get_x());
            y = y.max(size.get_y());
            z = z.max(size.get_z());
        }

        Vec3i::new(x, y, z)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#getShuffledJigsawBlocks
    fn get_shuffled_jigsaw_blocks(
        &self,
        template_manager: &dyn StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
        random: &mut dyn RandomSource,
    ) -> Vec<JigsawBlockInfo> {
        self.elements[0].get_shuffled_jigsaw_blocks(template_manager, pos, rotation, random)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#getBoundingBox
    fn get_bounding_box(
        &self,
        template_manager: &dyn StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
    ) -> BoundingBox {
        let boxes = self
            .elements
            .iter()
            .filter(|e| !e.is_empty_pool_element())
            .map(|e| e.get_bounding_box(template_manager, pos, rotation));
        BoundingBox::encapsulating_boxes(boxes)
            .expect("Unable to calculate boundingbox for ListPoolElement")
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#place
    fn place(
        &self,
        template_manager: &dyn StructureTemplateManager,
        level: &mut dyn WorldGenLevel,
        structure_manager: &dyn StructureManager,
        chunk_generator: &dyn ChunkGenerator,
        pos: BlockPos,
        pivot: BlockPos,
        rotation: Rotation,
        bounding_box: &BoundingBox,
        random: &mut dyn RandomSource,
        liquid_settings: LiquidSettings,
        keep_jigsaws: bool,
    ) -> bool {
        for element in &self.elements {
            if !element.place(
                template_manager,
                level,
                structure_manager,
                chunk_generator,
                pos,
                pivot,
                rotation,
                bounding_box,
                random,
                liquid_settings,
                keep_jigsaws,
            ) {
                return false;
            }
        }
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#getType
    fn get_type(&self) -> StructurePoolElementType {
        StructurePoolElementType::List
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#setProjection
    fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.set_projection_on_each_element(projection);
    }

    fn projection(&self) -> Projection {
        self.projection
    }
}

impl fmt::Display for ListPoolElement {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.ListPoolElement#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List[")?;
        for (i, element) in self.elements.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{element}")?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[derive(Debug)]
    struct MockManager;
    impl StructureTemplateManager for MockManager {}

    #[derive(Debug)]
    struct MockLevel;
    impl WorldGenLevel for MockLevel {}

    #[derive(Debug)]
    struct MockStructureManager;
    impl StructureManager for MockStructureManager {}

    #[derive(Debug)]
    struct MockChunkGenerator;
    impl ChunkGenerator for MockChunkGenerator {}

    #[derive(Debug)]
    struct MockRandom;
    impl RandomSource for MockRandom {}

    #[derive(Debug, Clone)]
    struct MockElement {
        size: Vec3i,
        bounds: BoundingBox,
        projection: Projection,
        jigsaw_count: usize,
        empty: bool,
    }

    impl MockElement {
        fn new(size: Vec3i, bounds: BoundingBox) -> Self {
            Self {
                size,
                bounds,
                projection: Projection::Rigid,
                jigsaw_count: 0,
                empty: false,
            }
        }

        fn empty() -> Self {
            Self {
                size: Vec3i::new(0, 0, 0),
                bounds: BoundingBox::new(0, 0, 0, 0, 0, 0),
                projection: Projection::Rigid,
                jigsaw_count: 0,
                empty: true,
            }
        }

        fn with_jigsaw_count(mut self, count: usize) -> Self {
            self.jigsaw_count = count;
            self
        }
    }

    impl fmt::Display for MockElement {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.empty {
                write!(f, "Empty")
            } else {
                write!(f, "Mock({},{},{})", self.size.x, self.size.y, self.size.z)
            }
        }
    }

    impl StructurePoolElement for MockElement {
        fn get_size(&self, _: &dyn StructureTemplateManager, _: Rotation) -> Vec3i {
            self.size
        }

        fn get_shuffled_jigsaw_blocks(
            &self,
            _: &dyn StructureTemplateManager,
            _: BlockPos,
            _: Rotation,
            _: &mut dyn RandomSource,
        ) -> Vec<JigsawBlockInfo> {
            vec![JigsawBlockInfo; self.jigsaw_count]
        }

        fn get_bounding_box(
            &self,
            _: &dyn StructureTemplateManager,
            _: BlockPos,
            _: Rotation,
        ) -> BoundingBox {
            self.bounds
        }

        fn place(
            &self,
            _: &dyn StructureTemplateManager,
            _: &mut dyn WorldGenLevel,
            _: &dyn StructureManager,
            _: &dyn ChunkGenerator,
            _: BlockPos,
            _: BlockPos,
            _: Rotation,
            _: &BoundingBox,
            _: &mut dyn RandomSource,
            _: LiquidSettings,
            _: bool,
        ) -> bool {
            true
        }

        fn get_type(&self) -> StructurePoolElementType {
            StructurePoolElementType::List
        }

        fn set_projection(&mut self, projection: Projection) {
            self.projection = projection;
        }

        fn projection(&self) -> Projection {
            self.projection
        }

        fn is_empty_pool_element(&self) -> bool {
            self.empty
        }
    }

    #[derive(Debug)]
    struct TrackingElement {
        projection: Projection,
        place_result: bool,
        place_count: Rc<Cell<u32>>,
    }

    impl TrackingElement {
        fn new(place_result: bool, place_count: Rc<Cell<u32>>) -> Self {
            Self {
                projection: Projection::Rigid,
                place_result,
                place_count,
            }
        }
    }

    impl fmt::Display for TrackingElement {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Tracking")
        }
    }

    impl StructurePoolElement for TrackingElement {
        fn get_size(&self, _: &dyn StructureTemplateManager, _: Rotation) -> Vec3i {
            Vec3i::new(0, 0, 0)
        }

        fn get_shuffled_jigsaw_blocks(
            &self,
            _: &dyn StructureTemplateManager,
            _: BlockPos,
            _: Rotation,
            _: &mut dyn RandomSource,
        ) -> Vec<JigsawBlockInfo> {
            vec![]
        }

        fn get_bounding_box(
            &self,
            _: &dyn StructureTemplateManager,
            _: BlockPos,
            _: Rotation,
        ) -> BoundingBox {
            BoundingBox::default()
        }

        fn place(
            &self,
            _: &dyn StructureTemplateManager,
            _: &mut dyn WorldGenLevel,
            _: &dyn StructureManager,
            _: &dyn ChunkGenerator,
            _: BlockPos,
            _: BlockPos,
            _: Rotation,
            _: &BoundingBox,
            _: &mut dyn RandomSource,
            _: LiquidSettings,
            _: bool,
        ) -> bool {
            self.place_count.set(self.place_count.get() + 1);
            self.place_result
        }

        fn get_type(&self) -> StructurePoolElementType {
            StructurePoolElementType::List
        }

        fn set_projection(&mut self, projection: Projection) {
            self.projection = projection;
        }

        fn projection(&self) -> Projection {
            self.projection
        }
    }

    #[test]
    #[should_panic(expected = "Elements are empty")]
    fn test_new_empty_panics() {
        let _ = ListPoolElement::new(vec![], Projection::Rigid);
    }

    #[test]
    fn test_get_size_max_negative_and_extreme() {
        let elements: Vec<Box<dyn StructurePoolElement>> = vec![
            Box::new(MockElement::new(
                Vec3i::new(i32::MIN, 10, 3),
                BoundingBox::default(),
            )),
            Box::new(MockElement::new(
                Vec3i::new(7, i32::MIN, i32::MAX),
                BoundingBox::default(),
            )),
            Box::new(MockElement::new(
                Vec3i::new(1, 8, -100),
                BoundingBox::default(),
            )),
        ];
        let list = ListPoolElement::new(elements, Projection::Rigid);
        let size = list.get_size(&MockManager, Rotation::None);
        assert_eq!(size, Vec3i::new(7, 10, i32::MAX));
        assert_eq!(list.get_type(), StructurePoolElementType::List);
    }

    #[test]
    fn test_get_shuffled_jigsaw_blocks_delegates_to_first() {
        let elements: Vec<Box<dyn StructurePoolElement>> = vec![
            Box::new(
                MockElement::new(Vec3i::new(1, 1, 1), BoundingBox::default())
                    .with_jigsaw_count(3),
            ),
            Box::new(
                MockElement::new(Vec3i::new(2, 2, 2), BoundingBox::default())
                    .with_jigsaw_count(5),
            ),
        ];
        let list = ListPoolElement::new(elements, Projection::Rigid);
        let mut random = MockRandom;
        let blocks = list.get_shuffled_jigsaw_blocks(
            &MockManager,
            BlockPos::new(0, 0, 0),
            Rotation::None,
            &mut random,
        );
        assert_eq!(blocks.len(), 3);
    }

    #[test]
    fn test_get_bounding_box_filters_empty_and_negative_coords() {
        let elements: Vec<Box<dyn StructurePoolElement>> = vec![
            Box::new(MockElement::empty()),
            Box::new(MockElement::new(
                Vec3i::new(1, 1, 1),
                BoundingBox::new(-5, -10, 2, 3, 4, 8),
            )),
            Box::new(MockElement::new(
                Vec3i::new(1, 1, 1),
                BoundingBox::new(0, -20, -1, 10, 5, 6),
            )),
        ];
        let list = ListPoolElement::new(elements, Projection::Rigid);
        let bb = list.get_bounding_box(&MockManager, BlockPos::new(0, 0, 0), Rotation::None);
        assert_eq!(bb, BoundingBox::new(-5, -20, -1, 10, 5, 8));
    }

    #[test]
    #[should_panic(expected = "Unable to calculate boundingbox for ListPoolElement")]
    fn test_get_bounding_box_all_empty_panics() {
        let elements: Vec<Box<dyn StructurePoolElement>> = vec![
            Box::new(MockElement::empty()),
            Box::new(MockElement::empty()),
        ];
        let list = ListPoolElement::new(elements, Projection::Rigid);
        let _ = list.get_bounding_box(&MockManager, BlockPos::new(0, 0, 0), Rotation::None);
    }

    #[test]
    fn test_place_projection_and_to_string() {
        let c1 = Rc::new(Cell::new(0u32));
        let c2 = Rc::new(Cell::new(0u32));
        let c3 = Rc::new(Cell::new(0u32));
        let elements: Vec<Box<dyn StructurePoolElement>> = vec![
            Box::new(TrackingElement::new(true, c1.clone())),
            Box::new(TrackingElement::new(false, c2.clone())),
            Box::new(TrackingElement::new(true, c3.clone())),
        ];
        let mut list = ListPoolElement::new(elements, Projection::TerrainMatching);
        let mut level = MockLevel;
        let mut random = MockRandom;

        let ok = list.place(
            &MockManager,
            &mut level,
            &MockStructureManager,
            &MockChunkGenerator,
            BlockPos::new(i32::MIN, -1, i32::MAX),
            BlockPos::new(0, 0, 0),
            Rotation::Clockwise180,
            &BoundingBox::default(),
            &mut random,
            LiquidSettings::IgnoreWaterlogging,
            true,
        );
        assert!(!ok);
        assert_eq!(c1.get(), 1);
        assert_eq!(c2.get(), 1);
        assert_eq!(c3.get(), 0);

        assert_eq!(list.projection(), Projection::TerrainMatching);
        for e in list.get_elements() {
            assert_eq!(e.projection(), Projection::TerrainMatching);
        }
        list.set_projection(Projection::Rigid);
        assert_eq!(list.projection(), Projection::Rigid);
        for e in list.get_elements() {
            assert_eq!(e.projection(), Projection::Rigid);
        }

        let s = list.to_string();
        assert_eq!(s, "List[Tracking, Tracking, Tracking]");

        let a1 = Rc::new(Cell::new(0u32));
        let a2 = Rc::new(Cell::new(0u32));
        let list2 = ListPoolElement::new(
            vec![
                Box::new(TrackingElement::new(true, a1.clone())),
                Box::new(TrackingElement::new(true, a2.clone())),
            ],
            Projection::Rigid,
        );
        let ok2 = list2.place(
            &MockManager,
            &mut level,
            &MockStructureManager,
            &MockChunkGenerator,
            BlockPos::new(0, 0, 0),
            BlockPos::new(0, 0, 0),
            Rotation::None,
            &BoundingBox::default(),
            &mut random,
            LiquidSettings::ApplyWaterlogging,
            false,
        );
        assert!(ok2);
        assert_eq!(a1.get(), 1);
        assert_eq!(a2.get(), 1);
    }
}
