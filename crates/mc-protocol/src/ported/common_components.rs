use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq)]
pub enum ComponentArg {
    Long(i64),
    Component(Component),
}

impl From<i64> for ComponentArg {
    fn from(value: i64) -> Self {
        ComponentArg::Long(value)
    }
}

impl From<Component> for ComponentArg {
    fn from(value: Component) -> Self {
        ComponentArg::Component(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ComponentKind {
    Empty,
    Literal(String),
    Translatable { key: String, args: Vec<ComponentArg> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    pub kind: ComponentKind,
    pub siblings: Vec<Component>,
}

pub type MutableComponent = Component;

impl Component {
    pub fn empty() -> MutableComponent {
        Component {
            kind: ComponentKind::Empty,
            siblings: Vec::new(),
        }
    }

    pub fn literal(text: impl Into<String>) -> MutableComponent {
        Component {
            kind: ComponentKind::Literal(text.into()),
            siblings: Vec::new(),
        }
    }

    pub fn translatable(key: impl Into<String>, args: Vec<ComponentArg>) -> MutableComponent {
        Component {
            kind: ComponentKind::Translatable {
                key: key.into(),
                args,
            },
            siblings: Vec::new(),
        }
    }

    pub fn append(&mut self, component: Component) -> &mut Self {
        self.siblings.push(component);
        self
    }

    pub fn copy(&self) -> Component {
        self.clone()
    }
}

fn format_list(components: &[Component], separator: &Component) -> Component {
    if components.is_empty() {
        return Component::empty();
    }
    if components.len() == 1 {
        return components[0].copy();
    }
    let mut result = Component::empty();
    let mut first = true;
    for component in components {
        if !first {
            result.append(separator.clone());
        }
        result.append(component.clone());
        first = false;
    }
    result
}

pub static EMPTY: LazyLock<Component> = LazyLock::new(Component::empty);
pub static OPTION_ON: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("options.on", vec![]));
pub static OPTION_OFF: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("options.off", vec![]));
pub static GUI_DONE: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.done", vec![]));
pub static GUI_CANCEL: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.cancel", vec![]));
pub static GUI_YES: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.yes", vec![]));
pub static GUI_NO: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.no", vec![]));
pub static GUI_OK: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.ok", vec![]));
pub static GUI_PROCEED: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.proceed", vec![]));
pub static GUI_CONTINUE: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.continue", vec![]));
pub static GUI_BACK: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.back", vec![]));
pub static GUI_TO_TITLE: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.toTitle", vec![]));
pub static GUI_ACKNOWLEDGE: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.acknowledge", vec![]));
pub static GUI_OPEN_IN_BROWSER: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("chat.link.open", vec![]));
pub static GUI_COPY_TO_CLIPBOARD: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("chat.copy", vec![]));
pub static GUI_COPY_LINK_TO_CLIPBOARD: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("gui.copy_link_to_clipboard", vec![]));
pub static GUI_DISCONNECT: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("menu.disconnect", vec![]));
pub static GUI_RETURN_TO_MENU: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("menu.returnToMenu", vec![]));
pub static TRANSFER_CONNECT_FAILED: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("connect.failed.transfer", vec![]));
pub static CONNECT_FAILED: LazyLock<Component> =
    LazyLock::new(|| Component::translatable("connect.failed", vec![]));
pub static NEW_LINE: LazyLock<Component> = LazyLock::new(|| Component::literal("\n"));
pub static NARRATION_SEPARATOR: LazyLock<Component> = LazyLock::new(|| Component::literal(". "));
pub static ELLIPSIS: LazyLock<Component> = LazyLock::new(|| Component::literal("..."));
pub static SPACE: LazyLock<Component> = LazyLock::new(space);

// JAVA: net.minecraft.network.chat.CommonComponents#space
pub fn space() -> MutableComponent {
    Component::literal(" ")
}

// JAVA: net.minecraft.network.chat.CommonComponents#days
pub fn days(days: i64) -> MutableComponent {
    Component::translatable("gui.days", vec![ComponentArg::Long(days)])
}

// JAVA: net.minecraft.network.chat.CommonComponents#hours
pub fn hours(hours: i64) -> MutableComponent {
    Component::translatable("gui.hours", vec![ComponentArg::Long(hours)])
}

// JAVA: net.minecraft.network.chat.CommonComponents#minutes
pub fn minutes(minutes: i64) -> MutableComponent {
    Component::translatable("gui.minutes", vec![ComponentArg::Long(minutes)])
}

// JAVA: net.minecraft.network.chat.CommonComponents#optionStatus
pub fn option_status(value: bool) -> Component {
    if value {
        OPTION_ON.clone()
    } else {
        OPTION_OFF.clone()
    }
}

// JAVA: net.minecraft.network.chat.CommonComponents#disconnectButtonLabel
pub fn disconnect_button_label(is_local_server: bool) -> Component {
    if is_local_server {
        GUI_RETURN_TO_MENU.clone()
    } else {
        GUI_DISCONNECT.clone()
    }
}

