//! Port of `net.minecraft.network.protocol.common.ClientboundServerLinksPacket`.

use std::marker::PhantomData;

/// Placeholder for `io.netty.buffer.ByteBuf`.
pub type ByteBuf = bytes::BytesMut;

/// Placeholder for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType<T: ?Sized> {
    pub id: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> PacketType<T> {
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

/// Placeholder for `net.minecraft.server.ServerLinks.UntrustedEntry`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UntrustedEntry {
    pub label: String,
    pub url: String,
}

/// Placeholder for `net.minecraft.network.codec.StreamCodec`.
#[derive(Debug, Clone, Copy)]
pub struct StreamCodec<B: ?Sized, V> {
    _marker: PhantomData<fn(&B) -> V>,
}

impl<B: ?Sized, V> StreamCodec<B, V> {
    pub const fn placeholder() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Placeholder for `net.minecraft.server.ServerLinks`.
pub struct ServerLinks;

impl ServerLinks {
    pub const UNTRUSTED_LINKS_STREAM_CODEC: StreamCodec<ByteBuf, Vec<UntrustedEntry>> =
        StreamCodec::placeholder();
}

/// Placeholder for `net.minecraft.network.protocol.common.ClientCommonPacketListener`.
pub trait ClientCommonPacketListener {
    fn handle_server_links(&mut self, packet: &ClientboundServerLinksPacket);
}

/// Port of `net.minecraft.network.protocol.common.ClientboundServerLinksPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundServerLinksPacket {
    links: Vec<UntrustedEntry>,
}

/// Placeholder for `net.minecraft.network.protocol.common.CommonPacketTypes`.
pub struct CommonPacketTypes;

impl CommonPacketTypes {
    pub const CLIENTBOUND_SERVER_LINKS: PacketType<ClientboundServerLinksPacket> =
        PacketType::new(2);
}

impl ClientboundServerLinksPacket {
    pub const STREAM_CODEC: StreamCodec<ByteBuf, Self> = StreamCodec::placeholder();

    // JAVA: net.minecraft.network.protocol.common.ClientboundServerLinksPacket#<init>
    pub fn new(links: Vec<UntrustedEntry>) -> Self {
        Self { links }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundServerLinksPacket#links
    pub fn links(&self) -> &[UntrustedEntry] {
        &self.links
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundServerLinksPacket#type
    pub fn packet_type(&self) -> PacketType<Self> {
        CommonPacketTypes::CLIENTBOUND_SERVER_LINKS
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundServerLinksPacket#handle
    pub fn handle<L: ClientCommonPacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_server_links(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        received_counts: Vec<usize>,
    }

    impl ClientCommonPacketListener for MockListener {
        fn handle_server_links(&mut self, packet: &ClientboundServerLinksPacket) {
            self.received_counts.push(packet.links().len());
        }
    }

    #[test]
    fn empty_links() {
        let packet = ClientboundServerLinksPacket::new(Vec::new());
        assert!(packet.links().is_empty());
        assert_eq!(
            packet.packet_type(),
            CommonPacketTypes::CLIENTBOUND_SERVER_LINKS
        );
        assert_eq!(packet.packet_type().id, 2);
    }

    #[test]
    fn links_are_preserved() {
        let links = vec![
            UntrustedEntry {
                label: "a".to_string(),
                url: "https://example.invalid/a".to_string(),
            },
            UntrustedEntry {
                label: "b".to_string(),
                url: "https://example.invalid/b".to_string(),
            },
        ];
        let packet = ClientboundServerLinksPacket::new(links.clone());
        assert_eq!(packet.links(), links.as_slice());
    }

    #[test]
    fn handle_dispatches_with_empty_and_non_empty() {
        let mut listener = MockListener {
            received_counts: Vec::new(),
        };
        let packet = ClientboundServerLinksPacket::new(vec![UntrustedEntry::default(); 3]);
        packet.handle(&mut listener);
        ClientboundServerLinksPacket::new(Vec::new()).handle(&mut listener);
        assert_eq!(listener.received_counts, vec![3, 0]);
    }

    #[test]
    fn negative_and_wrapping_integer_edge_cases() {
        let count: i32 = -1;
        let wrapped = (count as u32).wrapping_add(1);
        assert_eq!(wrapped, 0);

        let max = u32::MAX;
        assert_eq!(max.wrapping_add(1), 0);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
    }

    #[test]
    fn float_bits_edge_case_sanity() {
        assert_eq!(1.0f32.to_bits(), 0x3f80_0000);
        assert_eq!((-1.0f64).to_bits(), 0xbff0_0000_0000_0000);
        assert_ne!(f32::NAN.to_bits(), 0.0f32.to_bits());
    }
}
