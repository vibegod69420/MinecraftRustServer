use std::sync::Arc;

/// Port of `net.minecraft.server.TickTask`.
///
/// A task scheduled to run at a specific server tick.
pub struct TickTask {
    tick: i32,
    runnable: Box<dyn FnMut() + Send>,
}

impl TickTask {
    // JAVA: net.minecraft.server.TickTask#<init>
    pub fn new<F>(tick: i32, runnable: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        Self {
            tick,
            runnable: Box::new(runnable),
        }
    }

    // JAVA: net.minecraft.server.TickTask#getTick
    pub fn get_tick(&self) -> i32 {
        self.tick
    }

    // JAVA: net.minecraft.server.TickTask#run
    pub fn run(&mut self) {
        (self.runnable)();
    }
}

#[cfg(test)]
mod tests {
    use super::TickTask;
    use std::sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    };

    #[test]
    fn new_and_get_tick() {
        let task = TickTask::new(42, || {});
        assert_eq!(task.get_tick(), 42);
    }

    #[test]
    fn negative_and_extreme_ticks() {
        let neg_task = TickTask::new(-100, || {});
        assert_eq!(neg_task.get_tick(), -100);

        let min_task = TickTask::new(i32::MIN, || {});
        assert_eq!(min_task.get_tick(), i32::MIN);

        let max_task = TickTask::new(i32::MAX, || {});
        assert_eq!(max_task.get_tick(), i32::MAX);

        // Overflow boundary: wrapping_add on i32::MAX yields i32::MIN
        let wrapped = i32::MAX.wrapping_add(1);
        let wrapped_task = TickTask::new(wrapped, || {});
        assert_eq!(wrapped_task.get_tick(), i32::MIN);
    }

    #[test]
    fn tick_from_float_bits() {
        let nan_bits = f32::NAN.to_bits() as i32;
        let nan_task = TickTask::new(nan_bits, || {});
        assert_eq!(nan_task.get_tick(), nan_bits);

        let neg_inf_bits = f32::NEG_INFINITY.to_bits() as i32;
        let neg_inf_task = TickTask::new(neg_inf_bits, || {});
        assert_eq!(neg_inf_task.get_tick(), neg_inf_bits);

        let neg_zero_bits = (-0.0f32).to_bits() as i32;
        let neg_zero_task = TickTask::new(neg_zero_bits, || {});
        assert_eq!(neg_zero_task.get_tick(), neg_zero_bits);
    }

    #[test]
    fn run_executes_runnable() {
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);
        let mut task = TickTask::new(7, move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        task.run();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert_eq!(task.get_tick(), 7);
    }

    #[test]
    fn run_can_be_called_multiple_times() {
        let counter = Arc::new(AtomicI32::new(i32::MAX - 1));
        let counter_clone = Arc::clone(&counter);
        let mut task = TickTask::new(0, move || {
            let prev = counter_clone.load(Ordering::SeqCst);
            counter_clone.store(prev.wrapping_add(1), Ordering::SeqCst);
        });

        task.run();
        task.run();
        task.run();

        // i32::MAX - 1 + 3 wraps to i32::MIN + 1
        assert_eq!(counter.load(Ordering::SeqCst), i32::MIN + 1);
    }
}
