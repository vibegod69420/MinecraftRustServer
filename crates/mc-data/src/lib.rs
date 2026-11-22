#![forbid(unsafe_code)]
//! Generated Minecraft 1.21.11 registries, packet IDs, and block state data.

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tables_sanity() {
        if !HAS_GENERATED_DATA {
            eprintln!("[skip] reference/generated/reports not present");
            return;
        }
        assert_eq!(
            packet_id(
                ConnectionProtocol::Handshake,
                PacketDirection::Serverbound,
                "minecraft:intention"
            ),
            Some(0)
        );
        assert!(packet_id(ConnectionProtocol::Play, PacketDirection::Clientbound, "minecraft:login").is_some());
        let block_reg = get_registry("minecraft:block").expect("block registry missing");
        assert!(block_reg.get_id("minecraft:stone").is_some());
        let stone = block_by_name("minecraft:stone").expect("stone block missing");
        assert_eq!(block_for_state(stone.default_state_id).unwrap().id, "minecraft:stone");
        assert_eq!(TOTAL_BLOCK_STATES, STATE_TO_BLOCK.len());
    }
}
