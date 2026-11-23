use std::sync::Arc;

/// Minimal self-contained stand-in for `net.minecraft.resources.Identifier`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#<init>
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    // JAVA: net.minecraft.resources.Identifier#getNamespace
    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }

    // JAVA: net.minecraft.resources.Identifier#getPath
    pub fn get_path(&self) -> &str {
        &self.path
    }
}

/// Faithful port of `net.minecraft.util.IdentifierPattern`.
///
/// The Java CODEC field is omitted because Mojang codec types are not available in this crate.
/// Pattern matching uses a small self-contained regex subset compatible with common identifier patterns.
pub struct IdentifierPattern {
    namespace_pattern: Option<String>,
    namespace_predicate: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    path_pattern: Option<String>,
    path_predicate: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    location_predicate: Arc<dyn Fn(&Identifier) -> bool + Send + Sync>,
}

impl IdentifierPattern {
    // JAVA: net.minecraft.util.IdentifierPattern#<init>
    pub fn new(namespace_pattern: Option<String>, path_pattern: Option<String>) -> Self {
        let namespace_predicate = make_predicate(namespace_pattern.as_deref());
        let path_predicate = make_predicate(path_pattern.as_deref());
        let ns = Arc::clone(&namespace_predicate);
        let pp = Arc::clone(&path_predicate);
        let location_predicate: Arc<dyn Fn(&Identifier) -> bool + Send + Sync> =
            Arc::new(move |id: &Identifier| ns(id.get_namespace()) && pp(id.get_path()));
        Self {
            namespace_pattern,
            namespace_predicate,
            path_pattern,
            path_predicate,
            location_predicate,
        }
    }

    // JAVA: net.minecraft.util.IdentifierPattern#namespacePredicate
    pub fn namespace_predicate(&self) -> &(dyn Fn(&str) -> bool + Send + Sync) {
        &*self.namespace_predicate
    }

    // JAVA: net.minecraft.util.IdentifierPattern#pathPredicate
    pub fn path_predicate(&self) -> &(dyn Fn(&str) -> bool + Send + Sync) {
        &*self.path_predicate
    }

    // JAVA: net.minecraft.util.IdentifierPattern#locationPredicate
    pub fn location_predicate(&self) -> &(dyn Fn(&Identifier) -> bool + Send + Sync) {
        &*self.location_predicate
    }

    /// Returns the optional namespace regex pattern string.
    pub fn namespace_pattern(&self) -> Option<&str> {
        self.namespace_pattern.as_deref()
    }

    /// Returns the optional path regex pattern string.
    pub fn path_pattern(&self) -> Option<&str> {
        self.path_pattern.as_deref()
    }
}

fn make_predicate(pattern: Option<&str>) -> Arc<dyn Fn(&str) -> bool + Send + Sync> {
    match pattern {
        Some(p) => {
            let p = p.to_string();
            Arc::new(move |s: &str| regex_find(&p, s))
        }
        None => Arc::new(|_: &str| true),
    }
}

fn regex_find(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    if p.is_empty() {
        return true;
    }
    let anchor_start = p[0] == '^';
    let anchor_end = *p.last().unwrap_or(&' ') == '$';
    let ps = if anchor_start { 1 } else { 0 };
    let pe = if anchor_end { p.len().saturating_sub(1) } else { p.len() };
    if ps > pe {
        return !anchor_end || t.is_empty() || !anchor_start;
    }
    let pat = &p[ps..pe];

    if anchor_start {
        return matches_here(pat, 0, &t, 0, anchor_end);
    }
    let mut i = 0usize;
    while i <= t.len() {
        if matches_here(pat, 0, &t, i, anchor_end) {
            return true;
        }
        i += 1;
    }
    false
}

fn matches_here(p: &[char], pi: usize, t: &[char], ti: usize, end_anchor: bool) -> bool {
    if pi >= p.len() {
        return !end_anchor || ti == t.len();
    }
    let (atom_start, atom_end) = parse_atom(p, pi);
    let (qmin, qmax_raw, next_pi) = parse_quantifier(p, atom_end);
    let remaining = t.len().saturating_sub(ti);
    let qmax = if qmax_raw == usize::MAX {
        remaining.saturating_add(1)
    } else {
        qmax_raw
    };

    let mut positions: Vec<usize> = Vec::with_capacity(qmax.saturating_add(1));
    positions.push(ti);
    let mut cur = ti;
    let mut matched = 0usize;
    while matched < qmax {
        if let Some(nt) = match_atom(p, atom_start, atom_end, t, cur) {
            cur = nt;
            matched += 1;
            positions.push(cur);
        } else {
            break;
        }
    }
    if matched < qmin {
        return false;
    }

    let mut k = matched;
    loop {
        if k >= qmin && matches_here(p, next_pi, t, positions[k], end_anchor) {
            return true;
        }
        if k == qmin {
            break;
        }
        k -= 1;
    }
    false
}

