/// Minimal placeholder for `net.minecraft.network.PacketListener`.
pub trait PacketListener {}

/// Minimal placeholder for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType {
    pub id: String,
}

impl PacketType {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// Minimal placeholder for `net.minecraft.network.protocol.Packet`.
pub trait Packet<T: PacketListener> {
    fn handle(&self, listener: &mut T);
    fn packet_type(&self) -> PacketType;
}

/// JAVA: net.minecraft.network.protocol.BundleDelimiterPacket
pub trait BundleDelimiterPacket<T: PacketListener> {
    // JAVA: net.minecraft.network.protocol.BundleDelimiterPacket#type
    fn delimiter_type(&self) -> PacketType;
}

impl<T: PacketListener, P: BundleDelimiterPacket<T>> Packet<T> for P {
    // JAVA: net.minecraft.network.protocol.BundleDelimiterPacket#handle
    fn handle(&self, _listener: &mut T) {
        panic!("This packet should be handled by pipeline");
    }

    // JAVA: net.minecraft.network.protocol.BundleDelimiterPacket#type
    fn packet_type(&self) -> PacketType {
        self.delimiter_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener;
    impl PacketListener for TestListener {}

    struct MyDelimiter {
        id: &'static str,
    }

    impl BundleDelimiterPacket<TestListener> for MyDelimiter {
        // JAVA: net.minecraft.network.protocol.BundleDelimiterPacket#type
        fn delimiter_type(&self) -> PacketType {
            PacketType::new(self.id)
        }
    }

    #[test]
    fn delimiter_type_returns_configured_type() {
        let pkt = MyDelimiter { id: "bundle_delim" };
        assert_eq!(
            BundleDelimiterPacket::delimiter_type(&pkt).id,
            "bundle_delim"
        );
    }

    #[test]
    fn packet_type_delegates_to_delimiter_type() {
        let pkt = MyDelimiter { id: "delim" };
        let pt = Packet::<TestListener>::packet_type(&pkt);
        assert_eq!(pt.id, "delim");
    }

    #[test]
    #[should_panic(expected = "This packet should be handled by pipeline")]
    fn handle_panics_like_java_assertion_error() {
        let pkt = MyDelimiter { id: "delim" };
        let mut listener = TestListener;
        Packet::<TestListener>::handle(&pkt, &mut listener);
    }

    #[test]
    fn different_delimiters_have_different_types() {
        let a = MyDelimiter { id: "a" };
        let b = MyDelimiter { id: "b" };
        assert_ne!(
            Packet::<TestListener>::packet_type(&a),
            Packet::<TestListener>::packet_type(&b)
        );
    }
}
