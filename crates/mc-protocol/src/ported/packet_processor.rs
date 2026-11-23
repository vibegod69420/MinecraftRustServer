use std::collections::VecDeque;
use std::fmt;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

/// JAVA: net.minecraft.network.PacketListener
pub trait PacketListener: Send {
    /// JAVA: net.minecraft.network.PacketListener#shouldHandleMessage(Packet)
    fn should_handle_message(&self, packet: &dyn Packet<Self>) -> bool
    where
        Self: Sized;

    /// JAVA: net.minecraft.network.PacketListener#onPacketError(Packet, Exception)
    fn on_packet_error(&mut self, packet: &dyn Packet<Self>, error: PacketProcessError)
    where
        Self: Sized;
}

/// JAVA: net.minecraft.network.protocol.Packet
pub trait Packet<T: PacketListener + ?Sized>: Send + fmt::Debug {
    /// JAVA: net.minecraft.network.protocol.Packet#handle(PacketListener)
    fn handle(&self, listener: &mut T) -> Result<(), PacketProcessError>;

    fn type_name(&self) -> &'static str {
        "unknown"
    }
}

/// Error type used to model Java exceptions in packet processing.
#[derive(Debug)]
pub enum PacketProcessError {
    Reported(ReportedException),
    Other(String),
}

impl fmt::Display for PacketProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketProcessError::Reported(e) => write!(f, "ReportedException: {}", e.message),
            PacketProcessError::Other(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for PacketProcessError {}

/// JAVA: net.minecraft.ReportedException
#[derive(Debug)]
pub struct ReportedException {
    pub message: String,
    pub cause_is_out_of_memory: bool,
}

impl ReportedException {
    pub fn new(message: impl Into<String>, cause_is_out_of_memory: bool) -> Self {
        Self {
            message: message.into(),
            cause_is_out_of_memory,
        }
    }

    /// JAVA: net.minecraft.ReportedException#getCause
    pub fn cause_is_out_of_memory(&self) -> bool {
        self.cause_is_out_of_memory
    }
}

/// Equivalent of `PacketUtils.makeReportedException(...)`.
/// JAVA: net.minecraft.network.protocol.PacketUtils#makeReportedException
pub fn make_reported_exception(
    error: PacketProcessError,
    _packet_type_name: &str,
    _listener_name: &str,
) -> PacketProcessError {
    match error {
        PacketProcessError::Reported(_) => error,
        PacketProcessError::Other(msg) => PacketProcessError::Reported(ReportedException::new(
            format!("Packet processing error: {msg}"),
            false,
        )),
    }
}

/// A queued packet operation. This is type-erased so the processor can hold
/// heterogeneous listener/packet pairs like Java's generic record.
/// JAVA: net.minecraft.network.PacketProcessor.ListenerAndPacket
trait QueuedPacketOp: Send {
    fn handle(&mut self, logger: &dyn PacketProcessorLogger);
    fn debug_name(&self) -> &'static str;
}

/// JAVA: net.minecraft.network.PacketProcessor.ListenerAndPacket
struct ListenerAndPacket<T: PacketListener + 'static> {
    listener: Arc<Mutex<T>>,
    packet: Box<dyn Packet<T>>,
}

impl<T: PacketListener + 'static> ListenerAndPacket<T> {
    fn new(listener: Arc<Mutex<T>>, packet: Box<dyn Packet<T>>) -> Self {
        Self { listener, packet }
    }
}

impl<T: PacketListener + 'static> QueuedPacketOp for ListenerAndPacket<T> {
    /// JAVA: net.minecraft.network.PacketProcessor.ListenerAndPacket#handle
    fn handle(&mut self, logger: &dyn PacketProcessorLogger) {
        let mut listener = match self.listener.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        if listener.should_handle_message(self.packet.as_ref()) {
            let result = self.packet.handle(&mut *listener);
            if let Err(err) = result {
                let is_oom_reported = matches!(
                    &err,
                    PacketProcessError::Reported(re) if re.cause_is_out_of_memory()
                );
                if is_oom_reported {
                    // Java: throw PacketUtils.makeReportedException($$0, this.packet, this.listener);
                    // In Rust we cannot unwind through this trait object cleanly with Java's exact type;
                    // preserve the semantic by panicking with the reported exception payload.
                    let reported = make_reported_exception(
                        err,
                        self.packet.type_name(),
                        std::any::type_name::<T>(),
                    );
                    panic!("{reported}");
                }
                listener.on_packet_error(self.packet.as_ref(), err);
            }
        } else {
            logger.debug_ignoring_packet(self.packet.type_name());
        }
    }

    fn debug_name(&self) -> &'static str {
        self.packet.type_name()
    }
}