// JAVA: net.minecraft.network.chat.CommonComponents#optionStatus
pub fn option_status_composed(name: Component, value: bool) -> MutableComponent {
    Component::translatable(
        if value {
            "options.on.composed"
        } else {
            "options.off.composed"
        },
        vec![ComponentArg::Component(name)],
    )
}

// JAVA: net.minecraft.network.chat.CommonComponents#optionNameValue
pub fn option_name_value(name: Component, value: Component) -> MutableComponent {
    Component::translatable(
        "options.generic_value",
        vec![ComponentArg::Component(name), ComponentArg::Component(value)],
    )
}

// JAVA: net.minecraft.network.chat.CommonComponents#joinForNarration
pub fn join_for_narration(components: &[Component]) -> MutableComponent {
    let mut result = Component::empty();
    for (i, component) in components.iter().enumerate() {
        result.append(component.clone());
        if i != components.len().wrapping_sub(1) {
            result.append(NARRATION_SEPARATOR.clone());
        }
    }
    result
}

// JAVA: net.minecraft.network.chat.CommonComponents#joinLines
pub fn join_lines(components: &[Component]) -> Component {
    join_lines_from_collection(components)
}

// JAVA: net.minecraft.network.chat.CommonComponents#joinLines
pub fn join_lines_from_collection(components: &[Component]) -> Component {
    format_list(components, &NEW_LINE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_expected_content() {
        assert_eq!(*EMPTY, Component::empty());
        assert_eq!(*OPTION_ON, Component::translatable("options.on", vec![]));
        assert_eq!(*OPTION_OFF, Component::translatable("options.off", vec![]));
        assert_eq!(*NEW_LINE, Component::literal("\n"));
        assert_eq!(*NARRATION_SEPARATOR, Component::literal(". "));
        assert_eq!(*ELLIPSIS, Component::literal("..."));
        assert_eq!(*SPACE, Component::literal(" "));
    }

    #[test]
    fn option_and_disconnect_status_selection() {
        assert_eq!(option_status(true), *OPTION_ON);
        assert_eq!(option_status(false), *OPTION_OFF);
        assert_eq!(disconnect_button_label(true), *GUI_RETURN_TO_MENU);
        assert_eq!(disconnect_button_label(false), *GUI_DISCONNECT);
    }

    #[test]
    fn time_components_preserve_negative_and_extreme_i64() {
        assert_eq!(
            days(-1),
            Component::translatable("gui.days", vec![ComponentArg::Long(-1)])
        );
        assert_eq!(
            hours(i64::MAX),
            Component::translatable("gui.hours", vec![ComponentArg::Long(i64::MAX)])
        );
        assert_eq!(
            minutes(i64::MIN),
            Component::translatable("gui.minutes", vec![ComponentArg::Long(i64::MIN)])
        );
    }

    #[test]
    fn join_for_narration_handles_empty_and_separators() {
        assert_eq!(join_for_narration(&[]), Component::empty());

        let a = Component::literal("a");
        let b = Component::literal("b");
        let c = Component::literal("c");
        let joined = join_for_narration(&[a.clone(), b.clone(), c.clone()]);

        let mut expected = Component::empty();
        expected.append(a);
        expected.append(NARRATION_SEPARATOR.clone());
        expected.append(b);
        expected.append(NARRATION_SEPARATOR.clone());
        expected.append(c);
        assert_eq!(joined, expected);
    }

    #[test]
    fn join_lines_matches_format_list_edge_cases() {
        assert_eq!(join_lines(&[]), Component::empty());

        let one = Component::literal("one");
        assert_eq!(join_lines(&[one.clone()]), one);

        let a = Component::literal("a");
        let b = Component::literal("b");
        let joined = join_lines_from_collection(&[a.clone(), b.clone()]);

        let mut expected = Component::empty();
        expected.append(a);
        expected.append(NEW_LINE.clone());
        expected.append(b);
        assert_eq!(joined, expected);
    }

    #[test]
    fn composed_options_and_float_bits_preservation() {
        let name = Component::literal("FOV");
        let value = Component::literal("90");
        assert_eq!(
            option_name_value(name.clone(), value.clone()),
            Component::translatable(
                "options.generic_value",
                vec![
                    ComponentArg::Component(name.clone()),
                    ComponentArg::Component(value),
                ],
            )
        );
        assert_eq!(
            option_status_composed(name.clone(), true),
            Component::translatable(
                "options.on.composed",
                vec![ComponentArg::Component(name.clone())],
            )
        );
        assert_eq!(
            option_status_composed(name, false),
            Component::translatable(
                "options.off.composed",
                vec![ComponentArg::Component(Component::literal("FOV"))],
            )
        );

        let nan_bits = f64::NAN.to_bits();
        assert_eq!(f64::from_bits(nan_bits).to_bits(), nan_bits);
        let neg_zero_bits = (-0.0f32).to_bits();
        assert_eq!(f32::from_bits(neg_zero_bits).to_bits(), neg_zero_bits);
    }
}
