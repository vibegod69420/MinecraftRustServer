use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix porting.
#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Short(i16),
    Byte(i8),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

fn as_long(value: Option<&Dynamic>, default: i64) -> i64 {
    match value {
        Some(Dynamic::Long(v)) => *v,
        Some(Dynamic::Int(v)) => *v as i64,
        Some(Dynamic::Short(v)) => *v as i64,
        Some(Dynamic::Byte(v)) => *v as i64,
        Some(Dynamic::Double(v)) => *v as i64,
        Some(Dynamic::Float(v)) => *v as i64,
        _ => default,
    }
}

pub struct JukeboxTicksSinceSongStartedFix;

impl JukeboxTicksSinceSongStartedFix {
    // JAVA: net.minecraft.util.datafix.fixes.JukeboxTicksSinceSongStartedFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.JukeboxTicksSinceSongStartedFix#fixTag
    pub fn fix_tag(&self, data: Dynamic) -> Dynamic {
        let (tick_count, record_start_tick) = match &data {
            Dynamic::Map(map) => (
                as_long(map.get("TickCount"), 0_i64),
                as_long(map.get("RecordStartTick"), 0_i64),
            ),
            _ => (0_i64, 0_i64),
        };
        let diff = tick_count.wrapping_sub(record_start_tick);

        let mut map = match data {
            Dynamic::Map(map) => map,
            other => return other,
        };
        map.remove("IsPlaying");
        map.remove("TickCount");
        map.remove("RecordStartTick");
        if diff > 0_i64 {
            map.insert(
                "ticks_since_song_started".to_string(),
                Dynamic::Long(diff),
            );
        }
        Dynamic::Map(map)
    }

    // JAVA: net.minecraft.util.datafix.fixes.JukeboxTicksSinceSongStartedFix#fix
    pub fn fix(&self, data: Dynamic) -> Dynamic {
        self.fix_tag(data)
    }
}

impl Default for JukeboxTicksSinceSongStartedFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(pairs: &[(&str, Dynamic)]) -> Dynamic {
        let mut map = BTreeMap::new();
        for (k, v) in pairs {
            map.insert((*k).to_string(), v.clone());
        }
        Dynamic::Map(map)
    }

    #[test]
    fn sets_ticks_when_positive_and_removes_old_fields() {
        let input = obj(&[
            ("TickCount", Dynamic::Long(100)),
            ("RecordStartTick", Dynamic::Long(40)),
            ("IsPlaying", Dynamic::Bool(true)),
            ("id", Dynamic::String("minecraft:jukebox".to_string())),
        ]);
        let out = JukeboxTicksSinceSongStartedFix::new().fix_tag(input);
        let Dynamic::Map(map) = out else { panic!("expected map") };
        assert_eq!(
            map.get("ticks_since_song_started"),
            Some(&Dynamic::Long(60))
        );
        assert!(!map.contains_key("TickCount"));
        assert!(!map.contains_key("RecordStartTick"));
        assert!(!map.contains_key("IsPlaying"));
        assert_eq!(
            map.get("id"),
            Some(&Dynamic::String("minecraft:jukebox".to_string()))
        );
    }

    #[test]
    fn does_not_set_ticks_when_diff_is_zero() {
        let input = obj(&[
            ("TickCount", Dynamic::Long(40)),
            ("RecordStartTick", Dynamic::Long(40)),
        ]);
        let out = JukeboxTicksSinceSongStartedFix::new().fix_tag(input);
        let Dynamic::Map(map) = out else { panic!("expected map") };
        assert!(!map.contains_key("ticks_since_song_started"));
        assert!(!map.contains_key("TickCount"));
        assert!(!map.contains_key("RecordStartTick"));
    }

    #[test]
    fn does_not_set_ticks_when_diff_is_negative() {
        let input = obj(&[
            ("TickCount", Dynamic::Long(10)),
            ("RecordStartTick", Dynamic::Long(20)),
        ]);
        let out = JukeboxTicksSinceSongStartedFix::new().fix_tag(input);
        let Dynamic::Map(map) = out else { panic!("expected map") };
        assert!(!map.contains_key("ticks_since_song_started"));
    }

    #[test]
    fn missing_fields_default_to_zero() {
        let input = obj(&[("IsPlaying", Dynamic::Bool(false))]);
        let out = JukeboxTicksSinceSongStartedFix::new().fix_tag(input);
        let Dynamic::Map(map) = out else { panic!("expected map") };
        assert!(!map.contains_key("ticks_since_song_started"));
        assert!(!map.contains_key("IsPlaying"));
    }

    #[test]
    fn wrapping_subtraction_matches_java_long_semantics() {
        let input = obj(&[
            ("TickCount", Dynamic::Long(i64::MIN)),
            ("RecordStartTick", Dynamic::Long(1)),
        ]);
        let out = JukeboxTicksSinceSongStartedFix::new().fix(input);
        let Dynamic::Map(map) = out else { panic!("expected map") };
        assert_eq!(
            map.get("ticks_since_song_started"),
            Some(&Dynamic::Long(i64::MAX))
        );
    }

    #[test]
    fn accepts_int_numeric_fields() {
        let input = obj(&[
            ("TickCount", Dynamic::Int(25)),
            ("RecordStartTick", Dynamic::Int(5)),
        ]);
        let out = JukeboxTicksSinceSongStartedFix::new().fix_tag(input);
        let Dynamic::Map(map) = out else { panic!("expected map") };
        assert_eq!(
            map.get("ticks_since_song_started"),
            Some(&Dynamic::Long(20))
        );
    }
}
