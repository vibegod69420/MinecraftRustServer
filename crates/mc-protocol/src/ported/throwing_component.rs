use std::error::Error;
use std::fmt;

/// Minimal stub for `net.minecraft.network.chat.Component`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn get_string(&self) -> &str {
        &self.text
    }
}

/// Port of `net.minecraft.network.chat.ThrowingComponent`.
#[derive(Debug)]
pub struct ThrowingComponent {
    component: Component,
    cause: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl ThrowingComponent {
    // JAVA: net.minecraft.network.chat.ThrowingComponent#<init>
    pub fn new(component: Component) -> Self {
        Self {
            component,
            cause: None,
        }
    }

    // JAVA: net.minecraft.network.chat.ThrowingComponent#<init>
    pub fn with_cause(
        component: Component,
        cause: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            component,
            cause: Some(Box::new(cause)),
        }
    }

    // JAVA: net.minecraft.network.chat.ThrowingComponent#getComponent
    pub fn get_component(&self) -> &Component {
        &self.component
    }
}

impl fmt::Display for ThrowingComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.component.get_string())
    }
}

impl Error for ThrowingComponent {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_ref()
            .map(|c| &**c as &(dyn Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ThrowingComponent};
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct DummyCause;
    impl fmt::Display for DummyCause {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "dummy cause")
        }
    }
    impl Error for DummyCause {}

    #[test]
    fn new_sets_message_and_component() {
        let comp = Component::literal("bad component");
        let err = ThrowingComponent::new(comp.clone());
        assert_eq!(err.to_string(), "bad component");
        assert_eq!(err.get_component(), &comp);
        assert!(err.source().is_none());
    }

    #[test]
    fn with_cause_preserves_cause() {
        let comp = Component::literal("with cause");
        let err = ThrowingComponent::with_cause(comp.clone(), DummyCause);
        assert_eq!(err.to_string(), "with cause");
        assert_eq!(err.get_component(), &comp);
        let source = err.source().expect("cause should exist");
        assert_eq!(source.to_string(), "dummy cause");
    }

    #[test]
    fn get_component_returns_original() {
        let comp = Component::literal("original");
        let err = ThrowingComponent::new(comp.clone());
        assert_eq!(err.get_component().get_string(), "original");
    }

    #[test]
    fn display_matches_component_string_for_empty_text() {
        let err = ThrowingComponent::new(Component::literal(""));
        assert_eq!(err.to_string(), "");
    }
}
