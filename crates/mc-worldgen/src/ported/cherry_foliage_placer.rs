use std::fmt::Debug;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn above(&self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
        }
    }

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

pub trait IntProvider: Debug {
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
}

#[derive(Debug, Clone, Copy)]
pub struct ConstantIntProvider(pub i32);

impl IntProvider for ConstantIntProvider {
    fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FoliageAttachment {
    pos: BlockPos,
    radius_offset: i32,
    double_trunk: bool,
}

impl FoliageAttachment {
    pub fn new(pos: BlockPos, radius_offset: i32, double_trunk: bool) -> Self {
        Self {
            pos,
            radius_offset,
            double_trunk,
        }
    }

    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    pub fn radius_offset(&self) -> i32 {
        self.radius_offset
    }

    pub fn double_trunk(&self) -> bool {
        self.double_trunk
    }
}

pub trait LevelSimulatedReader {}

pub trait FoliageSetter {
    fn place_leaf(&mut self, pos: BlockPos);
    fn place_hanging_leaf(&mut self, pos: BlockPos);
}

#[derive(Debug, Default, Clone)]
pub struct TreeConfiguration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoliagePlacerType {
    CherryFoliagePlacer,
}

/// Abstraction over base FoliagePlacer row placement methods used by CherryFoliagePlacer.
/// This keeps the port self-contained while preserving CherryFoliagePlacer's exact call pattern.
pub trait FoliageRowPlacer {
    fn place_leaves_row(
        &mut self,
        level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        config: &TreeConfiguration,
        pos: BlockPos,
        range: i32,
        y_offset: i32,
        large: bool,
    );

    fn place_leaves_row_with_hanging_leaves_below(
        &mut self,
        level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        config: &TreeConfiguration,
        pos: BlockPos,
        range: i32,
        y_offset: i32,
        large: bool,
        hanging_leaves_chance: f32,
        hanging_leaves_extension_chance: f32,
    );
}

pub struct CherryFoliagePlacer {
    pub radius: Box<dyn IntProvider>,
    pub offset: Box<dyn IntProvider>,
    pub height: Box<dyn IntProvider>,
    pub wide_bottom_layer_hole_chance: f32,
    pub corner_hole_chance: f32,
    pub hanging_leaves_chance: f32,
    pub hanging_leaves_extension_chance: f32,
}

impl Debug for CherryFoliagePlacer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CherryFoliagePlacer")
            .field("radius", &self.radius)
            .field("offset", &self.offset)
            .field("height", &self.height)
            .field(
                "wide_bottom_layer_hole_chance",
                &self.wide_bottom_layer_hole_chance,
            )
            .field("corner_hole_chance", &self.corner_hole_chance)
            .field("hanging_leaves_chance", &self.hanging_leaves_chance)
            .field(
                "hanging_leaves_extension_chance",
                &self.hanging_leaves_extension_chance,
            )
            .finish()
    }
}

