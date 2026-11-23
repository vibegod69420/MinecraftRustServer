//! Faithful port of `net.minecraft.util.debug.DebugBrainDump` (Java 1.21.11).

use std::collections::HashSet;

/// Minimal block position used by debug brain dumps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Minimal byte buffer abstraction for the Java `FriendlyByteBuf` read/write calls.
pub trait DebugByteBuf {
    fn read_utf(&mut self) -> String;
    fn read_int(&mut self) -> i32;
    fn read_float(&mut self) -> f32;
    fn read_boolean(&mut self) -> bool;
    fn read_list_utf(&mut self) -> Vec<String>;
    fn read_block_pos_set(&mut self) -> HashSet<BlockPos>;
    fn write_utf(&mut self, value: &str);
    fn write_int(&mut self, value: i32);
    fn write_float(&mut self, value: f32);
    fn write_boolean(&mut self, value: bool);
    fn write_utf_collection(&mut self, values: &[String]);
    fn write_block_pos_collection(&mut self, values: &HashSet<BlockPos>);
}

/// Minimal level abstraction for resolving entity UUIDs during debug description formatting.
pub trait DebugLevel {
    fn get_entity_name_by_uuid(&self, uuid: u128) -> Option<String>;
}

/// Value model for `getShortDescription`'s Java pattern-matching switch.
#[derive(Clone, Debug, PartialEq)]
pub enum DebugObject {
    Uuid(u128),
    NamedEntity(String),
    WalkTarget(Box<DebugObject>),
    EntityTracker(Box<DebugObject>),
    GlobalPos { pos: BlockPos },
    BlockPosTracker { pos: BlockPos },
    BlockPosValue(BlockPos),
    DamageSource {
        entity: Option<Box<DebugObject>>,
        fallback: String,
    },
    Collection(Vec<DebugObject>),
    Long(i64),
    Other(String),
}

/// View of Java `ExpirableValue<?>`.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpirableValueView {
    pub value: DebugObject,
    pub can_expire: bool,
    pub time_to_live: i64,
}

/// View of a brain memory map entry.
#[derive(Clone, Debug, PartialEq)]
pub struct MemoryEntryView {
    pub type_path: String,
    pub is_heard_bell_time: bool,
    pub value: Option<ExpirableValueView>,
}

/// Flattened villager gossip entry view.
#[derive(Clone, Debug, PartialEq)]
pub struct GossipEntryView {
    pub target_name: String,
    pub gossip_type: String,
    pub value: i32,
}

/// Minimal brain access required by `DebugBrainDump`.
pub trait BrainDumpBrain {
    fn active_activities(&self) -> Vec<String>;
    fn running_behaviors(&self) -> Vec<String>;
    fn memory_entries(&self) -> Vec<MemoryEntryView>;
    fn get_global_pos_memory(&self, memory_type_path: &str) -> Option<BlockPos>;
}

/// Minimal villager access required by `DebugBrainDump`.
pub trait BrainDumpVillager {
    fn profession_name(&self) -> String;
    fn villager_xp(&self) -> i32;
    fn wants_to_spawn_golem(&self, game_time: i64) -> bool;
    fn gossip_entries(&self) -> Vec<GossipEntryView>;
}

/// Minimal inventory carrier access required by `DebugBrainDump`.
pub trait BrainDumpInventoryCarrier {
    fn inventory_debug_string(&self) -> String;
}

/// Minimal warden access required by `DebugBrainDump`.
pub trait BrainDumpWarden {
    fn client_anger_level(&self) -> i32;
}

/// Minimal living entity access required by `DebugBrainDump`.
pub trait BrainDumpLivingEntity {
    fn debug_entity_name(&self) -> String;
    fn health(&self) -> f32;
    fn max_health(&self) -> f32;
    fn brain(&self) -> &dyn BrainDumpBrain;
    fn level_game_time(&self) -> i64;
    fn as_villager(&self) -> Option<&dyn BrainDumpVillager>;
    fn as_inventory_carrier(&self) -> Option<&dyn BrainDumpInventoryCarrier>;
    fn as_warden(&self) -> Option<&dyn BrainDumpWarden>;
}

