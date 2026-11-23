use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Int(i32),
    Long(i64),
    String(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    fn create_string(s: impl Into<String>) -> Self {
        Dynamic::String(s.into())
    }

    fn create_int(v: i32) -> Self {
        Dynamic::Int(v)
    }

    fn create_long(v: i64) -> Self {
        Dynamic::Long(v)
    }

    fn create_list(items: Vec<Dynamic>) -> Self {
        Dynamic::List(items)
    }

    fn as_long_stream(&self) -> Vec<i64> {
        match self {
            Dynamic::List(items) => items
                .iter()
                .filter_map(|it| match it {
                    Dynamic::Long(v) => Some(*v),
                    Dynamic::Int(v) => Some(i64::from(*v)),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    fn as_map(&self) -> Option<&BTreeMap<String, Dynamic>> {
        match self {
            Dynamic::Map(m) => Some(m),
            _ => None,
        }
    }

    fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    fn update<F: FnOnce(Dynamic) -> Dynamic>(self, key: &str, f: F) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                if let Some(v) = m.remove(key) {
                    m.insert(key.to_string(), f(v));
                }
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    fn rename_and_fix_field<F: FnOnce(Dynamic) -> Dynamic>(
        self,
        old_key: &str,
        new_key: &str,
        f: F,
    ) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                if let Some(v) = m.remove(old_key) {
                    m.insert(new_key.to_string(), f(v));
                }
                Dynamic::Map(m)
            }
            other => other,
        }
    }
}

pub struct ForcedChunkToTicketFix;

impl ForcedChunkToTicketFix {
    // JAVA: net.minecraft.util.datafix.fixes.ForcedChunkToTicketFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ForcedChunkToTicketFix#makeRule
    pub fn apply(&self, value: Dynamic) -> Dynamic {
        value.update("data", |data| {
            data.rename_and_fix_field("Forced", "tickets", |forced| {
                let tickets: Vec<Dynamic> = forced
                    .as_long_stream()
                    .into_iter()
                    .map(|chunk_pos| {
                        Dynamic::empty_map()
                            .set("type", Dynamic::create_string("minecraft:forced"))
                            .set("level", Dynamic::create_int(31))
                            .set("ticks_left", Dynamic::create_long(0i64))
                            .set("chunk_pos", Dynamic::create_long(chunk_pos))
                    })
                    .collect();
                Dynamic::create_list(tickets)
            })
        })
    }
}

impl Default for ForcedChunkToTicketFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn converts_forced_chunks_to_tickets() {
        let input = map(vec![(
            "data",
            map(vec![(
                "Forced",
                Dynamic::List(vec![Dynamic::Long(0), Dynamic::Long(1)]),
            )]),
        )]);
        let out = ForcedChunkToTicketFix::new().apply(input);
        let data = out.as_map().unwrap().get("data").unwrap().as_map().unwrap();
        let tickets = match data.get("tickets").unwrap() {
            Dynamic::List(t) => t,
            _ => panic!("expected list"),
        };
        assert_eq!(tickets.len(), 2);
        let expected = map(vec![
            ("chunk_pos", Dynamic::Long(0)),
            ("level", Dynamic::Int(31)),
            ("ticks_left", Dynamic::Long(0)),
            ("type", Dynamic::String("minecraft:forced".to_string())),
        ]);
        assert_eq!(tickets[0], expected);
        assert!(data.get("Forced").is_none());
    }

    #[test]
    fn handles_negative_and_extreme_chunk_pos() {
        let pos = -9_151_314_442_816_847_872i64;
        let input = map(vec![(
            "data",
            map(vec![(
                "Forced",
                Dynamic::List(vec![Dynamic::Long(pos), Dynamic::Long(i64::MIN)]),
            )]),
        )]);
        let out = ForcedChunkToTicketFix::new().apply(input);
        let data = out.as_map().unwrap().get("data").unwrap().as_map().unwrap();
        let tickets = match data.get("tickets").unwrap() {
            Dynamic::List(t) => t,
            _ => panic!("expected list"),
        };
        assert_eq!(
            tickets[0].as_map().unwrap().get("chunk_pos"),
            Some(&Dynamic::Long(pos))
        );
        assert_eq!(
            tickets[1].as_map().unwrap().get("chunk_pos"),
            Some(&Dynamic::Long(i64::MIN))
        );
    }

    #[test]
    fn missing_forced_is_noop() {
        let input = map(vec![(
            "data",
            map(vec![("Other", Dynamic::Int(1))]),
        )]);
        let out = ForcedChunkToTicketFix::new().apply(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn preserves_other_fields_and_handles_empty_list() {
        let input = map(vec![
            (
                "data",
                map(vec![
                    ("Forced", Dynamic::List(vec![])),
                    ("Keep", Dynamic::String("yes".to_string())),
                ]),
            ),
            ("extra", Dynamic::Int(7)),
        ]);
        let out = ForcedChunkToTicketFix::new().apply(input);
        let root = out.as_map().unwrap();
        assert_eq!(root.get("extra"), Some(&Dynamic::Int(7)));
        let data = root.get("data").unwrap().as_map().unwrap();
        assert_eq!(data.get("Keep"), Some(&Dynamic::String("yes".to_string())));
        assert_eq!(data.get("tickets"), Some(&Dynamic::List(vec![])));
    }
}
