use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Entry interface for [`DependencySorter`].
pub trait Entry<K> {
    // JAVA: net.minecraft.util.DependencySorter$Entry#visitRequiredDependencies
    fn visit_required_dependencies(&self, visitor: &mut dyn FnMut(&K));
    // JAVA: net.minecraft.util.DependencySorter$Entry#visitOptionalDependencies
    fn visit_optional_dependencies(&self, visitor: &mut dyn FnMut(&K));
}

/// Faithful port of `net.minecraft.util.DependencySorter`.
pub struct DependencySorter<K, V> {
    contents: HashMap<K, V>,
}

impl<K: Eq + Hash + Clone, V: Entry<K>> DependencySorter<K, V> {
    // JAVA: net.minecraft.util.DependencySorter#<init>
    pub fn new() -> Self {
        Self {
            contents: HashMap::new(),
        }
    }

    // JAVA: net.minecraft.util.DependencySorter#addEntry
    pub fn add_entry(&mut self, key: K, value: V) -> &mut Self {
        self.contents.insert(key, value);
        self
    }

    // JAVA: net.minecraft.util.DependencySorter#visitDependenciesAndElement
    fn visit_dependencies_and_element<F>(
        &self,
        deps: &HashMap<K, Vec<K>>,
        visited: &mut HashSet<K>,
        key: &K,
        callback: &mut F,
    ) where
        F: FnMut(&K, &V),
    {
        if visited.insert(key.clone()) {
            if let Some(dep_keys) = deps.get(key) {
                let dep_keys = dep_keys.clone();
                for dep in dep_keys.iter() {
                    self.visit_dependencies_and_element(deps, visited, dep, callback);
                }
            }
            if let Some(value) = self.contents.get(key) {
                callback(key, value);
            }
        }
    }

    // JAVA: net.minecraft.util.DependencySorter#isCyclic
    fn is_cyclic(deps: &HashMap<K, Vec<K>>, from: &K, to: &K) -> bool {
        if let Some(collection) = deps.get(to) {
            if collection.contains(from) {
                return true;
            }
            for dep in collection.iter() {
                if Self::is_cyclic(deps, from, dep) {
                    return true;
                }
            }
        }
        false
    }

    // JAVA: net.minecraft.util.DependencySorter#addDependencyIfNotCyclic
    fn add_dependency_if_not_cyclic(deps: &mut HashMap<K, Vec<K>>, key: K, dep: K) {
        if !Self::is_cyclic(deps, &key, &dep) {
            let entry = deps.entry(key).or_default();
            if !entry.contains(&dep) {
                entry.push(dep);
            }
        }
    }

    // JAVA: net.minecraft.util.DependencySorter#orderByDependencies
    pub fn order_by_dependencies<F>(&self, mut callback: F)
    where
        F: FnMut(&K, &V),
    {
        let mut deps: HashMap<K, Vec<K>> = HashMap::new();
        for (key, value) in &self.contents {
            value.visit_required_dependencies(&mut |dep| {
                Self::add_dependency_if_not_cyclic(&mut deps, key.clone(), dep.clone());
            });
        }
        for (key, value) in &self.contents {
            value.visit_optional_dependencies(&mut |dep| {
                Self::add_dependency_if_not_cyclic(&mut deps, key.clone(), dep.clone());
            });
        }
        let mut visited: HashSet<K> = HashSet::new();
        let keys: Vec<K> = self.contents.keys().cloned().collect();
        for key in keys.iter() {
            self.visit_dependencies_and_element(&deps, &mut visited, key, &mut callback);
        }
    }
}

impl<K: Eq + Hash + Clone, V: Entry<K>> Default for DependencySorter<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntry {
        required: Vec<i32>,
        optional: Vec<i32>,
    }

    impl Entry<i32> for TestEntry {
        fn visit_required_dependencies(&self, visitor: &mut dyn FnMut(&i32)) {
            for dep in &self.required {
                visitor(dep);
            }
        }

        fn visit_optional_dependencies(&self, visitor: &mut dyn FnMut(&i32)) {
            for dep in &self.optional {
                visitor(dep);
            }
        }
    }

    #[test]
    fn orders_dependencies_before_dependents() {
        let mut sorter = DependencySorter::new();
        sorter.add_entry(1, TestEntry { required: vec![2], optional: vec![] });
        sorter.add_entry(2, TestEntry { required: vec![], optional: vec![] });
        let mut order = Vec::new();
        sorter.order_by_dependencies(|k, _| order.push(*k));
        let pos1 = order.iter().position(|x| *x == 1).unwrap();
        let pos2 = order.iter().position(|x| *x == 2).unwrap();
        assert!(pos2 < pos1);
    }

    #[test]
    fn respects_optional_dependencies() {
        let mut sorter = DependencySorter::new();
        sorter.add_entry(1, TestEntry { required: vec![], optional: vec![2] });
        sorter.add_entry(2, TestEntry { required: vec![], optional: vec![] });
        let mut order = Vec::new();
        sorter.order_by_dependencies(|k, _| order.push(*k));
        let pos1 = order.iter().position(|x| *x == 1).unwrap();
        let pos2 = order.iter().position(|x| *x == 2).unwrap();
        assert!(pos2 < pos1);
    }

    #[test]
    fn ignores_cyclic_dependency() {
        let mut sorter = DependencySorter::new();
        sorter.add_entry(1, TestEntry { required: vec![2], optional: vec![] });
        sorter.add_entry(2, TestEntry { required: vec![1], optional: vec![] });
        let mut order = Vec::new();
        sorter.order_by_dependencies(|k, _| order.push(*k));
        assert_eq!(order.len(), 2);
        assert!(order.contains(&1));
        assert!(order.contains(&2));
    }

    #[test]
    fn missing_dependency_does_not_block_element() {
        let mut sorter = DependencySorter::new();
        sorter.add_entry(1, TestEntry { required: vec![99], optional: vec![] });
        let mut order = Vec::new();
        sorter.order_by_dependencies(|k, _| order.push(*k));
        assert_eq!(order, vec![1]);
    }

    #[test]
    fn visits_element_only_once_for_diamond() {
        let mut sorter = DependencySorter::new();
        sorter.add_entry(1, TestEntry { required: vec![2, 3], optional: vec![] });
        sorter.add_entry(2, TestEntry { required: vec![4], optional: vec![] });
        sorter.add_entry(3, TestEntry { required: vec![4], optional: vec![] });
        sorter.add_entry(4, TestEntry { required: vec![], optional: vec![] });
        let mut order = Vec::new();
        sorter.order_by_dependencies(|k, _| order.push(*k));
        assert_eq!(order.len(), 4);
        let pos = |x: i32| order.iter().position(|v| *v == x).unwrap();
        assert!(pos(4) < pos(2));
        assert!(pos(4) < pos(3));
        assert!(pos(2) < pos(1));
        assert!(pos(3) < pos(1));
    }

    #[test]
    fn add_entry_chains() {
        let mut sorter = DependencySorter::new();
        sorter
            .add_entry(1, TestEntry { required: vec![], optional: vec![] })
            .add_entry(2, TestEntry { required: vec![], optional: vec![] });
        let mut order = Vec::new();
        sorter.order_by_dependencies(|k, _| order.push(*k));
        assert_eq!(order.len(), 2);
        assert!(order.contains(&1));
        assert!(order.contains(&2));
    }
}
