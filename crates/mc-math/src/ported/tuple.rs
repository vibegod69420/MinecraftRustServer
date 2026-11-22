#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tuple<A, B> {
    a: A,
    b: B,
}

impl<A, B> Tuple<A, B> {
    // JAVA: net.minecraft.util.Tuple#<init>
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }

    // JAVA: net.minecraft.util.Tuple#getA
    pub fn get_a(&self) -> &A {
        &self.a
    }

    // JAVA: net.minecraft.util.Tuple#setA
    pub fn set_a(&mut self, a: A) {
        self.a = a;
    }

    // JAVA: net.minecraft.util.Tuple#getB
    pub fn get_b(&self) -> &B {
        &self.b
    }

    // JAVA: net.minecraft.util.Tuple#setB
    pub fn set_b(&mut self, b: B) {
        self.b = b;
    }
}

#[cfg(test)]
mod tests {
    use super::Tuple;

    #[test]
    fn get_and_set() {
        let mut t = Tuple::new(10i32, "hello");
        assert_eq!(*t.get_a(), 10);
        assert_eq!(*t.get_b(), "hello");
        t.set_a(20);
        t.set_b("world");
        assert_eq!(*t.get_a(), 20);
        assert_eq!(*t.get_b(), "world");
    }

    #[test]
    fn different_types_and_negative() {
        let t = Tuple::new(-1i64, -2.5f64);
        assert_eq!(*t.get_a(), -1i64);
        assert_eq!((*t.get_b()).to_bits(), (-2.5f64).to_bits());
    }

    #[test]
    fn overwrite_with_extreme_values() {
        let mut t = Tuple::new(i32::MAX, i32::MIN);
        t.set_a(i32::MIN);
        t.set_b(i32::MAX);
        assert_eq!(*t.get_a(), i32::MIN);
        assert_eq!(*t.get_b(), i32::MAX);
    }
}
