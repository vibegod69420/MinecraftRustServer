use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunningOnDifferentThreadException;

impl RunningOnDifferentThreadException {
    pub const RUNNING_ON_DIFFERENT_THREAD: Self = Self;
}

impl fmt::Display for RunningOnDifferentThreadException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Running on different thread")
    }
}

impl Error for RunningOnDifferentThreadException {}

#[derive(Debug, Clone, Default)]
pub struct CrashReportCategory {
    pub name: String,
    pub details: Vec<(String, String)>,
}

impl CrashReportCategory {
    pub fn set_detail(&mut self, key: &str, value: impl FnOnce() -> String) {
        self.details.push((key.to_string(), value()));
    }
}

#[derive(Debug, Clone)]
pub struct CrashReport {
    pub title: String,
    pub message: String,
    pub categories: Vec<CrashReportCategory>,
}

impl CrashReport {
    pub fn for_throwable(message: &str, title: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            categories: Vec::new(),
        }
    }

    pub fn add_category(&mut self, name: &str) -> &mut CrashReportCategory {
        self.categories.push(CrashReportCategory {
            name: name.to_string(),
            details: Vec::new(),
        });
        self.categories.last_mut().expect("category just pushed")
    }
}

#[derive(Debug, Clone)]
pub struct ReportedException {
    pub report: CrashReport,
}

impl ReportedException {
    pub fn new(report: CrashReport) -> Self {
        Self { report }
    }

    pub fn get_report(&self) -> &CrashReport {
        &self.report
    }

    pub fn get_report_mut(&mut self) -> &mut CrashReport {
        &mut self.report
    }
}

impl fmt::Display for ReportedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.report.title, self.report.message)
    }
}

impl Error for ReportedException {}

#[derive(Debug)]
pub enum HandledPacketException {
    Reported(ReportedException),
    Other(String),
}

pub trait PacketInfo {
    fn type_name(&self) -> String;
    fn is_terminal(&self) -> bool;
    fn is_skippable(&self) -> bool;
}

pub trait PacketListenerInfo {
    fn fill_crash_report(&self, report: &mut CrashReport);
}

pub trait PacketProcessor<L: ?Sized, P: ?Sized> {
    fn is_same_thread(&self) -> bool;
    fn schedule_if_possible(&self, listener: &L, packet: &P);
}

pub trait MinecraftServer<L: ?Sized, P: ?Sized> {
    type Processor: PacketProcessor<L, P>;
    fn packet_processor(&self) -> &Self::Processor;
}

pub trait ServerLevel<L: ?Sized, P: ?Sized> {
    type Server: MinecraftServer<L, P>;
    fn get_server(&self) -> &Self::Server;
}

// JAVA: net.minecraft.network.protocol.PacketUtils#ensureRunningOnSameThread
pub fn ensure_running_on_same_thread<P: ?Sized, L: ?Sized, PP: PacketProcessor<L, P> + ?Sized>(
    packet: &P,
    listener: &L,
    processor: &PP,
) -> Result<(), RunningOnDifferentThreadException> {
    if !processor.is_same_thread() {
        processor.schedule_if_possible(listener, packet);
        return Err(RunningOnDifferentThreadException::RUNNING_ON_DIFFERENT_THREAD);
    }
    Ok(())
}

// JAVA: net.minecraft.network.protocol.PacketUtils#ensureRunningOnSameThread
pub fn ensure_running_on_same_thread_for_level<
    P: ?Sized,
    L: ?Sized,
    SL: ServerLevel<L, P> + ?Sized,
>(
    packet: &P,
    listener: &L,
    level: &SL,
) -> Result<(), RunningOnDifferentThreadException> {
    ensure_running_on_same_thread(packet, listener, level.get_server().packet_processor())
}

