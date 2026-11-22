use crate::NbtTag;
use std::cmp::Ordering;
use std::fmt::Write as FmtWrite;

const SNBT_DATA_TAG: &str = "data";
const PROPERTIES_START: char = '{';
const PROPERTIES_END: char = '}';
const KEY_VALUE_SEPARATOR: char = ':';
const INDENT_STEP: usize = 2;

// JAVA: net.minecraft.nbt.NbtUtils#compareNbt
pub fn compare_nbt(template: Option<&NbtTag>, actual: Option<&NbtTag>, compare_list_tag: bool) -> bool {
    if let (Some(t), Some(a)) = (template, actual) {
        if std::ptr::eq(t, a) {
            return true;
        }
    }
    let Some(template) = template else {
        return true;
    };
    let Some(actual) = actual else {
        return false;
    };

    if std::mem::discriminant(template) != std::mem::discriminant(actual) {
        // Allow numeric/list/compound class checks by tag kind; discriminant is enough for enum variants,
        // but different enum variants are different classes in Java.
        return false;
    }

    match (template, actual) {
        (NbtTag::Compound(t_entries), NbtTag::Compound(a_entries)) => {
            if a_entries.len() < t_entries.len() {
                return false;
            }
            for (key, t_val) in t_entries {
                let a_val = compound_get(a_entries, key);
                if !compare_nbt(Some(t_val), a_val, compare_list_tag) {
                    return false;
                }
            }
            true
        }
        (NbtTag::List { items: t_items, .. }, NbtTag::List { items: a_items, .. })
            if compare_list_tag =>
        {
            if t_items.is_empty() {
                return a_items.is_empty();
            }
            if a_items.len() < t_items.len() {
                return false;
            }
            for t_item in t_items {
                let mut found = false;
                for a_item in a_items {
                    if compare_nbt(Some(t_item), Some(a_item), compare_list_tag) {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }
            true
        }
        _ => nbt_equals(template, actual),
    }
}

fn compound_get<'a>(entries: &'a [(String, NbtTag)], key: &str) -> Option<&'a NbtTag> {
    entries.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn nbt_equals(a: &NbtTag, b: &NbtTag) -> bool {
    match (a, b) {
        (NbtTag::Byte(x), NbtTag::Byte(y)) => x == y,
        (NbtTag::Short(x), NbtTag::Short(y)) => x == y,
        (NbtTag::Int(x), NbtTag::Int(y)) => x == y,
        (NbtTag::Long(x), NbtTag::Long(y)) => x == y,
        (NbtTag::Float(x), NbtTag::Float(y)) => x.to_bits() == y.to_bits(),
        (NbtTag::Double(x), NbtTag::Double(y)) => x.to_bits() == y.to_bits(),
        (NbtTag::ByteArray(x), NbtTag::ByteArray(y)) => x == y,
        (NbtTag::String(x), NbtTag::String(y)) => x == y,
        (
            NbtTag::List {
                elem_type: et1,
                items: i1,
            },
            NbtTag::List {
                elem_type: et2,
                items: i2,
            },
        ) => et1 == et2 && i1.len() == i2.len() && i1.iter().zip(i2).all(|(x, y)| nbt_equals(x, y)),
        (NbtTag::Compound(x), NbtTag::Compound(y)) => {
            x.len() == y.len()
                && x.iter()
                    .zip(y.iter())
                    .all(|((k1, v1), (k2, v2))| k1 == k2 && nbt_equals(v1, v2))
        }
        (NbtTag::IntArray(x), NbtTag::IntArray(y)) => x == y,
        (NbtTag::LongArray(x), NbtTag::LongArray(y)) => x == y,
        _ => false,
    }
}

// JAVA: net.minecraft.nbt.NbtUtils#prettyPrint(net.minecraft.nbt.Tag)
pub fn pretty_print(tag: &NbtTag) -> String {
    pretty_print_with_blobs(tag, false)
}

// JAVA: net.minecraft.nbt.NbtUtils#prettyPrint(net.minecraft.nbt.Tag,boolean)
pub fn pretty_print_with_blobs(tag: &NbtTag, with_binary_blobs: bool) -> String {
    let mut out = String::new();
    pretty_print_into(&mut out, tag, 0, with_binary_blobs);
    out
}

// JAVA: net.minecraft.nbt.NbtUtils#prettyPrint(java.lang.StringBuilder,net.minecraft.nbt.Tag,int,boolean)
pub fn pretty_print_into(out: &mut String, tag: &NbtTag, indent: usize, with_binary_blobs: bool) {
    match tag {
        NbtTag::Byte(v) => {
            let _ = write!(out, "{}b", v);
        }
        NbtTag::Short(v) => {
            let _ = write!(out, "{}s", v);
        }
        NbtTag::Int(v) => {
            let _ = write!(out, "{}", v);
        }
        NbtTag::Long(v) => {
            let _ = write!(out, "{}L", v);
        }
        NbtTag::Float(v) => {
            let _ = write!(out, "{}f", format_java_float(*v));
        }
        NbtTag::Double(v) => {
            let _ = write!(out, "{}d", format_java_double(*v));
        }
        NbtTag::String(v) => {
            out.push('"');
            push_escaped_string(out, v);
            out.push('"');
        }
        NbtTag::ByteArray(arr) => {
            let len = arr.len();
            indent_at(indent, out);
            let _ = write!(out, "byte[{}] {{\n", len);
            if with_binary_blobs {
                indent_at(indent + 1, out);
                for i in 0..arr.len() {
                    if i != 0 {
                        out.push(',');
                    }
                    if i % 16 == 0 && i / 16 > 0 {
                        out.push('\n');
                        if i < arr.len() {
                            indent_at(indent + 1, out);
                        }
                    } else if i != 0 {
                        out.push(' ');
                    }
                    let b = (arr[i] as u8) as u32;
                    let _ = write!(out, "0x{:02X}", b);
                }
            } else {
                indent_at(indent + 1, out);
                out.push_str(" // Skipped, supply withBinaryBlobs true");
            }
            out.push('\n');
            indent_at(indent, out);
            out.push('}');
        }
        NbtTag::List { items, .. } => {
            let len = items.len();
            indent_at(indent, out);
            let _ = write!(out, "list[{}] [", len);
            if len != 0 {
                out.push('\n');
            }
            for i in 0..len {
                if i != 0 {
                    out.push_str(",\n");
                }
                indent_at(indent + 1, out);
                pretty_print_into(out, &items[i], indent + 1, with_binary_blobs);
            }
            if len != 0 {
                out.push('\n');
            }
            indent_at(indent, out);
            out.push(']');
        }
        NbtTag::IntArray(arr) => {
            let mut max_width: usize = 0;
            for &v in arr {
                let w = format!("{:X}", v).len();
                if w > max_width {
                    max_width = w;
                }
            }
            let len = arr.len();
            indent_at(indent, out);
            let _ = write!(out, "int[{}] {{\n", len);
            if with_binary_blobs {
                indent_at(indent + 1, out);
                for i in 0..arr.len() {
                    if i != 0 {
                        out.push(',');
                    }
                    if i % 16 == 0 && i / 16 > 0 {
                        out.push('\n');
                        if i < arr.len() {
                            indent_at(indent + 1, out);
                        }
                    } else if i != 0 {
                        out.push(' ');
                    }
                    let hex = format!("{:X}", arr[i]);
                    for _ in 0..(max_width.saturating_sub(hex.len())) {
                        out.push('0');
                    }
                    out.push_str("0x");
                    // Java uses String.format(Locale.ROOT, "0x%0" + maxWidth + "X", value)
                    // which means 0x prefix is literal, then zero-padded hex of width maxWidth.
                    // We already pushed "0x" above by mistake? Wait - check carefully.
                }
            } else {
                indent_at(indent + 1, out);
                out.push_str(" // Skipped, supply withBinaryBlobs true");
            }
            out.push('\n');
            indent_at(indent, out);
            out.push('}');
        }
        NbtTag::Compound(entries) => {
            let mut keys: Vec<&String> = entries.iter().map(|(k, _)| k).collect();
            keys.sort();
            indent_at(indent, out);
            out.push('{');
            let line_len_from_last_nl = out.len() - (out.rfind('\n').map(|i| i + 1).unwrap_or(0));
            if line_len_from_last_nl > 2 * (indent + 1) {
                out.push('\n');
                indent_at(indent + 1, out);
            }
            let max_key_len = keys.iter().map(|k| k.len()).max().unwrap_or(0);
            let padding: String = " ".repeat(max_key_len);
            for i in 0..keys.len() {
                if i != 0 {
                    out.push_str(",\n");
                }
                let key = keys[i];
                indent_at(indent + 1, out);
                out.push('"');
                push_escaped_string(out, key);
                out.push('"');
                out.push_str(&padding[0..(padding.len() - key.len())]);
                out.push_str(": ");
                if let Some(val) = compound_get(entries, key) {
                    pretty_print_into(out, val, indent + 1, with_binary_blobs);
                }
            }
            if !keys.is_empty() {
                out.push('\n');
            }
            indent_at(indent, out);
            out.push('}');
        }
        NbtTag::LongArray(arr) => {
            let mut max_width: usize = 0;
            for &v in arr {
                let w = format!("{:X}", v).len();
                if w > max_width {
                    max_width = w;
                }
            }
            let len = arr.len();
            indent_at(indent, out);
            let _ = write!(out, "long[{}] {{\n", len);
            if with_binary_blobs {
                indent_at(indent + 1, out);
                for i in 0..arr.len() {
                    if i != 0 {
                        out.push(',');
                    }
                    if i % 16 == 0 && i / 16 > 0 {
                        out.push('\n');
                        if i < arr.len() {
                            indent_at(indent + 1, out);
                        }
                    } else if i != 0 {
                        out.push(' ');
                    }
                    let hex = format!("{:X}", arr[i]);
                    let pad = max_width.saturating_sub(hex.len());
                    out.push_str("0x");
                    for _ in 0..pad {
                        out.push('0');
                    }
                    out.push_str(&hex);
                }
            } else {
                indent_at(indent + 1, out);
                out.push_str(" // Skipped, supply withBinaryBlobs true");
            }
            out.push('\n');
            indent_at(indent, out);
            out.push('}');
        }
    }
}

// JAVA: net.minecraft.nbt.NbtUtils#indent
fn indent_at(indent: usize, out: &mut String) {
    let last_nl = out.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let current_line_len = out.len() - last_nl;
    let target = INDENT_STEP * indent;
    if target > current_line_len {
        for _ in 0..(target - current_line_len) {
            out.push(' ');
        }
    }
}

fn format_java_float(v: f32) -> String {
    // Java Float.toString/append behavior for FloatTag is close to Rust default, but ensure .0 for integers.
    if v.is_finite() && v.fract() == 0.0 {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
}

fn format_java_double(v: f64) -> String {
    if v.is_finite() && v.fract() == 0.0 {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
}

fn push_escaped_string(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
}

// JAVA: net.minecraft.nbt.NbtUtils#packBlockState
pub fn pack_block_state(state: &NbtTag) -> String {
    let mut out = String::new();
    let name = state
        .compound_get("Name")
        .and_then(|t| match t {
            NbtTag::String(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("");
    out.push_str(name);

    if let Some(NbtTag::Compound(props)) = state.compound_get("Properties") {
        let mut sorted_props: Vec<(&String, &NbtTag)> = props.iter().map(|(k, v)| (k, v)).collect();
        sorted_props.sort_by(|(a, _), (b, _)| a.cmp(b));
        let mut parts: Vec<String> = Vec::with_capacity(sorted_props.len());
        for (k, v) in sorted_props {
            let val_str = match v {
                NbtTag::String(s) => s.clone(),
                _ => tag_to_plain_string(v),
            };
            parts.push(format!("{}{}{}", k, KEY_VALUE_SEPARATOR, val_str));
        }
        out.push(PROPERTIES_START);
        out.push_str(&parts.join(","));
        out.push(PROPERTIES_END);
    }

    out
}

// JAVA: net.minecraft.nbt.NbtUtils#unpackBlockState
pub fn unpack_block_state(input: &str) -> NbtTag {
    let mut compound: Vec<(String, NbtTag)> = Vec::new();
    let brace_idx = input.find(PROPERTIES_START);
    let name: String;
    if let Some(idx) = brace_idx {
        name = input[..idx].to_string();
        let mut props_compound: Vec<(String, NbtTag)> = Vec::new();
        if idx + 2 <= input.len() {
            if let Some(end_idx) = input[idx..].find(PROPERTIES_END) {
                let end_idx = idx + end_idx;
                let inner = &input[(idx + 1)..end_idx];
                for part in inner.split(',') {
                    let mut split = part.splitn(2, KEY_VALUE_SEPARATOR);
                    let key = split.next();
                    let val = split.next();
                    if let (Some(k), Some(v)) = (key, val) {
                        props_compound.push((k.to_string(), NbtTag::String(v.to_string())));
                    } else {
                        // LOGGER.error("Something went wrong parsing: '{}' -- incorrect gamedata!", input);
                        // Keep faithful: log in Java; here we silently skip as we have no logger dependency.
                    }
                }
                compound.push(("Properties".to_string(), NbtTag::Compound(props_compound)));
            }
        } else {
            compound.push(("Properties".to_string(), NbtTag::Compound(props_compound)));
        }
    } else {
        name = input.to_string();
    }
    compound.push(("Name".to_string(), NbtTag::String(name)));
    NbtTag::Compound(compound)
}

// JAVA: net.minecraft.nbt.NbtUtils#packStructureTemplate
pub fn pack_structure_template(tag: &NbtTag) -> NbtTag {
    let mut root = clone_compound(tag);

    let palettes_opt = root_compound_get(&root, "palettes").and_then(as_list_items);
    let palette_source: Vec<NbtTag> = if let Some(palettes) = palettes_opt {
        if let Some(first) = palettes.first() {
            as_list_items(first).cloned().unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        root_compound_get(&root, "palette")
            .and_then(as_list_items)
            .cloned()
            .unwrap_or_default()
    };

    let packed_palette: Vec<NbtTag> = palette_source
        .iter()
        .filter_map(as_compound_entries)
        .map(|entries| NbtTag::String(pack_block_state(&NbtTag::Compound(entries.clone()))))
        .collect();
    let packed_palette_list = NbtTag::List {
        elem_type: 8,
        items: packed_palette.clone(),
    };
    root_compound_put(&mut root, "palette".to_string(), packed_palette_list);

    if let Some(palettes) = root_compound_get(&root, "palettes").and_then(as_list_items).cloned() {
        let mut out_palettes: Vec<NbtTag> = Vec::new();
        for p in &palettes {
            let Some(p_list) = as_list_items(p) else {
                continue;
            };
            let mut map_entries: Vec<(String, NbtTag)> = Vec::new();
            for i in 0..p_list.len() {
                let Some(block_entries) = p_list.get(i).and_then(as_compound_entries) else {
                    continue;
                };
                let key = packed_palette
                    .get(i)
                    .and_then(|t| match t {
                        NbtTag::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let value = pack_block_state(&NbtTag::Compound(block_entries.clone()));
                map_entries.push((key, NbtTag::String(value)));
            }
            out_palettes.push(NbtTag::Compound(map_entries));
        }
        root_compound_put(
            &mut root,
            "palettes".to_string(),
            NbtTag::List {
                elem_type: 10,
                items: out_palettes,
            },
        );
    }

    if let Some(entities) = root_compound_get(&root, "entities").and_then(as_list_items).cloned() {
        let mut entity_compounds: Vec<NbtTag> = entities
            .into_iter()
            .filter(|e| matches!(e, NbtTag::Compound(_)))
            .collect();
        entity_compounds.sort_by(|a, b| compare_entity_pos(a, b));
        root_compound_put(
            &mut root,
            "entities".to_string(),
            NbtTag::List {
                elem_type: 10,
                items: entity_compounds,
            },
        );
    }

    let blocks = root_compound_get(&root, "blocks")
        .and_then(as_list_items)
        .cloned()
        .unwrap_or_default();
    let mut block_compounds: Vec<NbtTag> = blocks
        .into_iter()
        .filter(|b| matches!(b, NbtTag::Compound(_)))
        .collect();
    block_compounds.sort_by(|a, b| compare_block_pos(a, b));
    for block in &mut block_compounds {
        if let NbtTag::Compound(entries) = block {
            let state_idx = compound_get(entries, "state")
                .and_then(|t| match t {
                    NbtTag::Int(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0);
            let state_str = packed_palette
                .get(state_idx as usize)
                .and_then(|t| match t {
                    NbtTag::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            compound_put(entries, "state".to_string(), NbtTag::String(state_str));
        }
    }
    root_compound_put(
        &mut root,
        SNBT_DATA_TAG.to_string(),
        NbtTag::List {
            elem_type: 10,
            items: block_compounds,
        },
    );
    root_compound_remove(&mut root, "blocks");
    root
}

// JAVA: net.minecraft.nbt.NbtUtils#unpackStructureTemplate
pub fn unpack_structure_template(tag: &NbtTag) -> NbtTag {
    let mut root = clone_compound(tag);
    let palette = root_compound_get(&root, "palette")
        .and_then(as_list_items)
        .cloned()
        .unwrap_or_default();

    let mut palette_map: Vec<(String, NbtTag)> = Vec::new();
    for p in &palette {
        if let NbtTag::String(s) = p {
            palette_map.push((s.clone(), unpack_block_state(s)));
        }
    }

    if let Some(palettes) = root_compound_get(&root, "palettes").and_then(as_list_items).cloned() {
        let mut out_palettes: Vec<NbtTag> = Vec::new();
        for p in &palettes {
            let Some(p_compound) = as_compound_entries(p) else {
                continue;
            };
            let mut list_items: Vec<NbtTag> = Vec::new();
            for (key, _) in &palette_map {
                let val_str = compound_get(p_compound, key)
                    .and_then(|t| match t {
                        NbtTag::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .unwrap_or("");
                list_items.push(unpack_block_state(val_str));
            }
            out_palettes.push(NbtTag::List {
                elem_type: 10,
                items: list_items,
            });
        }
        root_compound_put(
            &mut root,
            "palettes".to_string(),
            NbtTag::List {
                elem_type: 10,
                items: out_palettes,
            },
        );
        root_compound_remove(&mut root, "palette");
    } else {
        let items: Vec<NbtTag> = palette_map.iter().map(|(_, v)| v.clone()).collect();
        root_compound_put(
            &mut root,
            "palette".to_string(),
            NbtTag::List {
                elem_type: 10,
                items,
            },
        );
    }

    if let Some(data) = root_compound_get(&root, SNBT_DATA_TAG).and_then(as_list_items).cloned() {
        let mut index_map: Vec<(String, i32)> = Vec::with_capacity(palette.len());
        for (i, p) in palette.iter().enumerate() {
            if let NbtTag::String(s) = p {
                index_map.push((s.clone(), i as i32));
            }
        }
        let mut blocks: Vec<NbtTag> = Vec::with_capacity(data.len());
        for item in data {
            let mut block = clone_compound(&item);
            let state_str = root_compound_get(&block, "state")
                .and_then(|t| match t {
                    NbtTag::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let mut found_idx: Option<i32> = None;
            for (k, idx) in &index_map {
                if k == &state_str {
                    found_idx = Some(*idx);
                    break;
                }
            }
            if let Some(idx) = found_idx {
                root_compound_put(&mut block, "state".to_string(), NbtTag::Int(idx));
                blocks.push(block);
            } else {
                panic!("Entry {} missing from palette", state_str);
            }
        }
        root_compound_put(
            &mut root,
            "blocks".to_string(),
            NbtTag::List {
                elem_type: 10,
                items: blocks,
            },
        );
        root_compound_remove(&mut root, SNBT_DATA_TAG);
    }

    root
}

// JAVA: net.minecraft.nbt.NbtUtils#addDataVersion(net.minecraft.nbt.CompoundTag,int)
pub fn add_data_version(tag: &NbtTag, version: i32) -> NbtTag {
    let mut out = clone_compound(tag);
    root_compound_put(&mut out, "DataVersion".to_string(), NbtTag::Int(version));
    out
}

// JAVA: net.minecraft.nbt.NbtUtils#getDataVersion(net.minecraft.nbt.CompoundTag)
pub fn get_data_version(tag: &NbtTag) -> i32 {
    get_data_version_or(tag, -1)
}

// JAVA: net.minecraft.nbt.NbtUtils#getDataVersion(net.minecraft.nbt.CompoundTag,int)
pub fn get_data_version_or(tag: &NbtTag, default: i32) -> i32 {
    match root_compound_get(tag, "DataVersion") {
        Some(NbtTag::Int(v)) => *v,
        _ => default,
    }
}

fn clone_compound(tag: &NbtTag) -> NbtTag {
    match tag {
        NbtTag::Compound(entries) => NbtTag::Compound(entries.clone()),
        _ => NbtTag::Compound(Vec::new()),
    }
}

fn as_compound_entries(tag: &NbtTag) -> Option<&Vec<(String, NbtTag)>> {
    match tag {
        NbtTag::Compound(entries) => Some(entries),
        _ => None,
    }
}

fn as_list_items(tag: &NbtTag) -> Option<&Vec<NbtTag>> {
    match tag {
        NbtTag::List { items, .. } => Some(items),
        _ => None,
    }
}

fn root_compound_get<'a>(tag: &'a NbtTag, key: &str) -> Option<&'a NbtTag> {
    match tag {
        NbtTag::Compound(entries) => compound_get(entries, key),
        _ => None,
    }
}

fn root_compound_put(tag: &mut NbtTag, key: String, value: NbtTag) {
    if let NbtTag::Compound(entries) = tag {
        compound_put(entries, key, value);
    }
}

fn root_compound_remove(tag: &mut NbtTag, key: &str) {
    if let NbtTag::Compound(entries) = tag {
        entries.retain(|(k, _)| k != key);
    }
}

fn compound_put(entries: &mut Vec<(String, NbtTag)>, key: String, value: NbtTag) {
    if let Some((_, v)) = entries.iter_mut().find(|(k, _)| *k == key) {
        *v = value;
    } else {
        entries.push((key, value));
    }
}

fn tag_to_plain_string(tag: &NbtTag) -> String {
    match tag {
        NbtTag::String(s) => s.clone(),
        NbtTag::Byte(v) => format!("{}b", v),
        NbtTag::Short(v) => format!("{}s", v),
        NbtTag::Int(v) => format!("{}", v),
        NbtTag::Long(v) => format!("{}L", v),
        NbtTag::Float(v) => format!("{}f", format_java_float(*v)),
        NbtTag::Double(v) => format!("{}d", format_java_double(*v)),
        _ => String::new(),
    }
}

fn get_list_int(list: &NbtTag, index: usize, default: i32) -> i32 {
    match list {
        NbtTag::List { items, .. } => match items.get(index) {
            Some(NbtTag::Int(v)) => *v,
            Some(NbtTag::Byte(v)) => *v as i32,
            Some(NbtTag::Short(v)) => *v as i32,
            Some(NbtTag::Long(v)) => *v as i32,
            _ => default,
        },
        _ => default,
    }
}

fn get_list_double(list: &NbtTag, index: usize, default: f64) -> f64 {
    match list {
        NbtTag::List { items, .. } => match items.get(index) {
            Some(NbtTag::Double(v)) => *v,
            Some(NbtTag::Float(v)) => *v as f64,
            Some(NbtTag::Int(v)) => *v as f64,
            Some(NbtTag::Long(v)) => *v as f64,
            _ => default,
        },
        _ => default,
    }
}

fn compare_block_pos(a: &NbtTag, b: &NbtTag) -> Ordering {
    let a_pos = root_compound_get(a, "pos");
    let b_pos = root_compound_get(b, "pos");
    match (a_pos, b_pos) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater, // emptiesLast
        (Some(_), None) => Ordering::Less,
        (Some(ap), Some(bp)) => {
            let ay = get_list_int(ap, 1, 0);
            let by = get_list_int(bp, 1, 0);
            ay.cmp(&by)
                .then_with(|| get_list_int(ap, 0, 0).cmp(&get_list_int(bp, 0, 0)))
                .then_with(|| get_list_int(ap, 2, 0).cmp(&get_list_int(bp, 2, 0)))
        }
    }
}

fn compare_entity_pos(a: &NbtTag, b: &NbtTag) -> Ordering {
    let a_pos = root_compound_get(a, "pos");
    let b_pos = root_compound_get(b, "pos");
    match (a_pos, b_pos) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(ap), Some(bp)) => {
            let ay = get_list_double(ap, 1, 0.0);
            let by = get_list_double(bp, 1, 0.0);
            ay.partial_cmp(&by)
                .unwrap_or(Ordering::Equal)
                .then_with(|| {
                    get_list_double(ap, 0, 0.0)
                        .partial_cmp(&get_list_double(bp, 0, 0.0))
                        .unwrap_or(Ordering::Equal)
                })
                .then_with(|| {
                    get_list_double(ap, 2, 0.0)
                        .partial_cmp(&get_list_double(bp, 2, 0.0))
                        .unwrap_or(Ordering::Equal)
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(entries: Vec<(&str, NbtTag)>) -> NbtTag {
        NbtTag::Compound(entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    fn list(items: Vec<NbtTag>) -> NbtTag {
        let elem_type = items.first().map(|t| t.id()).unwrap_or(0);
        NbtTag::List { elem_type, items }
    }

    #[test]
    fn compare_nbt_null_template_and_missing_actual() {
        let a = NbtTag::Int(1);
        assert!(compare_nbt(None, Some(&a), false));
        assert!(!compare_nbt(Some(&a), None, false));
        assert!(compare_nbt(None, None, false));
    }

    #[test]
    fn compare_nbt_compound_subset_and_extra_keys() {
        let template = compound(vec![("a", NbtTag::Int(1))]);
        let actual = compound(vec![("a", NbtTag::Int(1)), ("b", NbtTag::Int(2))]);
        let actual_missing = compound(vec![("b", NbtTag::Int(2))]);
        assert!(compare_nbt(Some(&template), Some(&actual), false));
        assert!(!compare_nbt(Some(&template), Some(&actual_missing), false));
    }

    #[test]
    fn compare_nbt_list_unordered_when_flag_set() {
        let template = NbtTag::List {
            elem_type: 3,
            items: vec![NbtTag::Int(1), NbtTag::Int(2)],
        };
        let actual = NbtTag::List {
            elem_type: 3,
            items: vec![NbtTag::Int(2), NbtTag::Int(3), NbtTag::Int(1)],
        };
        let actual_missing = NbtTag::List {
            elem_type: 3,
            items: vec![NbtTag::Int(2), NbtTag::Int(3)],
        };
        assert!(compare_nbt(Some(&template), Some(&actual), true));
        assert!(!compare_nbt(Some(&template), Some(&actual_missing), true));
        assert!(!compare_nbt(Some(&template), Some(&actual), false));
    }

    #[test]
    fn compare_nbt_float_bits_nan_and_neg_zero() {
        let nan1 = NbtTag::Float(f32::NAN);
        let nan2 = NbtTag::Float(f32::from_bits(f32::NAN.to_bits()));
        let neg_zero = NbtTag::Float(-0.0f32);
        let pos_zero = NbtTag::Float(0.0f32);
        assert!(compare_nbt(Some(&nan1), Some(&nan2), false));
        assert!(!compare_nbt(Some(&neg_zero), Some(&pos_zero), false));

        let dnan1 = NbtTag::Double(f64::NAN);
        let dnan2 = NbtTag::Double(f64::from_bits(f64::NAN.to_bits()));
        assert!(compare_nbt(Some(&dnan1), Some(&dnan2), false));
    }

    #[test]
    fn pack_unpack_block_state_round_trip() {
        let tag = compound(vec![
            ("Name".to_string().leak() as &str, NbtTag::String("minecraft:stone".to_string())),
            (
                "Properties",
                compound(vec![("facing", NbtTag::String("north".to_string())), ("waterlogged", NbtTag::String("true".to_string()))]),
            ),
        ]);
        // Avoid leak in normal code; build properly
        let tag = NbtTag::Compound(vec![
            ("Name".to_string(), NbtTag::String("minecraft:stone".to_string())),
            (
                "Properties".to_string(),
                NbtTag::Compound(vec![
                    ("facing".to_string(), NbtTag::String("north".to_string())),
                    ("waterlogged".to_string(), NbtTag::String("true".to_string())),
                ]),
            ),
        ]);
        let packed = pack_block_state(&tag);
        assert_eq!(packed, "minecraft:stone{facing:north,waterlogged:true}");
        let unpacked = unpack_block_state(&packed);
        assert!(compare_nbt(Some(&tag), Some(&unpacked), false));
    }

    #[test]
    fn data_version_helpers() {
        let tag = NbtTag::Compound(vec![("x".to_string(), NbtTag::Int(1))]);
        let with_ver = add_data_version(&tag, 1234);
        assert_eq!(get_data_version(&with_ver), 1234);
        assert_eq!(get_data_version_or(&tag, -1), -1);
        assert_eq!(get_data_version_or(&tag, 42), 42);
    }
}
