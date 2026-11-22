#[derive(Debug, Clone)]
pub struct TickThrottler {
    increment_step: i32,
    threshold: i32,
    count: i32,
}

impl TickThrottler {
    // JAVA: net.minecraft.util.TickThrottler#<init>
    pub fn new(increment_step: i32, threshold: i32) -> Self {
        Self {
            increment_step,
            threshold,
            count: 0,
        }
    }

    // JAVA: net.minecraft.util.TickThrottler#increment
    pub fn increment(&mut self) {
        self.count = self.count.wrapping_add(self.increment_step);
    }

    // JAVA: net.minecraft.util.TickThrottler#tick
    pub fn tick(&mut self) {
        if self.count > 0 {
            self.count = self.count.wrapping_sub(1);
        }
    }

    // JAVA: net.minecraft.util.TickThrottler#isUnderThreshold
    pub fn is_under_threshold(&self) -> bool {
        self.count < self.threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_threshold_behavior() {
        let mut t = TickThrottler::new(5, 10);
        assert!(t.is_under_threshold());
        t.increment(); // 5
        assert!(t.is_under_threshold());
        t.increment(); // 10
        assert!(!t.is_under_threshold());
        t.increment(); // 15
        assert!(!t.is_under_threshold());
    }

    #[test]
    fn tick_decrements_only_when_positive() {
        let mut t = TickThrottler::new(2, 10);
        t.tick(); // count 0, no change
        assert_eq!(t.is_under_threshold(), true);
        t.increment(); // 2
        t.increment(); // 4
        t.tick(); // 3
        t.tick(); // 2
        t.tick(); // 1
        t.tick(); // 0
        t.tick(); // still 0
        assert!(t.is_under_threshold());
    }

    #[test]
    fn increment_wraps_on_overflow() {
        let mut t = TickThrottler::new(i32::MAX, 0);
        t.increment(); // i32::MAX
        assert!(!t.is_under_threshold());
        t.increment(); // i32::MAX.wrapping_add(i32::MAX) == -2
        assert!(t.is_under_threshold());
    }

    #[test]
    fn negative_increment_and_threshold() {
        let mut t = TickThrottler::new(-5, 0);
        t.increment(); // -5
        assert!(t.is_under_threshold());
        t.tick(); // negative, should not decrement
        assert!(t.is_under_threshold());
        let mut t2 = TickThrottler::new(1, -1);
        assert!(!t2.is_under_threshold()); // 0 < -1 is false
    }
}