fn parse_atom(p: &[char], pi: usize) -> (usize, usize) {
    if pi >= p.len() {
        return (pi, pi);
    }
    match p[pi] {
        '[' => {
            let mut j = pi + 1;
            if j < p.len() && p[j] == '^' {
                j += 1;
            }
            if j < p.len() && p[j] == ']' {
                j += 1;
            }
            while j < p.len() && p[j] != ']' {
                j += 1;
            }
            if j < p.len() {
                (pi, j + 1)
            } else {
                (pi, pi + 1)
            }
        }
        '\\' if pi + 1 < p.len() => (pi, pi + 2),
        _ => (pi, pi + 1),
    }
}

fn parse_quantifier(p: &[char], atom_end: usize) -> (usize, usize, usize) {
    if atom_end < p.len() {
        match p[atom_end] {
            '*' => (0, usize::MAX, atom_end + 1),
            '+' => (1, usize::MAX, atom_end + 1),
            '?' => (0, 1, atom_end + 1),
            _ => (1, 1, atom_end),
        }
    } else {
        (1, 1, atom_end)
    }
}

fn match_atom(p: &[char], atom_start: usize, atom_end: usize, t: &[char], ti: usize) -> Option<usize> {
    if ti >= t.len() || atom_start >= atom_end {
        return None;
    }
    let c = t[ti];
    let ok = match p[atom_start] {
        '.' => true,
        '[' => match_char_class(p, atom_start, atom_end, c),
        '\\' if atom_start + 1 < atom_end => c == p[atom_start + 1],
        ch => c == ch,
    };
    if ok {
        Some(ti + 1)
    } else {
        None
    }
}

fn match_char_class(p: &[char], atom_start: usize, atom_end: usize, c: char) -> bool {
    let mut i = atom_start + 1;
    let mut end = atom_end;
    if end > atom_start && p[end - 1] == ']' {
        end -= 1;
    }
    let negated = i < end && p[i] == '^';
    if negated {
        i += 1;
    }
    let mut matched = false;
    while i < end {
        let ch = if p[i] == '\\' && i + 1 < end {
            let esc = p[i + 1];
            i += 2;
            esc
        } else {
            let ch = p[i];
            i += 1;
            ch
        };
        if i + 1 < end && p[i] == '-' {
            let range_end = if p[i + 1] == '\\' && i + 2 < end {
                let esc = p[i + 2];
                i += 3;
                esc
            } else {
                let re = p[i + 1];
                i += 2;
                re
            };
            if c >= ch && c <= range_end {
                matched = true;
            }
        } else if c == ch {
            matched = true;
        }
    }
    if negated {
        !matched
    } else {
        matched
    }
}

#[cfg(test)]
mod tests {
    use super::{Identifier, IdentifierPattern};

    #[test]
    fn no_patterns_match_all() {
        let pat = IdentifierPattern::new(None, None);
        assert!((pat.namespace_predicate())("minecraft"));
        assert!((pat.path_predicate())("stone"));
        let id = Identifier::new("minecraft", "stone");
        assert!((pat.location_predicate())(&id));
    }

    #[test]
    fn namespace_and_path_patterns_filter() {
        let pat = IdentifierPattern::new(Some("^mine.*".to_string()), Some("stone$".to_string()));
        assert!((pat.namespace_predicate())("minecraft"));
        assert!(!(pat.namespace_predicate())("other"));
        assert!((pat.path_predicate())("smooth_stone"));
        assert!(!(pat.path_predicate())("stone_bricks"));
    }

    #[test]
    fn location_predicate_combines_namespace_and_path() {
        let pat = IdentifierPattern::new(Some("minecraft".to_string()), Some("^block/.*".to_string()));
        assert!((pat.location_predicate())(&Identifier::new("minecraft", "block/stone")));
        assert!(!(pat.location_predicate())(&Identifier::new("modid", "block/stone")));
        assert!(!(pat.location_predicate())(&Identifier::new("minecraft", "item/apple")));
    }

    #[test]
    fn char_class_and_quantifiers() {
        let pat = IdentifierPattern::new(Some("^[a-z0-9_.-]+$".to_string()), None);
        assert!((pat.namespace_predicate())("minecraft"));
        assert!((pat.namespace_predicate())("my-mod_1.2"));
        assert!(!(pat.namespace_predicate())("BadName"));
        assert!(!(pat.namespace_predicate())(""));
    }

    #[test]
    fn pattern_strings_are_preserved() {
        let pat = IdentifierPattern::new(Some("ns".to_string()), Some("pa.*".to_string()));
        assert_eq!(pat.namespace_pattern(), Some("ns"));
        assert_eq!(pat.path_pattern(), Some("pa.*"));
    }
}
