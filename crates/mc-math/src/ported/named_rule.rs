//! Faithful port of `net.minecraft.util.parsing.packrat.NamedRule`.

/// Minimal placeholder for `net.minecraft.util.parsing.packrat.Atom`.
///
/// Kept local so this module is self-contained.
pub trait Atom<T> {}

/// Minimal placeholder for `net.minecraft.util.parsing.packrat.Rule`.
///
/// Kept local so this module is self-contained.
pub trait Rule<S, T> {}

/// Port of `net.minecraft.util.parsing.packrat.NamedRule`.
pub trait NamedRule<S, T> {
    type Name: Atom<T>;
    type Value: Rule<S, T>;

    // JAVA: net.minecraft.util.parsing.packrat.NamedRule#name
    fn name(&self) -> &Self::Name;

    // JAVA: net.minecraft.util.parsing.packrat.NamedRule#value
    fn value(&self) -> &Self::Value;
}

#[cfg(test)]
mod tests {
    use super::{Atom, NamedRule, Rule};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestAtom(&'static str);
    impl Atom<i32> for TestAtom {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestRule(u32);
    impl Rule<(), i32> for TestRule {}

    struct TestNamedRule {
        name: TestAtom,
        value: TestRule,
    }

    impl NamedRule<(), i32> for TestNamedRule {
        type Name = TestAtom;
        type Value = TestRule;

        // JAVA: net.minecraft.util.parsing.packrat.NamedRule#name
        fn name(&self) -> &Self::Name {
            &self.name
        }

        // JAVA: net.minecraft.util.parsing.packrat.NamedRule#value
        fn value(&self) -> &Self::Value {
            &self.value
        }
    }

    #[test]
    fn name_and_value_are_accessible() {
        let rule = TestNamedRule {
            name: TestAtom("demo"),
            value: TestRule(7),
        };
        assert_eq!(rule.name(), &TestAtom("demo"));
        assert_eq!(rule.value(), &TestRule(7));
    }

    #[test]
    fn works_with_negative_and_boundary_payloads() {
        let rule = TestNamedRule {
            name: TestAtom("edge"),
            value: TestRule(u32::MAX),
        };
        assert_eq!(rule.name().0, "edge");
        assert_eq!(rule.value().0, u32::MAX);
    }

    #[test]
    fn trait_object_like_usage_via_generics() {
        fn read_name<S, T, R: NamedRule<S, T>>(r: &R) -> &R::Name {
            r.name()
        }
        let rule = TestNamedRule {
            name: TestAtom("generic"),
            value: TestRule(0),
        };
        let _ = read_name(&rule);
        assert_eq!(rule.value().0, 0);
    }
}
