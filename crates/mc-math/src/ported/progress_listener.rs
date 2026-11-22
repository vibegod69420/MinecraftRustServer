/// Minimal text component placeholder for self-contained porting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

pub trait ProgressListener {
    // JAVA: net.minecraft.util.ProgressListener#progressStartNoAbort
    fn progress_start_no_abort(&mut self, component: &Component);

    // JAVA: net.minecraft.util.ProgressListener#progressStart
    fn progress_start(&mut self, component: &Component);

    // JAVA: net.minecraft.util.ProgressListener#progressStage
    fn progress_stage(&mut self, component: &Component);

    // JAVA: net.minecraft.util.ProgressListener#progressStagePercentage
    fn progress_stage_percentage(&mut self, percentage: i32);

    // JAVA: net.minecraft.util.ProgressListener#stop
    fn stop(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockProgress {
        events: Vec<String>,
        last_percentage: i32,
        stopped: bool,
    }

    impl ProgressListener for MockProgress {
        fn progress_start_no_abort(&mut self, component: &Component) {
            self.events.push(format!("start_no_abort:{}", component.text));
        }

        fn progress_start(&mut self, component: &Component) {
            self.events.push(format!("start:{}", component.text));
        }

        fn progress_stage(&mut self, component: &Component) {
            self.events.push(format!("stage:{}", component.text));
        }

        fn progress_stage_percentage(&mut self, percentage: i32) {
            self.last_percentage = percentage;
        }

        fn stop(&mut self) {
            self.stopped = true;
        }
    }

    #[test]
    fn component_literal() {
        let c = Component::literal("hello");
        assert_eq!(c.text, "hello");
    }

    #[test]
    fn progress_listener_lifecycle() {
        let mut p = MockProgress::default();
        p.progress_start(&Component::literal("loading"));
        p.progress_stage(&Component::literal("step"));
        p.progress_stage_percentage(50);
        p.stop();
        assert_eq!(p.events, vec!["start:loading", "stage:step"]);
        assert_eq!(p.last_percentage, 50);
        assert!(p.stopped);
    }

    #[test]
    fn progress_start_no_abort_records_event() {
        let mut p = MockProgress::default();
        p.progress_start_no_abort(&Component::literal("no abort"));
        assert_eq!(p.events, vec!["start_no_abort:no abort"]);
        assert!(!p.stopped);
    }

    #[test]
    fn progress_percentage_handles_negative_and_large_values() {
        let mut p = MockProgress::default();
        p.progress_stage_percentage(-1);
        assert_eq!(p.last_percentage, -1);
        p.progress_stage_percentage(100);
        assert_eq!(p.last_percentage, 100);
        p.progress_stage_percentage(i32::MAX);
        assert_eq!(p.last_percentage, i32::MAX);
    }
}
