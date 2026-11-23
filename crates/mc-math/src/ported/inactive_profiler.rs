use std::collections::HashSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfileResults;

pub struct EmptyProfileResults;

impl EmptyProfileResults {
    pub const EMPTY: ProfileResults = ProfileResults;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetricCategory(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zone;

impl Zone {
    pub const INACTIVE: Zone = Zone;
}

pub trait ProfilerFiller {
    fn start_tick(&mut self);
    fn end_tick(&mut self);
    fn push(&mut self, name: &str);
    fn push_supplier(&mut self, name_supplier: &dyn Fn() -> String);
    fn mark_for_charting(&mut self, category: &MetricCategory);
    fn pop(&mut self);
    fn pop_push(&mut self, name: &str);
    fn pop_push_supplier(&mut self, name_supplier: &dyn Fn() -> String);
    fn zone(&mut self, name: &str) -> Zone;
    fn zone_supplier(&mut self, name_supplier: &dyn Fn() -> String) -> Zone;
    fn increment_counter(&mut self, name: &str, count: i32);
    fn increment_counter_supplier(&mut self, name_supplier: &dyn Fn() -> String, count: i32);
}

pub trait ProfileCollector: ProfilerFiller {
    fn get_results(&self) -> ProfileResults;
    fn get_entry(&self, name: &str) -> Option<PathEntry>;
    fn get_charted_paths(&self) -> HashSet<(String, MetricCategory)>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InactiveProfiler;

impl InactiveProfiler {
    pub const INSTANCE: InactiveProfiler = InactiveProfiler;

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#<init>
    fn new() -> Self {
        Self
    }
}

impl ProfilerFiller for InactiveProfiler {
    // JAVA: net.minecraft.util.profiling.InactiveProfiler#startTick
    fn start_tick(&mut self) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#endTick
    fn end_tick(&mut self) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#push(String)
    fn push(&mut self, _name: &str) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#push(Supplier)
    fn push_supplier(&mut self, _name_supplier: &dyn Fn() -> String) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#markForCharting
    fn mark_for_charting(&mut self, _category: &MetricCategory) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#pop
    fn pop(&mut self) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#popPush(String)
    fn pop_push(&mut self, _name: &str) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#popPush(Supplier)
    fn pop_push_supplier(&mut self, _name_supplier: &dyn Fn() -> String) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#zone(String)
    fn zone(&mut self, _name: &str) -> Zone {
        Zone::INACTIVE
    }

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#zone(Supplier)
    fn zone_supplier(&mut self, _name_supplier: &dyn Fn() -> String) -> Zone {
        Zone::INACTIVE
    }

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#incrementCounter(String,int)
    fn increment_counter(&mut self, _name: &str, _count: i32) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#incrementCounter(Supplier,int)
    fn increment_counter_supplier(&mut self, _name_supplier: &dyn Fn() -> String, _count: i32) {}
}

impl ProfileCollector for InactiveProfiler {
    // JAVA: net.minecraft.util.profiling.InactiveProfiler#getResults
    fn get_results(&self) -> ProfileResults {
        EmptyProfileResults::EMPTY
    }

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#getEntry
    fn get_entry(&self, _name: &str) -> Option<PathEntry> {
        None
    }

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#getChartedPaths
    fn get_charted_paths(&self) -> HashSet<(String, MetricCategory)> {
        HashSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_is_usable_and_noop() {
        let mut p = InactiveProfiler::INSTANCE;
        p.start_tick();
        p.push("root");
        p.push_supplier(&|| "supplier".to_string());
        p.mark_for_charting(&MetricCategory("test".to_string()));
        p.pop_push("next");
        p.pop_push_supplier(&|| "next2".to_string());
        p.increment_counter("c", i32::MAX);
        p.increment_counter_supplier(&|| "c2".to_string(), i32::MIN);
        p.pop();
        p.end_tick();
    }

    #[test]
    fn zones_are_inactive() {
        let mut p = InactiveProfiler::new();
        assert_eq!(p.zone("a"), Zone::INACTIVE);
        assert_eq!(p.zone_supplier(&|| "b".to_string()), Zone::INACTIVE);
    }

    #[test]
    fn results_and_entries_are_empty() {
        let p = InactiveProfiler::INSTANCE;
        let _results = p.get_results();
        assert!(p.get_entry("missing").is_none());
        assert!(p.get_charted_paths().is_empty());
    }

    #[test]
    fn empty_profile_results_constant_exists() {
        let _ = EmptyProfileResults::EMPTY;
        let _ = ProfileResults::default();
    }
}
