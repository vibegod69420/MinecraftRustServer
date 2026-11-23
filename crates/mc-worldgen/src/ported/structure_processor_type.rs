//! Port of `net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorType`.
//!
//! Codec and registry wiring are represented as lightweight placeholders because
//! the full Mojang serialization/registry infrastructure is not available in this
//! self-contained module.

use std::fmt::Debug;
use std::marker::PhantomData;

/// Placeholder for a Mojang `MapCodec`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MapCodecPlaceholder;

/// Placeholder for a Mojang `Codec`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CodecPlaceholder;

/// Placeholder for a Mojang `Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderPlaceholder<T>(pub T);

/// Minimal placeholder for `StructureProcessor`.
///
/// The real processor hierarchy is not ported here; this trait exists so
/// `StructureProcessorType` can preserve its generic registration shape.
// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessor
pub trait StructureProcessor: Debug {
    /// Returns the processor type descriptor.
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessor#getType
    fn get_type(&self) -> StructureProcessorType;
}

/// Minimal placeholder for `StructureProcessorList`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructureProcessorList {
    /// Placeholder for the wrapped processor list.
    pub processors: Vec<String>,
}

impl StructureProcessorList {
    /// Creates a placeholder processor list from names.
    pub fn new(processors: Vec<String>) -> Self {
        Self { processors }
    }

    /// Returns the wrapped list.
    pub fn list(&self) -> &[String] {
        &self.processors
    }
}

/// Port of `StructureProcessorType<P extends StructureProcessor>`.
///
/// The Java interface is modeled as a type descriptor carrying the registered
/// name. Codec fields are exposed as placeholders to keep the module self-contained.
// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructureProcessorType {
    /// Registered processor type name.
    pub name: &'static str,
}

/// `Codec<StructureProcessor>` placeholder.
pub const SINGLE_CODEC: CodecPlaceholder = CodecPlaceholder;
/// `Codec<StructureProcessorList>` placeholder.
pub const LIST_OBJECT_CODEC: CodecPlaceholder = CodecPlaceholder;
/// `Codec<StructureProcessorList>` placeholder.
pub const DIRECT_CODEC: CodecPlaceholder = CodecPlaceholder;
/// `Codec<Holder<StructureProcessorList>>` placeholder.
pub const LIST_CODEC: CodecPlaceholder = CodecPlaceholder;

/// `BLOCK_IGNORE` processor type.
pub const BLOCK_IGNORE: StructureProcessorType = StructureProcessorType {
    name: "block_ignore",
};
/// `BLOCK_ROT` processor type.
pub const BLOCK_ROT: StructureProcessorType = StructureProcessorType { name: "block_rot" };
/// `GRAVITY` processor type.
pub const GRAVITY: StructureProcessorType = StructureProcessorType { name: "gravity" };
/// `JIGSAW_REPLACEMENT` processor type.
pub const JIGSAW_REPLACEMENT: StructureProcessorType = StructureProcessorType {
    name: "jigsaw_replacement",
};
/// `RULE` processor type.
pub const RULE: StructureProcessorType = StructureProcessorType { name: "rule" };
/// `NOP` processor type.
pub const NOP: StructureProcessorType = StructureProcessorType { name: "nop" };
/// `BLOCK_AGE` processor type.
pub const BLOCK_AGE: StructureProcessorType = StructureProcessorType { name: "block_age" };
/// `BLACKSTONE_REPLACE` processor type.
pub const BLACKSTONE_REPLACE: StructureProcessorType = StructureProcessorType {
    name: "blackstone_replace",
};
/// `LAVA_SUBMERGED_BLOCK` processor type.
pub const LAVA_SUBMERGED_BLOCK: StructureProcessorType = StructureProcessorType {
    name: "lava_submerged_block",
};
/// `PROTECTED_BLOCKS` processor type.
pub const PROTECTED_BLOCKS: StructureProcessorType = StructureProcessorType {
    name: "protected_blocks",
};
/// `CAPPED` processor type.
pub const CAPPED: StructureProcessorType = StructureProcessorType { name: "capped" };

impl StructureProcessorType {
    /// Returns the placeholder codec for this processor type.
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorType#codec
    pub fn codec<P: StructureProcessor>(&self) -> MapCodecPlaceholder {
        let _typed: PhantomData<P> = PhantomData;
        MapCodecPlaceholder
    }

    /// Registers a processor type by name.
    ///
    /// In Java this registers into `BuiltInRegistries.STRUCTURE_PROCESSOR`.
    /// This self-contained port returns the type descriptor.
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorType#register
    pub fn register<P: StructureProcessor>(name: &'static str, _codec: MapCodecPlaceholder) -> Self {
        let _typed: PhantomData<P> = PhantomData;
        Self { name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyProcessor;

    impl StructureProcessor for DummyProcessor {
        fn get_type(&self) -> StructureProcessorType {
            BLOCK_IGNORE
        }
    }

    #[test]
    fn register_returns_named_type() {
        let t = StructureProcessorType::register::<DummyProcessor>("custom", MapCodecPlaceholder);
        assert_eq!(t.name, "custom");
    }

    #[test]
    fn constants_match_java_names() {
        assert_eq!(BLOCK_IGNORE.name, "block_ignore");
        assert_eq!(BLOCK_ROT.name, "block_rot");
        assert_eq!(GRAVITY.name, "gravity");
        assert_eq!(JIGSAW_REPLACEMENT.name, "jigsaw_replacement");
        assert_eq!(RULE.name, "rule");
        assert_eq!(NOP.name, "nop");
        assert_eq!(BLOCK_AGE.name, "block_age");
        assert_eq!(BLACKSTONE_REPLACE.name, "blackstone_replace");
        assert_eq!(LAVA_SUBMERGED_BLOCK.name, "lava_submerged_block");
        assert_eq!(PROTECTED_BLOCKS.name, "protected_blocks");
        assert_eq!(CAPPED.name, "capped");
    }

    #[test]
    fn type_equality_hash_and_copy() {
        use std::collections::HashSet;
        let a = CAPPED;
        let b = a;
        let c = StructureProcessorType { name: "capped" };
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_ne!(CAPPED, NOP);

        let mut set = HashSet::new();
        set.insert(CAPPED);
        set.insert(StructureProcessorType { name: "capped" });
        set.insert(NOP);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn codec_placeholders_and_processor_trait() {
        let _single = SINGLE_CODEC;
        let _list_obj = LIST_OBJECT_CODEC;
        let _direct = DIRECT_CODEC;
        let _list = LIST_CODEC;
        let _codec: MapCodecPlaceholder = CAPPED.codec::<DummyProcessor>();

        let p = DummyProcessor;
        assert_eq!(p.get_type(), BLOCK_IGNORE);
    }

    #[test]
    fn processor_list_placeholder() {
        let list = StructureProcessorList::new(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(list.list(), &["a".to_string(), "b".to_string()]);
        let empty = StructureProcessorList::default();
        assert!(empty.list().is_empty());
    }
}
