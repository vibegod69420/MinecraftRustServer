// JAVA: net.minecraft.util.datafix.fixes.LevelFlatGeneratorInfoFix

pub struct LevelFlatGeneratorInfoFix;

impl LevelFlatGeneratorInfoFix {
    pub fn fix_string(input: &str) -> String {
        if input.is_empty() {
            return "minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;1;village".to_string();
        }

        let parts: Vec<String> = split_limited(input, ';', 5);
        let mut iter = parts.into_iter();
        let first = iter.next().unwrap_or_default();

        let (version, layers): (i32, String) = if iter.clone().next().is_some() {
            (to_int(&first, 0), iter.next().unwrap_or_default())
        } else {
            (0, first)
        };

        if !(0..=3).contains(&version) {
            return "minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;1;village".to_string();
        }

        let amount_splitter = if version < 3 { 'x' } else { '*' };
        let mut out = String::new();

        for (i, layer) in split_no_limit(&layers, ',').iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let amount_parts: Vec<String> = split_limited(layer, amount_splitter, 2);
            let (amount, block_spec) = if amount_parts.len() == 2 {
                (to_int(&amount_parts[0], 0), amount_parts[1].clone())
            } else {
                (1, amount_parts.first().cloned().unwrap_or_default())
            };

            let block_parts: Vec<String> = split_limited(&block_spec, ':', 3);
            let prefix = if block_parts.first().map(|s| s == "minecraft").unwrap_or(false) {
                1usize
            } else {
                0usize
            };
            let name = block_parts.get(prefix).cloned().unwrap_or_default();
            let data = if block_parts.len() > prefix + 1 {
                to_int(block_parts.get(prefix + 1).map(|s| s.as_str()).unwrap_or("0"), 0)
            } else {
                0
            };

            let new_name = if version == 3 {
                format!("minecraft:{}", name)
            } else {
                let id = to_int(&name, 0);
                old_block_name(id << 4 | data)
            };

            if amount != 1 {
                out.push_str(&amount.to_string());
                out.push('*');
            }
            out.push_str(&new_name);
        }

        for rest in iter {
            out.push(';');
            out.push_str(&rest);
        }

        out
    }
}

fn to_int(s: &str, default: i32) -> i32 {
    s.trim().parse::<i32>().unwrap_or(default)
}

fn split_limited(s: &str, sep: char, limit: usize) -> Vec<String> {
    if limit == 0 {
        return vec![s.to_string()];
    }
    let mut result = Vec::new();
    let mut current = String::new();
    let mut count = 0;
    for ch in s.chars() {
        if ch == sep && count + 1 < limit {
            result.push(current.clone());
            current.clear();
            count += 1;
        } else {
            current.push(ch);
        }
    }
    result.push(current);
    result
}

fn split_no_limit(s: &str, sep: char) -> Vec<String> {
    s.split(sep).map(|x| x.to_string()).collect()
}

fn old_block_name(id: i32) -> String {
    let base_id = (id >> 4) & 0xFFF;
    match base_id {
        0 => "minecraft:air",
        1 => "minecraft:stone",
        2 => "minecraft:grass_block",
        3 => "minecraft:dirt",
        4 => "minecraft:cobblestone",
        5 => "minecraft:oak_planks",
        7 => "minecraft:bedrock",
        8 | 9 => "minecraft:water",
        10 | 11 => "minecraft:lava",
        12 => "minecraft:sand",
        13 => "minecraft:gravel",
        14 => "minecraft:gold_ore",
        15 => "minecraft:iron_ore",
        16 => "minecraft:coal_ore",
        17 => "minecraft:oak_log",
        18 => "minecraft:oak_leaves",
        19 => "minecraft:sponge",
        20 => "minecraft:glass",
        21 => "minecraft:lapis_ore",
        22 => "minecraft:lapis_block",
        24 => "minecraft:sandstone",
        35 => "minecraft:white_wool",
        41 => "minecraft:gold_block",
        42 => "minecraft:iron_block",
        45 => "minecraft:bricks",
        46 => "minecraft:tnt",
        48 => "minecraft:mossy_cobblestone",
        49 => "minecraft:obsidian",
        56 => "minecraft:diamond_ore",
        57 => "minecraft:diamond_block",
        73 => "minecraft:redstone_ore",
        78 => "minecraft:snow",
        79 => "minecraft:ice",
        80 => "minecraft:snow_block",
        82 => "minecraft:clay",
        87 => "minecraft:netherrack",
        88 => "minecraft:soul_sand",
        89 => "minecraft:glowstone",
        98 => "minecraft:stone_bricks",
        110 => "minecraft:mycelium",
        112 => "minecraft:nether_bricks",
        121 => "minecraft:end_stone",
        129 => "minecraft:emerald_ore",
        133 => "minecraft:emerald_block",
        155 => "minecraft:quartz_block",
        168 => "minecraft:prismarine",
        169 => "minecraft:sea_lantern",
        172 => "minecraft:terracotta",
        174 => "minecraft:packed_ice",
        201 => "minecraft:purpur_block",
        203 => "minecraft:purpur_pillar",
        206 => "minecraft:end_bricks",
        210 => "minecraft:structure_void",
        251 => "minecraft:concrete",
        252 => "minecraft:concrete_powder",
        _ => "minecraft:air",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::LevelFlatGeneratorInfoFix;

    #[test]
    fn empty_becomes_default() {
        assert_eq!(
            LevelFlatGeneratorInfoFix::fix_string(""),
            "minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;1;village"
        );
    }

    #[test]
    fn invalid_version_becomes_default() {
        assert_eq!(
            LevelFlatGeneratorInfoFix::fix_string("4;1,2"),
            "minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;1;village"
        );
    }

    #[test]
    fn version_three_modernizes_names() {
        assert_eq!(
            LevelFlatGeneratorInfoFix::fix_string("3;2*stone,grass_block"),
            "2*minecraft:stone,minecraft:grass_block"
        );
    }

    #[test]
    fn old_numeric_ids_map() {
        assert_eq!(
            LevelFlatGeneratorInfoFix::fix_string("0;7,2x3,2"),
            "minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block"
        );
    }

    #[test]
    fn trailing_sections_preserved() {
        assert_eq!(
            LevelFlatGeneratorInfoFix::fix_string("1;1;village;extra"),
            "minecraft:stone;village;extra"
        );
    }

    #[test]
    fn amount_one_omitted() {
        assert_eq!(
            LevelFlatGeneratorInfoFix::fix_string("3;1*stone"),
            "minecraft:stone"
        );
    }
}
