#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientFpsEvent {
    pub fps: i32,
}

impl ClientFpsEvent {
    pub const EVENT_NAME: &'static str = "minecraft.ClientFps";

    // JAVA: net.minecraft.util.profiling.jfr.event.ClientFpsEvent#<init>
    pub fn new(fps: i32) -> Self {
        Self { fps }
    }
}

pub struct ClientFpsEventFields;

impl ClientFpsEventFields {
    pub const FPS: &'static str = "fps";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_fps() {
        let ev = ClientFpsEvent::new(60);
        assert_eq!(ev.fps, 60);
    }

    #[test]
    fn constants() {
        assert_eq!(ClientFpsEvent::EVENT_NAME, "minecraft.ClientFps");
        assert_eq!(ClientFpsEventFields::FPS, "fps");
    }

    #[test]
    fn negative_and_zero_fps() {
        assert_eq!(ClientFpsEvent::new(0).fps, 0);
        assert_eq!(ClientFpsEvent::new(-1).fps, -1);
    }

    #[test]
    fn fps_integer_extremes() {
        assert_eq!(ClientFpsEvent::new(i32::MAX).fps, i32::MAX);
        assert_eq!(ClientFpsEvent::new(i32::MIN).fps, i32::MIN);
    }
}
