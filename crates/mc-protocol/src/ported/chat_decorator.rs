/// Minimal stub for `net.minecraft.network.chat.Component`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Minimal stub for `net.minecraft.server.level.ServerPlayer`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerPlayer {
    pub name: String,
}

/// Port of `net.minecraft.network.chat.ChatDecorator`.
pub trait ChatDecorator {
    // JAVA: net.minecraft.network.chat.ChatDecorator#decorate
    fn decorate(&self, player: Option<&ServerPlayer>, component: Component) -> Component;
}

/// Plain decorator implementation for `ChatDecorator.PLAIN`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlainChatDecorator;

impl ChatDecorator for PlainChatDecorator {
    // JAVA: net.minecraft.network.chat.ChatDecorator#decorate
    fn decorate(&self, _player: Option<&ServerPlayer>, component: Component) -> Component {
        component
    }
}

/// Port of `ChatDecorator.PLAIN`.
pub const PLAIN: PlainChatDecorator = PlainChatDecorator;

#[cfg(test)]
mod tests {
    use super::{ChatDecorator, Component, PlainChatDecorator, ServerPlayer, PLAIN};

    #[test]
    fn plain_returns_same_component() {
        let input = Component::literal("hello");
        let out = PLAIN.decorate(None, input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn plain_ignores_player() {
        let player = ServerPlayer {
            name: "Steve".to_string(),
        };
        let input = Component::literal("msg");
        let out = PLAIN.decorate(Some(&player), input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn custom_decorator_can_modify_component() {
        struct PrefixDecorator;
        impl ChatDecorator for PrefixDecorator {
            fn decorate(&self, _player: Option<&ServerPlayer>, component: Component) -> Component {
                Component::literal(format!("[x] {}", component.text))
            }
        }

        let out = PrefixDecorator.decorate(None, Component::literal("hi"));
        assert_eq!(out.text, "[x] hi");
    }

    #[test]
    fn plain_struct_default_works() {
        let decorator = PlainChatDecorator;
        let out = decorator.decorate(None, Component::literal("test"));
        assert_eq!(out.text, "test");
    }
}
