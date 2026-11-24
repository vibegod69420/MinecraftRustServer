use std::collections::HashMap;
use std::fmt;

/// Minimal self-contained dynamic value representation for particle data.
/// Replaces Mojang DataFixerUpper Dynamic<T> for this 1:1 port.
#[derive(Clone, Debug, PartialEq)]
pub enum ParticleValue {
    String(String),
    Int(i32),
    Float(f32),
    List(Vec<ParticleValue>),
    Map(HashMap<String, ParticleValue>),
}

/// Minimal self-contained dynamic wrapper, mirroring the Java Dynamic<T> API
/// surface used by ParticleUnflatteningFix.
#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: ParticleValue,
}

impl Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#fix (helper: createString)
    pub fn create_string(s: &str) -> Self {
        Dynamic {
            value: ParticleValue::String(s.to_string()),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#fix (helper: createInt)
    pub fn create_int(i: i32) -> Self {
        Dynamic {
            value: ParticleValue::Int(i),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateDust (helper: createFloat)
    pub fn create_float(f: f32) -> Self {
        Dynamic {
            value: ParticleValue::Float(f),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#fix (helper: createMap)
    pub fn create_map(entries: Vec<(String, Dynamic)>) -> Self {
        let mut map = HashMap::new();
        for (k, v) in entries {
            map.insert(k, v.value);
        }
        Dynamic {
            value: ParticleValue::Map(map),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateBlock (helper: emptyMap)
    pub fn empty_map() -> Self {
        Dynamic {
            value: ParticleValue::Map(HashMap::new()),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#readVector (helper: createList)
    pub fn create_list(items: Vec<Dynamic>) -> Self {
        Dynamic {
            value: ParticleValue::List(items.into_iter().map(|d| d.value).collect()),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateVibration (helper: createIntList)
    pub fn create_int_list(values: &[i32]) -> Self {
        Dynamic {
            value: ParticleValue::List(values.iter().map(|&i| ParticleValue::Int(i)).collect()),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#fix (helper: asString)
    pub fn as_string(&self) -> Option<String> {
        match &self.value {
            ParticleValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Set a field on this dynamic if it is a map, returning a new Dynamic.
    // JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateItem (helper: set)
    pub fn set(&self, key: &str, val: Dynamic) -> Self {
        match &self.value {
            ParticleValue::Map(m) => {
                let mut new_map = m.clone();
                new_map.insert(key.to_string(), val.value);
                Dynamic {
                    value: ParticleValue::Map(new_map),
                }
            }
            _ => self.clone(),
        }
    }

    /// Get a string field from a map dynamic.
    pub fn get_string(&self, key: &str) -> Option<String> {
        match &self.value {
            ParticleValue::Map(m) => match m.get(key) {
                Some(ParticleValue::String(s)) => Some(s.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    /// Get a field from a map dynamic.
    pub fn get(&self, key: &str) -> Option<Dynamic> {
        match &self.value {
            ParticleValue::Map(m) => m.get(key).map(|v| Dynamic { value: v.clone() }),
            _ => None,
        }
    }
}

/// Minimal StringReader matching the brigadier StringReader behavior used here.
struct StringReader {
    string: Vec<char>,
    cursor: usize,
}

#[derive(Debug)]
struct ReaderError;

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StringReader error")
    }
}

impl StringReader {
    fn new(s: &str) -> Self {
        StringReader {
            string: s.chars().collect(),
            cursor: 0,
        }
    }

    fn can_read(&self) -> bool {
        self.cursor < self.string.len()
    }

    fn peek(&self) -> char {
        self.string[self.cursor]
    }

    fn skip(&mut self) {
        self.cursor += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.can_read() && self.string[self.cursor].is_whitespace() {
            self.cursor += 1;
        }
    }

    fn expect(&mut self, c: char) -> Result<(), ReaderError> {
        if self.can_read() && self.peek() == c {
            self.cursor += 1;
            Ok(())
        } else {
            Err(ReaderError)
        }
    }

    fn read_float(&mut self) -> Result<f32, ReaderError> {
        let s = self.read_number_string()?;
        s.parse::<f32>().map_err(|_| ReaderError)
    }

    fn read_double(&mut self) -> Result<f64, ReaderError> {
        let s = self.read_number_string()?;
        s.parse::<f64>().map_err(|_| ReaderError)
    }

    fn read_int(&mut self) -> Result<i32, ReaderError> {
        let s = self.read_number_string()?;
        s.parse::<i32>().map_err(|_| ReaderError)
    }

    fn read_number_string(&mut self) -> Result<String, ReaderError> {
        let start = self.cursor;
        while self.can_read() {
            let c = self.peek();
            if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
                self.cursor += 1;
            } else {
                break;
            }
        }
        if self.cursor == start {
            return Err(ReaderError);
        }
        Ok(self.string[start..self.cursor].iter().collect())
    }

    /// Reads an unquoted string (brigadier StringReader.readString behavior for
    /// unquoted strings: reads until whitespace or special delimiter).
    fn read_string(&mut self) -> Result<String, ReaderError> {
        if !self.can_read() {
            return Err(ReaderError);
        }
        let start = self.cursor;
        while self.can_read() {
            let c = self.peek();
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '+' || c == ':' {
                self.cursor += 1;
            } else {
                break;
            }
        }
        if self.cursor == start {
            return Err(ReaderError);
        }
        Ok(self.string[start..self.cursor].iter().collect())
    }
}

/// Ensures a namespaced string has a `minecraft:` prefix if no namespace is present.
/// Mirrors `NamespacedSchema.ensureNamespaced`.
fn ensure_namespaced(s: &str) -> String {
    if s.contains(':') {
        s.to_string()
    } else {
        format!("minecraft:{s}")
    }
}

/// Parse a simple SNBT-like tag string. Returns None on failure.
/// This is a minimal stub: Java uses TagParser.create(ops).parseFully(string).
/// For the particle unflattening fix, we store the raw tag string as a string value
/// when we cannot fully parse it, but the Java code stores the parsed tag Dynamic.
/// We provide a best-effort parse for simple cases and fall back gracefully.
// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#parseTag
fn parse_tag(raw: &str) -> Option<Dynamic> {
    // Minimal SNBT parser for the cases encountered in particle datafixing.
    // Java uses TagParser which produces NBT; we represent it as a Dynamic tree.
    let trimmed = raw.trim();
    parse_snbt_value(trimmed).map(|(v, _)| v)
}

fn parse_snbt_value(s: &str) -> Option<(Dynamic, usize)> {
    let s = s.trim_start();
    if s.is_empty() {
        return None;
    }
    match s.as_bytes()[0] {
        b'{' => parse_snbt_compound(s),
        b'[' => parse_snbt_list(s),
        b'"' | b'\'' => parse_snbt_quoted_string(s),
        _ => parse_snbt_unquoted(s),
    }
}

fn parse_snbt_compound(s: &str) -> Option<(Dynamic, usize)> {
    if !s.starts_with('{') {
        return None;
    }
    let mut map: HashMap<String, ParticleValue> = HashMap::new();
    let mut i = 1; // skip '{'
    let bytes = s.as_bytes();
    loop {
        // skip whitespace
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        if bytes[i] == b'}' {
            i += 1;
            return Some((
                Dynamic {
                    value: ParticleValue::Map(map),
                },
                i,
            ));
        }
        // read key
        let (key, consumed) = parse_snbt_key(&s[i..])?;
        i += consumed;
        // skip whitespace
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b':' {
            return None;
        }
        i += 1; // skip ':'
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        let (val, consumed) = parse_snbt_value(&s[i..])?;
        i += consumed;
        map.insert(key, val.value);
        // skip whitespace
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b',' {
            i += 1;
        }
    }
}

fn parse_snbt_key(s: &str) -> Option<(String, usize)> {
    let s_trimmed = s.trim_start();
    let offset = s.len() - s_trimmed.len();
    if s_trimmed.is_empty() {
        return None;
    }
    if s_trimmed.starts_with('"') || s_trimmed.starts_with('\'') {
        let (d, consumed) = parse_snbt_quoted_string(s_trimmed)?;
        if let ParticleValue::String(k) = d.value {
            return Some((k, offset + consumed));
        }
        return None;
    }
    let mut i = 0;
    let bytes = s_trimmed.as_bytes();
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '+' {
            i += 1;
        } else {
            break;
        }
    }
    if i == 0 {
        return None;
    }
    Some((s_trimmed[..i].to_string(), offset + i))
}

fn parse_snbt_quoted_string(s: &str) -> Option<(Dynamic, usize)> {
    let quote = s.as_bytes()[0] as char;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let mut i = 1;
    let bytes = s.as_bytes();
    let mut result = String::new();
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '\\' && i + 1 < bytes.len() {
            i += 1;
            result.push(bytes[i] as char);
            i += 1;
        } else if c == quote {
            i += 1;
            return Some((Dynamic::create_string(&result), i));
        } else {
            result.push(c);
            i += 1;
        }
    }
    None
}

fn parse_snbt_list(s: &str) -> Option<(Dynamic, usize)> {
    if !s.starts_with('[') {
        return None;
    }
    let mut items: Vec<Dynamic> = Vec::new();
    let mut i = 1;
    let bytes = s.as_bytes();
    // Skip optional type prefix like I; B; L;
    if i + 1 < bytes.len()
        && (bytes[i] == b'I' || bytes[i] == b'B' || bytes[i] == b'L')
        && bytes[i + 1] == b';'
    {
        i += 2;
    }
    loop {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        if bytes[i] == b']' {
            i += 1;
            return Some((Dynamic::create_list(items), i));
        }
        let (val, consumed) = parse_snbt_value(&s[i..])?;
        i += consumed;
        items.push(val);
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b',' {
            i += 1;
        }
    }
}

fn parse_snbt_unquoted(s: &str) -> Option<(Dynamic, usize)> {
    let mut i = 0;
    let bytes = s.as_bytes();
    // Try number
    let start = i;
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+' || bytes[i].is_ascii_digit()) {
        while i < bytes.len()
            && (bytes[i].is_ascii_digit()
                || bytes[i] == b'.'
                || bytes[i] == b'-'
                || bytes[i] == b'+'
                || bytes[i] == b'e'
                || bytes[i] == b'E')
        {
            i += 1;
        }
        // Check for type suffix
        let mut suffix = None;
        if i < bytes.len() {
            let c = bytes[i] as char;
            if c == 'f' || c == 'F' || c == 'd' || c == 'D' || c == 'b' || c == 'B'
                || c == 's' || c == 'S' || c == 'l' || c == 'L'
            {
                suffix = Some(c);
                i += 1;
            }
        }
        let num_str: String = s[start..i - if suffix.is_some() { 1 } else { 0 }]
            .iter()
            .collect::<String>();
        // Actually just slice properly
        let end_num = if suffix.is_some() { i - 1 } else { i };
        let num_str = &s[start..end_num];
        if !num_str.is_empty()
            && num_str
                .chars()
                .any(|c| c.is_ascii_digit())
        {
            match suffix {
                Some('f') | Some('F') => {
                    if let Ok(v) = num_str.parse::<f32>() {
                        return Some((Dynamic::create_float(v), i));
                    }
                }
                Some('d') | Some('D') => {
                    if let Ok(v) = num_str.parse::<f32>() {
                        return Some((Dynamic::create_float(v), i));
                    }
                }
                Some('b') | Some('B') | Some('s') | Some('S') | Some('l') | Some('L') => {
                    if let Ok(v) = num_str.parse::<i32>() {
                        return Some((Dynamic::create_int(v), i));
                    }
                }
                None => {
                    if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
                        if let Ok(v) = num_str.parse::<f32>() {
                            return Some((Dynamic::create_float(v), i));
                        }
                    } else if let Ok(v) = num_str.parse::<i32>() {
                        return Some((Dynamic::create_int(v), i));
                    }
                }
            }
        }
        i = start; // reset
    }

    // Unquoted string / identifier
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '+' || c == ':' {
            i += 1;
        } else {
            break;
        }
    }
    if i == 0 {
        return None;
    }
    let token = &s[..i];
    // Handle booleans
    if token == "true" {
        return Some((Dynamic::create_int(1), i));
    }
    if token == "false" {
        return Some((Dynamic::create_int(0), i));
    }
    Some((Dynamic::create_string(token), i))
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#fix
pub fn fix(input: &Dynamic) -> Dynamic {
    let opt_string = input.as_string();
    if opt_string.is_none() {
        return input.clone();
    }

    let raw = opt_string.unwrap();
    let parts: Vec<&str> = raw.splitn(2, ' ').collect();
    let name = ensure_namespaced(parts[0]);
    let tail = if parts.len() > 1 { Some(parts[1]) } else { None };

    let mut result = Dynamic::create_map(vec![(
        "type".to_string(),
        Dynamic::create_string(&name),
    )]);

    result = match name.as_str() {
        "minecraft:item" => {
            if let Some(t) = tail {
                update_item(&result, t)
            } else {
                result
            }
        }
        "minecraft:block"
        | "minecraft:block_marker"
        | "minecraft:falling_dust"
        | "minecraft:dust_pillar" => {
            if let Some(t) = tail {
                update_block(&result, t)
            } else {
                result
            }
        }
        "minecraft:dust" => {
            if let Some(t) = tail {
                update_dust(&result, t)
            } else {
                result
            }
        }
        "minecraft:dust_color_transition" => {
            if let Some(t) = tail {
                update_dust_transition(&result, t)
            } else {
                result
            }
        }
        "minecraft:sculk_charge" => {
            if let Some(t) = tail {
                update_sculk_charge(&result, t)
            } else {
                result
            }
        }
        "minecraft:vibration" => {
            if let Some(t) = tail {
                update_vibration(&result, t)
            } else {
                result
            }
        }
        "minecraft:shriek" => {
            if let Some(t) = tail {
                update_shriek(&result, t)
            } else {
                result
            }
        }
        _ => result,
    };

    result
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateItem
fn update_item(particle: &Dynamic, options: &str) -> Dynamic {
    let brace_idx = options.find('{');
    let mut item = Dynamic::create_map(vec![(
        "Count".to_string(),
        Dynamic::create_int(1),
    )]);

    if brace_idx.is_none() {
        item = item.set("id", Dynamic::create_string(options));
    } else {
        let idx = brace_idx.unwrap();
        item = item.set("id", Dynamic::create_string(&options[..idx]));
        let tag_str = &options[idx..];
        if let Some(tag) = parse_tag(tag_str) {
            item = item.set("tag", tag);
        }
        // On parse failure, Java logs warning and returns null tag - we skip setting tag
    }

    particle.set("item", item)
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateBlock
fn update_block(particle: &Dynamic, options: &str) -> Dynamic {
    let bracket_idx = options.find('[');
    let mut block_state = Dynamic::empty_map();

    if bracket_idx.is_none() {
        block_state = block_state.set(
            "Name",
            Dynamic::create_string(&ensure_namespaced(options)),
        );
    } else {
        let idx = bracket_idx.unwrap();
        block_state = block_state.set(
            "Name",
            Dynamic::create_string(&ensure_namespaced(&options[..idx])),
        );
        let props = parse_block_properties(&options[idx..]);
        if !props.is_empty() {
            let entries: Vec<(String, Dynamic)> = props
                .into_iter()
                .map(|(k, v)| (k, Dynamic::create_string(&v)))
                .collect();
            block_state = block_state.set("Properties", Dynamic::create_map(entries));
        }
    }

    particle.set("block_state", block_state)
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#parseBlockProperties
fn parse_block_properties(input: &str) -> HashMap<String, String> {
    let result = (|| -> Result<HashMap<String, String>, ReaderError> {
        let mut props = HashMap::new();
        let mut reader = StringReader::new(input);
        reader.expect('[')?;
        reader.skip_whitespace();

        while reader.can_read() && reader.peek() != ']' {
            reader.skip_whitespace();
            let key = reader.read_string()?;
            reader.skip_whitespace();
            reader.expect('=')?;
            reader.skip_whitespace();
            let val = reader.read_string()?;
            reader.skip_whitespace();
            props.insert(key, val);
            if reader.can_read() {
                if reader.peek() != ',' {
                    break;
                }
                reader.skip();
            }
        }

        reader.expect(']')?;
        Ok(props)
    })();

    match result {
        Ok(m) => m,
        Err(_) => {
            // LOGGER.warn("Failed to parse block properties: {}", input);
            HashMap::new()
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#readVector
fn read_vector(reader: &mut StringReader) -> Result<Dynamic, ReaderError> {
    let x = reader.read_float()?;
    reader.expect(' ')?;
    let y = reader.read_float()?;
    reader.expect(' ')?;
    let z = reader.read_float()?;
    Ok(Dynamic::create_list(vec![
        Dynamic::create_float(x),
        Dynamic::create_float(y),
        Dynamic::create_float(z),
    ]))
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateDust
fn update_dust(particle: &Dynamic, options: &str) -> Dynamic {
    let result = (|| -> Result<Dynamic, ReaderError> {
        let mut reader = StringReader::new(options);
        let color = read_vector(&mut reader)?;
        reader.expect(' ')?;
        let scale = reader.read_float()?;
        Ok(particle
            .set("color", color)
            .set("scale", Dynamic::create_float(scale)))
    })();

    match result {
        Ok(d) => d,
        Err(_) => {
            // LOGGER.warn("Failed to parse particle options: {}", options);
            particle.clone()
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateDustTransition
fn update_dust_transition(particle: &Dynamic, options: &str) -> Dynamic {
    let result = (|| -> Result<Dynamic, ReaderError> {
        let mut reader = StringReader::new(options);
        let from_color = read_vector(&mut reader)?;
        reader.expect(' ')?;
        let scale = reader.read_float()?;
        reader.expect(' ')?;
        let to_color = read_vector(&mut reader)?;
        Ok(particle
            .set("from_color", from_color)
            .set("to_color", to_color)
            .set("scale", Dynamic::create_float(scale)))
    })();

    match result {
        Ok(d) => d,
        Err(_) => {
            // LOGGER.warn("Failed to parse particle options: {}", options);
            particle.clone()
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateSculkCharge
fn update_sculk_charge(particle: &Dynamic, options: &str) -> Dynamic {
    let result = (|| -> Result<Dynamic, ReaderError> {
        let mut reader = StringReader::new(options);
        let roll = reader.read_float()?;
        Ok(particle.set("roll", Dynamic::create_float(roll)))
    })();

    match result {
        Ok(d) => d,
        Err(_) => {
            // LOGGER.warn("Failed to parse particle options: {}", options);
            particle.clone()
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateVibration
fn update_vibration(particle: &Dynamic, options: &str) -> Dynamic {
    let result = (|| -> Result<Dynamic, ReaderError> {
        let mut reader = StringReader::new(options);
        let x = reader.read_double()? as f32;
        reader.expect(' ')?;
        let y = reader.read_double()? as f32;
        reader.expect(' ')?;
        let z = reader.read_double()? as f32;
        reader.expect(' ')?;
        let arrival_in_ticks = reader.read_int()?;

        let pos = Dynamic::create_int_list(&[
            crate::mth::floor_f32(x),
            crate::mth::floor_f32(y),
            crate::mth::floor_f32(z),
        ]);
        let destination = Dynamic::create_map(vec![
            (
                "type".to_string(),
                Dynamic::create_string("minecraft:block"),
            ),
            ("pos".to_string(), pos),
        ]);
        Ok(particle
            .set("destination", destination)
            .set("arrival_in_ticks", Dynamic::create_int(arrival_in_ticks)))
    })();

    match result {
        Ok(d) => d,
        Err(_) => {
            // LOGGER.warn("Failed to parse particle options: {}", options);
            particle.clone()
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ParticleUnflatteningFix#updateShriek
fn update_shriek(particle: &Dynamic, options: &str) -> Dynamic {
    let result = (|| -> Result<Dynamic, ReaderError> {
        let mut reader = StringReader::new(options);
        let delay = reader.read_int()?;
        Ok(particle.set("delay", Dynamic::create_int(delay)))
    })();

    match result {
        Ok(d) => d,
        Err(_) => {
            // LOGGER.warn("Failed to parse particle options: {}", options);
            particle.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_string_input(s: &str) -> Dynamic {
        Dynamic::create_string(s)
    }

    #[test]
    fn test_simple_particle() {
        let input = make_string_input("minecraft:smoke");
        let result = fix(&input);
        assert_eq!(result.get_string("type"), Some("minecraft:smoke".to_string()));
    }

    #[test]
    fn test_namespace_ensured() {
        let input = make_string_input("smoke");
        let result = fix(&input);
        assert_eq!(result.get_string("type"), Some("minecraft:smoke".to_string()));
    }

    #[test]
    fn test_non_string_passthrough() {
        let input = Dynamic::create_int(42);
        let result = fix(&input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_item_particle_no_tag() {
        let input = make_string_input("minecraft:item minecraft:diamond");
        let result = fix(&input);
        assert_eq!(result.get_string("type"), Some("minecraft:item".to_string()));
        let item = result.get("item").unwrap();
        assert_eq!(item.get_string("id"), Some("minecraft:diamond".to_string()));
        match item.get("Count").unwrap().value {
            ParticleValue::Int(1) => {}
            ref v => panic!("expected Count=1, got {:?}", v),
        }
    }

    #[test]
    fn test_item_particle_with_tag() {
        let input = make_string_input("minecraft:item minecraft:diamond{Enchantments:[]}");
        let result = fix(&input);
        assert_eq!(result.get_string("type"), Some("minecraft:item".to_string()));
        let item = result.get("item").unwrap();
        assert_eq!(item.get_string("id"), Some("minecraft:diamond".to_string()));
        assert!(item.get("tag").is_some());
    }

    #[test]
    fn test_block_particle_with_properties() {
        let input = make_string_input("minecraft:block minecraft:oak_log[axis=y]");
        let result = fix(&input);
        assert_eq!(result.get_string("type"), Some("minecraft:block".to_string()));
        let bs = result.get("block_state").unwrap();
        assert_eq!(bs.get_string("Name"), Some("minecraft:oak_log".to_string()));
        let props = bs.get("Properties").unwrap();
        assert_eq!(props.get_string("axis"), Some("y".to_string()));
    }

    #[test]
    fn test_dust_particle() {
        let input = make_string_input("minecraft:dust 1.0 0.5 0.0 2.0");
        let result = fix(&input);
        assert_eq!(result.get_string("type"), Some("minecraft:dust".to_string()));
        match result.get("scale").unwrap().value {
            ParticleValue::Float(f) => assert!((f - 2.0).abs() < 1e-6),
            ref v => panic!("expected float scale, got {:?}", v),
        }
        let color = result.get("color").unwrap();
        match &color.value {
            ParticleValue::List(items) => {
                assert_eq!(items.len(), 3);
                match &items[0] {
                    ParticleValue::Float(f) => assert!((f - 1.0).abs() < 1e-6),
                    _ => panic!("expected float"),
                }
            }
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_dust_color_transition() {
        let input = make_string_input("minecraft:dust_color_transition 1.0 0.0 0.0 1.5 0.0 0.0 1.0");
        let result = fix(&input);
        assert_eq!(
            result.get_string("type"),
            Some("minecraft:dust_color_transition".to_string())
        );
        assert!(result.get("from_color").is_some());
        assert!(result.get("to_color").is_some());
        match result.get("scale").unwrap().value {
            ParticleValue::Float(f) => assert!((f - 1.5).abs() < 1e-6),
            ref v => panic!("expected float scale, got {:?}", v),
        }
    }

    #[test]
    fn test_sculk_charge() {
        let input = make_string_input("minecraft:sculk_charge 0.5");
        let result = fix(&input);
        match result.get("roll").unwrap().value {
            ParticleValue::Float(f) => assert!((f - 0.5).abs() < 1e-6),
            ref v => panic!("expected float roll, got {:?}", v),
        }
    }

    #[test]
    fn test_vibration_negative_floor() {
        let input = make_string_input("minecraft:vibration -0.5 -1.7 2.3 10");
        let result = fix(&input);
        let dest = result.get("destination").unwrap();
        let pos = dest.get("pos").unwrap();
        match &pos.value {
            ParticleValue::List(items) => {
                // Mth.floor(-0.5) = -1, Mth.floor(-1.7) = -2, Mth.floor(2.3) = 2
                assert_eq!(items[0], ParticleValue::Int(-1));
                assert_eq!(items[1], ParticleValue::Int(-2));
                assert_eq!(items[2], ParticleValue::Int(2));
            }
            _ => panic!("expected list"),
        }
        match result.get("arrival_in_ticks").unwrap().value {
            ParticleValue::Int(10) => {}
            ref v => panic!("expected int 10, got {:?}", v),
        }
    }

    #[test]
    fn test_shriek() {
        let input = make_string_input("minecraft:shriek 42");
        let result = fix(&input);
        match result.get("delay").unwrap().value {
            ParticleValue::Int(42) => {}
            ref v => panic!("expected int 42, got {:?}", v),
        }
    }

    #[test]
    fn test_malformed_dust_returns_original_structure() {
        let input = make_string_input("minecraft:dust bad_data");
        let result = fix(&input);
        assert_eq!(result.get_string("type"), Some("minecraft:dust".to_string()));
        // Should not have color/scale on parse failure
        assert!(result.get("color").is_none());
        assert!(result.get("scale").is_none());
    }
}