// JAVA: net.minecraft.network.protocol.PacketUtils#makeReportedException
pub fn make_reported_exception<P: PacketInfo + ?Sized, L: PacketListenerInfo + ?Sized>(
    error: HandledPacketException,
    packet: Option<&P>,
    listener: &L,
) -> ReportedException {
    match error {
        HandledPacketException::Reported(mut reported) => {
            fill_crash_report(&mut reported.report, listener, packet);
            reported
        }
        HandledPacketException::Other(msg) => {
            let mut report = CrashReport::for_throwable(&msg, "Main thread packet handler");
            fill_crash_report(&mut report, listener, packet);
            ReportedException::new(report)
        }
    }
}

// JAVA: net.minecraft.network.protocol.PacketUtils#fillCrashReport
pub fn fill_crash_report<P: PacketInfo + ?Sized, L: PacketListenerInfo + ?Sized>(
    report: &mut CrashReport,
    listener: &L,
    packet: Option<&P>,
) {
    if let Some(packet) = packet {
        let category = report.add_category("Incoming Packet");
        let type_name = packet.type_name();
        category.set_detail("Type", || type_name);
        let is_terminal = packet.is_terminal().to_string();
        category.set_detail("Is Terminal", || is_terminal);
        let is_skippable = packet.is_skippable().to_string();
        category.set_detail("Is Skippable", || is_skippable);
    }

    listener.fill_crash_report(report);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct TestPacket {
        type_name: String,
        terminal: bool,
        skippable: bool,
    }

    impl PacketInfo for TestPacket {
        fn type_name(&self) -> String {
            self.type_name.clone()
        }

        fn is_terminal(&self) -> bool {
            self.terminal
        }

        fn is_skippable(&self) -> bool {
            self.skippable
        }
    }

    struct TestListener {
        filled: RefCell<usize>,
    }

    impl TestListener {
        fn new() -> Self {
            Self {
                filled: RefCell::new(0),
            }
        }
    }

    impl PacketListenerInfo for TestListener {
        fn fill_crash_report(&self, report: &mut CrashReport) {
            *self.filled.borrow_mut() += 1;
            report
                .add_category("Listener")
                .set_detail("filled", || "true".to_string());
        }
    }

    struct TestProcessor {
        same_thread: bool,
        scheduled: RefCell<Vec<String>>,
    }

    impl TestProcessor {
        fn new(same_thread: bool) -> Self {
            Self {
                same_thread,
                scheduled: RefCell::new(Vec::new()),
            }
        }
    }

    impl PacketProcessor<TestListener, TestPacket> for TestProcessor {
        fn is_same_thread(&self) -> bool {
            self.same_thread
        }

        fn schedule_if_possible(&self, _listener: &TestListener, packet: &TestPacket) {
            self.scheduled.borrow_mut().push(packet.type_name.clone());
        }
    }

    struct TestServer {
        processor: TestProcessor,
    }

    impl MinecraftServer<TestListener, TestPacket> for TestServer {
        type Processor = TestProcessor;

        fn packet_processor(&self) -> &Self::Processor {
            &self.processor
        }
    }

    struct TestLevel {
        server: TestServer,
    }

    impl ServerLevel<TestListener, TestPacket> for TestLevel {
        type Server = TestServer;

        fn get_server(&self) -> &Self::Server {
            &self.server
        }
    }

    #[test]
    fn ensure_running_on_same_thread_ok_when_same_thread() {
        let packet = TestPacket {
            type_name: "p".into(),
            terminal: false,
            skippable: false,
        };
        let listener = TestListener::new();
        let processor = TestProcessor::new(true);
        let result = ensure_running_on_same_thread(&packet, &listener, &processor);
        assert!(result.is_ok());
        assert!(processor.scheduled.borrow().is_empty());
    }

    #[test]
    fn ensure_running_on_same_thread_schedules_and_errors_when_different_thread() {
        let packet = TestPacket {
            type_name: "p".into(),
            terminal: false,
            skippable: true,
        };
        let listener = TestListener::new();
        let processor = TestProcessor::new(false);
        let result = ensure_running_on_same_thread(&packet, &listener, &processor);
        assert_eq!(
            result,
            Err(RunningOnDifferentThreadException::RUNNING_ON_DIFFERENT_THREAD)
        );
        let scheduled = processor.scheduled.borrow();
        assert_eq!(scheduled.len(), 1);
        assert_eq!(scheduled[0], "p");
    }

    #[test]
    fn ensure_running_on_same_thread_for_level_delegates_to_processor() {
        let packet = TestPacket {
            type_name: "lvl".into(),
            terminal: true,
            skippable: false,
        };
        let listener = TestListener::new();
        let level = TestLevel {
            server: TestServer {
                processor: TestProcessor::new(false),
            },
        };
        let result = ensure_running_on_same_thread_for_level(&packet, &listener, &level);
        assert_eq!(
            result,
            Err(RunningOnDifferentThreadException::RUNNING_ON_DIFFERENT_THREAD)
        );
        assert_eq!(level.server.processor.scheduled.borrow().len(), 1);
    }

    #[test]
    fn fill_crash_report_adds_packet_category_and_listener_details() {
        let mut report = CrashReport::for_throwable("boom", "Main thread packet handler");
        let listener = TestListener::new();
        let packet = TestPacket {
            type_name: "test:packet_neg_-2147483648_nan_7fc00000".into(),
            terminal: true,
            skippable: false,
        };
        fill_crash_report(&mut report, &listener, Some(&packet));
        assert_eq!(*listener.filled.borrow(), 1);
        let cat = report
            .categories
            .iter()
            .find(|c| c.name == "Incoming Packet")
            .expect("packet category");
        assert!(cat
            .details
            .iter()
            .any(|(k, v)| k == "Type" && v == "test:packet_neg_-2147483648_nan_7fc00000"));
        assert!(cat
            .details
            .iter()
            .any(|(k, v)| k == "Is Terminal" && v == "true"));
        assert!(cat
            .details
            .iter()
            .any(|(k, v)| k == "Is Skippable" && v == "false"));
        assert!(report.categories.iter().any(|c| c.name == "Listener"));
    }

    #[test]
    fn fill_crash_report_without_packet_only_calls_listener() {
        let mut report = CrashReport::for_throwable("err", "ctx");
        let listener = TestListener::new();
        let packet: Option<&TestPacket> = None;
        fill_crash_report(&mut report, &listener, packet);
        assert_eq!(*listener.filled.borrow(), 1);
        assert!(!report
            .categories
            .iter()
            .any(|c| c.name == "Incoming Packet"));
        assert!(report.categories.iter().any(|c| c.name == "Listener"));
    }

    #[test]
    fn make_reported_exception_wraps_other_and_reuses_reported() {
        let listener = TestListener::new();
        let packet = TestPacket {
            type_name: "pkt_nan_7fc00000".into(),
            terminal: false,
            skippable: true,
        };

        let other = make_reported_exception(
            HandledPacketException::Other("bad".into()),
            Some(&packet),
            &listener,
        );
        assert_eq!(other.report.title, "Main thread packet handler");
        assert_eq!(other.report.message, "bad");
        assert!(other
            .report
            .categories
            .iter()
            .any(|c| c.name == "Incoming Packet"));

        let mut existing_report = CrashReport::for_throwable("old", "old_ctx");
        existing_report
            .add_category("Existing")
            .set_detail("neg", || i32::MIN.to_string());
        let reused = make_reported_exception(
            HandledPacketException::Reported(ReportedException::new(existing_report)),
            Some(&packet),
            &listener,
        );
        assert_eq!(reused.report.title, "old_ctx");
        assert_eq!(reused.report.message, "old");
        assert!(reused.report.categories.iter().any(|c| c.name == "Existing"));
        let cat = reused
            .report
            .categories
            .iter()
            .find(|c| c.name == "Incoming Packet")
            .expect("incoming packet category");
        assert!(cat
            .details
            .iter()
            .any(|(k, v)| k == "Type" && v == "pkt_nan_7fc00000"));
        assert_eq!(*listener.filled.borrow(), 2);
    }
}
