pub trait ProfilerFiller {
    fn add_zone_text(&mut self, text: &str);
    fn add_zone_value(&mut self, value: i64);
    fn set_zone_color(&mut self, color: i32);
    fn pop(&mut self);
}

pub struct Zone<'a> {
    profiler: Option<&'a mut dyn ProfilerFiller>,
}

impl<'a> Zone<'a> {
    pub const INACTIVE: Zone<'static> = Zone { profiler: None };

    // JAVA: net.minecraft.util.profiling.Zone#<init>
    pub fn new(profiler: Option<&'a mut dyn ProfilerFiller>) -> Self {
        Self { profiler }
    }

    // JAVA: net.minecraft.util.profiling.Zone#addText(String)
    pub fn add_text(&mut self, text: &str) -> &mut Self {
        if let Some(profiler) = self.profiler.as_deref_mut() {
            profiler.add_zone_text(text);
        }
        self
    }

    // JAVA: net.minecraft.util.profiling.Zone#addText(Supplier)
    pub fn add_text_supplier<F: FnOnce() -> String>(&mut self, text: F) -> &mut Self {
        if let Some(profiler) = self.profiler.as_deref_mut() {
            profiler.add_zone_text(&text());
        }
        self
    }

    // JAVA: net.minecraft.util.profiling.Zone#addValue
    pub fn add_value(&mut self, value: i64) -> &mut Self {
        if let Some(profiler) = self.profiler.as_deref_mut() {
            profiler.add_zone_value(value);
        }
        self
    }

    // JAVA: net.minecraft.util.profiling.Zone#setColor
    pub fn set_color(&mut self, color: i32) -> &mut Self {
        if let Some(profiler) = self.profiler.as_deref_mut() {
            profiler.set_zone_color(color);
        }
        self
    }

    // JAVA: net.minecraft.util.profiling.Zone#close
    pub fn close(&mut self) {
        if let Some(profiler) = self.profiler.take() {
            profiler.pop();
        }
    }
}

impl Drop for Zone<'_> {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockProfiler {
        texts: Vec<String>,
        values: Vec<i64>,
        colors: Vec<i32>,
        pops: i32,
    }

    impl ProfilerFiller for MockProfiler {
        fn add_zone_text(&mut self, text: &str) {
            self.texts.push(text.to_string());
        }
        fn add_zone_value(&mut self, value: i64) {
            self.values.push(value);
        }
        fn set_zone_color(&mut self, color: i32) {
            self.colors.push(color);
        }
        fn pop(&mut self) {
            self.pops += 1;
        }
    }

    #[test]
    fn inactive_zone_is_noop() {
        let mut z = Zone::INACTIVE;
        z.add_text("x").add_value(1).set_color(0xFF);
        z.add_text_supplier(|| "y".to_string());
        z.close();
    }

    #[test]
    fn active_zone_records_calls() {
        let mut p = MockProfiler::default();
        {
            let mut z = Zone::new(Some(&mut p));
            z.add_text("hello")
                .add_text_supplier(|| "world".to_string())
                .add_value(i64::MIN)
                .add_value(i64::MAX)
                .set_color(-1);
        }
        assert_eq!(p.texts, vec!["hello", "world"]);
        assert_eq!(p.values, vec![i64::MIN, i64::MAX]);
        assert_eq!(p.colors, vec![-1]);
        assert_eq!(p.pops, 1);
    }

    #[test]
    fn close_pops_once() {
        let mut p = MockProfiler::default();
        let mut z = Zone::new(Some(&mut p));
        z.close();
        z.close();
        drop(z);
        assert_eq!(p.pops, 1);
    }

    #[test]
    fn drop_pops_when_not_closed() {
        let mut p = MockProfiler::default();
        {
            let _z = Zone::new(Some(&mut p));
        }
        assert_eq!(p.pops, 1);
    }
}
