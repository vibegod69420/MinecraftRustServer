// JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceSerializationContext
#[derive(Clone, Debug)]
pub struct StructurePieceSerializationContext {
   pub resource_manager: ResourceManager,
   pub registry_access: RegistryAccess,
   pub structure_template_manager: StructureTemplateManager,
}

// Self-contained stub types for compilation without external Minecraft classes
#[derive(Clone, Debug)]
pub struct ResourceManager;
#[derive(Clone, Debug)]
pub struct RegistryAccess;
#[derive(Clone, Debug)]
pub struct StructureTemplateManager;

// JAVA: net.minecraft.server.MinecraftServer
#[derive(Clone, Debug)]
pub struct MinecraftServer;

impl MinecraftServer {
   // JAVA: net.minecraft.server.MinecraftServer#getResourceManager
   pub fn get_resource_manager(&self) -> ResourceManager {
       ResourceManager
   }
   // JAVA: net.minecraft.server.MinecraftServer#registryAccess
   pub fn registry_access(&self) -> RegistryAccess {
       RegistryAccess
   }
   // JAVA: net.minecraft.server.MinecraftServer#getStructureManager
   pub fn get_structure_manager(&self) -> StructureTemplateManager {
       StructureTemplateManager
   }
}

// JAVA: net.minecraft.world.level.ServerLevel#getServer
#[derive(Clone, Debug)]
pub struct ServerLevel;

impl ServerLevel {
   // JAVA: net.minecraft.world.level.ServerLevel#getServer
   pub fn get_server(&self) -> &MinecraftServer {
       &MinecraftServer
   }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceSerializationContext#fromLevel
pub fn from_level(server_level: &ServerLevel) -> StructurePieceSerializationContext {
   let server = server_level.get_server();
   StructurePieceSerializationContext {
      resource_manager: server.get_resource_manager(),
      registry_access: server.registry_access(),
      structure_template_manager: server.get_structure_manager(),
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_struct_creation() {
      let ctx = StructurePieceSerializationContext {
         resource_manager: ResourceManager,
         registry_access: RegistryAccess,
         structure_template_manager: StructureTemplateManager,
      };
      assert!(matches!(ctx.resource_manager, ResourceManager));
      assert!(matches!(ctx.registry_access, RegistryAccess));
      assert!(matches!(ctx.structure_template_manager, StructureTemplateManager));
   }

   #[test]
   fn test_from_level() {
      let server_level = ServerLevel;
      let ctx = from_level(&server_level);
      assert!(matches!(ctx.registry_access, RegistryAccess));
   }

   #[test]
   fn test_wrapping_arithmetic() {
      // Java long wrapping semantics match Rust i64 wrapping
      let max: i64 = i64::MAX;
      let min: i64 = i64::MIN;
      assert_eq!(max.wrapping_add(1), min);
      assert_eq!(min.wrapping_sub(1), max);
      assert_eq!(max.wrapping_sub(1), max - 1);
      assert_eq!(min.wrapping_add(1), min + 1);
   }

   #[test]
   fn test_negative_wrapping() {
      let neg: i64 = -1;
      assert_eq!(neg.wrapping_add(1), 0);
      assert_eq!(neg.wrapping_sub(1), -2);
   }
}
