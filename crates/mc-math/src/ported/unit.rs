#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unit {
    Instance,
}

impl Unit {
    // JAVA: net.minecraft.util.Unit#valueOf-like singleton access
    pub fn instance() -> Self {
        Unit::Instance
    }
}

#[cfg(test)]
mod tests {
    use super::Unit;

    #[test]
    fn singleton_identity() {
        let a = Unit::Instance;
        let b = Unit::instance();
        assert_eq!(a, b);
    }

    #[test]
    fn copy_and_clone() {
        let a = Unit::Instance;
        let b = a;
        let c = a;
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_format() {
        assert_eq!(format!("{:?}", Unit::Instance), "Instance");
    }
}
