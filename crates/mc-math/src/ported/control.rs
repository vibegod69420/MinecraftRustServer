//! Faithful port of `net.minecraft.util.parsing.packrat.Control`.

/// Port of `net.minecraft.util.parsing.packrat.Control`.
pub trait Control {
    // JAVA: net.minecraft.util.parsing.packrat.Control#cut
    fn cut(&mut self);

    // JAVA: net.minecraft.util.parsing.packrat.Control#hasCut
    fn has_cut(&self) -> bool;
}

/// Rust equivalent of Java's `Control.UNBOUND` anonymous instance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnboundControl;

impl Control for UnboundControl {
    // JAVA: net.minecraft.util.parsing.packrat.Control#cut
    fn cut(&mut self) {}

    // JAVA: net.minecraft.util.parsing.packrat.Control#hasCut
    fn has_cut(&self) -> bool {
        false
    }
}

/// Port of `Control.UNBOUND`.
pub const UNBOUND: UnboundControl = UnboundControl;

#[cfg(test)]
mod tests {
    use super::{Control, UnboundControl, UNBOUND};

    #[test]
    fn unbound_has_cut_is_false() {
        assert!(!UNBOUND.has_cut());
        let c = UnboundControl;
        assert!(!c.has_cut());
    }

    #[test]
    fn unbound_cut_is_noop() {
        let mut c = UNBOUND;
        c.cut();
        assert!(!c.has_cut());
        c.cut();
        c.cut();
        assert!(!c.has_cut());
    }

    #[test]
    fn custom_control_tracks_cut_state() {
        #[derive(Default)]
        struct TestControl(bool);
        impl Control for TestControl {
            // JAVA: net.minecraft.util.parsing.packrat.Control#cut
            fn cut(&mut self) {
                self.0 = true;
            }
            // JAVA: net.minecraft.util.parsing.packrat.Control#hasCut
            fn has_cut(&self) -> bool {
                self.0
            }
        }

        let mut c = TestControl::default();
        assert!(!c.has_cut());
        c.cut();
        assert!(c.has_cut());
    }

    #[test]
    fn unbound_is_copy_and_eq() {
        let a = UNBOUND;
        let b = a;
        assert_eq!(a, b);
        assert!(!b.has_cut());
    }
}
