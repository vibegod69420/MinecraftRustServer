use std::sync::LazyLock;

/// Minimal local representation of `net.minecraft.world.level.chunk.status.ChunkStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkStatus {
    Empty,
    StructureStarts,
    StructureReferences,
    Biomes,
    Noise,
    Surface,
    Carvers,
    Features,
    InitializeLight,
    Light,
    Spawn,
    Full,
}

impl ChunkStatus {
    /// Returns the Java ordinal/index used by chunk status pyramids.
    pub fn get_index(self) -> i32 {
        self as i32
    }
}

/// Minimal local representation of chunk status tasks referenced by the pyramid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkTask {
    GenerateStructureStarts,
    GenerateStructureReferences,
    GenerateBiomes,
    GenerateNoise,
    GenerateSurface,
    GenerateCarvers,
    GenerateFeatures,
    InitializeLight,
    Light,
    GenerateSpawn,
    Full,
    LoadStructureStarts,
}

/// Minimal local representation of `net.minecraft.world.level.chunk.status.ChunkStep`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkStep {
    pub target_status: ChunkStatus,
    pub requirements: Vec<(ChunkStatus, i32)>,
    pub block_state_write_radius: i32,
    pub task: Option<ChunkTask>,
}

/// Minimal local representation of `net.minecraft.world.level.chunk.status.ChunkStep$Builder`.
#[derive(Debug, Clone)]
pub struct ChunkStepBuilder {
    status: ChunkStatus,
    requirements: Vec<(ChunkStatus, i32)>,
    block_state_write_radius: i32,
    task: Option<ChunkTask>,
}

impl ChunkStepBuilder {
    pub fn new(status: ChunkStatus) -> Self {
        Self {
            status,
            requirements: Vec::new(),
            block_state_write_radius: 0,
            task: None,
        }
    }

    pub fn new_with_parent(status: ChunkStatus, _parent: &ChunkStep) -> Self {
        // Parent is accepted to match the Java builder shape used by ChunkPyramid.
        Self::new(status)
    }

    pub fn add_requirement(mut self, status: ChunkStatus, radius: i32) -> Self {
        self.requirements.push((status, radius));
        self
    }

    pub fn block_state_write_radius(mut self, radius: i32) -> Self {
        self.block_state_write_radius = radius;
        self
    }

    pub fn set_task(mut self, task: ChunkTask) -> Self {
        self.task = Some(task);
        self
    }

    pub fn build(self) -> ChunkStep {
        ChunkStep {
            target_status: self.status,
            requirements: self.requirements,
            block_state_write_radius: self.block_state_write_radius,
            task: self.task,
        }
    }
}

/// Port of `net.minecraft.world.level.chunk.status.ChunkPyramid`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkPyramid {
    steps: Vec<ChunkStep>,
}

