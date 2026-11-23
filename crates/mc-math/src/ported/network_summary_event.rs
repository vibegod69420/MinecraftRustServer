use std::sync::atomic::{AtomicI32, AtomicI64, Ordering};

// JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent#<init>(String)
#[derive(Debug, Clone)]
pub struct NetworkSummaryEvent {
   pub remote_address: String,
   pub sent_bytes: i64,
   pub sent_packets: i32,
   pub received_bytes: i64,
   pub received_packets: i32,
}

impl NetworkSummaryEvent {
   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent#<init>(String)
   pub fn new(remote_address: String) -> Self {
      Self {
         remote_address,
         sent_bytes: 0,
         sent_packets: 0,
         received_bytes: 0,
         received_packets: 0,
      }
   }

   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.Fields#REMOTE_ADDRESS
   pub const REMOTE_ADDRESS: &str = "remoteAddress";
   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.Fields#SENT_BYTES
   pub const SENT_BYTES: &str = "sentBytes";
   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.Fields#SENT_PACKETS
   pub const SENT_PACKETS: &str = "sentPackets";
   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.Fields#RECEIVED_BYTES
   pub const RECEIVED_BYTES: &str = "receivedBytes";
   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.Fields#RECEIVED_PACKETS
   pub const RECEIVED_PACKETS: &str = "receivedPackets";
}

// JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.SumAggregation#<init>(String)
pub struct SumAggregation {
   sent_bytes: AtomicI64,
   sent_packets: AtomicI32,
   received_bytes: AtomicI64,
   received_packets: AtomicI32,
   event: NetworkSummaryEvent,
}

impl SumAggregation {
   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.SumAggregation#<init>(String)
   pub fn new(remote_address: String) -> Self {
      let event = NetworkSummaryEvent::new(remote_address);
      Self {
         sent_bytes: AtomicI64::new(0),
         sent_packets: AtomicI32::new(0),
         received_bytes: AtomicI64::new(0),
         received_packets: AtomicI32::new(0),
         event,
      }
   }

   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.SumAggregation#trackSentPacket(int)
   pub fn track_sent_packet(&mut self, bytes: i32) {
      self.sent_packets.fetch_add(1, Ordering::Relaxed);
      self.sent_bytes.fetch_add(bytes as i64, Ordering::Relaxed);
   }

   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.SumAggregation#trackReceivedPacket(int)
   pub fn track_received_packet(&mut self, bytes: i32) {
      self.received_packets.fetch_add(1, Ordering::Relaxed);
      self.received_bytes.fetch_add(bytes as i64, Ordering::Relaxed);
   }

   // JAVA: net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent.SumAggregation#commitEvent()
   pub fn commit_event(&mut self) {
      self.event.sent_bytes = self.sent_bytes.load(Ordering::Relaxed);
      self.event.sent_packets = self.sent_packets.load(Ordering::Relaxed);
      self.event.received_bytes = self.received_bytes.load(Ordering::Relaxed);
      self.event.received_packets = self.received_packets.load(Ordering::Relaxed);
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_network_summary_event_construction() {
      let event = NetworkSummaryEvent::new("127.0.0.1:25565".to_string());
      assert_eq!(event.remote_address, "127.0.0.1:25565");
      assert_eq!(event.sent_bytes, 0);
      assert_eq!(event.sent_packets, 0);
      assert_eq!(event.received_bytes, 0);
      assert_eq!(event.received_packets, 0);
   }

   #[test]
   fn test_sum_aggregation_track_and_commit() {
      let mut agg = SumAggregation::new("localhost".to_string());
      agg.track_sent_packet(100);
      agg.track_sent_packet(200);
      agg.track_received_packet(50);
      agg.track_received_packet(75);

      assert_eq!(agg.sent_packets.load(Ordering::Relaxed), 2);
      assert_eq!(agg.sent_bytes.load(Ordering::Relaxed), 300);
      assert_eq!(agg.received_packets.load(Ordering::Relaxed), 2);
      assert_eq!(agg.received_bytes.load(Ordering::Relaxed), 125);

      agg.commit_event();

      assert_eq!(agg.event.sent_packets, 2);
      assert_eq!(agg.event.sent_bytes, 300);
      assert_eq!(agg.event.received_packets, 2);
      assert_eq!(agg.event.received_bytes, 125);
   }

   #[test]
   fn test_sum_aggregation_edge_cases() {
      let mut agg = SumAggregation::new("edge".to_string());
      // Track zero bytes
      agg.track_sent_packet(0);
      assert_eq!(agg.sent_bytes.load(Ordering::Relaxed), 0);
      agg.commit_event();
      assert_eq!(agg.event.sent_bytes, 0);

      // Track negative bytes (i64 can be negative)
      agg.track_sent_packet(-10);
      assert_eq!(agg.sent_bytes.load(Ordering::Relaxed), -10);
      agg.commit_event();
      assert_eq!(agg.event.sent_bytes, -10);
   }
}