/// Logger hook for `PacketProcessor`.
pub trait PacketProcessorLogger: Send + Sync {
    fn debug_ignoring_packet(&self, packet_debug: &str);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopPacketProcessorLogger;

impl PacketProcessorLogger for NoopPacketProcessorLogger {
    fn debug_ignoring_packet(&self, _packet_debug: &str) {}
}

/// JAVA: net.minecraft.network.PacketProcessor
pub struct PacketProcessor {
    packets_to_be_handled: Mutex<VecDeque<Box<dyn QueuedPacketOp>>>,
    running_thread: ThreadId,
    closed: Mutex<bool>,
    logger: Arc<dyn PacketProcessorLogger>,
}

impl fmt::Debug for PacketProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let closed = self.closed.lock().map(|g| *g).unwrap_or(true);
        let queued = self
            .packets_to_be_handled
            .lock()
            .map(|q| q.len())
            .unwrap_or(0);
        f.debug_struct("PacketProcessor")
            .field("running_thread", &self.running_thread)
            .field("closed", &closed)
            .field("queued", &queued)
            .finish()
    }
}

impl PacketProcessor {
    // JAVA: net.minecraft.network.PacketProcessor#<init>
    pub fn new(running_thread: ThreadId) -> Self {
        Self::with_logger(running_thread, Arc::new(NoopPacketProcessorLogger))
    }

    pub fn with_logger(running_thread: ThreadId, logger: Arc<dyn PacketProcessorLogger>) -> Self {
        Self {
            packets_to_be_handled: Mutex::new(VecDeque::new()),
            running_thread,
            closed: Mutex::new(false),
            logger,
        }
    }

    // JAVA: net.minecraft.network.PacketProcessor#isSameThread
    pub fn is_same_thread(&self) -> bool {
        std::thread::current().id() == self.running_thread
    }

    // JAVA: net.minecraft.network.PacketProcessor#scheduleIfPossible
    pub fn schedule_if_possible<T: PacketListener + 'static>(
        &self,
        listener: Arc<Mutex<T>>,
        packet: Box<dyn Packet<T>>,
    ) -> Result<(), io::Error> {
        let closed = self.is_closed();
        if closed {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Server already shutting down",
            ));
        }
        let mut queue = match self.packets_to_be_handled.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        // Recheck closed under queue lock to reduce shutdown race window.
        if self.is_closed() {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Server already shutting down",
            ));
        }
        queue.push_back(Box::new(ListenerAndPacket::new(listener, packet)));
        Ok(())
    }

    // JAVA: net.minecraft.network.PacketProcessor#processQueuedPackets
    pub fn process_queued_packets(&self) {
        if self.is_closed() {
            return;
        }

        loop {
            let mut op = {
                let mut queue = match self.packets_to_be_handled.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                queue.pop_front()
            };

            let Some(ref mut packet_op) = op else {
                break;
            };
            packet_op.handle(self.logger.as_ref());

            if self.is_closed() {
                break;
            }
        }
    }

    // JAVA: net.minecraft.network.PacketProcessor#close
    pub fn close(&self) {
        let mut closed = match self.closed.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        *closed = true;
    }

    pub fn is_closed(&self) -> bool {
        match self.closed.lock() {
            Ok(g) => *g,
            Err(poisoned) => *poisoned.into_inner(),
        }
    }

    pub fn queued_len(&self) -> usize {
        match self.packets_to_be_handled.lock() {
            Ok(g) => g.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }
}

impl Drop for PacketProcessor {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestListener {
        should_handle: bool,
        handled: AtomicUsize,
        errors: AtomicUsize,
        last_error_oom_reported: Mutex<Option<bool>>,
    }

    impl TestListener {
        fn new(should_handle: bool) -> Self {
            Self {
                should_handle,
                handled: AtomicUsize::new(0),
                errors: AtomicUsize::new(0),
                last_error_oom_reported: Mutex::new(None),
            }
        }
    }

    impl PacketListener for TestListener {
        fn should_handle_message(&self, _packet: &dyn Packet<Self>) -> bool
        where
            Self: Sized,
        {
            self.should_handle
        }

