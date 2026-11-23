// JAVA: net.minecraft.util.profiling.jfr.event.ChunkGenerationEvent#<init>(ChunkPos, ResourceKey<Level>, String)
#[derive(Debug, Clone)]
pub struct ChunkGenerationEvent {
   pub world_pos_x: i32,
   pub world_pos_z: i32,
   pub chunk_pos_x: i32,
   pub chunk_pos_z: i32,
   pub target_status: String,
   pub level: String,
}

impl ChunkGenerationEvent {
   // JAVA: net.minecraft.util.profiling.jfr.event.ChunkGenerationEvent#<init>(ChunkPos, ResourceKey<Level>, String)
   pub fn new(world_pos_x: i32, world_pos_z: i32, chunk_pos_x: i32, chunk_pos_z: i32, target_status: String, level: String) -> Self {
      Self {
         world_pos_x,
         world_pos_z,
         chunk_pos_x,
         chunk_pos_z,
         target_status,
         level,
      }
   }

   // JAVA: net.minecraft.util.profiling.jfr.event.ChunkGenerationEvent.Fields#WORLD_POS_X
   pub const WORLD_POS_X: &str = "worldPosX";
   // JAVA: net.minecraft.util.profiling.jfr.event.ChunkGenerationEvent.Fields#WORLD_POS_Z
   pub const WORLD_POS_Z: &str = "worldPosZ";
   // JAVA: net.minecraft.util.profiling.jfr.event.ChunkGenerationEvent.Fields#CHUNK_POS_X
   pub const CHUNK_POS_X: &str = "chunkPosX";
   // JAVA: net.minecraft.util.profiling.jfr.event.ChunkGenerationEvent.Fields#CHUNK_POS_Z
   pub const CHUNK_POS_Z: &str = "chunkPosZ";
   // JAVA: net.minecraft.util.profiling.jfr.event.ChunkGenerationEvent.Fields#STATUS
   pub const STATUS: &str = "status";
   // JAVA: net.minecraft.util.profiling.jfr.event.ChunkGenerationEvent.Fields#LEVEL
   pub const LEVEL: &str = "level";
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_construction_with_sample_values() {
      let event = ChunkGenerationEvent::new(16, 64, 2, 3, "STARTED".to_string(), "world_nether".to_string());
      assert_eq!(event.world_pos_x, 16);
      assert_eq!(event.world_pos_z, 64);
      assert_eq!(event.chunk_pos_x, 2);
      assert_eq!(event.chunk_pos_z, 3);
      assert_eq!(event.target_status, "STARTED");
      assert_eq!(event.level, "world_nether");
   }

   #[test]
   fn test_construction_with_negative_coords() {
      let event = ChunkGenerationEvent::new(-32, -128, -1, -2, "CANCELLED".to_string(), "world".to_string());
      assert_eq!(event.world_pos_x, -32);
      assert_eq!(event.world_pos_z, -128);
      assert_eq!(event.chunk_pos_x, -1);
      assert_eq!(event.chunk_pos_z, -2);
      assert_eq!(event.target_status, "CANCELLED");
   }

   #[test]
   fn test_constants() {
      assert_eq!(ChunkGenerationEvent::WORLD_POS_X, "worldPosX");
      assert_eq!(ChunkGenerationEvent::WORLD_POS_Z, "worldPosZ");
      assert_eq!(ChunkGenerationEvent::CHUNK_POS_X, "chunkPosX");
      assert_eq!(ChunkGenerationEvent::CHUNK_POS_Z, "chunkPosZ");
      assert_eq!(ChunkGenerationEvent::STATUS, "status");
      assert_eq!(ChunkGenerationEvent::LEVEL, "level");
   }
}
