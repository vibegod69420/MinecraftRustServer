use std::sync::{Arc, Mutex};

// JAVA: net.minecraft.network.RateKickingConnection
pub trait PacketLike {
    fn name(&self) -> &'static str;
}

pub trait PacketSendListenerLike {
    fn on_result(&self, success: bool, err: Option<&str>);
}

pub trait ConnectionLike {
    fn tick_second_base(&mut self);
    fn average_received_packets(&self) -> f32;
    fn send_packet(&mut self, packet: Box<dyn PacketLike>);
    fn disconnect(&mut self, reason: &str);
    fn set_read_only(&mut self);
}

#[derive(Clone)]
pub struct DisconnectPacket {
    pub reason: String,
}

impl DisconnectPacket {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl PacketLike for DisconnectPacket {
    fn name(&self) -> &'static str {
        "disconnect"
    }
}

pub struct RateKickingConnection {
    rate_limit_packets_per_second: i32,
    average_received_packets: f32,
    read_only: bool,
    disconnected: bool,
    last_reason: Option<String>,
    sent_packets: Vec<String>,
    on_disconnect_listeners: Vec<Arc<dyn Fn() + Send + Sync>>,
}

impl RateKickingConnection {
    pub const EXCEED_REASON: &'static str = "disconnect.exceeded_packet_rate";

    // JAVA: net.minecraft.network.RateKickingConnection#<init>
    pub fn new(rate_limit_packets_per_second: i32) -> Self {
        Self {
            rate_limit_packets_per_second,
            average_received_packets: 0.0,
            read_only: false,
            disconnected: false,
            last_reason: None,
            sent_packets: Vec::new(),
            on_disconnect_listeners: Vec::new(),
        }
    }

    pub fn set_average_received_packets(&mut self, avg: f32) {
        self.average_received_packets = avg;
    }

    pub fn average_received_packets_value(&self) -> f32 {
        self.average_received_packets
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected
    }

    pub fn last_reason(&self) -> Option<&str> {
        self.last_reason.as_deref()
    }

    pub fn sent_packet_names(&self) -> &[String] {
        &self.sent_packets
    }

    pub fn add_disconnect_listener<F>(&mut self, f: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_disconnect_listeners.push(Arc::new(f));
    }

    // JAVA: net.minecraft.network.RateKickingConnection#tickSecond
    pub fn tick_second(&mut self) {
        self.tick_second_base_internal();
        let avg = self.average_received_packets;
        if avg > (self.rate_limit_packets_per_second as f32) {
            eprintln!(
                "Player exceeded rate-limit (sent {} packets per second)",
                avg
            );
            let reason = Self::EXCEED_REASON.to_string();
            let packet = DisconnectPacket::new(reason.clone());
            self.send_packet(Box::new(packet));
            self.disconnect(&reason);
            for listener in &self.on_disconnect_listeners {
                listener();
            }
            self.set_read_only();
        }
    }

    fn tick_second_base_internal(&mut self) {
        // Placeholder for super.tickSecond() behavior in this self-contained port.
    }
}

impl ConnectionLike for RateKickingConnection {
    fn tick_second_base(&mut self) {
        self.tick_second_base_internal();
    }

    fn average_received_packets(&self) -> f32 {
        self.average_received_packets
    }

    fn send_packet(&mut self, packet: Box<dyn PacketLike>) {
        self.sent_packets.push(packet.name().to_string());
    }

    fn disconnect(&mut self, reason: &str) {
        self.disconnected = true;
        self.last_reason = Some(reason.to_string());
    }

    fn set_read_only(&mut self) {
        self.read_only = true;
    }
}

#[cfg(test)]
mod tests {
    use super::RateKickingConnection;

    #[test]
    fn test_under_limit_no_disconnect() {
        let mut conn = RateKickingConnection::new(10);
        conn.set_average_received_packets(5.0);
        conn.tick_second();
        assert!(!conn.is_disconnected());
        assert!(!conn.is_read_only());
        assert!(conn.last_reason().is_none());
        assert!(conn.sent_packet_names().is_empty());
    }

    #[test]
    fn test_equal_limit_no_disconnect() {
        let mut conn = RateKickingConnection::new(10);
        conn.set_average_received_packets(10.0);
        conn.tick_second();
        assert!(!conn.is_disconnected());
        assert!(!conn.is_read_only());
    }

    #[test]
    fn test_over_limit_disconnects_and_sets_read_only() {
        let mut conn = RateKickingConnection::new(10);
        conn.set_average_received_packets(10.5);
        conn.tick_second();
        assert!(conn.is_disconnected());
        assert!(conn.is_read_only());
        assert_eq!(
            conn.last_reason(),
            Some(RateKickingConnection::EXCEED_REASON)
        );
        assert_eq!(conn.sent_packet_names().len(), 1);
    }

    #[test]
    fn test_negative_average_no_disconnect() {
        let mut conn = RateKickingConnection::new(10);
        conn.set_average_received_packets(-1.0);
        conn.tick_second();
        assert!(!conn.is_disconnected());
        assert!(!conn.is_read_only());
    }
}