impl CherryFoliagePlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.CherryFoliagePlacer#<init>
    pub fn new(
        radius: Box<dyn IntProvider>,
        offset: Box<dyn IntProvider>,
        height: Box<dyn IntProvider>,
        wide_bottom_layer_hole_chance: f32,
        corner_hole_chance: f32,
        hanging_leaves_chance: f32,
        hanging_leaves_extension_chance: f32,
    ) -> Self {
        Self {
            radius,
            offset,
            height,
            wide_bottom_layer_hole_chance,
            corner_hole_chance,
            hanging_leaves_chance,
            hanging_leaves_extension_chance,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.CherryFoliagePlacer#type
    pub fn foliage_placer_type(&self) -> FoliagePlacerType {
        FoliagePlacerType::CherryFoliagePlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.CherryFoliagePlacer#createFoliage
    pub fn create_foliage(
        &self,
        row_placer: &mut dyn FoliageRowPlacer,
        level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        config: &TreeConfiguration,
        _max_free_tree_height: i32,
        attachment: &FoliageAttachment,
        foliage_height: i32,
        foliage_radius: i32,
        offset: i32,
    ) {
        let large: bool = attachment.double_trunk();
        let pos: BlockPos = attachment.pos().above(offset);
        let radius: i32 = foliage_radius
            .wrapping_add(attachment.radius_offset())
            .wrapping_sub(1);
        row_placer.place_leaves_row(
            level,
            setter,
            random,
            config,
            pos,
            radius.wrapping_sub(2),
            foliage_height.wrapping_sub(3),
            large,
        );
        row_placer.place_leaves_row(
            level,
            setter,
            random,
            config,
            pos,
            radius.wrapping_sub(1),
            foliage_height.wrapping_sub(4),
            large,
        );

        let mut y: i32 = foliage_height.wrapping_sub(5);
        while y >= 0 {
            row_placer.place_leaves_row(level, setter, random, config, pos, radius, y, large);
            y = y.wrapping_sub(1);
        }

        row_placer.place_leaves_row_with_hanging_leaves_below(
            level,
            setter,
            random,
            config,
            pos,
            radius,
            -1,
            large,
            self.hanging_leaves_chance,
            self.hanging_leaves_extension_chance,
        );
        row_placer.place_leaves_row_with_hanging_leaves_below(
            level,
            setter,
            random,
            config,
            pos,
            radius.wrapping_sub(1),
            -2,
            large,
            self.hanging_leaves_chance,
            self.hanging_leaves_extension_chance,
        );
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.CherryFoliagePlacer#foliageHeight
    pub fn foliage_height(
        &self,
        random: &mut dyn RandomSource,
        _height: i32,
        _config: &TreeConfiguration,
    ) -> i32 {
        self.height.sample(random)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.CherryFoliagePlacer#shouldSkipLocation
    pub fn should_skip_location(
        &self,
        random: &mut dyn RandomSource,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        range: i32,
        _large: bool,
    ) -> bool {
        if local_y == -1
            && (local_x == range || local_z == range)
            && random.next_float() < self.wide_bottom_layer_hole_chance
        {
            return true;
        }

        let is_corner: bool = local_x == range && local_z == range;
        let is_wide: bool = range > 2;
        if is_wide {
            is_corner
                || (local_x.wrapping_add(local_z) > range.wrapping_mul(2).wrapping_sub(2)
                    && random.next_float() < self.corner_hole_chance)
        } else {
            is_corner && random.next_float() < self.corner_hole_chance
        }
    }
}

/// Default base-FoliagePlacer row placement implementation using CherryFoliagePlacer's skip logic.
pub struct DefaultFoliageRowPlacer<'a> {
    pub placer: &'a CherryFoliagePlacer,
}

impl<'a> DefaultFoliageRowPlacer<'a> {
    pub fn new(placer: &'a CherryFoliagePlacer) -> Self {
        Self { placer }
    }

    fn should_skip_location_signed(
        &self,
        random: &mut dyn RandomSource,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        range: i32,
        large: bool,
    ) -> bool {
        let abs_x = if local_x < 0 {
            local_x.wrapping_neg()
        } else {
            local_x
        };
        let abs_z = if local_z < 0 {
            local_z.wrapping_neg()
        } else {
            local_z
        };
        self.placer
            .should_skip_location(random, abs_x, local_y, abs_z, range, large)
    }
}

impl FoliageRowPlacer for DefaultFoliageRowPlacer<'_> {
    fn place_leaves_row(
        &mut self,
        _level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        _config: &TreeConfiguration,
        pos: BlockPos,
        range: i32,
        y_offset: i32,
        large: bool,
    ) {
        let i: i32 = if large { 1 } else { 0 };
        let mut dx: i32 = -range;
        while dx <= range.wrapping_add(i) {
            let mut dz: i32 = -range;
            while dz <= range.wrapping_add(i) {
                if !self.should_skip_location_signed(random, dx, y_offset, dz, range, large) {
                    setter.place_leaf(pos.offset(dx, y_offset, dz));
                }
                dz = dz.wrapping_add(1);
            }
            dx = dx.wrapping_add(1);
        }
    }

    fn place_leaves_row_with_hanging_leaves_below(
        &mut self,
        level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        config: &TreeConfiguration,
        pos: BlockPos,
        range: i32,
        y_offset: i32,
        large: bool,
        hanging_leaves_chance: f32,
        hanging_leaves_extension_chance: f32,
    ) {
        self.place_leaves_row(level, setter, random, config, pos, range, y_offset, large);
        let i: i32 = if large { 1 } else { 0 };
        let mut dx: i32 = -range;
        while dx <= range.wrapping_add(i) {
            let mut dz: i32 = -range;
            while dz <= range.wrapping_add(i) {
                if !self.should_skip_location_signed(random, dx, y_offset, dz, range, large) {
                    let below1 = pos.offset(dx, y_offset.wrapping_sub(1), dz);
                    setter.place_leaf(below1);
                    if random.next_float() < hanging_leaves_chance {
                        setter.place_hanging_leaf(below1);
                    }
                    if random.next_float() < hanging_leaves_extension_chance {
                        let below2 = pos.offset(dx, y_offset.wrapping_sub(2), dz);
                        setter.place_hanging_leaf(below2);
                    }
                }
                dz = dz.wrapping_add(1);
            }
            dx = dx.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct ConstRandom(f32);
    impl RandomSource for ConstRandom {
        fn next_float(&mut self) -> f32 {
            self.0
        }
    }

    #[derive(Debug)]
    struct SeqRandom {
        values: Vec<f32>,
        idx: usize,
    }
    impl SeqRandom {
        fn new(values: Vec<f32>) -> Self {
            Self { values, idx: 0 }
        }
    }
    impl RandomSource for SeqRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.values[self.idx];
            self.idx += 1;
            v
        }
    }

    #[derive(Debug)]
    struct ConstInt(i32);
    impl IntProvider for ConstInt {
        fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
            self.0
        }
    }

    struct NoopLevel;
    impl LevelSimulatedReader for NoopLevel {}

    struct NoopSetter;
    impl FoliageSetter for NoopSetter {
        fn place_leaf(&mut self, _pos: BlockPos) {}
        fn place_hanging_leaf(&mut self, _pos: BlockPos) {}
    }

    #[derive(Default)]
    struct RecordingPlacer {
        rows: Vec<(BlockPos, i32, i32, bool)>,
        hanging_rows: Vec<(BlockPos, i32, i32, bool, f32, f32)>,
    }
    impl FoliageRowPlacer for RecordingPlacer {
        fn place_leaves_row(
            &mut self,
            _level: &dyn LevelSimulatedReader,
            _setter: &mut dyn FoliageSetter,
            _random: &mut dyn RandomSource,
            _config: &TreeConfiguration,
            pos: BlockPos,
            range: i32,
            y_offset: i32,
            large: bool,
        ) {
            self.rows.push((pos, range, y_offset, large));
        }

        fn place_leaves_row_with_hanging_leaves_below(
            &mut self,
            _level: &dyn LevelSimulatedReader,
            _setter: &mut dyn FoliageSetter,
            _random: &mut dyn RandomSource,
            _config: &TreeConfiguration,
            pos: BlockPos,
            range: i32,
            y_offset: i32,
            large: bool,
            hanging_leaves_chance: f32,
            hanging_leaves_extension_chance: f32,
        ) {
            self.hanging_rows.push((
                pos,
                range,
                y_offset,
                large,
                hanging_leaves_chance,
                hanging_leaves_extension_chance,
            ));
        }
    }

    fn make_placer(
        wide_bottom: f32,
        corner: f32,
        hanging: f32,
        ext: f32,
    ) -> CherryFoliagePlacer {
        CherryFoliagePlacer::new(
            Box::new(ConstInt(0)),
            Box::new(ConstInt(0)),
            Box::new(ConstInt(5)),
            wide_bottom,
            corner,
            hanging,
            ext,
        )
    }

    #[test]
    fn should_skip_location_wide_bottom_hole_and_rng_consumption() {
        let placer = make_placer(0.5, 0.0, 0.0, 0.0);
        let mut rng = ConstRandom(0.4);
        assert!(placer.should_skip_location(&mut rng, 3, -1, 0, 3, false));

        let mut rng = ConstRandom(0.6);
        assert!(!placer.should_skip_location(&mut rng, 3, -1, 0, 3, false));

        // y != -1: first rule does not consume RNG
        let mut rng = SeqRandom::new(vec![]);
        assert!(!placer.should_skip_location(&mut rng, 3, 0, 0, 3, false));
    }

    #[test]
    fn should_skip_location_corner_logic_wide_and_narrow() {
        let placer = make_placer(0.0, 0.5, 0.0, 0.0);
        let mut rng = SeqRandom::new(vec![]);
        assert!(placer.should_skip_location(&mut rng, 4, 0, 4, 4, false));

        let mut rng = ConstRandom(0.3);
        assert!(placer.should_skip_location(&mut rng, 3, 0, 4, 4, false));
        let mut rng = ConstRandom(0.7);
        assert!(!placer.should_skip_location(&mut rng, 3, 0, 4, 4, false));

        let narrow = make_placer(0.0, 0.5, 0.0, 0.0);
        let mut rng = ConstRandom(0.3);
        assert!(narrow.should_skip_location(&mut rng, 2, 0, 2, 2, false));
        let mut rng = SeqRandom::new(vec![]);
        assert!(!narrow.should_skip_location(&mut rng, 1, 0, 2, 2, false));
    }

    #[test]
    fn should_skip_location_overflow_negative_and_float_bits() {
        let placer = make_placer(0.0, 1.0, 0.0, 0.0);
        let mut rng = ConstRandom(0.0);
        assert!(placer.should_skip_location(&mut rng, -1, 0, -1, -1, false));

        // Wrapping overflow: no RNG consumed because comparison is false
        let mut rng = SeqRandom::new(vec![]);
        let result = placer.should_skip_location(&mut rng, i32::MAX, 0, 1, i32::MAX, false);
        assert!(!result);

        // Float bits: 0.0 < 0.0 is false, NaN comparisons are false
        let zero_placer = make_placer(0.0, 0.0, 0.0, 0.0);
        let mut rng = ConstRandom(0.0);
        assert!(!zero_placer.should_skip_location(&mut rng, 3, -1, 0, 3, false));

        let nan_placer = make_placer(f32::NAN, f32::NAN, 0.0, 0.0);
        let mut rng = ConstRandom(0.5);
        assert!(!nan_placer.should_skip_location(&mut rng, 3, -1, 0, 3, false));

        let neg_zero_placer = make_placer(-0.0_f32, 0.0, 0.0, 0.0);
        let mut rng = ConstRandom(0.0);
        assert!(!neg_zero_placer.should_skip_location(&mut rng, 3, -1, 0, 3, false));
    }

    #[test]
    fn foliage_height_samples_provider_and_supports_negative() {
        let placer = CherryFoliagePlacer::new(
            Box::new(ConstInt(0)),
            Box::new(ConstInt(0)),
            Box::new(ConstInt(7)),
            0.0,
            0.0,
            0.0,
            0.0,
        );
        let mut rng = ConstRandom(0.0);
        assert_eq!(placer.foliage_height(&mut rng, 10, &TreeConfiguration), 7);

        let neg = CherryFoliagePlacer::new(
            Box::new(ConstInt(0)),
            Box::new(ConstInt(0)),
            Box::new(ConstInt(-3)),
            0.0,
            0.0,
            0.0,
            0.0,
        );
        let mut rng = ConstRandom(0.0);
        assert_eq!(neg.foliage_height(&mut rng, 0, &TreeConfiguration), -3);
    }

    #[test]
    fn create_foliage_call_pattern_and_overflow() {
        let placer = make_placer(0.0, 0.0, 0.3, 0.6);
        let level = NoopLevel;
        let mut setter = NoopSetter;
        let mut rng = ConstRandom(0.0);
        let config = TreeConfiguration;

        let mut rec = RecordingPlacer::default();
        let attachment = FoliageAttachment::new(BlockPos::new(10, 20, 30), 1, false);
        placer.create_foliage(
            &mut rec, &level, &mut setter, &mut rng, &config, 0, &attachment, 6, 4, 2,
        );

        let pos = BlockPos::new(10, 22, 30);
        assert_eq!(
            rec.rows,
            vec![
                (pos, 2, 3, false), // radius-2, height-3
                (pos, 3, 2, false), // radius-1, height-4
                (pos, 4, 1, false), // height-5 down to 0
                (pos, 4, 0, false),
            ]
        );
        assert_eq!(
            rec.hanging_rows,
            vec![
                (pos, 4, -1, false, 0.3, 0.6),
                (pos, 3, -2, false, 0.3, 0.6),
            ]
        );

        // Overflow/wrapping behavior in position and radius arithmetic
        let mut rec2 = RecordingPlacer::default();
        let attachment2 = FoliageAttachment::new(BlockPos::new(0, i32::MAX, 0), i32::MAX, true);
        placer.create_foliage(
            &mut rec2, &level, &mut setter, &mut rng, &config, 0, &attachment2, 5, i32::MAX, 1,
        );
        assert_eq!(rec2.rows[0].0, BlockPos::new(0, i32::MIN, 0));
        let expected_radius = i32::MAX.wrapping_add(i32::MAX).wrapping_sub(1);
        assert_eq!(rec2.rows[0].1, expected_radius.wrapping_sub(2));
        assert!(rec2.rows[0].3);
    }

    #[test]
    fn foliage_placer_type_is_cherry() {
        let placer = make_placer(0.0, 0.0, 0.0, 0.0);
        assert_eq!(
            placer.foliage_placer_type(),
            FoliagePlacerType::CherryFoliagePlacer
        );
    }
}
