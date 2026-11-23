/// Faithful port of `net.minecraft.util.profiling.jfr.event.WorldLoadFinishedEvent`.
///
/// JFR runtime types are not modeled; this module keeps the event metadata
/// in a self-contained representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventType {
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorldLoadFinishedEvent;

impl WorldLoadFinishedEvent {
    pub const EVENT_NAME: &'static str = "minecraft.LoadWorld";
    pub const TYPE: EventType = EventType {
        name: Self::EVENT_NAME,
    };

    // JAVA: net.minecraft.util.profiling.jfr.event.WorldLoadFinishedEvent#<init>
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_name_and_type_match() {
        assert_eq!(WorldLoadFinishedEvent::EVENT_NAME, "minecraft.LoadWorld");
        assert_eq!(WorldLoadFinishedEvent::TYPE.name, "minecraft.LoadWorld");
        assert_eq!(WorldLoadFinishedEvent::TYPE, EventType { name: "minecraft.LoadWorld" });
    }

    #[test]
    fn constructor_and_default_create_event() {
        let a = WorldLoadFinishedEvent::new();
        let b = WorldLoadFinishedEvent::default();
        assert_eq!(a, b);
    }

    #[test]
    fn event_is_copy_and_clone() {
        let a = WorldLoadFinishedEvent::new();
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }
}
