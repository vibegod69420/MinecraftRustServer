// JAVA: net.minecraft.util.profiling.jfr.event.StructureGenerationEvent#<init>
#[derive(Debug, Clone)]
pub struct StructureGenerationEvent {
   pub chunk_pos_x: i32,
   pub chunk_pos_z: i32,
   pub structure: String,
   pub level: String,
   pub success: bool,
}

impl StructureGenerationEvent {
   // JAVA: net.minecraft.util.profiling.jfr.event.StructureGenerationEvent#<init>(ChunkPos, Holder<Structure>, ResourceKey<Level>)
   pub fn new(chunk_pos_x: i32, chunk_pos_z: i32, structure: String, level: String, success: bool) -> Self {
      Self {
         chunk_pos_x,
         chunk_pos_z,
         structure,
         level,
         success,
      }
   }

   // JAVA: net.minecraft.util.profiling.jfr.event.StructureGenerationEvent.Fields#CHUNK_POS_X
   pub const CHUNK_POS_X: &str = "chunkPosX";
   // JAVA: net.minecraft.util.profiling.jfr.event.StructureGenerationEvent.Fields#CHUNK_POS_Z
   pub const CHUNK_POS_Z: &str = "chunkPosZ";
   // JAVA: net.minecraft.util.profiling.jfr.event.StructureGenerationEvent.Fields#STRUCTURE
   pub const STRUCTURE: &str = "structure";
   // JAVA: net.minecraft.util.profiling.jfr.event.StructureGenerationEvent.Fields#LEVEL
   pub const LEVEL: &str = "level";
   // JAVA: net.minecraft.util.profiling.jfr.event.StructureGenerationEvent.Fields#SUCCESS
   pub const SUCCESS: &str = "success";
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_construction_with_valid_values() {
      let event = StructureGenerationEvent::new(10, 20, "village".to_string(), "overworld".to_string(), true);
      assert_eq!(event.chunk_pos_x, 10);
      assert_eq!(event.chunk_pos_z, 20);
      assert_eq!(event.structure, "village");
      assert_eq!(event.level, "overworld");
      assert!(event.success);
   }

   #[test]
   fn test_construction_with_negative_chunk_pos() {
      let event = StructureGenerationEvent::new(-5, -15, "fortress".to_string(), "the_end".to_string(), false);
      assert_eq!(event.chunk_pos_x, -5);
      assert_eq!(event.chunk_pos_z, -15);
      assert!(!event.success);
   }

   #[test]
   fn test_constants() {
      assert_eq!(StructureGenerationEvent::CHUNK_POS_X, "chunkPosX");
      assert_eq!(StructureGenerationEvent::CHUNK_POS_Z, "chunkPosZ");
      assert_eq!(StructureGenerationEvent::STRUCTURE, "structure");
      assert_eq!(StructureGenerationEvent::LEVEL, "level");
      assert_eq!(StructureGenerationEvent::SUCCESS, "success");
   }
}
