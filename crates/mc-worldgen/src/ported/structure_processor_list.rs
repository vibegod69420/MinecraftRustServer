use std::fmt;

/// Minimal self-contained placeholder for unported StructureProcessor.
pub trait StructureProcessor: fmt::Debug {}

#[derive(Debug)]
pub struct StructureProcessorList {
    list: Vec<Box<dyn StructureProcessor>>,
}

impl StructureProcessorList {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorList#<init>
    pub fn new(list: Vec<Box<dyn StructureProcessor>>) -> Self {
        Self { list }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorList#list
    pub fn list(&self) -> &[Box<dyn StructureProcessor>] {
        &self.list
    }
}

impl fmt::Display for StructureProcessorList {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorList#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProcessorList[{:?}]", self.list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyProcessor(i32);
    impl StructureProcessor for DummyProcessor {}

    #[test]
    fn new_and_list_len() {
        let list = StructureProcessorList::new(vec![
            Box::new(DummyProcessor(1)),
            Box::new(DummyProcessor(-2)),
        ]);
        assert_eq!(list.list().len(), 2);
    }

    #[test]
    fn to_string_empty() {
        let list = StructureProcessorList::new(vec![]);
        assert_eq!(list.to_string(), "ProcessorList[[]]");
    }

    #[test]
    fn to_string_with_negative_and_min() {
        let list = StructureProcessorList::new(vec![
            Box::new(DummyProcessor(0)),
            Box::new(DummyProcessor(i32::MIN)),
            Box::new(DummyProcessor(-1)),
        ]);
        let s = list.to_string();
        assert!(s.starts_with("ProcessorList[["));
        assert!(s.ends_with("]]"));
        assert!(s.contains("DummyProcessor(0)"));
        assert!(s.contains(&format!("DummyProcessor({})", i32::MIN)));
        assert!(s.contains("DummyProcessor(-1)"));
    }

    #[test]
    fn list_preserves_order() {
        let list = StructureProcessorList::new(vec![
            Box::new(DummyProcessor(10)),
            Box::new(DummyProcessor(20)),
            Box::new(DummyProcessor(i32::MAX)),
        ]);
        let s = format!("{:?}", list.list());
        let i10 = s.find("10").unwrap();
        let i20 = s.find("20").unwrap();
        let imax = s.find(&i32::MAX.to_string()).unwrap();
        assert!(i10 < i20);
        assert!(i20 < imax);
    }
}
