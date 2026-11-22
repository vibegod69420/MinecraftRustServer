use std::error::Error;
use std::fmt;

pub trait ServerboundPacketListener {}

#[derive(Debug)]
pub struct ReportedException {
    pub message: String,
}

impl ReportedException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ReportedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ReportedException {}

pub trait Packet: fmt::Debug {}

pub trait ServerPacketListener: ServerboundPacketListener {
    // JAVA: net.minecraft.network.protocol.game.ServerPacketListener#onPacketError
    fn on_packet_error<P: Packet + ?Sized>(
        &self,
        packet: &P,
        err: &(dyn Error + 'static),
    ) -> Result<(), ReportedException> {
        eprintln!(
            "Failed to handle packet {:?}, suppressing error: {}",
            packet, err
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyPacket;

    impl Packet for DummyPacket {}

    struct DummyListener;

    impl ServerboundPacketListener for DummyListener {}
    impl ServerPacketListener for DummyListener {}

    #[derive(Debug)]
    struct TestError(&'static str);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Error for TestError {}

    #[test]
    fn on_packet_error_suppresses_error() {
        let listener = DummyListener;
        let packet = DummyPacket;
        let err = TestError("boom");
        let result = listener.on_packet_error(&packet, &err);
        assert!(result.is_ok());
    }

    #[test]
    fn on_packet_error_handles_empty_error_message() {
        let listener = DummyListener;
        let packet = DummyPacket;
        let err = TestError("");
        assert!(listener.on_packet_error(&packet, &err).is_ok());
    }

    #[test]
    fn reported_exception_display_and_debug() {
        let err = ReportedException::new("test failure");
        assert_eq!(err.to_string(), "test failure");
        let dbg = format!("{:?}", err);
        assert!(dbg.contains("ReportedException"));
        assert!(dbg.contains("test failure"));
    }

    #[test]
    fn on_packet_error_accepts_negative_payload_packet() {
        #[derive(Debug)]
        struct SignedPacket(i32);

        impl Packet for SignedPacket {}

        let listener = DummyListener;
        let packet = SignedPacket(i32::MIN);
        let err = TestError("negative edge case");
        assert!(listener.on_packet_error(&packet, &err).is_ok());
    }

    #[test]
    fn reported_exception_from_owned_string() {
        let msg = String::from("owned");
        let err = ReportedException::new(msg);
        assert_eq!(err.message, "owned");
    }
}