/// Faithful data record for `net.minecraft.util.debug.DebugBrainDump`.
#[derive(Clone, Debug, PartialEq)]
pub struct DebugBrainDump {
    pub name: String,
    pub profession: String,
    pub xp: i32,
    pub health: f32,
    pub max_health: f32,
    pub inventory: String,
    pub wants_golem: bool,
    pub anger_level: i32,
    pub activities: Vec<String>,
    pub behaviors: Vec<String>,
    pub memories: Vec<String>,
    pub gossips: Vec<String>,
    pub pois: HashSet<BlockPos>,
    pub potential_pois: HashSet<BlockPos>,
}

impl DebugBrainDump {
    // JAVA: net.minecraft.util.debug.DebugBrainDump#DebugBrainDump
    pub fn from_buf(buf: &mut dyn DebugByteBuf) -> Self {
        Self {
            name: buf.read_utf(),
            profession: buf.read_utf(),
            xp: buf.read_int(),
            health: buf.read_float(),
            max_health: buf.read_float(),
            inventory: buf.read_utf(),
            wants_golem: buf.read_boolean(),
            anger_level: buf.read_int(),
            activities: buf.read_list_utf(),
            behaviors: buf.read_list_utf(),
            memories: buf.read_list_utf(),
            gossips: buf.read_list_utf(),
            pois: buf.read_block_pos_set(),
            potential_pois: buf.read_block_pos_set(),
        }
    }

    // JAVA: net.minecraft.util.debug.DebugBrainDump#write
    pub fn write(&self, buf: &mut dyn DebugByteBuf) {
        buf.write_utf(&self.name);
        buf.write_utf(&self.profession);
        buf.write_int(self.xp);
        buf.write_float(self.health);
        buf.write_float(self.max_health);
        buf.write_utf(&self.inventory);
        buf.write_boolean(self.wants_golem);
        buf.write_int(self.anger_level);
        buf.write_utf_collection(&self.activities);
        buf.write_utf_collection(&self.behaviors);
        buf.write_utf_collection(&self.memories);
        buf.write_utf_collection(&self.gossips);
        buf.write_block_pos_collection(&self.pois);
        buf.write_block_pos_collection(&self.potential_pois);
    }

