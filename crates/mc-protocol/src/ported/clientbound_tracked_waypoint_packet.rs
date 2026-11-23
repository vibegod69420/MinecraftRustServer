use std::fmt;

/// Minimal self-contained UUID representation matching Java `java.util.UUID` bit layout.
// JAVA: java.util.UUID#<init>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

/// Minimal self-contained 3D integer vector matching `net.minecraft.core.Vec3i`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Minimal self-contained chunk position matching `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// Minimal self-contained waypoint icon matching `net.minecraft.world.waypoints.Waypoint.Icon`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WaypointIcon {
    pub style: i32,
    pub color: i32,
}

/// Minimal self-contained tracked waypoint data matching `net.minecraft.world.waypoints.TrackedWaypoint`.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackedWaypointData {
    Empty,
    Vec3i(Vec3i),
    Chunk(ChunkPos),
    Azimuth(f32),
}

/// Minimal self-contained tracked waypoint matching `net.minecraft.world.waypoints.TrackedWaypoint`.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackedWaypoint {
    pub id: Uuid,
    pub icon: WaypointIcon,
    pub data: TrackedWaypointData,
}

impl TrackedWaypoint {
    // JAVA: net.minecraft.world.waypoints.TrackedWaypoint#empty
    pub fn empty(id: Uuid) -> Self {
        Self {
            id,
            icon: WaypointIcon::default(),
            data: TrackedWaypointData::Empty,
        }
    }

    // JAVA: net.minecraft.world.waypoints.TrackedWaypoint#setPosition
    pub fn set_position(id: Uuid, icon: WaypointIcon, pos: Vec3i) -> Self {
        Self {
            id,
            icon,
            data: TrackedWaypointData::Vec3i(pos),
        }
    }

    // JAVA: net.minecraft.world.waypoints.TrackedWaypoint#setChunk
    pub fn set_chunk(id: Uuid, icon: WaypointIcon, chunk_pos: ChunkPos) -> Self {
        Self {
            id,
            icon,
            data: TrackedWaypointData::Chunk(chunk_pos),
        }
    }

    // JAVA: net.minecraft.world.waypoints.TrackedWaypoint#setAzimuth
    pub fn set_azimuth(id: Uuid, icon: WaypointIcon, angle: f32) -> Self {
        Self {
            id,
            icon,
            data: TrackedWaypointData::Azimuth(angle),
        }
    }
}

/// Minimal packet type identifier matching `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType {
    pub id: &'static str,
}

impl PacketType {
    pub const CLIENTBOUND_WAYPOINT: Self = Self {
        id: "minecraft:waypoint",
    };
}

/// Minimal listener trait matching `net.minecraft.network.protocol.game.ClientGamePacketListener`.
pub trait ClientGamePacketListener {
    fn handle_waypoint(&mut self, packet: &ClientboundTrackedWaypointPacket);
}

/// Minimal manager trait matching `net.minecraft.world.waypoints.TrackedWaypointManager` / `WaypointManager`.
pub trait TrackedWaypointManager {
    fn track_waypoint(&mut self, waypoint: &TrackedWaypoint);
    fn untrack_waypoint(&mut self, waypoint: &TrackedWaypoint);
    fn update_waypoint(&mut self, waypoint: &TrackedWaypoint);
}

/// Port of `net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket.Operation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    Track,
    Untrack,
    Update,
}

