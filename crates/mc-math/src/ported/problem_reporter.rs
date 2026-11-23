// === FILE: problem_reporter.rs ===
//! Faithful port of `net.minecraft.util.ProblemReporter` (Java 1.21.11).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub trait ProblemLogger {
    fn name(&self) -> &str;
    fn warn(&self, msg: &str);
}

pub trait Problem {
    // JAVA: net.minecraft.util.ProblemReporter$Problem#description
    fn description(&self) -> String;
}

impl Problem for String {
    // JAVA: net.minecraft.util.ProblemReporter$Problem#description
    fn description(&self) -> String {
        self.clone()
    }
}

impl Problem for &'static str {
    // JAVA: net.minecraft.util.ProblemReporter$Problem#description
    fn description(&self) -> String {
        (*self).to_string()
    }
}

pub trait PathElement {
    // JAVA: net.minecraft.util.ProblemReporter$PathElement#get
    fn get(&self) -> String;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub identifier: String,
    pub registry: String,
}

impl ResourceKey {
    pub fn new(identifier: impl Into<String>, registry: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            registry: registry.into(),
        }
    }
}

// JAVA: net.minecraft.util.ProblemReporter$ElementReferencePathElement
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ElementReferencePathElement {
    pub id: ResourceKey,
}

impl PathElement for ElementReferencePathElement {
    // JAVA: net.minecraft.util.ProblemReporter$ElementReferencePathElement#get
    fn get(&self) -> String {
        format!("->{{{}@{}}}", self.id.identifier, self.id.registry)
    }
}

// JAVA: net.minecraft.util.ProblemReporter$FieldPathElement
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldPathElement {
    pub name: String,
}

impl PathElement for FieldPathElement {
    // JAVA: net.minecraft.util.ProblemReporter$FieldPathElement#get
    fn get(&self) -> String {
        format!(".{}", self.name)
    }
}

// JAVA: net.minecraft.util.ProblemReporter$IndexedFieldPathElement
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IndexedFieldPathElement {
    pub name: String,
    pub index: i32,
}

impl PathElement for IndexedFieldPathElement {
    // JAVA: net.minecraft.util.ProblemReporter$IndexedFieldPathElement#get
    fn get(&self) -> String {
        format!(".{}[{}]", self.name, self.index)
    }
}

// JAVA: net.minecraft.util.ProblemReporter$IndexedPathElement
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IndexedPathElement {
    pub index: i32,
}

impl PathElement for IndexedPathElement {
    // JAVA: net.minecraft.util.ProblemReporter$IndexedPathElement#get
    fn get(&self) -> String {
        format!("[{}]", self.index)
    }
}

// JAVA: net.minecraft.util.ProblemReporter$RootElementPathElement
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RootElementPathElement {
    pub id: ResourceKey,
}

impl PathElement for RootElementPathElement {
    // JAVA: net.minecraft.util.ProblemReporter$RootElementPathElement#get
    fn get(&self) -> String {
        format!("{{{}@{}}}", self.id.identifier, self.id.registry)
    }
}

// JAVA: net.minecraft.util.ProblemReporter$RootFieldPathElement
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RootFieldPathElement {
    pub name: String,
}

impl PathElement for RootFieldPathElement {
    // JAVA: net.minecraft.util.ProblemReporter$RootFieldPathElement#get
    fn get(&self) -> String {
        self.name.clone()
    }
}

// JAVA: net.minecraft.util.ProblemReporter$Collector#EMPTY_ROOT
#[derive(Clone, Copy, Debug, Default)]
pub struct EmptyRoot;

impl PathElement for EmptyRoot {
    // JAVA: net.minecraft.util.ProblemReporter$PathElement#get
    fn get(&self) -> String {
        String::new()
    }
}

pub const EMPTY_ROOT: EmptyRoot = EmptyRoot;

pub trait ProblemReporter {
    // JAVA: net.minecraft.util.ProblemReporter#forChild
    fn for_child(&self, element: Box<dyn PathElement>) -> Box<dyn ProblemReporter>;

    // JAVA: net.minecraft.util.ProblemReporter#report
    fn report(&self, problem: Box<dyn Problem>);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DiscardingReporter;

pub const DISCARDING: DiscardingReporter = DiscardingReporter;

impl ProblemReporter for DiscardingReporter {
    // JAVA: net.minecraft.util.ProblemReporter#forChild
    fn for_child(&self, _element: Box<dyn PathElement>) -> Box<dyn ProblemReporter> {
        Box::new(DiscardingReporter)
    }