    // JAVA: net.minecraft.util.debug.DebugBrainDump#takeBrainDump
    pub fn take_brain_dump(level: &dyn DebugLevel, entity: &dyn BrainDumpLivingEntity) -> Self {
        let name = entity.debug_entity_name();
        let (profession, xp) = if let Some(v) = entity.as_villager() {
            (v.profession_name(), v.villager_xp())
        } else {
            (String::new(), 0)
        };
        let health = entity.health();
        let max_health = entity.max_health();
        let brain = entity.brain();
        let game_time = entity.level_game_time();
        let inventory = if let Some(c) = entity.as_inventory_carrier() {
            c.inventory_debug_string()
        } else {
            String::new()
        };
        let wants_golem = if let Some(v) = entity.as_villager() {
            v.wants_to_spawn_golem(game_time)
        } else {
            false
        };
        let anger_level = if let Some(w) = entity.as_warden() {
            w.client_anger_level()
        } else {
            -1
        };
        let activities = brain.active_activities();
        let behaviors = brain.running_behaviors();
        let memories = get_memory_descriptions(level, &brain.memory_entries(), game_time)
            .into_iter()
            .map(|s| truncate_string_if_necessary(&s, 255, true))
            .collect();
        let pois = get_known_block_positions(brain, &["job_site", "home", "meeting_point"]);
        let potential_pois = get_known_block_positions(brain, &["potential_job_site"]);
        let gossips = if let Some(v) = entity.as_villager() {
            get_villager_gossips(&v.gossip_entries())
        } else {
            Vec::new()
        };
        Self {
            name,
            profession,
            xp,
            health,
            max_health,
            inventory,
            wants_golem,
            anger_level,
            activities,
            behaviors,
            memories,
            gossips,
            pois,
            potential_pois,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugBrainDump#hasPoi
    pub fn has_poi(&self, pos: &BlockPos) -> bool {
        self.pois.contains(pos)
    }

    // JAVA: net.minecraft.util.debug.DebugBrainDump#hasPotentialPoi
    pub fn has_potential_poi(&self, pos: &BlockPos) -> bool {
        self.potential_pois.contains(pos)
    }
}

// JAVA: net.minecraft.util.debug.DebugBrainDump#getKnownBlockPositions
fn get_known_block_positions(
    brain: &dyn BrainDumpBrain,
    memory_types: &[&str],
) -> HashSet<BlockPos> {
    memory_types
        .iter()
        .filter_map(|mt| brain.get_global_pos_memory(mt))
        .collect()
}

// JAVA: net.minecraft.util.debug.DebugBrainDump#getVillagerGossips
fn get_villager_gossips(entries: &[GossipEntryView]) -> Vec<String> {
    let mut out = Vec::new();
    for e in entries {
        out.push(format!(
            "{}: {}: {}",
            e.target_name, e.gossip_type, e.value
        ));
    }
    out
}

// JAVA: net.minecraft.util.debug.DebugBrainDump#getMemoryDescriptions
fn get_memory_descriptions(
    level: &dyn DebugLevel,
    memories: &[MemoryEntryView],
    game_time: i64,
) -> Vec<String> {
    let mut out: Vec<String> = memories
        .iter()
        .map(|m| {
            get_memory_description(
                level,
                game_time,
                &m.type_path,
                m.value.as_ref(),
                m.is_heard_bell_time,
            )
        })
        .collect();
    out.sort();
    out
}

// JAVA: net.minecraft.util.debug.DebugBrainDump#getMemoryDescription
fn get_memory_description(
    level: &dyn DebugLevel,
    game_time: i64,
    memory_type_path: &str,
    memory: Option<&ExpirableValueView>,
    is_heard_bell_time: bool,
) -> String {
    let desc = match memory {
        Some(ev) => {
            if is_heard_bell_time {
                let bell_time = match &ev.value {
                    DebugObject::Long(v) => *v,
                    _ => 0i64,
                };
                let delta = game_time.wrapping_sub(bell_time);
                format!("{} ticks ago", delta)
            } else if ev.can_expire {
                format!(
                    "{} (ttl: {})",
                    get_short_description(level, Some(&ev.value)),
                    ev.time_to_live
                )
            } else {
                get_short_description(level, Some(&ev.value))
            }
        }
        None => "-".to_string(),
    };
    format!("{}: {}", memory_type_path, desc)
}

// JAVA: net.minecraft.util.debug.DebugBrainDump#getShortDescription
fn get_short_description(level: &dyn DebugLevel, value: Option<&DebugObject>) -> String {
    match value {
        None => "-".to_string(),
        Some(DebugObject::Uuid(uuid)) => match level.get_entity_name_by_uuid(*uuid) {
            Some(name) => get_short_description(level, Some(&DebugObject::NamedEntity(name))),
            None => get_short_description(level, None),
        },
        Some(DebugObject::NamedEntity(name)) => name.clone(),
        Some(DebugObject::WalkTarget(target)) => {
            get_short_description(level, Some(target.as_ref()))
        }
        Some(DebugObject::EntityTracker(entity)) => {
            get_short_description(level, Some(entity.as_ref()))
        }
        Some(DebugObject::GlobalPos { pos }) => {
            get_short_description(level, Some(&DebugObject::BlockPosValue(*pos)))
        }
        Some(DebugObject::BlockPosTracker { pos }) => {
            get_short_description(level, Some(&DebugObject::BlockPosValue(*pos)))
        }
        Some(DebugObject::BlockPosValue(pos)) => {
            format!("BlockPos{{x={}, y={}, z={}}}", pos.x, pos.y, pos.z)
        }
        Some(DebugObject::DamageSource { entity, fallback }) => match entity {
            Some(e) => get_short_description(level, Some(e.as_ref())),
            None => fallback.clone(),
        },
        Some(DebugObject::Collection(items)) => {
            let inner: Vec<String> = items
                .iter()
                .map(|i| get_short_description(level, Some(i)))
                .collect();
            format!("[{}]", inner.join(", "))
        }
        Some(DebugObject::Long(v)) => v.to_string(),
        Some(DebugObject::Other(s)) => s.clone(),
    }
}

fn truncate_string_if_necessary(s: &str, max_length: usize, add_ellipsis: bool) -> String {
    let char_count = s.chars().count();
    if char_count <= max_length {
        s.to_string()
    } else if add_ellipsis && max_length >= 3 {
        let take = max_length - 3;
        let mut out: String = s.chars().take(take).collect();
        out.push_str("...");
        out
    } else {
        s.chars().take(max_length).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockBuf {
        bytes: Vec<u8>,
        read_pos: usize,
    }

    impl DebugByteBuf for MockBuf {
        fn read_utf(&mut self) -> String {
            let len = self.read_int() as usize;
            let s = String::from_utf8(self.bytes[self.read_pos..self.read_pos + len].to_vec())
                .unwrap();
            self.read_pos += len;
            s
        }
        fn read_int(&mut self) -> i32 {
            let mut b = [0u8; 4];
            b.copy_from_slice(&self.bytes[self.read_pos..self.read_pos + 4]);
            self.read_pos += 4;
            i32::from_be_bytes(b)
        }
        fn read_float(&mut self) -> f32 {
            let mut b = [0u8; 4];
            b.copy_from_slice(&self.bytes[self.read_pos..self.read_pos + 4]);
            self.read_pos += 4;
            f32::from_be_bytes(b)
        }
        fn read_boolean(&mut self) -> bool {
            let v = self.bytes[self.read_pos];
            self.read_pos += 1;
            v != 0
        }
        fn read_list_utf(&mut self) -> Vec<String> {
            let n = self.read_int() as usize;
            (0..n).map(|_| self.read_utf()).collect()
        }
        fn read_block_pos_set(&mut self) -> HashSet<BlockPos> {
            let n = self.read_int() as usize;
            (0..n)
                .map(|_| BlockPos {
                    x: self.read_int(),
                    y: self.read_int(),
                    z: self.read_int(),
                })
                .collect()
        }
        fn write_utf(&mut self, value: &str) {
            let bytes = value.as_bytes();
            self.write_int(bytes.len() as i32);
            self.bytes.extend_from_slice(bytes);
        }
        fn write_int(&mut self, value: i32) {
            self.bytes.extend_from_slice(&value.to_be_bytes());
        }
        fn write_float(&mut self, value: f32) {
            self.bytes.extend_from_slice(&value.to_be_bytes());
        }
        fn write_boolean(&mut self, value: bool) {
            self.bytes.push(if value { 1 } else { 0 });
        }
        fn write_utf_collection(&mut self, values: &[String]) {
            self.write_int(values.len() as i32);
            for v in values {
                self.write_utf(v);
            }
        }
        fn write_block_pos_collection(&mut self, values: &HashSet<BlockPos>) {
            self.write_int(values.len() as i32);
            let mut sorted: Vec<BlockPos> = values.iter().copied().collect();
            sorted.sort();
            for p in sorted {
                self.write_int(p.x);
                self.write_int(p.y);
                self.write_int(p.z);
            }
        }
    }

    #[test]
    fn read_write_roundtrip_and_float_bits() {
        let mut pois = HashSet::new();
        pois.insert(BlockPos { x: 1, y: -2, z: 3 });
        let mut potential_pois = HashSet::new();
        potential_pois.insert(BlockPos {
            x: -10,
            y: 64,
            z: 999,
        });

        let dump = DebugBrainDump {
            name: "Villager#1".to_string(),
            profession: "minecraft:farmer".to_string(),
            xp: -5,
            health: f32::from_bits(0x7FC0_0001),
            max_health: 20.0,
            inventory: "inv".to_string(),
            wants_golem: true,
            anger_level: -1,
            activities: vec!["idle".to_string()],
            behaviors: vec!["look".to_string()],
            memories: vec!["home: -".to_string()],
            gossips: vec!["g: minor: 1".to_string()],
            pois,
            potential_pois,
        };

        let mut buf = MockBuf::default();
        dump.write(&mut buf);
        buf.read_pos = 0;
        let decoded = DebugBrainDump::from_buf(&mut buf);

        assert_eq!(decoded.name, dump.name);
        assert_eq!(decoded.xp, -5);
        assert_eq!(decoded.health.to_bits(), dump.health.to_bits());
        assert_eq!(decoded.max_health.to_bits(), 20.0f32.to_bits());
        assert_eq!(decoded.anger_level, -1);
        assert!(decoded.wants_golem);
        assert_eq!(decoded.pois, dump.pois);
        assert_eq!(decoded.potential_pois, dump.potential_pois);
    }

    #[test]
    fn has_poi_and_potential_poi_checks() {
        let mut pois = HashSet::new();
        pois.insert(BlockPos { x: 1, y: 2, z: 3 });
        let dump = DebugBrainDump {
            name: String::new(),
            profession: String::new(),
            xp: 0,
            health: 0.0,
            max_health: 0.0,
            inventory: String::new(),
            wants_golem: false,
            anger_level: 0,
            activities: vec![],
            behaviors: vec![],
            memories: vec![],
            gossips: vec![],
            pois,
            potential_pois: HashSet::new(),
        };
        assert!(dump.has_poi(&BlockPos { x: 1, y: 2, z: 3 }));
        assert!(!dump.has_poi(&BlockPos { x: 9, y: 9, z: 9 }));
        assert!(!dump.has_potential_poi(&BlockPos { x: 1, y: 2, z: 3 }));
    }

    #[test]
    fn short_description_cases() {
        struct Lvl;
        impl DebugLevel for Lvl {
            fn get_entity_name_by_uuid(&self, uuid: u128) -> Option<String> {
                if uuid == 1 {
                    Some("Entity#1".to_string())
                } else {
                    None
                }
            }
        }
        let lvl = Lvl;
        assert_eq!(get_short_description(&lvl, None), "-");
        assert_eq!(
            get_short_description(&lvl, Some(&DebugObject::Uuid(1))),
            "Entity#1"
        );
        assert_eq!(get_short_description(&lvl, Some(&DebugObject::Uuid(2))), "-");
        assert_eq!(
            get_short_description(
                &lvl,
                Some(&DebugObject::GlobalPos {
                    pos: BlockPos { x: 1, y: -2, z: 3 }
                })
            ),
            "BlockPos{x=1, y=-2, z=3}"
        );
        assert_eq!(
            get_short_description(
                &lvl,
                Some(&DebugObject::WalkTarget(Box::new(DebugObject::BlockPosValue(
                    BlockPos { x: 0, y: 0, z: 0 }
                ))))
            ),
            "BlockPos{x=0, y=0, z=0}"
        );
        let coll = DebugObject::Collection(vec![
            DebugObject::Other("a".into()),
            DebugObject::Long(-7),
        ]);
        assert_eq!(get_short_description(&lvl, Some(&coll)), "[a, -7]");
        assert_eq!(
            get_short_description(
                &lvl,
                Some(&DebugObject::DamageSource {
                    entity: None,
                    fallback: "dmg".into()
                })
            ),
            "dmg"
        );
    }

    #[test]
    fn memory_description_edge_cases() {
        struct Lvl;
        impl DebugLevel for Lvl {
            fn get_entity_name_by_uuid(&self, _uuid: u128) -> Option<String> {
                None
            }
        }
        let lvl = Lvl;
        assert_eq!(
            get_memory_description(&lvl, 100, "home", None, false),
            "home: -"
        );

        let bell = ExpirableValueView {
            value: DebugObject::Long(i64::MAX),
            can_expire: false,
            time_to_live: 0,
        };
        let desc = get_memory_description(&lvl, i64::MIN + 1, "heard_bell_time", Some(&bell), true);
        let expected_delta = (i64::MIN + 1i64).wrapping_sub(i64::MAX);
        assert_eq!(
            desc,
            format!("heard_bell_time: {} ticks ago", expected_delta)
        );

        let exp = ExpirableValueView {
            value: DebugObject::Other("x".into()),
            can_expire: true,
            time_to_live: 42,
        };
        assert_eq!(
            get_memory_description(&lvl, 0, "job", Some(&exp), false),
            "job: x (ttl: 42)"
        );
    }

    #[test]
    fn take_brain_dump_dispatch_and_truncation() {
        struct Lvl;
        impl DebugLevel for Lvl {
            fn get_entity_name_by_uuid(&self, _uuid: u128) -> Option<String> {
                None
            }
        }
        struct Brain;
        impl BrainDumpBrain for Brain {
            fn active_activities(&self) -> Vec<String> {
                vec!["idle".into()]
            }
            fn running_behaviors(&self) -> Vec<String> {
                vec!["walk".into()]
            }
            fn memory_entries(&self) -> Vec<MemoryEntryView> {
                vec![
                    MemoryEntryView {
                        type_path: "home".into(),
                        is_heard_bell_time: false,
                        value: Some(ExpirableValueView {
                            value: DebugObject::Other("v".into()),
                            can_expire: false,
                            time_to_live: 0,
                        }),
                    },
                    MemoryEntryView {
                        type_path: "long".into(),
                        is_heard_bell_time: false,
                        value: Some(ExpirableValueView {
                            value: DebugObject::Other("x".repeat(300)),
                            can_expire: false,
                            time_to_live: 0,
                        }),
                    },
                ]
            }
            fn get_global_pos_memory(&self, memory_type_path: &str) -> Option<BlockPos> {
                match memory_type_path {
                    "job_site" => Some(BlockPos { x: 1, y: 2, z: 3 }),
                    "potential_job_site" => Some(BlockPos { x: 4, y: 5, z: 6 }),
                    _ => None,
                }
            }
        }
        struct Villager;
        impl BrainDumpVillager for Villager {
            fn profession_name(&self) -> String {
                "farmer".into()
            }
            fn villager_xp(&self) -> i32 {
                7
            }
            fn wants_to_spawn_golem(&self, _game_time: i64) -> bool {
                true
            }
            fn gossip_entries(&self) -> Vec<GossipEntryView> {
                vec![GossipEntryView {
                    target_name: "P".into(),
                    gossip_type: "minor".into(),
                    value: 1,
                }]
            }
        }
        struct Carrier;
        impl BrainDumpInventoryCarrier for Carrier {
            fn inventory_debug_string(&self) -> String {
                "items".into()
            }
        }
        struct Entity {
            brain: Brain,
            villager: Villager,
            carrier: Carrier,
        }
        impl BrainDumpLivingEntity for Entity {
            fn debug_entity_name(&self) -> String {
                "V#1".into()
            }
            fn health(&self) -> f32 {
                20.0
            }
            fn max_health(&self) -> f32 {
                20.0
            }
            fn brain(&self) -> &dyn BrainDumpBrain {
                &self.brain
            }
            fn level_game_time(&self) -> i64 {
                100
            }
            fn as_villager(&self) -> Option<&dyn BrainDumpVillager> {
                Some(&self.villager)
            }
            fn as_inventory_carrier(&self) -> Option<&dyn BrainDumpInventoryCarrier> {
                Some(&self.carrier)
            }
            fn as_warden(&self) -> Option<&dyn BrainDumpWarden> {
                None
            }
        }

        let dump = DebugBrainDump::take_brain_dump(
            &Lvl,
            &Entity {
                brain: Brain,
                villager: Villager,
                carrier: Carrier,
            },
        );
        assert_eq!(dump.name, "V#1");
        assert_eq!(dump.profession, "farmer");
        assert_eq!(dump.xp, 7);
        assert_eq!(dump.inventory, "items");
        assert!(dump.wants_golem);
        assert_eq!(dump.anger_level, -1);
        assert!(dump.has_poi(&BlockPos { x: 1, y: 2, z: 3 }));
        assert!(dump.has_potential_poi(&BlockPos { x: 4, y: 5, z: 6 }));
        assert_eq!(dump.gossips, vec!["P: minor: 1"]);
        assert_eq!(dump.memories.len(), 2);
        assert_eq!(dump.memories[0], "home: v");
        assert_eq!(dump.memories[1].chars().count(), 255);
        assert!(dump.memories[1].ends_with("..."));
    }
}
