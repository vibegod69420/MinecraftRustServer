use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub struct Graph;

impl Graph {
    // JAVA: net.minecraft.util.Graph#depthFirstSearch
    pub fn depth_first_search<T, F>(
        graph: &HashMap<T, HashSet<T>>,
        discovered: &mut HashSet<T>,
        visiting: &mut HashSet<T>,
        output: &mut F,
        current: T,
    ) -> bool
    where
        T: Eq + Hash + Clone,
        F: FnMut(T),
    {
        if discovered.contains(&current) {
            return false;
        }

        if visiting.contains(&current) {
            return true;
        }

        visiting.insert(current.clone());

        let neighbors: Vec<T> = graph
            .get(&current)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default();

        for neighbor in neighbors {
            if Self::depth_first_search(graph, discovered, visiting, output, neighbor) {
                return true;
            }
        }

        visiting.remove(&current);
        discovered.insert(current.clone());
        output(current);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::Graph;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn detects_cycle() {
        let mut graph: HashMap<i32, HashSet<i32>> = HashMap::new();
        graph.insert(1, HashSet::from([2]));
        graph.insert(2, HashSet::from([1]));

        let mut discovered = HashSet::new();
        let mut visiting = HashSet::new();
        let mut out = Vec::new();
        let has_cycle =
            Graph::depth_first_search(&graph, &mut discovered, &mut visiting, &mut |v| out.push(v), 1);

        assert!(has_cycle);
    }

    #[test]
    fn dag_produces_postorder_output() {
        let mut graph: HashMap<i32, HashSet<i32>> = HashMap::new();
        graph.insert(1, HashSet::from([2]));
        graph.insert(2, HashSet::from([3]));
        // 3 has no outgoing edges

        let mut discovered = HashSet::new();
        let mut visiting = HashSet::new();
        let mut out = Vec::new();
        let has_cycle =
            Graph::depth_first_search(&graph, &mut discovered, &mut visiting, &mut |v| out.push(v), 1);

        assert!(!has_cycle);
        assert_eq!(out, vec![3, 2, 1]);
    }

    #[test]
    fn already_discovered_returns_false() {
        let graph: HashMap<i32, HashSet<i32>> = HashMap::new();
        let mut discovered = HashSet::from([42]);
        let mut visiting = HashSet::new();
        let mut out = Vec::new();
        let has_cycle =
            Graph::depth_first_search(&graph, &mut discovered, &mut visiting, &mut |v| out.push(v), 42);

        assert!(!has_cycle);
        assert!(out.is_empty());
    }

    #[test]
    fn already_visiting_returns_true() {
        let graph: HashMap<i32, HashSet<i32>> = HashMap::new();
        let mut discovered = HashSet::new();
        let mut visiting = HashSet::from([7]);
        let mut out = Vec::new();
        let has_cycle =
            Graph::depth_first_search(&graph, &mut discovered, &mut visiting, &mut |v| out.push(v), 7);

        assert!(has_cycle);
        assert!(out.is_empty());
    }

    #[test]
    fn missing_node_treated_as_no_neighbors() {
        let graph: HashMap<i32, HashSet<i32>> = HashMap::new();
        let mut discovered = HashSet::new();
        let mut visiting = HashSet::new();
        let mut out = Vec::new();
        let has_cycle =
            Graph::depth_first_search(&graph, &mut discovered, &mut visiting, &mut |v| out.push(v), 99);

        assert!(!has_cycle);
        assert_eq!(out, vec![99]);
        assert!(discovered.contains(&99));
        assert!(visiting.is_empty());
    }
}
