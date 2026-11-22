//! Port of `net.minecraft.network.chat.numbers.NumberFormat`.

/// Minimal, self-contained representation of a mutable chat component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutableComponent {
    pub text: String,
}

impl MutableComponent {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Minimal, self-contained representation of a number format type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFormatType {
    pub name: String,
}

/// Port of `net.minecraft.network.chat.numbers.NumberFormat`.
pub trait NumberFormat {
    // JAVA: net.minecraft.network.chat.numbers.NumberFormat#format
    fn format(&self, value: i32) -> MutableComponent;

    // JAVA: net.minecraft.network.chat.numbers.NumberFormat#type
    fn r#type(&self) -> NumberFormatType;
}

#[cfg(test)]
mod tests {
    use super::{MutableComponent, NumberFormat, NumberFormatType};

    struct DummyFormat {
        type_name: String,
    }

    impl NumberFormat for DummyFormat {
        fn format(&self, value: i32) -> MutableComponent {
            MutableComponent::literal(value.to_string())
        }

        fn r#type(&self) -> NumberFormatType {
            NumberFormatType {
                name: self.type_name.clone(),
            }
        }
    }

    #[test]
    fn format_positive_integer() {
        let fmt = DummyFormat {
            type_name: "dummy".to_string(),
        };
        assert_eq!(fmt.format(42).text, "42");
    }

    #[test]
    fn format_negative_integer() {
        let fmt = DummyFormat {
            type_name: "dummy".to_string(),
        };
        assert_eq!(fmt.format(-7).text, "-7");
    }

    #[test]
    fn format_i32_min() {
        let fmt = DummyFormat {
            type_name: "dummy".to_string(),
        };
        assert_eq!(fmt.format(i32::MIN).text, i32::MIN.to_string());
    }

    #[test]
    fn format_i32_max() {
        let fmt = DummyFormat {
            type_name: "dummy".to_string(),
        };
        assert_eq!(fmt.format(i32::MAX).text, i32::MAX.to_string());
    }

    #[test]
    fn type_returns_configured_number_format_type() {
        let fmt = DummyFormat {
            type_name: "example_type".to_string(),
        };
        let ty = fmt.r#type();
        assert_eq!(ty, NumberFormatType {
            name: "example_type".to_string()
        });
    }
}