        fn on_packet_error(&mut self, _packet: &dyn Packet<Self>, error: PacketProcessError)
        where
            Self: Sized,
        {
            self.errors.fetch_add(1, Ordering::SeqCst);
            let is_oom = matches!(error, PacketProcessError::Reported(ref r) if r.cause_is_out_of_memory());
            *self.last_error_oom_reported.lock().unwrap() = Some(is_oom);
        }
    }

    #[derive(Debug)]
    struct OkPacket;
    impl Packet<TestListener> for OkPacket {
        fn handle(&self, listener: &mut TestListener) -> Result<(), PacketProcessError> {
            listener.handled.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn type_name(&self) -> &'static str {
            "OkPacket"
        }
    }

    #[derive(Debug)]
    struct ErrPacket;
    impl Packet<TestListener> for ErrPacket {
        fn handle(&self, _listener: &mut TestListener) -> Result<(), PacketProcessError> {
            Err(PacketProcessError::Other("boom".to_string()))
        }
        fn type_name(&self) -> &'static str {
            "ErrPacket"
        }
    }

    #[derive(Debug)]
    struct OomPacket;
    impl Packet<TestListener> for OomPacket {
        fn handle(&self, _listener: &mut TestListener) -> Result<(), PacketProcessError> {
            Err(PacketProcessError::Reported(ReportedException::new(
                "oom",
                true,
            )))
        }
        fn type_name(&self) -> &'static str {
            "OomPacket"
        }
    }

    #[derive(Default)]
    struct RecordingLogger {
        ignored: Mutex<Vec<String>>,
    }
    impl PacketProcessorLogger for RecordingLogger {
        fn debug_ignoring_packet(&self, packet_debug: &str) {
            self.ignored.lock().unwrap().push(packet_debug.to_string());
        }
    }

    #[test]
    fn is_same_thread_matches_current() {
        let proc = PacketProcessor::new(std::thread::current().id());
        assert!(proc.is_same_thread());
    }

    #[test]
    fn schedule_and_process_handles_packet() {
        let proc = PacketProcessor::new(std::thread::current().id());
        let listener = Arc::new(Mutex::new(TestListener::new(true)));
        proc.schedule_if_possible(listener.clone(), Box::new(OkPacket))
            .expect("schedule");
        assert_eq!(proc.queued_len(), 1);
        proc.process_queued_packets();
        assert_eq!(proc.queued_len(), 0);
        assert_eq!(listener.lock().unwrap().handled.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn schedule_after_close_returns_error() {
        let proc = PacketProcessor::new(std::thread::current().id());
        proc.close();
        let listener = Arc::new(Mutex::new(TestListener::new(true)));
        let err = proc
            .schedule_if_possible(listener, Box::new(OkPacket))
            .expect_err("should reject after close");
        assert_eq!(err.kind(), io::ErrorKind::WouldBlock);
    }

    #[test]
    fn disconnected_listener_ignores_packet_and_logs() {
        let logger = Arc::new(RecordingLogger::default());
        let proc = PacketProcessor::with_logger(std::thread::current().id(), logger.clone());
        let listener = Arc::new(Mutex::new(TestListener::new(false)));
        proc.schedule_if_possible(listener.clone(), Box::new(OkPacket))
            .expect("schedule");
        proc.process_queued_packets();
        assert_eq!(listener.lock().unwrap().handled.load(Ordering::SeqCst), 0);
        let ignored = logger.ignored.lock().unwrap();
        assert_eq!(ignored.len(), 1);
        assert_eq!(ignored[0], "OkPacket");
    }

    #[test]
    fn handler_error_calls_on_packet_error() {
        let proc = PacketProcessor::new(std::thread::current().id());
        let listener = Arc::new(Mutex::new(TestListener::new(true)));
        proc.schedule_if_possible(listener.clone(), Box::new(ErrPacket))
            .expect("schedule");
        proc.process_queued_packets();
        let l = listener.lock().unwrap();
        assert_eq!(l.errors.load(Ordering::SeqCst), 1);
        assert_eq!(*l.last_error_oom_reported.lock().unwrap(), Some(false));
    }

    #[test]
    fn oom_reported_exception_panics_like_java_throw() {
        let proc = PacketProcessor::new(std::thread::current().id());
        let listener = Arc::new(Mutex::new(TestListener::new(true)));
        proc.schedule_if_possible(listener, Box::new(OomPacket))
            .expect("schedule");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            proc.process_queued_packets();
        }));
        assert!(result.is_err(), "OOM ReportedException should propagate as panic");
    }
}
