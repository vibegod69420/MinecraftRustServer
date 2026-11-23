use std::collections::HashMap;
use std::sync::OnceLock;

/// Minimal self-contained dynamic value representation for datafix porting.
/// Mirrors the subset of DFU `Dynamic<?>` operations used by this fix.
// JAVA: com.mojang.serialization.Dynamic (minimal subset)
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Long(i64),
    LongList(Vec<i64>),
    List(Vec<Dynamic>),
    Map(Vec<(Dynamic, Dynamic)>),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#get(String)
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(entries) => {
                for (k, v) in entries {
                    if k.as_string("") == key {
                        return v.clone();
                    }
                }
                Dynamic::Null
            }
            _ => Dynamic::Null,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#getMapValues()
    pub fn get_map_values(&self) -> Option<Vec<(Dynamic, Dynamic)>> {
        match self {
            Dynamic::Map(entries) => Some(entries.clone()),
            _ => None,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asString(String)
    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asLongStream()
    pub fn as_long_stream(&self) -> Vec<i64> {
        match self {
            Dynamic::LongList(v) => v.clone(),
            Dynamic::List(v) => {
                let mut out = Vec::with_capacity(v.len());
                for d in v {
                    match d {
                        Dynamic::Long(l) => out.push(*l),
                        _ => return Vec::new(),
                    }
                }
                out
            }
            _ => Vec::new(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asList(Function)
    pub fn as_list_identity(&self) -> Vec<Dynamic> {
        match self {
            Dynamic::List(v) => v.clone(),
            _ => Vec::new(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createMap(Map)
    pub fn create_map(&self, entries: Vec<(Dynamic, Dynamic)>) -> Dynamic {
        Dynamic::Map(entries)
    }

    // JAVA: com.mojang.serialization.Dynamic#createString(String)
    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    // JAVA: com.mojang.serialization.Dynamic#createLongList(LongStream)
    pub fn create_long_list<I: IntoIterator<Item = i64>>(&self, iter: I) -> Dynamic {
        Dynamic::LongList(iter.into_iter().collect())
    }

    // JAVA: com.mojang.serialization.Dynamic#update(String, Function)
    pub fn update<F>(&self, key: &str, f: F) -> Dynamic
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        let current = self.get(key);
        let updated = f(current);
        self.set(key, updated)
    }

    // JAVA: com.mojang.serialization.Dynamic#set(String, Dynamic)
    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(entries) => {
                let mut out: Vec<(Dynamic, Dynamic)> = Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k.as_string("") == key {
                        out.push((Dynamic::String(key.to_string()), value.clone()));
                        replaced = true;
                    } else {
                        out.push((k.clone(), v.clone()));
                    }
                }
                if !replaced {
                    out.push((Dynamic::String(key.to_string()), value));
                }
                Dynamic::Map(out)
            }
            _ => {
                let mut out = Vec::new();
                out.push((Dynamic::String(key.to_string()), value));
                Dynamic::Map(out)
            }
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix$Conversion
#[derive(Clone, Debug, PartialEq)]
pub struct Conversion {
    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix$Conversion#biomeMapping
    pub biome_mapping: HashMap<String, String>,
    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix$Conversion#fallback
    pub fallback: String,
}

impl Conversion {
    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix$Conversion#trivial(String)
    pub fn trivial(fallback: &str) -> Self {
        Self {
            biome_mapping: HashMap::new(),
            fallback: fallback.to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix$Conversion#biomeMapped(Map, String)
    pub fn biome_mapped(mapping: Vec<(Vec<&str>, &str)>, fallback: &str) -> Self {
        Self {
            biome_mapping: unpack(mapping),
            fallback: fallback.to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix$Conversion#unpack(Map)
    fn unpack_internal(mapping: Vec<(Vec<&str>, &str)>) -> HashMap<String, String> {
        unpack(mapping)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix$Conversion#unpack(Map)
fn unpack(mapping: Vec<(Vec<&str>, &str)>) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (keys, value) in mapping {
        for k in keys {
            out.insert(k.to_string(), value.to_string());
        }
    }
    out
}

// JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix
pub struct StructuresBecomeConfiguredFix {
    conversion_map: HashMap<String, Conversion>,
}

fn default_conversion_map() -> HashMap<String, Conversion> {
    let mut m = HashMap::new();
    m.insert(
        "mineshaft".to_string(),
        Conversion::biome_mapped(
            vec![(
                vec![
                    "minecraft:badlands",
                    "minecraft:eroded_badlands",
                    "minecraft:wooded_badlands",
                ],
                "minecraft:mineshaft_mesa",
            )],
            "minecraft:mineshaft",
        ),
    );
    m.insert(
        "shipwreck".to_string(),
        Conversion::biome_mapped(
            vec![(
                vec!["minecraft:beach", "minecraft:snowy_beach"],
                "minecraft:shipwreck_beached",
            )],
            "minecraft:shipwreck",
        ),
    );
    m.insert(
        "ocean_ruin".to_string(),
        Conversion::biome_mapped(
            vec![(
                vec![
                    "minecraft:warm_ocean",
                    "minecraft:lukewarm_ocean",
                    "minecraft:deep_lukewarm_ocean",
                ],
                "minecraft:ocean_ruin_warm",
            )],
            "minecraft:ocean_ruin_cold",
        ),
    );
    m.insert(
        "village".to_string(),
        Conversion::biome_mapped(
            vec![
                (vec!["minecraft:desert"], "minecraft:village_desert"),
                (vec!["minecraft:savanna"], "minecraft:village_savanna"),
                (vec!["minecraft:snowy_plains"], "minecraft:village_snowy"),
                (vec!["minecraft:taiga"], "minecraft:village_taiga"),
            ],
            "minecraft:village_plains",
        ),
    );
    m.insert(
        "ruined_portal".to_string(),
        Conversion::biome_mapped(
            vec![
                (vec!["minecraft:desert"], "minecraft:ruined_portal_desert"),
                (
                    vec![
                        "minecraft:badlands",
                        "minecraft:eroded_badlands",
                        "minecraft:wooded_badlands",
                        "minecraft:windswept_hills",
                        "minecraft:windswept_forest",
                        "minecraft:windswept_gravelly_hills",
                        "minecraft:savanna_plateau",
                        "minecraft:windswept_savanna",
                        "minecraft:stony_shore",
                        "minecraft:meadow",
                        "minecraft:frozen_peaks",
                        "minecraft:jagged_peaks",
                        "minecraft:stony_peaks",
                        "minecraft:snowy_slopes",
                    ],
                    "minecraft:ruined_portal_mountain",
                ),
                (
                    vec![
                        "minecraft:bamboo_jungle",
                        "minecraft:jungle",
                        "minecraft:sparse_jungle",
                    ],
                    "minecraft:ruined_portal_jungle",
                ),
                (
                    vec![
                        "minecraft:deep_frozen_ocean",
                        "minecraft:deep_cold_ocean",
                        "minecraft:deep_ocean",
                        "minecraft:deep_lukewarm_ocean",
                        "minecraft:frozen_ocean",
                        "minecraft:ocean",
                        "minecraft:cold_ocean",
                        "minecraft:lukewarm_ocean",
                        "minecraft:warm_ocean",
                    ],
                    "minecraft:ruined_portal_ocean",
                ),
            ],
            "minecraft:ruined_portal",
        ),
    );
    for (k, v) in [
        ("pillager_outpost", "minecraft:pillager_outpost"),
        ("mansion", "minecraft:mansion"),
        ("jungle_pyramid", "minecraft:jungle_pyramid"),
        ("desert_pyramid", "minecraft:desert_pyramid"),
        ("igloo", "minecraft:igloo"),
        ("swamp_hut", "minecraft:swamp_hut"),
        ("stronghold", "minecraft:stronghold"),
        ("monument", "minecraft:monument"),
        ("fortress", "minecraft:fortress"),
        ("endcity", "minecraft:end_city"),
        ("buried_treasure", "minecraft:buried_treasure"),
        ("nether_fossil", "minecraft:nether_fossil"),
        ("bastion_remnant", "minecraft:bastion_remnant"),
    ] {
        m.insert(k.to_string(), Conversion::trivial(v));
    }
    m
}

fn conversion_map() -> &'static HashMap<String, Conversion> {
    static MAP: OnceLock<HashMap<String, Conversion>> = OnceLock::new();
    MAP.get_or_init(default_conversion_map)
}

impl StructuresBecomeConfiguredFix {
    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix#<init>(Schema)
    pub fn new() -> Self {
        Self {
            conversion_map: default_conversion_map(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix#<init>(Schema)
    pub fn with_conversion_map(conversion_map: HashMap<String, Conversion>) -> Self {
        Self { conversion_map }
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix#fix(Dynamic)
    pub fn fix(&self, input: &Dynamic) -> Dynamic {
        input.update("structures", |structures| {
            let with_starts = structures.update("starts", |starts| self.update_starts(&starts, input));
            with_starts.update("References", |refs| self.update_references(&refs, input))
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix#updateStarts(Dynamic, Dynamic)
    fn update_starts(&self, starts: &Dynamic, chunk: &Dynamic) -> Dynamic {
        let map_values = starts.get_map_values().unwrap_or_default();
        let mut result: Vec<(Dynamic, Dynamic)> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();

        for (key, value) in map_values {
            let id = value.get("id").as_string("INVALID");
            if id == "INVALID" {
                continue;
            }
            if let Some(updated_type) = self.find_updated_structure_type(&key, chunk) {
                let updated_key_str = updated_type.as_string("");
                if let Some(&idx) = index.get(&updated_key_str) {
                    // Java computeIfAbsent: keep existing, do not replace
                    let _ = idx;
                } else {
                    let new_value = value.set("id", updated_type.clone());
                    index.insert(updated_key_str, result.len());
                    result.push((updated_type, new_value));
                }
            } else {
                // JAVA: LOGGER.warn("Encountered unknown structure in datafixer: {}", $$2x.asString("<missing key>"));
                // Warning omitted in self-contained port.
                let _ = key.as_string("<missing key>");
            }
        }

        chunk.create_map(result)
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix#updateReferences(Dynamic, Dynamic)
    fn update_references(&self, references: &Dynamic, chunk: &Dynamic) -> Dynamic {
        let map_values = references.get_map_values().unwrap_or_default();
        let mut result: Vec<(Dynamic, Dynamic)> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();

        for (key, value) in map_values {
            let stream = value.as_long_stream();
            if (stream.len() as i64) == 0 {
                continue;
            }
            if let Some(updated_type) = self.find_updated_structure_type(&key, chunk) {
                let updated_key_str = updated_type.as_string("");
                if let Some(&idx) = index.get(&updated_key_str) {
                    let existing = result[idx].1.as_long_stream();
                    let mut merged = Vec::with_capacity(existing.len() + stream.len());
                    merged.extend_from_slice(&existing);
                    merged.extend_from_slice(&stream);
                    result[idx].1 = value.create_long_list(merged);
                } else {
                    index.insert(updated_key_str, result.len());
                    result.push((updated_type, value));
                }
            } else {
                // JAVA: LOGGER.warn("Encountered unknown structure in datafixer: {}", $$2x.asString("<missing key>"));
                let _ = key.as_string("<missing key>");
            }
        }

        chunk.create_map(result)
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix#findUpdatedStructureType(Dynamic, Dynamic)
    fn find_updated_structure_type(&self, key: &Dynamic, chunk: &Dynamic) -> Option<Dynamic> {
        let structure_name = key.as_string("UNKNOWN").to_lowercase();
        let conversion = self.conversion_map.get(&structure_name)?;
        let mut fallback = conversion.fallback.clone();
        if !conversion.biome_mapping.is_empty() {
            if let Some(guess) = self.guess_configuration(chunk, conversion) {
                fallback = guess;
            }
        }
        Some(chunk.create_string(&fallback))
    }

    // JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix#guessConfiguration(Dynamic, Conversion)
    fn guess_configuration(&self, chunk: &Dynamic, conversion: &Conversion) -> Option<String> {
        // Object2IntArrayMap preserves insertion order; we replicate with Vec for deterministic max-on-tie behavior.
        let mut counts: Vec<(String, i32)> = Vec::new();

        let sections = chunk.get("sections").as_list_identity();
        for section in sections {
            let palette = section.get("biomes").get("palette").as_list_identity();
            for biome in palette {
                let biome_name = biome.as_string("");
                if let Some(mapped) = conversion.biome_mapping.get(&biome_name) {
                    let mut found = false;
                    for entry in &mut counts {
                        if &entry.0 == mapped {
                            entry.1 = entry.1.wrapping_add(1);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        counts.push((mapped.clone(), 1));
                    }
                }
            }
        }

        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| name)
    }
}

impl Default for StructuresBecomeConfiguredFix {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the default Java 1.21.11 conversion map.
// JAVA: net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix#CONVERSION_MAP
pub fn default_structures_become_configured_conversion_map() -> &'static HashMap<String, Conversion> {
    conversion_map()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dyn_map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(
            entries
                .into_iter()
                .map(|(k, v)| (Dynamic::String(k.to_string()), v))
                .collect(),
        )
    }

    fn dyn_str(s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    fn dyn_long_list(v: &[i64]) -> Dynamic {
        Dynamic::LongList(v.to_vec())
    }

    #[test]
    fn trivial_conversion_updates_start_id() {
        let fix = StructuresBecomeConfiguredFix::new();
        let chunk = dyn_map(vec![
            (
                "structures",
                dyn_map(vec![(
                    "starts",
                    dyn_map(vec![(
                        "mansion",
                        dyn_map(vec![("id", dyn_str("mansion"))]),
                    )]),
                )]),
            ),
            ("sections", Dynamic::List(vec![])),
        ]);
        let fixed = fix.fix(&chunk);
        let starts = fixed.get("structures").get("starts");
        let mansion = starts.get("minecraft:mansion");
        assert_eq!(mansion.get("id").as_string(""), "minecraft:mansion");
    }

    #[test]
    fn biome_mapped_conversion_picks_majority_biome() {
        let fix = StructuresBecomeConfiguredFix::new();
        let chunk = dyn_map(vec![
            (
                "sections",
                Dynamic::List(vec![
                    dyn_map(vec![(
                        "biomes",
                        dyn_map(vec![(
                            "palette",
                            Dynamic::List(vec![
                                dyn_str("minecraft:desert"),
                                dyn_str("minecraft:desert"),
                                dyn_str("minecraft:badlands"),
                            ]),
                        )]),
                    )]),
                    dyn_map(vec![(
                        "biomes",
                        dyn_map(vec![(
                            "palette",
                            Dynamic::List(vec![dyn_str("minecraft:desert")]),
                        )]),
                    )]),
                ]),
            ),
            (
                "structures",
                dyn_map(vec![(
                    "starts",
                    dyn_map(vec![(
                        "village",
                        dyn_map(vec![("id", dyn_str("village"))]),
                    )]),
                )]),
            ),
        ]);
        let fixed = fix.fix(&chunk);
        let starts = fixed.get("structures").get("starts");
        let village = starts.get("minecraft:village_desert");
        assert_eq!(village.get("id").as_string(""), "minecraft:village_desert");
        assert_eq!(starts.get("minecraft:village").as_string("MISSING"), "MISSING");
    }

    #[test]
    fn unknown_structure_is_skipped_and_invalid_id_ignored() {
        let fix = StructuresBecomeConfiguredFix::new();
        let chunk = dyn_map(vec![
            (
                "structures",
                dyn_map(vec![(
                    "starts",
                    dyn_map(vec![
                        ("unknown_thing", dyn_map(vec![("id", dyn_str("unknown_thing"))])),
                        ("broken", dyn_map(vec![("id", dyn_str("INVALID"))])),
                    ]),
                )]),
            ),
            ("sections", Dynamic::List(vec![])),
        ]);
        let fixed = fix.fix(&chunk);
        let starts = fixed.get("structures").get("starts");
        let values = starts.get_map_values().unwrap();
        assert!(values.is_empty());
    }

    #[test]
    fn references_are_concatenated_for_same_mapped_type() {
        let fix = StructuresBecomeConfiguredFix::new();
        let chunk = dyn_map(vec![
            (
                "structures",
                dyn_map(vec![(
                    "References",
                    dyn_map(vec![
                        ("mansion", dyn_long_list(&[1_i64, 2_i64])),
                        ("MANSION", dyn_long_list(&[3_i64])),
                    ]),
                )]),
            ),
            ("sections", Dynamic::List(vec![])),
        ]);
        let fixed = fix.fix(&chunk);
        let refs = fixed.get("structures").get("References");
        let mansion_refs = refs.get("minecraft:mansion").as_long_stream();
        assert_eq!(mansion_refs, vec![1_i64, 2_i64, 3_i64]);
    }

    #[test]
    fn empty_reference_stream_is_ignored() {
        let fix = StructuresBecomeConfiguredFix::new();
        let chunk = dyn_map(vec![
            (
                "structures",
                dyn_map(vec![(
                    "References",
                    dyn_map(vec![("mansion", dyn_long_list(&[]))]),
                )]),
            ),
            ("sections", Dynamic::List(vec![])),
        ]);
        let fixed = fix.fix(&chunk);
        let refs = fixed.get("structures").get("References");
        assert!(refs.get_map_values().unwrap().is_empty());
    }

    #[test]
    fn fallback_used_when_no_biome_mapping_match() {
        let fix = StructuresBecomeConfiguredFix::new();
        let chunk = dyn_map(vec![
            (
                "sections",
                Dynamic::List(vec![dyn_map(vec![(
                    "biomes",
                    dyn_map(vec![(
                        "palette",
                        Dynamic::List(vec![dyn_str("minecraft:plains")]),
                    )]),
                )])]),
            ),
            (
                "structures",
                dyn_map(vec![(
                    "starts",
                    dyn_map(vec![(
                        "village",
                        dyn_map(vec![("id", dyn_str("village"))]),
                    )]),
                )]),
            ),
        ]);
        let fixed = fix.fix(&chunk);
        let starts = fixed.get("structures").get("starts");
        let village = starts.get("minecraft:village_plains");
        assert_eq!(village.get("id").as_string(""), "minecraft:village_plains");
    }
}
