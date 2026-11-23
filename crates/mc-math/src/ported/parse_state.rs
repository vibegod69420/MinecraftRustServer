//! Faithful port of `net.minecraft.util.parsing.packrat.ParseState`.

use std::fmt;

/// Minimal placeholder for `net.minecraft.util.parsing.packrat.Scope`.
pub trait Scope: fmt::Debug {
    // JAVA: net.minecraft.util.parsing.packrat.Scope#hasOnlySingleFrame
    fn has_only_single_frame(&self) -> bool;
}

/// Minimal placeholder for `net.minecraft.util.parsing.packrat.ErrorCollector`.
pub trait ErrorCollector<S> {
    // JAVA: net.minecraft.util.parsing.packrat.ErrorCollector#finish
    fn finish(&mut self, cursor: i32);
}

/// Minimal placeholder for `net.minecraft.util.parsing.packrat.NamedRule`.
pub trait NamedRule<S, T> {}

/// Minimal placeholder for `net.minecraft.util.parsing.packrat.Control`.
pub trait Control {
    // JAVA: net.minecraft.util.parsing.packrat.Control#cut
    fn cut(&mut self);

    // JAVA: net.minecraft.util.parsing.packrat.Control#hasCut
    fn has_cut(&self) -> bool;
}

/// Port of `net.minecraft.util.parsing.packrat.ParseState`.
pub trait ParseState<S> {
    type Silent: ParseState<S>;

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#scope
    fn scope(&self) -> &dyn Scope;

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#errorCollector
    fn error_collector(&mut self) -> &mut dyn ErrorCollector<S>;

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#parseTopRule
    fn parse_top_rule<T>(&mut self, rule: &dyn NamedRule<S, T>) -> Option<T> {
        let result = self.parse(rule);
        if result.is_some() {
            let mark = self.mark();
            self.error_collector().finish(mark);
        }
        if !self.scope().has_only_single_frame() {
            panic!("Malformed scope: {:?}", self.scope());
        }
        result
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#parse
    fn parse<T>(&mut self, rule: &dyn NamedRule<S, T>) -> Option<T>;

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#input
    fn input(&self) -> &S;

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#mark
    fn mark(&self) -> i32;

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#restore
    fn restore(&mut self, mark: i32);

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#acquireControl
    fn acquire_control(&mut self) -> Box<dyn Control>;

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#releaseControl
    fn release_control(&mut self);

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#silent
    fn silent(&mut self) -> Self::Silent;
}

#[cfg(test)]
mod tests {
    use super::{Control, ErrorCollector, NamedRule, ParseState, Scope};
    use std::fmt;

    #[derive(Debug)]
    struct TestScope(bool);
    impl Scope for TestScope {
        // JAVA: net.minecraft.util.parsing.packrat.Scope#hasOnlySingleFrame
        fn has_only_single_frame(&self) -> bool {
            self.0
        }
    }

    #[derive(Default)]
    struct TestErrorCollector {
        finished: Vec<i32>,
    }
    impl ErrorCollector<String> for TestErrorCollector {
        // JAVA: net.minecraft.util.parsing.packrat.ErrorCollector#finish
        fn finish(&mut self, cursor: i32) {
            self.finished.push(cursor);
        }
    }

    struct DummyRule;
    impl NamedRule<String, i32> for DummyRule {}

    struct NoopControl(bool);
    impl Control for NoopControl {
        // JAVA: net.minecraft.util.parsing.packrat.Control#cut
        fn cut(&mut self) {
            self.0 = true;
        }
        // JAVA: net.minecraft.util.parsing.packrat.Control#hasCut
        fn has_cut(&self) -> bool {
            self.0
        }
    }

    struct MockState {
        input: String,
        mark: i32,
        scope_single: bool,
        errors: TestErrorCollector,
        released: i32,
    }

    struct MockSilentState {
        input: String,
        mark: i32,
        errors: TestErrorCollector,
    }

    impl ParseState<String> for MockSilentState {
        type Silent = MockSilentState;

        fn scope(&self) -> &dyn Scope {
            &TestScope(true)
        }
        fn error_collector(&mut self) -> &mut dyn ErrorCollector<String> {
            &mut self.errors
        }
        fn parse<T>(&mut self, _rule: &dyn NamedRule<String, T>) -> Option<T> {
            None
        }
        fn input(&self) -> &String {
            &self.input
        }
        fn mark(&self) -> i32 {
            self.mark
        }
        fn restore(&mut self, mark: i32) {
            self.mark = mark;
        }
        fn acquire_control(&mut self) -> Box<dyn Control> {
            Box::new(NoopControl(false))
        }
        fn release_control(&mut self) {}
        fn silent(&mut self) -> Self::Silent {
            MockSilentState {
                input: self.input.clone(),
                mark: self.mark,
                errors: TestErrorCollector::default(),
            }
        }
    }