impl ChunkPyramid {
    // JAVA: net.minecraft.world.level.chunk.status.ChunkPyramid#steps
    pub fn steps(&self) -> &[ChunkStep] {
        &self.steps
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkPyramid#getStepTo
    pub fn get_step_to(&self, status: ChunkStatus) -> &ChunkStep {
        let index = status.get_index() as usize;
        &self.steps[index]
    }
}

/// Port of `net.minecraft.world.level.chunk.status.ChunkPyramid$Builder`.
#[derive(Debug, Default)]
pub struct ChunkPyramidBuilder {
    steps: Vec<ChunkStep>,
}

impl ChunkPyramidBuilder {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkPyramid$Builder#build
    pub fn build(self) -> ChunkPyramid {
        ChunkPyramid { steps: self.steps }
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkPyramid$Builder#step
    pub fn step(
        mut self,
        status: ChunkStatus,
        op: impl FnOnce(ChunkStepBuilder) -> ChunkStepBuilder,
    ) -> Self {
        let builder = if self.steps.is_empty() {
            ChunkStepBuilder::new(status)
        } else {
            let parent = self
                .steps
                .last()
                .expect("ChunkPyramidBuilder steps should not be empty here");
            ChunkStepBuilder::new_with_parent(status, parent)
        };
        self.steps.push(op(builder).build());
        self
    }
}

pub static GENERATION_PYRAMID: LazyLock<ChunkPyramid> = LazyLock::new(|| {
    ChunkPyramidBuilder::new()
        .step(ChunkStatus::Empty, |b| b)
        .step(ChunkStatus::StructureStarts, |b| {
            b.set_task(ChunkTask::GenerateStructureStarts)
        })
        .step(ChunkStatus::StructureReferences, |b| {
            b.add_requirement(ChunkStatus::StructureStarts, 8)
                .set_task(ChunkTask::GenerateStructureReferences)
        })
        .step(ChunkStatus::Biomes, |b| {
            b.add_requirement(ChunkStatus::StructureStarts, 8)
                .set_task(ChunkTask::GenerateBiomes)
        })
        .step(ChunkStatus::Noise, |b| {
            b.add_requirement(ChunkStatus::StructureStarts, 8)
                .add_requirement(ChunkStatus::Biomes, 1)
                .block_state_write_radius(0)
                .set_task(ChunkTask::GenerateNoise)
        })
        .step(ChunkStatus::Surface, |b| {
            b.add_requirement(ChunkStatus::StructureStarts, 8)
                .add_requirement(ChunkStatus::Biomes, 1)
                .block_state_write_radius(0)
                .set_task(ChunkTask::GenerateSurface)
        })
        .step(ChunkStatus::Carvers, |b| {
            b.add_requirement(ChunkStatus::StructureStarts, 8)
                .block_state_write_radius(0)
                .set_task(ChunkTask::GenerateCarvers)
        })
        .step(ChunkStatus::Features, |b| {
            b.add_requirement(ChunkStatus::StructureStarts, 8)
                .add_requirement(ChunkStatus::Carvers, 1)
                .block_state_write_radius(1)
                .set_task(ChunkTask::GenerateFeatures)
        })
        .step(ChunkStatus::InitializeLight, |b| {
            b.set_task(ChunkTask::InitializeLight)
        })
        .step(ChunkStatus::Light, |b| {
            b.add_requirement(ChunkStatus::InitializeLight, 1)
                .set_task(ChunkTask::Light)
        })
        .step(ChunkStatus::Spawn, |b| {
            b.add_requirement(ChunkStatus::Biomes, 1)
                .set_task(ChunkTask::GenerateSpawn)
        })
        .step(ChunkStatus::Full, |b| b.set_task(ChunkTask::Full))
        .build()
});

pub static LOADING_PYRAMID: LazyLock<ChunkPyramid> = LazyLock::new(|| {
    ChunkPyramidBuilder::new()
        .step(ChunkStatus::Empty, |b| b)
        .step(ChunkStatus::StructureStarts, |b| {
            b.set_task(ChunkTask::LoadStructureStarts)
        })
        .step(ChunkStatus::StructureReferences, |b| b)
        .step(ChunkStatus::Biomes, |b| b)
        .step(ChunkStatus::Noise, |b| b)
        .step(ChunkStatus::Surface, |b| b)
        .step(ChunkStatus::Carvers, |b| b)
        .step(ChunkStatus::Features, |b| b)
        .step(ChunkStatus::InitializeLight, |b| {
            b.set_task(ChunkTask::InitializeLight)
        })
        .step(ChunkStatus::Light, |b| {
            b.add_requirement(ChunkStatus::InitializeLight, 1)
                .set_task(ChunkTask::Light)
        })
        .step(ChunkStatus::Spawn, |b| b)
        .step(ChunkStatus::Full, |b| b.set_task(ChunkTask::Full))
        .build()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_pyramid_structure() {
        let p = &*GENERATION_PYRAMID;
        assert_eq!(p.steps().len(), 12);
        assert_eq!(
            p.get_step_to(ChunkStatus::Empty).target_status,
            ChunkStatus::Empty
        );
        assert_eq!(
            p.get_step_to(ChunkStatus::Full).target_status,
            ChunkStatus::Full
        );

        let features = p.get_step_to(ChunkStatus::Features);
        assert_eq!(features.block_state_write_radius, 1);
        assert_eq!(
            features.requirements,
            vec![
                (ChunkStatus::StructureStarts, 8),
                (ChunkStatus::Carvers, 1)
            ]
        );
        assert_eq!(features.task, Some(ChunkTask::GenerateFeatures));
    }

    #[test]
    fn loading_pyramid_differences() {
        let p = &*LOADING_PYRAMID;
        assert_eq!(p.steps().len(), 12);

        let structure_starts = p.get_step_to(ChunkStatus::StructureStarts);
        assert_eq!(structure_starts.task, Some(ChunkTask::LoadStructureStarts));
        assert!(structure_starts.requirements.is_empty());

        let biomes = p.get_step_to(ChunkStatus::Biomes);
        assert_eq!(biomes.task, None);
        assert!(biomes.requirements.is_empty());
        assert_eq!(biomes.block_state_write_radius, 0);
    }

    #[test]
    fn get_step_to_all_statuses_match_indices() {
        let p = &*GENERATION_PYRAMID;
        let statuses = [
            ChunkStatus::Empty,
            ChunkStatus::StructureStarts,
            ChunkStatus::StructureReferences,
            ChunkStatus::Biomes,
            ChunkStatus::Noise,
            ChunkStatus::Surface,
            ChunkStatus::Carvers,
            ChunkStatus::Features,
            ChunkStatus::InitializeLight,
            ChunkStatus::Light,
            ChunkStatus::Spawn,
            ChunkStatus::Full,
        ];
        for (i, status) in statuses.iter().copied().enumerate() {
            let step = p.get_step_to(status);
            assert_eq!(step.target_status, status);
            assert_eq!(status.get_index() as usize, i);
        }
    }

    #[test]
    fn integer_semantics_wrapping_negative_and_shifts() {
        // Java int semantics: wrapping arithmetic, negative inputs, masked shifts.
        let idx: i32 = ChunkStatus::Empty.get_index();
        assert_eq!(idx.wrapping_sub(1), -1_i32);
        assert_eq!(i32::MAX.wrapping_add(1), i32::MIN);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
        assert_eq!((-8_i32).wrapping_mul(3_i32), -24_i32);

        let shift_mask_32: u32 = 32;
        assert_eq!(1_i32.wrapping_shl(shift_mask_32 & 31), 1_i32);
        assert_eq!((-16_i32) >> (4_i32 as u32 & 31), -1_i32);

        let shift_mask_64: u32 = 64;
        let v: i64 = 1_i64;
        assert_eq!(v.wrapping_shl(shift_mask_64 & 63), 1_i64);
    }

    #[test]
    fn float_bits_and_operation_order_discipline() {
        // Float bit patterns and operation order must be preserved exactly.
        let neg_zero = (-0.0_f32).to_bits();
        let pos_zero = 0.0_f32.to_bits();
        assert_ne!(neg_zero, pos_zero);

        let nan_bits: u32 = 0x7FC0_0001;
        let nan = f32::from_bits(nan_bits);
        assert!(nan.is_nan());
        assert_eq!(nan.to_bits(), nan_bits);

        let a = 1.0_f64;
        let b = 2.0_f64;
        let c = 3.0_f64;
        let left = (a / b) * c;
        let right = a / (b * c);
        assert_ne!(left.to_bits(), right.to_bits());
        assert_eq!(left.to_bits(), ((1.0_f64 / 2.0_f64) * 3.0_f64).to_bits());
    }
}
