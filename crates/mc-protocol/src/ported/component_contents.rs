use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandSourceStack;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommandSyntaxException {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapCodec {
    pub id: String,
}

impl MapCodec {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MutableComponent {
    pub created: bool,
}

impl MutableComponent {
    // JAVA: net.minecraft.network.chat.MutableComponent#create
    pub fn create<C: ComponentContents + ?Sized>(_contents: &C) -> Self {
        Self { created: true }
    }
}

pub trait ComponentContents {
    // JAVA: net.minecraft.network.chat.ComponentContents#visit
    fn visit_styled<T>(
        &self,
        _consumer: &mut dyn FnMut(&Style, &str) -> Option<T>,
        _style: &Style,
    ) -> Option<T> {
        None
    }

    // JAVA: net.minecraft.network.chat.ComponentContents#visit
    fn visit_content<T>(&self, _consumer: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        None
    }

    // JAVA: net.minecraft.network.chat.ComponentContents#resolve
    fn resolve(
        &self,
        _source: Option<&CommandSourceStack>,
        _entity: Option<&Entity>,
        _depth: i32,
    ) -> Result<MutableComponent, CommandSyntaxException> {
        Ok(MutableComponent::create(self))
    }

    // JAVA: net.minecraft.network.chat.ComponentContents#codec
    fn codec(&self) -> MapCodec;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyContents {
        codec_id: String,
    }

    impl ComponentContents for DummyContents {
        fn codec(&self) -> MapCodec {
            MapCodec::new(self.codec_id.clone())
        }
    }

    #[test]
    fn visit_styled_default_returns_none() {
        let c = DummyContents {
            codec_id: "dummy".into(),
        };
        let mut called = false;
        let result: Option<i32> =
            c.visit_styled(&mut |_style, _text| {
                called = true;
                Some(1)
            }, &Style);
        assert_eq!(result, None);
        assert!(!called);
    }

    #[test]
    fn visit_content_default_returns_none() {
        let c = DummyContents {
            codec_id: "dummy".into(),
        };
        let mut called = false;
        let result: Option<&str> = c.visit_content(&mut |_text| {
            called = true;
            Some("x")
        });
        assert_eq!(result, None);
        assert!(!called);
    }

    #[test]
    fn resolve_default_creates_component() {
        let c = DummyContents {
            codec_id: "dummy".into(),
        };
        let r = c.resolve(None, None, 0).unwrap();
        assert!(r.created);
    }

    #[test]
    fn resolve_with_negative_depth_and_present_refs() {
        let c = DummyContents {
            codec_id: "dummy".into(),
        };
        let src = CommandSourceStack;
        let ent = Entity;
        let r = c.resolve(Some(&src), Some(&ent), -1).unwrap();
        assert!(r.created);
    }

    #[test]
    fn codec_returns_configured_value() {
        let c = DummyContents {
            codec_id: "test_codec".into(),
        };
        assert_eq!(c.codec(), MapCodec::new("test_codec"));
    }
}
