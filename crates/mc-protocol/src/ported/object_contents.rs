use std::marker::PhantomData;

/// U+FFFC OBJECT REPLACEMENT CHARACTER, used as placeholder text for object contents.
// JAVA: net.minecraft.network.chat.contents.ObjectContents#PLACEHOLDER
pub const PLACEHOLDER: &str = "\u{FFFC}";

/// Minimal placeholder for Mojang's `MapCodec<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<T>(PhantomData<fn() -> T>);

impl<T> MapCodec<T> {
    /// Creates a placeholder codec marker.
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for MapCodec<T> {
    fn default() -> Self {
        Self::placeholder()
    }
}

/// Minimal style representation supporting font override.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    pub font: Option<String>,
}

impl Style {
    /// Returns a copy of this style with the given font applied.
    pub fn with_font(mut self, font: String) -> Self {
        self.font = Some(font);
        self
    }
}

/// Minimal equivalent of `FormattedText.ContentConsumer<T>`.
pub trait ContentConsumer<T> {
    fn accept(&mut self, text: String) -> Option<T>;
}

/// Minimal equivalent of `FormattedText.StyledContentConsumer<T>`.
pub trait StyledContentConsumer<T> {
    fn accept(&mut self, style: Style, text: &str) -> Option<T>;
}

/// Minimal equivalent of `net.minecraft.network.chat.contents.objects.ObjectInfo`.
pub trait ObjectInfo {
    fn description(&self) -> String;
    fn font_description(&self) -> String;
}

/// Port of `net.minecraft.network.chat.contents.ObjectContents`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectContents<I> {
    pub contents: I,
}

impl<I: ObjectInfo> ObjectContents<I> {
    /// Creates a new `ObjectContents`.
    pub fn new(contents: I) -> Self {
        Self { contents }
    }

    // JAVA: net.minecraft.network.chat.contents.ObjectContents#codec
    pub fn codec(&self) -> MapCodec<Self> {
        MapCodec::placeholder()
    }

    // JAVA: net.minecraft.network.chat.contents.ObjectContents#visit
    pub fn visit<T>(&self, consumer: &mut impl ContentConsumer<T>) -> Option<T> {
        consumer.accept(self.contents.description())
    }

    // JAVA: net.minecraft.network.chat.contents.ObjectContents#visit
    pub fn visit_styled<T>(
        &self,
        consumer: &mut impl StyledContentConsumer<T>,
        style: Style,
    ) -> Option<T> {
        consumer.accept(
            style.with_font(self.contents.font_description()),
            PLACEHOLDER,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockObjectInfo {
        desc: String,
        font: String,
    }

    impl ObjectInfo for MockObjectInfo {
        fn description(&self) -> String {
            self.desc.clone()
        }

        fn font_description(&self) -> String {
            self.font.clone()
        }
    }

    struct StringConsumer;

    impl ContentConsumer<String> for StringConsumer {
        fn accept(&mut self, text: String) -> Option<String> {
            Some(text)
        }
    }

    #[derive(Debug, Default)]
    struct StyledCapture {
        seen: Option<(Style, String)>,
    }

    impl StyledContentConsumer<(Style, String)> for StyledCapture {
        fn accept(&mut self, style: Style, text: &str) -> Option<(Style, String)> {
            let result = (style, text.to_string());
            self.seen = Some(result.clone());
            Some(result)
        }
    }

    #[test]
    fn placeholder_is_object_replacement_character() {
        assert_eq!(PLACEHOLDER, "\u{FFFC}");
        assert_eq!(PLACEHOLDER.chars().count(), 1);
        assert_eq!(PLACEHOLDER.chars().next().unwrap() as u32, 0xFFFC);
    }

    #[test]
    fn visit_returns_description() {
        let obj = ObjectContents::new(MockObjectInfo {
            desc: "hello".to_string(),
            font: "minecraft:default".to_string(),
        });
        let mut consumer = StringConsumer;
        assert_eq!(obj.visit(&mut consumer), Some("hello".to_string()));
    }

    #[test]
    fn visit_styled_applies_font_and_uses_placeholder() {
        let obj = ObjectContents::new(MockObjectInfo {
            desc: "ignored-description".to_string(),
            font: "minecraft:alt".to_string(),
        });
        let mut consumer = StyledCapture::default();
        let base_style = Style {
            font: Some("minecraft:base".to_string()),
        };

        let (out_style, out_text) = obj.visit_styled(&mut consumer, base_style).unwrap();
        assert_eq!(out_style.font.as_deref(), Some("minecraft:alt"));
        assert_eq!(out_text, PLACEHOLDER);
    }

    #[test]
    fn visit_handles_empty_and_unicode_edge_cases() {
        let empty_obj = ObjectContents::new(MockObjectInfo {
            desc: String::new(),
            font: String::new(),
        });
        let mut c1 = StringConsumer;
        assert_eq!(empty_obj.visit(&mut c1), Some(String::new()));

        let mut c2 = StyledCapture::default();
        let (style, text) = empty_obj
            .visit_styled(&mut c2, Style::default())
            .unwrap();
        assert_eq!(style.font.as_deref(), Some(""));
        assert_eq!(text, PLACEHOLDER);

        let unicode_obj = ObjectContents::new(MockObjectInfo {
            desc: "日本語🎮".to_string(),
            font: "ns:font/特殊".to_string(),
        });
        let mut c3 = StringConsumer;
        assert_eq!(unicode_obj.visit(&mut c3), Some("日本語🎮".to_string()));
    }

    #[test]
    fn codec_returns_placeholder_and_data_passes_numeric_edge_cases() {
        let neg_i32: i32 = -1;
        let wrapped_u32: u32 = u32::MAX.wrapping_add(1);
        let neg_zero_bits: u64 = (-0.0f64).to_bits();
        let nan_bits: u64 = f64::NAN.to_bits();

        let obj = ObjectContents::new(MockObjectInfo {
            desc: format!("{neg_i32},{wrapped_u32}"),
            font: format!("font:{neg_zero_bits}:{nan_bits}"),
        });

        let _codec: MapCodec<ObjectContents<MockObjectInfo>> = obj.codec();

        let mut c1 = StringConsumer;
        assert_eq!(obj.visit(&mut c1), Some("-1,0".to_string()));

        let mut c2 = StyledCapture::default();
        let (style, _) = obj
            .visit_styled(&mut c2, Style::default())
            .unwrap();
        assert_eq!(
            style.font.as_deref(),
            Some(format!("font:{neg_zero_bits}:{nan_bits}").as_str())
        );
        assert_eq!(neg_zero_bits, 0x8000_0000_0000_0000u64);
        assert!(f64::from_bits(nan_bits).is_nan());
        assert_eq!(wrapped_u32, 0u32);
    }
}
