use std::fmt;

/// Minimal trait for self-contained compilation.
/// A real `ChunkStatus` should implement this by returning its generation index.
pub trait ChunkStatusLike: fmt::Debug {
    fn get_index(&self) -> i32;
}

/// Port of `net.minecraft.world.level.chunk.status.ChunkDependencies`.
#[derive(Debug)]
pub struct ChunkDependencies<S> {
    dependency_by_radius: Vec<S>,
    radius_by_dependency: Vec<i32>,
}

impl<S: ChunkStatusLike> ChunkDependencies<S> {
    // JAVA: net.minecraft.world.level.chunk.status.ChunkDependencies#<init>
    pub fn new(dependencies: Vec<S>) -> Self {
        let len: i32 = if dependencies.is_empty() {
            0
        } else {
            dependencies[0].get_index().wrapping_add(1)
        };
        assert!(len >= 0, "NegativeArraySizeException: {}", len);
        let mut radius_by_dependency = vec![0i32; len as usize];

        let mut i: i32 = 0;
        let dep_len = dependencies.len() as i32;
        while i < dep_len {
            let idx = dependencies[i as usize].get_index();
            let mut j: i32 = 0;
            while j <= idx {
                radius_by_dependency[j as usize] = i;
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }

        Self {
            dependency_by_radius: dependencies,
            radius_by_dependency,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkDependencies#asList
    pub fn as_list(&self) -> &[S] {
        &self.dependency_by_radius
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkDependencies#size
    pub fn size(&self) -> i32 {
        self.dependency_by_radius.len() as i32
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkDependencies#getRadiusOf
    pub fn get_radius_of(&self, status: &S) -> i32 {
        let idx = status.get_index();
        if idx >= self.radius_by_dependency.len() as i32 {
            panic!(
                "Requesting a ChunkStatus({:?}) outside of dependency range({:?})",
                status, self.dependency_by_radius
            );
        }
        self.radius_by_dependency[idx as usize]
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkDependencies#getRadius
    pub fn get_radius(&self) -> i32 {
        let size = self.dependency_by_radius.len() as i32;
        std::cmp::max(0, size.wrapping_sub(1))
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkDependencies#get
    pub fn get(&self, index: i32) -> &S {
        &self.dependency_by_radius[index as usize]
    }
}

impl<S: fmt::Debug> fmt::Display for ChunkDependencies<S> {
    // JAVA: net.minecraft.world.level.chunk.status.ChunkDependencies#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.dependency_by_radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MockStatus {
        name: &'static str,
        index: i32,
    }

    impl MockStatus {
        fn new(name: &'static str, index: i32) -> Self {
            Self { name, index }
        }
    }

    impl ChunkStatusLike for MockStatus {
        fn get_index(&self) -> i32 {
            self.index
        }
    }

    #[test]
    fn basic_dependencies() {
        let deps = ChunkDependencies::new(vec![
            MockStatus::new("full", 2),
            MockStatus::new("features", 1),
            MockStatus::new("empty", 0),
        ]);
        assert_eq!(deps.size(), 3);
        assert_eq!(deps.get_radius(), 2);
        assert_eq!(deps.get(0).name, "full");
        assert_eq!(deps.get(2).name, "empty");
        assert_eq!(deps.as_list().len(), 3);
        assert_eq!(deps.to_string(), format!("{:?}", deps.as_list()));
    }

    #[test]
    fn get_radius_of_returns_correct_radius() {
        let deps = ChunkDependencies::new(vec![
            MockStatus::new("full", 2),
            MockStatus::new("features", 1),
            MockStatus::new("empty", 0),
        ]);
        assert_eq!(deps.get_radius_of(&MockStatus::new("q", 0)), 2);
        assert_eq!(deps.get_radius_of(&MockStatus::new("q", 1)), 1);
        assert_eq!(deps.get_radius_of(&MockStatus::new("q", 2)), 0);
    }

    #[test]
    #[should_panic(expected = "outside of dependency range")]
    fn get_radius_of_out_of_range_panics() {
        let deps = ChunkDependencies::new(vec![MockStatus::new("full", 2)]);
        deps.get_radius_of(&MockStatus::new("bad", 5));
    }

    #[test]
    fn empty_dependencies() {
        let deps: ChunkDependencies<MockStatus> = ChunkDependencies::new(vec![]);
        assert_eq!(deps.size(), 0);
        assert_eq!(deps.get_radius(), 0);
        assert!(deps.as_list().is_empty());
        assert_eq!(deps.to_string(), "[]");
    }

    #[test]
    #[should_panic]
    fn get_negative_index_panics() {
        let deps = ChunkDependencies::new(vec![MockStatus::new("a", 0)]);
        let _ = deps.get(-1);
    }

    #[test]
    #[should_panic]
    fn constructor_wrapping_overflow_panics() {
        // i32::MAX.wrapping_add(1) == i32::MIN, which triggers negative size panic.
        let _ = ChunkDependencies::new(vec![MockStatus::new("x", i32::MAX)]);
    }
}
