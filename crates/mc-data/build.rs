use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

use serde_json::Value;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let reports = Path::new(&manifest_dir).join("../../reference/generated/reports");

    println!("cargo:rerun-if-changed=../../reference/generated/reports/packets.json");
    println!("cargo:rerun-if-changed=../../reference/generated/reports/registries.json");
    println!("cargo:rerun-if-changed=../../reference/generated/reports/blocks.json");

    let has_reports = reports.join("packets.json").exists()
        && reports.join("registries.json").exists()
        && reports.join("blocks.json").exists();

    let mut out = String::new();
    out.push_str(&format!("pub const HAS_GENERATED_DATA: bool = {has_reports};\n"));

    if !has_reports {
        out.push_str(
            r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol { Handshake, Status, Login, Configuration, Play }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketDirection { Clientbound, Serverbound }
pub const PACKETS: &[(ConnectionProtocol, PacketDirection, u32, &str)] = &[];
pub fn packet_id(_: ConnectionProtocol, _: PacketDirection, _: &str) -> Option<u32> { None }
pub fn packet_name(_: ConnectionProtocol, _: PacketDirection, _: u32) -> Option<&'static str> { None }

#[derive(Debug, Clone, Copy)]
pub struct RegistryInfo { pub id: &'static str, pub protocol_id: i32, pub entries: &'static [(&'static str, i32)] }
impl RegistryInfo {
    pub fn get_id(&self, _: &str) -> Option<i32> { None }
    pub fn by_id(&self, _: i32) -> Option<&'static str> { None }
}
pub const REGISTRIES: &[RegistryInfo] = &[];
pub fn get_registry(_: &str) -> Option<&'static RegistryInfo> { None }

#[derive(Debug, Clone, Copy)]
pub struct BlockDef { pub id: &'static str, pub default_state_id: u32, pub first_state_id: u32, pub state_count: u16 }
pub const TOTAL_BLOCK_STATES: usize = 0;
pub const BLOCKS: &[BlockDef] = &[];
pub const STATE_TO_BLOCK: &[u16] = &[];
pub const STATE_PROPERTIES: &[&str] = &[];
pub fn block_by_name(_: &str) -> Option<&'static BlockDef> { None }
pub fn block_for_state(_: u32) -> Option<&'static BlockDef> { None }
pub fn state_properties(_: u32) -> Option<&'static str> { None }
"#,
        );
        fs::write(Path::new(&out_dir).join("generated.rs"), out).unwrap();
        return;
    }

    // --- Packets ---
    let pkts: Value = serde_json::from_str(&fs::read_to_string(reports.join("packets.json")).unwrap()).unwrap();
    let mut pkt_rows = Vec::new();
    for (proto, dirs) in pkts.as_object().unwrap() {
        let proto_enum = match proto.as_str() {
            "handshake" => "ConnectionProtocol::Handshake",
            "status" => "ConnectionProtocol::Status",
            "login" => "ConnectionProtocol::Login",
            "configuration" => "ConnectionProtocol::Configuration",
            "play" => "ConnectionProtocol::Play",
            other => panic!("unknown protocol {other}"),
        };
        for (dir, entries) in dirs.as_object().unwrap() {
            let dir_enum = match dir.as_str() {
                "clientbound" => "PacketDirection::Clientbound",
                "serverbound" => "PacketDirection::Serverbound",
                other => panic!("unknown direction {other}"),
            };
            for (name, meta) in entries.as_object().unwrap() {
                let id = meta["protocol_id"].as_u64().unwrap() as u32;
                pkt_rows.push((proto_enum, dir_enum, id, name.clone()));
            }
        }
    }
    pkt_rows.sort_by_key(|r| (r.0, r.1, r.2));
    out.push_str(
        r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol { Handshake, Status, Login, Configuration, Play }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketDirection { Clientbound, Serverbound }
pub const PACKETS: &[(ConnectionProtocol, PacketDirection, u32, &str)] = &[
"#,
    );
    for (p, d, id, name) in &pkt_rows {
        out.push_str(&format!("    ({p}, {d}, {id}, {name:?}),\n"));
    }
    out.push_str(
        r#"];
// JAVA: net.minecraft.network.protocol.PacketType
pub fn packet_id(proto: ConnectionProtocol, dir: PacketDirection, name: &str) -> Option<u32> {
    PACKETS.iter().find(|(p, d, _, n)| *p == proto && *d == dir && *n == name).map(|(_, _, id, _)| *id)
}
pub fn packet_name(proto: ConnectionProtocol, dir: PacketDirection, id: u32) -> Option<&'static str> {
    PACKETS.iter().find(|(p, d, pid, _)| *p == proto && *d == dir && *pid == id).map(|(_, _, _, n)| *n)
}
"#,
    );

    // --- Registries ---
    let regs: Value = serde_json::from_str(&fs::read_to_string(reports.join("registries.json")).unwrap()).unwrap();
    let mut reg_list: Vec<(String, i64, Vec<(String, i64)>)> = Vec::new();
    for (rname, robj) in regs.as_object().unwrap() {
        let pid = robj.get("protocol_id").and_then(|v| v.as_i64()).unwrap_or(-1);
        let mut entries: Vec<(String, i64)> = robj["entries"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v["protocol_id"].as_i64().unwrap()))
            .collect();
        entries.sort_by_key(|e| e.1);
        reg_list.push((rname.clone(), pid, entries));
    }
    reg_list.sort_by_key(|r| (r.1, r.0.clone()));
    out.push_str(
        r#"
#[derive(Debug, Clone, Copy)]
pub struct RegistryInfo { pub id: &'static str, pub protocol_id: i32, pub entries: &'static [(&'static str, i32)] }
impl RegistryInfo {
    // JAVA: net.minecraft.core.IdMap#getId
    pub fn get_id(&self, name: &str) -> Option<i32> {
        self.entries.iter().find(|(n, _)| *n == name).map(|(_, id)| *id)
    }
    // JAVA: net.minecraft.core.IdMap#byId
    pub fn by_id(&self, id: i32) -> Option<&'static str> {
        self.entries.iter().find(|(_, eid)| *eid == id).map(|(n, _)| *n)
    }
}
"#,
    );
    for (idx, (_, _, entries)) in reg_list.iter().enumerate() {
        out.push_str(&format!("const REG_{idx}: &[(&str, i32)] = &[\n"));
        for (ename, eid) in entries {
            out.push_str(&format!("    ({ename:?}, {eid}),\n"));
        }
        out.push_str("];\n");
    }
    out.push_str("pub const REGISTRIES: &[RegistryInfo] = &[\n");
    for (idx, (rname, pid, _)) in reg_list.iter().enumerate() {
        out.push_str(&format!(
            "    RegistryInfo {{ id: {rname:?}, protocol_id: {pid}, entries: REG_{idx} }},\n"
        ));
    }
    out.push_str(
        r#"];
// JAVA: net.minecraft.core.registries.BuiltInRegistries
pub fn get_registry(id: &str) -> Option<&'static RegistryInfo> {
    REGISTRIES.iter().find(|r| r.id == id)
}
"#,
    );

    // --- Blocks ---
    let blks: Value = serde_json::from_str(&fs::read_to_string(reports.join("blocks.json")).unwrap()).unwrap();
    let mut bnames: Vec<&String> = blks.as_object().unwrap().keys().collect();
    bnames.sort();
    let mut max_sid: usize = 0;
    let mut block_defs = Vec::with_capacity(bnames.len());
    let mut state_records = Vec::new();
    for (bidx, bname) in bnames.iter().enumerate() {
        let states = blks[*bname]["states"].as_array().unwrap();
        let mut first_id = u32::MAX;
        let mut default_id = None;
        for s in states {
            let sid = s["id"].as_u64().unwrap() as usize;
            max_sid = max_sid.max(sid);
            first_id = first_id.min(sid as u32);
            if s.get("default").and_then(|v| v.as_bool()).unwrap_or(false) {
                default_id = Some(sid as u32);
            }
            let mut props = BTreeMap::new();
            if let Some(pobj) = s.get("properties").and_then(|v| v.as_object()) {
                for (k, v) in pobj {
                    props.insert(k.as_str(), v.as_str().unwrap_or(""));
                }
            }
            let prop_str = props
                .into_iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(",");
            state_records.push((sid, bidx as u16, prop_str));
        }
        block_defs.push((
            (*bname).clone(),
            default_id.unwrap_or(first_id),
            first_id,
            states.len() as u16,
        ));
    }

    let total_states = max_sid + 1;
    let mut state_to_block = vec![0_u16; total_states];
    let mut state_props = vec![String::new(); total_states];
    for (sid, bidx, prop_str) in state_records {
        state_to_block[sid] = bidx;
        state_props[sid] = prop_str;
    }

    out.push_str(&format!(
        r#"
#[derive(Debug, Clone, Copy)]
pub struct BlockDef {{ pub id: &'static str, pub default_state_id: u32, pub first_state_id: u32, pub state_count: u16 }}
pub const TOTAL_BLOCK_STATES: usize = {total_states};
pub const BLOCKS: &[BlockDef] = &[
"#
    ));
    for (bname, def_id, first_id, cnt) in &block_defs {
        out.push_str(&format!(
            "    BlockDef {{ id: {bname:?}, default_state_id: {def_id}, first_state_id: {first_id}, state_count: {cnt} }},\n"
        ));
    }
    out.push_str("];\npub const STATE_TO_BLOCK: &[u16] = &[\n");
    for chunk in state_to_block.chunks(16) {
        let s = chunk.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",");
        out.push_str(&format!("    {s},\n"));
    }
    out.push_str("];\npub const STATE_PROPERTIES: &[&str] = &[\n");
    for p in &state_props {
        out.push_str(&format!("    {p:?},\n"));
    }
    out.push_str(
        r#"];
// JAVA: net.minecraft.world.level.block.Block#getId
pub fn block_by_name(name: &str) -> Option<&'static BlockDef> {
    BLOCKS.binary_search_by(|b| b.id.cmp(name)).ok().map(|idx| &BLOCKS[idx])
}
// JAVA: net.minecraft.world.level.block.Block#stateById
pub fn block_for_state(state_id: u32) -> Option<&'static BlockDef> {
    STATE_TO_BLOCK.get(state_id as usize).map(|&bidx| &BLOCKS[bidx as usize])
}
pub fn state_properties(state_id: u32) -> Option<&'static str> {
    STATE_PROPERTIES.get(state_id as usize).copied()
}
"#,
    );

    fs::write(Path::new(&out_dir).join("generated.rs"), out).unwrap();
}
