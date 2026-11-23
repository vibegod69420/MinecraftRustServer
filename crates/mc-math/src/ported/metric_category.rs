#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricCategory {
    PathFinding,
    EventLoops,
    ConsecutiveExecutors,
    TickLoop,
    Jvm,
    ChunkRendering,
    ChunkRenderingDispatching,
    Cpu,
    Gpu,
}

impl MetricCategory {
    // JAVA: net.minecraft.util.profiling.metrics.MetricCategory#getDescription
    pub fn get_description(&self) -> &'static str {
        match self {
            Self::PathFinding => "pathfinding",
            Self::EventLoops => "event-loops",
            Self::ConsecutiveExecutors => "consecutive-executors",
            Self::TickLoop => "ticking",
            Self::Jvm => "jvm",
            Self::ChunkRendering => "chunk rendering",
            Self::ChunkRenderingDispatching => "chunk rendering dispatching",
            Self::Cpu => "cpu",
            Self::Gpu => "gpu",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MetricCategory;

    #[test]
    fn descriptions_match_java() {
        let cases = [
            (MetricCategory::PathFinding, "pathfinding"),
            (MetricCategory::EventLoops, "event-loops"),
            (MetricCategory::ConsecutiveExecutors, "consecutive-executors"),
            (MetricCategory::TickLoop, "ticking"),
            (MetricCategory::Jvm, "jvm"),
            (MetricCategory::ChunkRendering, "chunk rendering"),
            (MetricCategory::ChunkRenderingDispatching, "chunk rendering dispatching"),
            (MetricCategory::Cpu, "cpu"),
            (MetricCategory::Gpu, "gpu"),
        ];
        for (cat, expected) in cases {
            assert_eq!(cat.get_description(), expected);
        }
    }

    #[test]
    fn all_nine_variants_present() {
        let all = [
            MetricCategory::PathFinding,
            MetricCategory::EventLoops,
            MetricCategory::ConsecutiveExecutors,
            MetricCategory::TickLoop,
            MetricCategory::Jvm,
            MetricCategory::ChunkRendering,
            MetricCategory::ChunkRenderingDispatching,
            MetricCategory::Cpu,
            MetricCategory::Gpu,
        ];
        assert_eq!(all.len(), 9);
    }

    #[test]
    fn copy_eq_and_hash_behavior() {
        use std::collections::HashSet;
        let a = MetricCategory::Cpu;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(MetricCategory::Cpu, MetricCategory::Gpu);
        let mut set = HashSet::new();
        set.insert(MetricCategory::Jvm);
        set.insert(MetricCategory::Jvm);
        set.insert(MetricCategory::Cpu);
        assert_eq!(set.len(), 2);
    }
}
