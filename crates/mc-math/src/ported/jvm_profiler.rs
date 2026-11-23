use std::marker::PhantomData;
use std::path::PathBuf;

pub struct Environment;

pub struct ConnectionProtocol;

pub struct PacketType;

pub struct RegionStorageInfo;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

pub struct RegionFileVersion;

pub struct ResourceKey<T> {
    _marker: PhantomData<T>,
}

impl<T> ResourceKey<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

pub struct Level;

pub struct Holder<T> {
    _marker: PhantomData<T>,
}

impl<T> Holder<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

pub struct Structure;

pub trait ProfiledDuration {
    fn commit(&self);
}

#[derive(Clone, Copy)]
pub struct NoOpCommit;

impl ProfiledDuration for NoOpCommit {
    fn commit(&self) {}
}

pub trait JvmProfiler {
    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#start
    fn start(&self, environment: Environment) -> bool;

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#stop
    fn stop(&self) -> PathBuf;

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#isRunning
    fn is_running(&self) -> bool;

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#isAvailable
    fn is_available(&self) -> bool;

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onServerTick
    fn on_server_tick(&self, tick_time: f32);

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onClientTick
    fn on_client_tick(&self, tick_count: i32);

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onPacketReceived
    fn on_packet_received(
        &self,
        protocol: ConnectionProtocol,
        packet_type: PacketType,
        address: std::net::SocketAddr,
        size: i32,
    );

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onPacketSent
    fn on_packet_sent(
        &self,
        protocol: ConnectionProtocol,
        packet_type: PacketType,
        address: std::net::SocketAddr,
        size: i32,
    );

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onRegionFileRead
    fn on_region_file_read(
        &self,
        info: RegionStorageInfo,
        pos: ChunkPos,
        version: RegionFileVersion,
        size: i32,
    );

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onRegionFileWrite
    fn on_region_file_write(
        &self,
        info: RegionStorageInfo,
        pos: ChunkPos,
        version: RegionFileVersion,
        size: i32,
    );

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onWorldLoadedStarted
    fn on_world_loaded_started(&self) -> Box<dyn ProfiledDuration>;

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onChunkGenerate
    fn on_chunk_generate(
        &self,
        pos: ChunkPos,
        level: ResourceKey<Level>,
        generator: String,
    ) -> Option<Box<dyn ProfiledDuration>>;

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onStructureGenerate
    fn on_structure_generate(
        &self,
        pos: ChunkPos,
        level: ResourceKey<Level>,
        structure: Holder<Structure>,
    ) -> Box<dyn ProfiledDuration>;
}

#[derive(Clone, Copy, Default)]
pub struct NoOpProfiler;

impl JvmProfiler for NoOpProfiler {
    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#start
    fn start(&self, _environment: Environment) -> bool {
        false
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#stop
    fn stop(&self) -> PathBuf {
        panic!("Attempted to stop Flight Recorder, but it's not supported on this JVM")
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#isRunning
    fn is_running(&self) -> bool {
        false
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#isAvailable
    fn is_available(&self) -> bool {
        false
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onServerTick
    fn on_server_tick(&self, _tick_time: f32) {}

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onClientTick
    fn on_client_tick(&self, _tick_count: i32) {}

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onPacketReceived
    fn on_packet_received(
        &self,
        _protocol: ConnectionProtocol,
        _packet_type: PacketType,
        _address: std::net::SocketAddr,
        _size: i32,
    ) {
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onPacketSent
    fn on_packet_sent(
        &self,
        _protocol: ConnectionProtocol,
        _packet_type: PacketType,
        _address: std::net::SocketAddr,
        _size: i32,
    ) {
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onRegionFileRead
    fn on_region_file_read(
        &self,
        _info: RegionStorageInfo,
        _pos: ChunkPos,
        _version: RegionFileVersion,
        _size: i32,
    ) {
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onRegionFileWrite
    fn on_region_file_write(
        &self,
        _info: RegionStorageInfo,
        _pos: ChunkPos,
        _version: RegionFileVersion,
        _size: i32,
    ) {
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onWorldLoadedStarted
    fn on_world_loaded_started(&self) -> Box<dyn ProfiledDuration> {
        Box::new(NoOpCommit)
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onChunkGenerate
    fn on_chunk_generate(
        &self,
        _pos: ChunkPos,
        _level: ResourceKey<Level>,
        _generator: String,
    ) -> Option<Box<dyn ProfiledDuration>> {
        None
    }

    // JAVA: net.minecraft.util.profiling.jfr.JvmProfiler#onStructureGenerate
    fn on_structure_generate(
        &self,
        _pos: ChunkPos,
        _level: ResourceKey<Level>,
        _structure: Holder<Structure>,
    ) -> Box<dyn ProfiledDuration> {
        Box::new(NoOpCommit)
    }
}

pub const INSTANCE: NoOpProfiler = NoOpProfiler;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_start_returns_false() {
        assert!(!INSTANCE.start(Environment));
    }

    #[test]
    fn noop_state_flags_are_false() {
        assert!(!INSTANCE.is_running());
        assert!(!INSTANCE.is_available());
    }

    #[test]
    fn noop_stop_panics() {
        let result = std::panic::catch_unwind(|| INSTANCE.stop());
        assert!(result.is_err());
    }

    #[test]
    fn noop_duration_callbacks_behave() {
        let world = INSTANCE.on_world_loaded_started();
        world.commit();

        let chunk = INSTANCE.on_chunk_generate(ChunkPos { x: 0, z: 0 }, ResourceKey::new(), String::new());
        assert!(chunk.is_none());

        let structure = INSTANCE.on_structure_generate(ChunkPos { x: 1, z: -1 }, ResourceKey::new(), Holder::new());
        structure.commit();
    }

    #[test]
    fn noop_packet_and_region_callbacks_do_not_panic() {
        let address = "127.0.0.1:25565".parse().unwrap();
        INSTANCE.on_packet_received(ConnectionProtocol, PacketType, address, 16);
        INSTANCE.on_packet_sent(ConnectionProtocol, PacketType, address, 32);
        INSTANCE.on_region_file_read(RegionStorageInfo, ChunkPos { x: 2, z: 3 }, RegionFileVersion, 64);
        INSTANCE.on_region_file_write(RegionStorageInfo, ChunkPos { x: 2, z: 3 }, RegionFileVersion, 64);
        INSTANCE.on_server_tick(1.0);
        INSTANCE.on_client_tick(20);
    }
}