impl Operation {
    const VALUES: [Operation; 3] = [Operation::Track, Operation::Untrack, Operation::Update];

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket$Operation#ordinal
    pub fn ordinal(self) -> i32 {
        match self {
            Operation::Track => 0,
            Operation::Untrack => 1,
            Operation::Update => 2,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket$Operation#BY_ID
    pub fn by_id(id: i32) -> Self {
        let len = Self::VALUES.len() as i32;
        // ByIdMap.OutOfBoundsStrategy.WRAP uses Math.floorMod semantics.
        let rem = id.wrapping_rem(len);
        let idx = rem.wrapping_add(len).wrapping_rem(len);
        Self::VALUES[idx as usize]
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket$Operation#action
    pub fn apply_action(self, manager: &mut dyn TrackedWaypointManager, waypoint: &TrackedWaypoint) {
        match self {
            Operation::Track => manager.track_waypoint(waypoint),
            Operation::Untrack => manager.untrack_waypoint(waypoint),
            Operation::Update => manager.update_waypoint(waypoint),
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Operation::Track => "TRACK",
            Operation::Untrack => "UNTRACK",
            Operation::Update => "UPDATE",
        };
        f.write_str(name)
    }
}

/// Port of `net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket`.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundTrackedWaypointPacket {
    pub operation: Operation,
    pub waypoint: TrackedWaypoint,
}

impl ClientboundTrackedWaypointPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#<init>
    pub fn new(operation: Operation, waypoint: TrackedWaypoint) -> Self {
        Self { operation, waypoint }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#removeWaypoint
    pub fn remove_waypoint(id: Uuid) -> Self {
        Self::new(Operation::Untrack, TrackedWaypoint::empty(id))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#addWaypointPosition
    pub fn add_waypoint_position(id: Uuid, icon: WaypointIcon, pos: Vec3i) -> Self {
        Self::new(Operation::Track, TrackedWaypoint::set_position(id, icon, pos))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#updateWaypointPosition
    pub fn update_waypoint_position(id: Uuid, icon: WaypointIcon, pos: Vec3i) -> Self {
        Self::new(
            Operation::Update,
            TrackedWaypoint::set_position(id, icon, pos),
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#addWaypointChunk
    pub fn add_waypoint_chunk(id: Uuid, icon: WaypointIcon, chunk_pos: ChunkPos) -> Self {
        Self::new(
            Operation::Track,
            TrackedWaypoint::set_chunk(id, icon, chunk_pos),
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#updateWaypointChunk
    pub fn update_waypoint_chunk(id: Uuid, icon: WaypointIcon, chunk_pos: ChunkPos) -> Self {
        Self::new(
            Operation::Update,
            TrackedWaypoint::set_chunk(id, icon, chunk_pos),
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#addWaypointAzimuth
    pub fn add_waypoint_azimuth(id: Uuid, icon: WaypointIcon, angle: f32) -> Self {
        Self::new(
            Operation::Track,
            TrackedWaypoint::set_azimuth(id, icon, angle),
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#updateWaypointAzimuth
    pub fn update_waypoint_azimuth(id: Uuid, icon: WaypointIcon, angle: f32) -> Self {
        Self::new(
            Operation::Update,
            TrackedWaypoint::set_azimuth(id, icon, angle),
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::CLIENTBOUND_WAYPOINT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_waypoint(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket#apply
    pub fn apply(&self, manager: &mut dyn TrackedWaypointManager) {
        self.operation.apply_action(manager, &self.waypoint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockManager {
        tracked: Vec<TrackedWaypoint>,
        untracked: Vec<TrackedWaypoint>,
        updated: Vec<TrackedWaypoint>,
    }

    impl TrackedWaypointManager for MockManager {
        fn track_waypoint(&mut self, waypoint: &TrackedWaypoint) {
            self.tracked.push(waypoint.clone());
        }

        fn untrack_waypoint(&mut self, waypoint: &TrackedWaypoint) {
            self.untracked.push(waypoint.clone());
        }

        fn update_waypoint(&mut self, waypoint: &TrackedWaypoint) {
            self.updated.push(waypoint.clone());
        }
    }

    struct MockListener {
        handled: Option<ClientboundTrackedWaypointPacket>,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_waypoint(&mut self, packet: &ClientboundTrackedWaypointPacket) {
            self.handled = Some(packet.clone());
        }
    }

    fn sample_uuid() -> Uuid {
        Uuid {
            most_significant_bits: -123_i64,
            least_significant_bits: 456_i64,
        }
    }

    fn sample_icon() -> WaypointIcon {
        WaypointIcon {
            style: 7,
            color: -1,
        }
    }

    #[test]
    fn operation_by_id_wrap_semantics() {
        assert_eq!(Operation::by_id(0), Operation::Track);
        assert_eq!(Operation::by_id(1), Operation::Untrack);
        assert_eq!(Operation::by_id(2), Operation::Update);
        assert_eq!(Operation::by_id(3), Operation::Track);
        assert_eq!(Operation::by_id(-1), Operation::Update);
        assert_eq!(Operation::by_id(-4), Operation::Update);

        // i32::MIN wrapping/floorMod behavior must not panic and must be deterministic.
        let min = Operation::by_id(i32::MIN);
        let len = 3_i32;
        let rem = i32::MIN.wrapping_rem(len);
        let idx = rem.wrapping_add(len).wrapping_rem(len);
        let expected = match idx {
            0 => Operation::Track,
            1 => Operation::Untrack,
            2 => Operation::Update,
            _ => panic!("invalid wrapped index"),
        };
        assert_eq!(min, expected);
    }

    #[test]
    fn operation_ordinal_roundtrip() {
        for op in [Operation::Track, Operation::Untrack, Operation::Update] {
            assert_eq!(Operation::by_id(op.ordinal()), op);
        }
    }

    #[test]
    fn factory_methods_build_expected_packets() {
        let id = sample_uuid();
        let icon = sample_icon();
        let pos = Vec3i { x: -10, y: 64, z: 999 };
        let chunk = ChunkPos { x: -3, z: 7 };

        let remove = ClientboundTrackedWaypointPacket::remove_waypoint(id);
        assert_eq!(remove.operation, Operation::Untrack);
        assert_eq!(remove.waypoint, TrackedWaypoint::empty(id));

        let add_pos = ClientboundTrackedWaypointPacket::add_waypoint_position(id, icon, pos);
        assert_eq!(add_pos.operation, Operation::Track);
        assert_eq!(add_pos.waypoint.data, TrackedWaypointData::Vec3i(pos));

        let upd_pos = ClientboundTrackedWaypointPacket::update_waypoint_position(id, icon, pos);
        assert_eq!(upd_pos.operation, Operation::Update);
        assert_eq!(upd_pos.waypoint.icon, icon);

        let add_chunk = ClientboundTrackedWaypointPacket::add_waypoint_chunk(id, icon, chunk);
        assert_eq!(add_chunk.operation, Operation::Track);
        assert_eq!(add_chunk.waypoint.data, TrackedWaypointData::Chunk(chunk));

        let upd_chunk = ClientboundTrackedWaypointPacket::update_waypoint_chunk(id, icon, chunk);
        assert_eq!(upd_chunk.operation, Operation::Update);
        assert_eq!(upd_chunk.waypoint.id, id);
    }

    #[test]
    fn azimuth_float_bits_preserved() {
        let id = sample_uuid();
        let icon = sample_icon();
        let cases = [
            0.0_f32,
            -0.0_f32,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::from_bits(0x7FC0_0001), // quiet NaN payload
            -123.456_f32,
        ];

        for bits in cases.into_iter().map(f32::to_bits) {
            let angle = f32::from_bits(bits);
            let add = ClientboundTrackedWaypointPacket::add_waypoint_azimuth(id, icon, angle);
            let upd = ClientboundTrackedWaypointPacket::update_waypoint_azimuth(id, icon, angle);

            match add.waypoint.data {
                TrackedWaypointData::Azimuth(a) => assert_eq!(a.to_bits(), bits),
                _ => panic!("expected azimuth waypoint"),
            }
            match upd.waypoint.data {
                TrackedWaypointData::Azimuth(a) => assert_eq!(a.to_bits(), bits),
                _ => panic!("expected azimuth waypoint"),
            }
            assert_eq!(add.operation, Operation::Track);
            assert_eq!(upd.operation, Operation::Update);
        }
    }

    #[test]
    fn apply_and_handle_dispatch() {
        let id = sample_uuid();
        let icon = sample_icon();
        let pos = Vec3i {
            x: i32::MIN,
            y: -1,
            z: i32::MAX,
        };

        let track = ClientboundTrackedWaypointPacket::add_waypoint_position(id, icon, pos);
        let untrack = ClientboundTrackedWaypointPacket::remove_waypoint(id);
        let update = ClientboundTrackedWaypointPacket::update_waypoint_position(id, icon, pos);

        let mut manager = MockManager::default();
        track.apply(&mut manager);
        untrack.apply(&mut manager);
        update.apply(&mut manager);

        assert_eq!(manager.tracked.len(), 1);
        assert_eq!(manager.untracked.len(), 1);
        assert_eq!(manager.updated.len(), 1);
        assert_eq!(manager.tracked[0].data, TrackedWaypointData::Vec3i(pos));

        let mut listener = MockListener { handled: None };
        track.handle(&mut listener);
        assert_eq!(listener.handled.as_ref(), Some(&track));
        assert_eq!(track.packet_type(), PacketType::CLIENTBOUND_WAYPOINT);
    }
}
