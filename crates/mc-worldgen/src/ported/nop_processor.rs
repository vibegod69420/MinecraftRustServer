use std::marker::PhantomData;

/// Minimal self-contained placeholder for StructureProcessorType.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureProcessorType {
    Nop,
}

/// Minimal self-contained placeholder for the StructureProcessor base.
pub trait StructureProcessor {
    fn get_type(&self) -> StructureProcessorType;
}

/// Minimal self-contained placeholder for Mojang MapCodec.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T>(PhantomData<T>);

impl<T> MapCodec<T> {
    pub const fn unit(_factory: fn() -> T) -> Self {
        Self(PhantomData)
    }
}

fn nop_instance() -> NopProcessor {
    NopProcessor::INSTANCE
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NopProcessor;

impl NopProcessor {
    pub const INSTANCE: NopProcessor = NopProcessor;
    pub const CODEC: MapCodec<NopProcessor> = MapCodec::unit(nop_instance);

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.NopProcessor#<init>
    const fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.NopProcessor#getType
    fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::Nop
    }
}

impl StructureProcessor for NopProcessor {
    fn get_type(&self) -> StructureProcessorType {
        NopProcessor::get_type(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_is_nop() {
        let p = NopProcessor::INSTANCE;
        assert_eq!(NopProcessor::get_type(&p), StructureProcessorType::Nop);
    }

    #[test]
    fn trait_get_type_returns_nop() {
        let p = NopProcessor;
        assert_eq!(StructureProcessor::get_type(&p), StructureProcessorType::Nop);
    }

    #[test]
    fn instance_equality_and_copy() {
        let a = NopProcessor::INSTANCE;
        let b = a;
        assert_eq!(a, b);
        assert_eq!(a, NopProcessor::new());
    }

    #[test]
    fn codec_placeholder_constructible() {
        let _codec: MapCodec<NopProcessor> = NopProcessor::CODEC;
        let _codec2 = MapCodec::<NopProcessor>::unit(nop_instance);
        let produced = nop_instance();
        assert_eq!(produced, NopProcessor::INSTANCE);
    }

    #[test]
    fn debug_format_contains_name() {
        let s = format!("{:?}", NopProcessor::INSTANCE);
        assert!(s.contains("NopProcessor"));
    }
}