    impl MockState {
        fn new(input: &str, scope_single: bool) -> Self {
            Self {
                input: input.to_string(),
                mark: 0,
                scope_single,
                errors: TestErrorCollector::default(),
                released: 0,
            }
        }
    }

    impl ParseState<String> for MockState {
        type Silent = MockSilentState;

        // JAVA: net.minecraft.util.parsing.packrat.ParseState#scope
        fn scope(&self) -> &dyn Scope {
            // Leak a small box-free static-like alternative is awkward; use a helper via thread-local is overkill.
            // Instead, store scope as field? We need &dyn Scope from &self. We'll use a simple workaround:
            // return reference to a static for true/false.
            static SINGLE: TestScope = TestScope(true);
            static MULTI: TestScope = TestScope(false);
            if self.scope_single {
                &SINGLE
            } else {
                &MULTI
            }
        }

        // JAVA: net.minecraft.util.parsing.packrat.ParseState#errorCollector
        fn error_collector(&mut self) -> &mut dyn ErrorCollector<String> {
            &mut self.errors
        }

        // JAVA: net.minecraft.util.parsing.packrat.ParseState#parse
        fn parse<T>(&mut self, _rule: &dyn NamedRule<String, T>) -> Option<T> {
            None
        }

        // JAVA: net.minecraft.util.parsing.packrat.ParseState#input
        fn input(&self) -> &String {
            &self.input
        }

        // JAVA: net.minecraft.util.parsing.packrat.ParseState#mark
        fn mark(&self) -> i32 {
            self.mark
        }

        // JAVA: net.minecraft.util.parsing.packrat.ParseState#restore
        fn restore(&mut self, mark: i32) {
            self.mark = mark;
        }

        // JAVA: net.minecraft.util.parsing.packrat.ParseState#acquireControl
        fn acquire_control(&mut self) -> Box<dyn Control> {
            Box::new(NoopControl(false))
        }

        // JAVA: net.minecraft.util.parsing.packrat.ParseState#releaseControl
        fn release_control(&mut self) {
            self.released += 1;
        }

        // JAVA: net.minecraft.util.parsing.packrat.ParseState#silent
        fn silent(&mut self) -> Self::Silent {
            MockSilentState {
                input: self.input.clone(),
                mark: self.mark,
                errors: TestErrorCollector::default(),
            }
        }
    }

    #[test]
    fn parse_top_rule_returns_none_when_parse_fails() {
        let mut state = MockState::new("input", true);
        state.mark = 5;
        let result = state.parse_top_rule(&DummyRule);
        assert_eq!(result, None);
        assert!(state.errors.finished.is_empty());
    }

    #[test]
    fn parse_top_rule_panics_on_malformed_scope() {
        let mut state = MockState::new("abc", false);
        let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.parse_top_rule(&DummyRule)
        }));
        assert!(err.is_err());
    }

    #[test]
    fn mark_and_restore_roundtrip_including_negative() {
        let mut state = MockState::new("x", true);
        assert_eq!(state.mark(), 0);
        state.restore(-1);
        assert_eq!(state.mark(), -1);
        state.restore(i32::MIN);
        assert_eq!(state.mark(), i32::MIN);
        state.restore(i32::MAX);
        assert_eq!(state.mark(), i32::MAX);
    }

    #[test]
    fn control_acquire_release_and_silent() {
        let mut state = MockState::new("hello", true);
        let mut ctrl = state.acquire_control();
        assert!(!ctrl.has_cut());
        ctrl.cut();
        assert!(ctrl.has_cut());
        state.release_control();
        state.release_control();
        assert_eq!(state.released, 2);

        let mut silent = state.silent();
        assert_eq!(silent.input(), "hello");
        assert_eq!(silent.mark(), 0);
        silent.restore(3);
        assert_eq!(silent.mark(), 3);
    }

    #[test]
    fn input_is_preserved() {
        let state = MockState::new("", true);
        assert_eq!(state.input(), "");
        let state2 = MockState::new("unicode-α", true);
        assert_eq!(state2.input(), "unicode-α");
    }
}
