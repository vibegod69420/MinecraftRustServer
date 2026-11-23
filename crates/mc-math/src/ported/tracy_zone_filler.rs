// ported/tracy_zone_filler.rs

use std::collections::HashMap;

mod shared_constants {
    pub const IS_RUNNING_IN_IDE: bool = false;
}

pub trait ZoneName {
    fn into_name(self) -> String;
}

impl ZoneName for &str {
    fn into_name(self) -> String {
        self.to_string()
    }
}

impl ZoneName for String {
    fn into_name(self) -> String {
        self
    }
}

impl<F> ZoneName for F
where
    F: FnOnce() -> String,
{
    fn into_name(self) -> String {
        self()
    }
}

pub trait ProfilerFiller {
    fn start_tick(&mut self);
    fn end_tick(&mut self);
    fn push<N: ZoneName>(&mut self, name: N);
    fn pop(&mut self);
    fn pop_push<N: ZoneName>(&mut self, name: N);
    fn mark_for_charting(&mut self, category: MetricCategory);
    fn increment_counter<N: ZoneName>(&mut self, name: N, amount: i32);
    fn add_zone_text(&mut self, text: &str);
    fn add_zone_value(&mut self, value: i64);
    fn set_zone_color(&mut self, color: i32);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MetricCategory;

#[derive(Debug, Default)]
pub struct Zone {}

impl Zone {
    // JAVA: net.minecraft.util.profiling.TracyZoneFiller.Zone#close
    pub fn close(&mut self) {}

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller.Zone#addText
    pub fn add_text(&mut self, _text: &str) {}

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller.Zone#addValue
    pub fn add_value(&mut self, _value: i64) {}

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller.Zone#setColor
    pub fn set_color(&mut self, _color: i32) {}
}

pub struct Plot {
    #[allow(dead_code)]
    name: String,
}

impl Plot {
    // JAVA: com.mojang.jtracy.Plot#setValue
    pub fn set_value(&mut self, _value: i32) {}
}

pub struct TracyClient;

impl TracyClient {
    // JAVA: com.mojang.jtracy.TracyClient#beginZone
    pub fn begin_zone(_name: &str, _method: &str, _file: &str, _line: i32) -> Zone {
        Zone {}
    }

    // JAVA: com.mojang.jtracy.TracyClient#createPlot
    pub fn create_plot(name: &str) -> Plot {
        Plot {
            name: name.to_string(),
        }
    }
}

pub struct PlotAndValue {
    plot: Plot,
    value: i32,
}

impl PlotAndValue {
    // JAVA: net.minecraft.util.profiling.TracyZoneFiller.PlotAndValue#<init>
    pub fn new(name: String) -> Self {
        Self {
            plot: TracyClient::create_plot(&name),
            value: 0,
        }
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller.PlotAndValue#set
    pub fn set(&mut self, value: i32) {
        self.value = value;
        self.plot.set_value(value);
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller.PlotAndValue#add
    pub fn add(&mut self, amount: i32) {
        self.set(self.value.wrapping_add(amount));
    }
}

pub struct TracyZoneFiller {
    active_zones: Vec<Zone>,
    plots: HashMap<String, PlotAndValue>,
    name: String,
}

impl TracyZoneFiller {
    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#<init>
    pub fn new() -> Self {
        Self {
            active_zones: Vec::new(),
            plots: HashMap::new(),
            name: std::thread::current()
                .name()
                .unwrap_or("main")
                .to_string(),
        }
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#activeZone
    fn active_zone(&mut self) -> &mut Zone {
        self.active_zones
            .last_mut()
            .expect("no active Tracy zone")
    }
}

impl ProfilerFiller for TracyZoneFiller {
    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#startTick
    fn start_tick(&mut self) {}

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#endTick
    fn end_tick(&mut self) {
        for plot in self.plots.values_mut() {
            plot.set(0);
        }
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#push
    fn push<N: ZoneName>(&mut self, name: N) {
        let name = name.into_name();

        let (method, file, line) = if shared_constants::IS_RUNNING_IN_IDE {
            ("", "", 0)
        } else {
            ("", "", 0)
        };

        let zone = TracyClient::begin_zone(&name, method, file, line);
        self.active_zones.push(zone);
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#pop
    fn pop(&mut self) {
        if self.active_zones.is_empty() {
            eprintln!("Tried to pop one too many times! Mismatched push() and pop()?");
        } else {
            let mut zone = self.active_zones.remove(self.active_zones.len() - 1);
            zone.close();
        }
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#popPush
    fn pop_push<N: ZoneName>(&mut self, name: N) {
        self.pop();
        self.push(name);
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#markForCharting
    fn mark_for_charting(&mut self, _category: MetricCategory) {}

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#incrementCounter
    fn increment_counter<N: ZoneName>(&mut self, name: N, amount: i32) {
        let name = name.into_name();
        let thread_name = self.name.clone();
        let plot = self
            .plots
            .entry(name.clone())
            .or_insert_with(|| PlotAndValue::new(format!("{} {}", thread_name, name)));
        plot.add(amount);
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#addZoneText
    fn add_zone_text(&mut self, text: &str) {
        self.active_zone().add_text(text);
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#addZoneValue
    fn add_zone_value(&mut self, value: i64) {
        self.active_zone().add_value(value);
    }

    // JAVA: net.minecraft.util.profiling.TracyZoneFiller#setZoneColor
    fn set_zone_color(&mut self, color: i32) {
        self.active_zone().set_color(color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop_track_active_zones() {
        let mut filler = TracyZoneFiller::new();
        filler.push("root");
        assert_eq!(filler.active_zones.len(), 1);
        filler.pop();
        assert!(filler.active_zones.is_empty());
    }

    #[test]
    fn pop_on_empty_does_not_panic() {
        let mut filler = TracyZoneFiller::new();
        filler.pop();
        assert!(filler.active_zones.is_empty());
    }

    #[test]
    fn increment_counter_creates_and_accumulates_plot() {
        let mut filler = TracyZoneFiller::new();
        filler.increment_counter("ticks", 3);
        filler.increment_counter("ticks", 4);
        assert_eq!(filler.plots.get("ticks").unwrap().value, 7);
    }

    #[test]
    fn end_tick_resets_plot_values() {
        let mut filler = TracyZoneFiller::new();
        filler.increment_counter("ticks", 5);
        filler.end_tick();
        assert_eq!(filler.plots.get("ticks").unwrap().value, 0);
    }

    #[test]
    fn pop_push_replaces_active_zone() {
        let mut filler = TracyZoneFiller::new();
        filler.push("first");
        filler.pop_push("second");
        assert_eq!(filler.active_zones.len(), 1);
    }

    #[test]
    fn supplier_name_is_evaluated() {
        let mut filler = TracyZoneFiller::new();
        filler.push(|| "dynamic".to_string());
        assert_eq!(filler.active_zones.len(), 1);
    }
}
