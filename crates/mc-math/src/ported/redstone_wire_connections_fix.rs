use std::collections::BTreeMap;

/// Self-contained port of `net.minecraft.util.datafix.fixes.RedstoneWireConnectionsFix`.
///
/// The original operates on DFU `Dynamic` block states; this module uses a simple
/// block-state representation while preserving the redstone connection update logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub name: String,
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct RedstoneWireConnectionsFix;

impl RedstoneWireConnectionsFix {
    // JAVA: net.minecraft.util.datafix.fixes.RedstoneWireConnectionsFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.RedstoneWireConnectionsFix#makeRule
    pub fn make_rule(&self, state: &BlockState) -> BlockState {
        self.update_redstone_connections(state)
    }

    // JAVA: net.minecraft.util.datafix.fixes.RedstoneWireConnectionsFix#updateRedstoneConnections
    fn update_redstone_connections(&self, state: &BlockState) -> BlockState {
        if state.name != "minecraft:redstone_wire" {
            return state.clone();
        }

        let mut properties = state.properties.clone();
        let east = properties.get("east").cloned().unwrap_or_else(|| "none".to_string());
        let west = properties.get("west").cloned().unwrap_or_else(|| "none".to_string());
        let north = properties.get("north").cloned().unwrap_or_else(|| "none".to_string());
        let south = properties.get("south").cloned().unwrap_or_else(|| "none".to_string());

        let east_west_connected = Self::is_connected(&east) || Self::is_connected(&west);
        let north_south_connected = Self::is_connected(&north) || Self::is_connected(&south);

        let new_east = if !Self::is_connected(&east) && !north_south_connected {
            "side".to_string()
        } else {
            east
        };
        let new_west = if !Self::is_connected(&west) && !north_south_connected {
            "side".to_string()
        } else {
            west
        };
        let new_north = if !Self::is_connected(&north) && !east_west_connected {
            "side".to_string()
        } else {
            north
        };
        let new_south = if !Self::is_connected(&south) && !east_west_connected {
            "side".to_string()
        } else {
            south
        };

        properties.insert("east".to_string(), new_east);
        properties.insert("west".to_string(), new_west);
        properties.insert("north".to_string(), new_north);
        properties.insert("south".to_string(), new_south);

        BlockState {
            name: state.name.clone(),
            properties,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.RedstoneWireConnectionsFix#isConnected
    fn is_connected(value: &str) -> bool {
        value != "none"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(name: &str, props: &[(&str, &str)]) -> BlockState {
        BlockState {
            name: name.to_string(),
            properties: props
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn non_redstone_wire_unchanged() {
        let fix = RedstoneWireConnectionsFix::new();
        let s = state("minecraft:stone", &[("facing", "north")]);
        assert_eq!(fix.make_rule(&s), s);
    }

    #[test]
    fn all_none_becomes_all_side() {
        let fix = RedstoneWireConnectionsFix::new();
        let s = state(
            "minecraft:redstone_wire",
            &[("east", "none"), ("west", "none"), ("north", "none"), ("south", "none")],
        );
        let out = fix.make_rule(&s);
        assert_eq!(out.properties.get("east").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("west").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("north").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("south").map(String::as_str), Some("side"));
    }

    #[test]
    fn horizontal_connection_fills_horizontal_none_only() {
        let fix = RedstoneWireConnectionsFix::new();
        let s = state(
            "minecraft:redstone_wire",
            &[("east", "side"), ("west", "none"), ("north", "none"), ("south", "none")],
        );
        let out = fix.make_rule(&s);
        assert_eq!(out.properties.get("east").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("west").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("north").map(String::as_str), Some("none"));
        assert_eq!(out.properties.get("south").map(String::as_str), Some("none"));
    }

    #[test]
    fn existing_connections_preserved() {
        let fix = RedstoneWireConnectionsFix::new();
        let s = state(
            "minecraft:redstone_wire",
            &[("east", "up"), ("west", "side"), ("north", "side"), ("south", "none")],
        );
        let out = fix.make_rule(&s);
        assert_eq!(out.properties.get("east").map(String::as_str), Some("up"));
        assert_eq!(out.properties.get("west").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("north").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("south").map(String::as_str), Some("none"));
    }

    #[test]
    fn missing_properties_default_to_none_and_other_preserved() {
        let fix = RedstoneWireConnectionsFix::new();
        let s = state("minecraft:redstone_wire", &[("power", "15")]);
        let out = fix.make_rule(&s);
        assert_eq!(out.properties.get("east").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("west").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("north").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("south").map(String::as_str), Some("side"));
        assert_eq!(out.properties.get("power").map(String::as_str), Some("15"));
    }
}
