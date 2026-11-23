pub trait SampleLogger {
    fn log_full_sample(&mut self, values: &[i64]);
    fn log_sample(&mut self, value: i64);
    fn log_partial_sample(&mut self, value: i64, index: i32);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteDebugSampleType {
    pub subscription_id: i32,
}

impl RemoteDebugSampleType {
    pub fn subscription(&self) -> i32 {
        self.subscription_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundDebugSamplePacket {
    pub sample: Vec<i64>,
    pub sample_type: RemoteDebugSampleType,
}

impl ClientboundDebugSamplePacket {
    pub fn new(sample: Vec<i64>, sample_type: RemoteDebugSampleType) -> Self {
        Self { sample, sample_type }
    }
}

pub trait ServerDebugSubscribers {
    fn has_any_subscriber_for(&self, subscription: i32) -> bool;
    fn broadcast_to_all(&self, subscription: i32, packet: ClientboundDebugSamplePacket);
}

pub struct RemoteSampleLogger<S: ServerDebugSubscribers> {
    pub defaults: Vec<i64>,
    pub sample: Vec<i64>,
    pub subscribers: S,
    pub sample_type: RemoteDebugSampleType,
}

impl<S: ServerDebugSubscribers> RemoteSampleLogger<S> {
    // JAVA: net.minecraft.util.debugchart.RemoteSampleLogger#<init>
    pub fn new(size: usize, subscribers: S, sample_type: RemoteDebugSampleType) -> Self {
        Self::with_defaults(size, subscribers, sample_type, vec![0i64; size])
    }

    // JAVA: net.minecraft.util.debugchart.RemoteSampleLogger#<init>
    pub fn with_defaults(
        size: usize,
        subscribers: S,
        sample_type: RemoteDebugSampleType,
        defaults: Vec<i64>,
    ) -> Self {
        if defaults.len() != size {
            panic!("defaults have incorrect length of {}", defaults.len());
        }
        Self {
            sample: vec![0i64; size],
            defaults,
            subscribers,
            sample_type,
        }
    }

    // JAVA: net.minecraft.util.debugchart.RemoteSampleLogger#useSample
    fn use_sample(&mut self) {
        let subscription = self.sample_type.subscription();
        if self.subscribers.has_any_subscriber_for(subscription) {
            self.subscribers.broadcast_to_all(
                subscription,
                ClientboundDebugSamplePacket::new(self.sample.clone(), self.sample_type.clone()),
            );
        }
    }

    // JAVA: net.minecraft.util.debugchart.AbstractSampleLogger#resetSample
    fn reset_sample(&mut self) {
        self.sample.copy_from_slice(&self.defaults);
    }

    // JAVA: net.minecraft.util.debugchart.AbstractSampleLogger#logFullSample
    pub fn log_full_sample(&mut self, values: &[i64]) {
        self.sample[..values.len()].copy_from_slice(values);
        self.use_sample();
        self.reset_sample();
    }

    // JAVA: net.minecraft.util.debugchart.AbstractSampleLogger#logSample
    pub fn log_sample(&mut self, value: i64) {
        self.sample[0] = value;
        self.use_sample();
        self.reset_sample();
    }

    // JAVA: net.minecraft.util.debugchart.AbstractSampleLogger#logPartialSample
    pub fn log_partial_sample(&mut self, value: i64, index: i32) {
        let idx = index as usize;
        if index >= 1 && idx < self.sample.len() {
            self.sample[idx] = value;
        } else {
            panic!(
                "{} out of bounds for dimensions {}",
                index,
                self.sample.len()
            );
        }
    }
}

impl<S: ServerDebugSubscribers> SampleLogger for RemoteSampleLogger<S> {
    fn log_full_sample(&mut self, values: &[i64]) {
        Self::log_full_sample(self, values);
    }

    fn log_sample(&mut self, value: i64) {
        Self::log_sample(self, value);
    }

    fn log_partial_sample(&mut self, value: i64, index: i32) {
        Self::log_partial_sample(self, value, index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockSubscribers {
        has: bool,
        broadcasts: RefCell<Vec<(i32, ClientboundDebugSamplePacket)>>,
    }

    impl MockSubscribers {
        fn new(has: bool) -> Self {
            Self {
                has,
                broadcasts: RefCell::new(Vec::new()),
            }
        }
    }

    impl ServerDebugSubscribers for MockSubscribers {
        fn has_any_subscriber_for(&self, _subscription: i32) -> bool {
            self.has
        }

        fn broadcast_to_all(&self, subscription: i32, packet: ClientboundDebugSamplePacket) {
            self.broadcasts.borrow_mut().push((subscription, packet));
        }
    }

    #[test]
    fn broadcasts_when_subscribed_and_resets() {
        let subs = MockSubscribers::new(true);
        let st = RemoteDebugSampleType { subscription_id: 7 };
        let mut logger = RemoteSampleLogger::new(2, subs, st.clone());
        logger.log_sample(123);
        let b = logger.subscribers.broadcasts.borrow();
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].0, 7);
        assert_eq!(b[0].1.sample, vec![123i64, 0]);
        assert_eq!(b[0].1.sample_type, st);
        drop(b);
        assert_eq!(logger.sample, vec![0i64, 0]);
    }

    #[test]
    fn no_broadcast_when_not_subscribed() {
        let subs = MockSubscribers::new(false);
        let st = RemoteDebugSampleType { subscription_id: 1 };
        let mut logger = RemoteSampleLogger::new(1, subs, st);
        logger.log_sample(5);
        assert!(logger.subscribers.broadcasts.borrow().is_empty());
        assert_eq!(logger.sample, vec![0i64]);
    }

    #[test]
    fn with_defaults_handles_negative_and_extreme_values() {
        let subs = MockSubscribers::new(true);
        let st = RemoteDebugSampleType { subscription_id: 2 };
        let mut logger =
            RemoteSampleLogger::with_defaults(2, subs, st, vec![-1i64, i64::MIN]);
        logger.log_full_sample(&[i64::MIN, i64::MAX]);
        let b = logger.subscribers.broadcasts.borrow();
        assert_eq!(b[0].1.sample, vec![i64::MIN, i64::MAX]);
        drop(b);
        assert_eq!(logger.sample, vec![-1i64, i64::MIN]);
    }

    #[test]
    fn log_partial_sample_bounds() {
        let subs = MockSubscribers::new(false);
        let st = RemoteDebugSampleType { subscription_id: 0 };
        let mut logger = RemoteSampleLogger::new(3, subs, st);
        logger.log_partial_sample(-9, 2);
        assert_eq!(logger.sample[2], -9);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            logger.log_partial_sample(1, 0);
        }))
        .is_err());
    }
}