    // JAVA: net.minecraft.util.ProblemReporter#report
    fn report(&self, _problem: Box<dyn Problem>) {}
}

struct CollectorNode {
    parent: Option<Rc<CollectorNode>>,
    element: Rc<dyn PathElement>,
}

struct CollectorEntry {
    source: Rc<CollectorNode>,
    problem: Rc<dyn Problem>,
}

#[derive(Clone)]
pub struct Collector {
    node: Rc<CollectorNode>,
    problems: Rc<RefCell<Vec<CollectorEntry>>>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    // JAVA: net.minecraft.util.ProblemReporter$Collector#Collector
    pub fn new() -> Self {
        Self::with_element(Box::new(EmptyRoot))
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#Collector
    pub fn with_element(element: Box<dyn PathElement>) -> Self {
        Self {
            node: Rc::new(CollectorNode {
                parent: None,
                element: Rc::from(element),
            }),
            problems: Rc::new(RefCell::new(Vec::new())),
        }
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#Collector
    fn from_parent(parent: &Collector, element: Box<dyn PathElement>) -> Self {
        Self {
            node: Rc::new(CollectorNode {
                parent: Some(parent.node.clone()),
                element: Rc::from(element),
            }),
            problems: parent.problems.clone(),
        }
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#forChild
    pub fn for_child(&self, element: Box<dyn PathElement>) -> Collector {
        Self::from_parent(self, element)
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#report
    pub fn report(&self, problem: Box<dyn Problem>) {
        let entry = CollectorEntry {
            source: self.node.clone(),
            problem: Rc::from(problem),
        };
        let mut problems = self.problems.borrow_mut();
        let exists = problems.iter().any(|e| {
            Rc::ptr_eq(&e.source, &entry.source)
                && e.problem.description() == entry.problem.description()
        });
        if !exists {
            problems.push(entry);
        }
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#isEmpty
    pub fn is_empty(&self) -> bool {
        self.problems.borrow().is_empty()
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#forEach
    pub fn for_each(&self, mut callback: impl FnMut(String, &dyn Problem)) {
        let mut path_elements: Vec<Rc<dyn PathElement>> = Vec::new();
        let mut path = String::new();
        let problems = self.problems.borrow();
        for entry in problems.iter() {
            let mut current: Option<Rc<CollectorNode>> = Some(entry.source.clone());
            while let Some(node) = current {
                path_elements.push(node.element.clone());
                current = node.parent.clone();
            }
            for i in (0..path_elements.len()).rev() {
                path.push_str(&path_elements[i].get());
            }
            callback(path.clone(), &*entry.problem);
            path.clear();
            path_elements.clear();
        }
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#getReport
    pub fn get_report(&self) -> String {
        let mut keys: Vec<String> = Vec::new();
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        self.for_each(|path, problem| {
            let desc = problem.description();
            if let Some(values) = map.get_mut(&path) {
                values.push(desc);
            } else {
                keys.push(path.clone());
                map.insert(path, vec![desc]);
            }
        });
        keys.into_iter()
            .map(|k| {
                let descs = map.get(&k).unwrap().join("; ");
                format!(" at {}: {}", k, descs)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#getTreeReport
    pub fn get_tree_report(&self) -> String {
        let mut path_elements: Vec<Rc<dyn PathElement>> = Vec::new();
        let mut root = ProblemTreeNode::new(self.node.element.get());
        let problems = self.problems.borrow();
        for entry in problems.iter() {
            let mut current = Some(entry.source.clone());
            let mut found = false;
            while let Some(node) = current {
                if Rc::ptr_eq(&node, &self.node) {
                    found = true;
                    break;
                }
                path_elements.push(node.element.clone());
                current = node.parent.clone();
            }
            if !found {
                path_elements.clear();
                continue;
            }

            let mut target = &mut root;
            for i in (0..path_elements.len()).rev() {
                let key = path_elements[i].get();
                target = target.child(&key);
            }
            path_elements.clear();
            target.problems.push(entry.problem.description());
        }
        drop(problems);
        root.get_lines().join("\n")
    }
}

impl ProblemReporter for Collector {
    // JAVA: net.minecraft.util.ProblemReporter$Collector#forChild
    fn for_child(&self, element: Box<dyn PathElement>) -> Box<dyn ProblemReporter> {
        Box::new(Self::from_parent(self, element))
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#report
    fn report(&self, problem: Box<dyn Problem>) {
        Collector::report(self, problem);
    }
}

struct ProblemTreeNode {
    element: String,
    problems: Vec<String>,
    children: Vec<(String, ProblemTreeNode)>,
}

impl ProblemTreeNode {
    // JAVA: net.minecraft.util.ProblemReporter$Collector$ProblemTreeNode#ProblemTreeNode
    fn new(element: String) -> Self {
        Self {
            element,
            problems: Vec::new(),
            children: Vec::new(),
        }
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector$ProblemTreeNode#child
    fn child(&mut self, key: &str) -> &mut ProblemTreeNode {
        if let Some(pos) = self.children.iter().position(|(k, _)| k == key) {
            return &mut self.children[pos].1;
        }
        let owned = key.to_string();
        self.children
            .push((owned.clone(), ProblemTreeNode::new(owned)));
        let last = self.children.len() - 1;
        &mut self.children[last].1
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector$ProblemTreeNode#getLines
    fn get_lines(&self) -> Vec<String> {
        let problem_count = self.problems.len();
        let child_count = self.children.len();
        if problem_count == 0 && child_count == 0 {
            return Vec::new();
        }

        if problem_count == 0 && child_count == 1 {
            let mut lines = Vec::new();
            for (_k, child) in &self.children {
                lines.extend(child.get_lines());
            }
            if !lines.is_empty() {
                lines[0] = format!("{}{}", self.element, lines[0]);
            }
            return lines;
        }

        if problem_count == 1 && child_count == 0 {
            return vec![format!("{}: {}", self.element, self.problems[0])];
        }

        let mut lines = Vec::new();
        for (_k, child) in &self.children {
            lines.extend(child.get_lines());
        }
        for line in &mut lines {
            *line = format!("  {}", line);
        }
        for problem in &self.problems {
            lines.push(format!("  {}", problem));
        }
        lines.insert(0, format!("{}:", self.element));
        lines
    }
}

pub struct ScopedCollector {
    collector: Collector,
    logger: Box<dyn ProblemLogger>,
    closed: bool,
}

impl std::ops::Deref for ScopedCollector {
    type Target = Collector;
    fn deref(&self) -> &Self::Target {
        &self.collector
    }
}

impl std::ops::DerefMut for ScopedCollector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.collector
    }
}

impl ScopedCollector {
    // JAVA: net.minecraft.util.ProblemReporter$ScopedCollector#ScopedCollector
    pub fn new(logger: Box<dyn ProblemLogger>) -> Self {
        Self {
            collector: Collector::new(),
            logger,
            closed: false,
        }
    }

    // JAVA: net.minecraft.util.ProblemReporter$ScopedCollector#ScopedCollector
    pub fn with_element(element: Box<dyn PathElement>, logger: Box<dyn ProblemLogger>) -> Self {
        Self {
            collector: Collector::with_element(element),
            logger,
            closed: false,
        }
    }

    // JAVA: net.minecraft.util.ProblemReporter$ScopedCollector#close
    pub fn close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        if !self.collector.is_empty() {
            let report = self.collector.get_tree_report();
            let msg = format!(
                "[{}] Serialization errors:\n{}",
                self.logger.name(),
                report
            );
            self.logger.warn(&msg);
        }
    }
}

impl Drop for ScopedCollector {
    // JAVA: net.minecraft.util.ProblemReporter$ScopedCollector#close
    fn drop(&mut self) {
        if !self.closed {
            self.close();
        }
    }
}

impl ProblemReporter for ScopedCollector {
    // JAVA: net.minecraft.util.ProblemReporter$Collector#forChild
    fn for_child(&self, element: Box<dyn PathElement>) -> Box<dyn ProblemReporter> {
        ProblemReporter::for_child(&self.collector, element)
    }

    // JAVA: net.minecraft.util.ProblemReporter$Collector#report
    fn report(&self, problem: Box<dyn Problem>) {
        ProblemReporter::report(&self.collector, problem);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn path_elements_and_integer_edges() {
        assert_eq!(FieldPathElement { name: "foo".into() }.get(), ".foo");
        assert_eq!(RootFieldPathElement { name: "root".into() }.get(), "root");
        assert_eq!(IndexedPathElement { index: 0 }.get(), "[0]");
        assert_eq!(IndexedPathElement { index: -1 }.get(), "[-1]");
        assert_eq!(
            IndexedPathElement { index: i32::MIN }.get(),
            "[-2147483648]"
        );
        assert_eq!(
            IndexedPathElement { index: i32::MAX }.get(),
            "[2147483647]"
        );
        assert_eq!(
            IndexedFieldPathElement {
                name: "arr".into(),
                index: -5,
            }
            .get(),
            ".arr[-5]"
        );
        let key = ResourceKey::new("stone", "minecraft:block");
        assert_eq!(
            ElementReferencePathElement { id: key.clone() }.get(),
            "->{stone@minecraft:block}"
        );
        assert_eq!(
            RootElementPathElement { id: key }.get(),
            "{stone@minecraft:block}"
        );
    }

    #[test]
    fn discarding_and_collector_for_each() {
        let d: Box<dyn ProblemReporter> = Box::new(DiscardingReporter);
        d.report(Box::new("ignored".to_string()));
        let c = d.for_child(Box::new(FieldPathElement { name: "x".into() }));
        c.report(Box::new("still ignored".to_string()));

        let root = Collector::new();
        assert!(root.is_empty());
        root.report(Box::new("root problem".to_string()));
        let child = root.for_child(Box::new(FieldPathElement { name: "items".into() }));
        child.report(Box::new("bad item".to_string()));
        let indexed = child.for_child(Box::new(IndexedPathElement { index: -1 }));
        indexed.report(Box::new("neg".to_string()));

        let mut seen = Vec::new();
        root.for_each(|path, p| seen.push((path, p.description())));
        assert_eq!(
            seen,
            vec![
                ("".to_string(), "root problem".to_string()),
                (".items".to_string(), "bad item".to_string()),
                (".items[-1]".to_string(), "neg".to_string()),
            ]
        );
    }

    #[test]
    fn get_report_groups_and_preserves_float_bits() {
        let root = Collector::new();
        let child = root.for_child(Box::new(FieldPathElement { name: "v".into() }));
        let inf_bits = f64::INFINITY.to_bits();
        let neg_zero_bits = (-0.0f64).to_bits();
        child.report(Box::new(format!("inf={inf_bits}")));
        child.report(Box::new(format!("neg_zero={neg_zero_bits}")));

        let report = root.get_report();
        assert_eq!(
            report,
            format!(" at .v: inf={inf_bits}; neg_zero={neg_zero_bits}")
        );
    }

    #[test]
    fn tree_report_collapses_and_nests() {
        let root = Collector::new();
        let a = root.for_child(Box::new(FieldPathElement { name: "a".into() }));
        let b = a.for_child(Box::new(FieldPathElement { name: "b".into() }));
        b.report(Box::new("p1".to_string()));
        assert_eq!(root.get_tree_report(), ".a.b: p1");

        let root2 = Collector::new();
        let a2 = root2.for_child(Box::new(FieldPathElement { name: "a".into() }));
        a2.report(Box::new("p1".to_string()));
        let c0 = a2.for_child(Box::new(IndexedPathElement { index: 0 }));
        c0.report(Box::new("p2".to_string()));
        let c1 = a2.for_child(Box::new(IndexedPathElement { index: 1 }));
        c1.report(Box::new("p3".to_string()));
        let expected = ".a:\n  [0]: p2\n  [1]: p3\n  p1";
        assert_eq!(root2.get_tree_report(), expected);
    }

    #[test]
    fn scoped_collector_logs_once_and_dedups() {
        struct TestLogger {
            name: String,
            warnings: Rc<RefCell<Vec<String>>>,
        }
        impl ProblemLogger for TestLogger {
            fn name(&self) -> &str {
                &self.name
            }
            fn warn(&self, msg: &str) {
                self.warnings.borrow_mut().push(msg.to_string());
            }
        }

        let warnings = Rc::new(RefCell::new(Vec::new()));
        {
            let logger = TestLogger {
                name: "Test".to_string(),
                warnings: warnings.clone(),
            };
            let mut sc = ScopedCollector::new(Box::new(logger));
            sc.report(Box::new("dup".to_string()));
            sc.report(Box::new("dup".to_string()));
            sc.close();
            sc.close();
        }
        assert_eq!(warnings.borrow().len(), 1);
        let msg = &warnings.borrow()[0];
        assert!(msg.contains("[Test] Serialization errors:"));
        assert!(msg.contains("dup"));

        let warnings2 = Rc::new(RefCell::new(Vec::new()));
        {
            let logger = TestLogger {
                name: "Drop".to_string(),
                warnings: warnings2.clone(),
            };
            let sc = ScopedCollector::new(Box::new(logger));
            sc.report(Box::new("dropped".to_string()));
        }
        assert_eq!(warnings2.borrow().len(), 1);
        assert!(warnings2.borrow()[0].contains("dropped"));

        let warnings3 = Rc::new(RefCell::new(Vec::new()));
        {
            let logger = TestLogger {
                name: "Empty".to_string(),
                warnings: warnings3.clone(),
            };
            let _sc = ScopedCollector::new(Box::new(logger));
        }
        assert!(warnings3.borrow().is_empty());
    }
}
