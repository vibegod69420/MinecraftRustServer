use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

impl fmt::Display for PacketFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketFlow::Clientbound => write!(f, "CLIENTBOUND"),
            PacketFlow::Serverbound => write!(f, "SERVERBOUND"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Handshaking,
    Play,
    Status,
    Login,
    Configuration,
}

impl ConnectionProtocol {
    pub fn id(&self) -> &'static str {
        match self {
            ConnectionProtocol::Handshaking => "handshaking",
            ConnectionProtocol::Play => "play",
            ConnectionProtocol::Status => "status",
            ConnectionProtocol::Login => "login",
            ConnectionProtocol::Configuration => "configuration",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisconnectionDetails {
    pub reason: Component,
}

impl DisconnectionDetails {
    pub fn new(reason: Component) -> Self {
        Self { reason }
    }
}

pub trait Packet: fmt::Debug {}

#[derive(Debug)]
pub struct ReportedException {
    pub message: String,
}

impl fmt::Display for ReportedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ReportedException {}

#[derive(Debug, Clone)]
pub struct CrashReportCategory {
    pub title: String,
    pub details: Vec<(String, String)>,
}

impl CrashReportCategory {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            details: Vec::new(),
        }
    }

    pub fn set_detail<F: FnOnce() -> String>(&mut self, name: &str, value: F) {
        self.details.push((name.to_string(), value()));
    }
}

#[derive(Debug, Clone, Default)]
pub struct CrashReport {
    pub categories: Vec<CrashReportCategory>,
}

impl CrashReport {
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
        }
    }
}

pub trait PacketListener {
    // JAVA: net.minecraft.network.PacketListener#flow
    fn flow(&self) -> PacketFlow;

    // JAVA: net.minecraft.network.PacketListener#protocol
    fn protocol(&self) -> ConnectionProtocol;

    // JAVA: net.minecraft.network.PacketListener#onDisconnect
    fn on_disconnect(&self, details: DisconnectionDetails);

    // JAVA: net.minecraft.network.PacketListener#onPacketError
    fn on_packet_error(
        &self,
        packet: &dyn Packet,
        error: &(dyn std::error::Error + 'static),
    ) -> ReportedException {
        ReportedException {
            message: format!("Packet error in {:?}: {}", packet, error),
        }
    }

    // JAVA: net.minecraft.network.PacketListener#createDisconnectionInfo
    fn create_disconnection_info(
        &self,
        reason: Component,
        _throwable: Option<&(dyn std::error::Error + 'static)>,
    ) -> DisconnectionDetails {
        DisconnectionDetails::new(reason)
    }

    // JAVA: net.minecraft.network.PacketListener#isAcceptingMessages
    fn is_accepting_messages(&self) -> bool;

    // JAVA: net.minecraft.network.PacketListener#shouldHandleMessage
    fn should_handle_message(&self, _packet: &dyn Packet) -> bool {
        self.is_accepting_messages()
    }

    // JAVA: net.minecraft.network.PacketListener#fillCrashReport
    fn fill_crash_report(&self, report: &mut CrashReport) {
        let protocol_id = self.protocol().id().to_string();
        let flow = self.flow().to_string();
        let mut category = CrashReportCategory::new("Connection");
        category.set_detail("Protocol", || protocol_id);
        category.set_detail("Flow", || flow);
        self.fill_listener_specific_crash_details(report, &mut category);
        report.categories.push(category);
    }

    // JAVA: net.minecraft.network.PacketListener#fillListenerSpecificCrashDetails
    fn fill_listener_specific_crash_details(
        &self,
        _report: &mut CrashReport,
        _category: &mut CrashReportCategory,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyPacket;
    impl Packet for DummyPacket {}

    struct TestListener {
        accepting: bool,
    }

    impl PacketListener for TestListener {
        fn flow(&self) -> PacketFlow {
            PacketFlow::Serverbound
        }

        fn protocol(&self) -> ConnectionProtocol {
            ConnectionProtocol::Play
        }

        fn on_disconnect(&self, _details: DisconnectionDetails) {}

        fn is_accepting_messages(&self) -> bool {
            self.accepting
        }
    }

    #[test]
    fn should_handle_message_delegates_to_accepting() {
        let l = TestListener { accepting: true };
        let p = DummyPacket;
        assert!(l.should_handle_message(&p));
        let l2 = TestListener { accepting: false };
        assert!(!l2.should_handle_message(&p));
    }

    #[test]
    fn create_disconnection_info_wraps_reason() {
        let l = TestListener { accepting: true };
        let reason = Component::literal("bye");
        let details = l.create_disconnection_info(reason.clone(), None);
        assert_eq!(details.reason, reason);
    }

    #[test]
    fn fill_crash_report_adds_connection_category() {
        let l = TestListener { accepting: true };
        let mut report = CrashReport::new();
        l.fill_crash_report(&mut report);
        assert_eq!(report.categories.len(), 1);
        let cat = &report.categories[0];
        assert_eq!(cat.title, "Connection");
        assert!(cat.details.iter().any(|(k, v)| k == "Protocol" && v == "play"));
        assert!(cat.details.iter().any(|(k, v)| k == "Flow" && v == "SERVERBOUND"));
    }

    #[test]
    fn on_packet_error_returns_reported_exception() {
        let l = TestListener { accepting: true };
        let p = DummyPacket;
        let err = std::io::Error::other("boom");
        let reported = l.on_packet_error(&p, &err);
        assert!(reported.message.contains("boom"));
    }
}
