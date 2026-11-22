use std::any::Any;
use std::fmt;
use std::marker::PhantomData;

use bytes::{Buf, BufMut};

// ---------------------------------------------------------------------------
// Minimal self-contained protocol types
// ---------------------------------------------------------------------------

/// JAVA: net.minecraft.network.protocol.PacketFlow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

impl PacketFlow {
    // JAVA: net.minecraft.network.protocol.PacketFlow#name (enum name)
    pub fn name(&self) -> &'static str {
        match self {
            PacketFlow::Clientbound => "CLIENTBOUND",
            PacketFlow::Serverbound => "SERVERBOUND",
        }
    }
}

/// JAVA: net.minecraft.network.PacketListener
pub trait PacketListener: Send + Sync + 'static {}

/// Identifies a packet type: its numeric id and protocol flow.
/// JAVA: net.minecraft.network.protocol.PacketType (identity/flow aspects)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketTypeInfo {
    pub id: i32,
    pub flow: PacketFlow,
}

/// Typed packet type handle.
/// JAVA: net.minecraft.network.protocol.PacketType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType<T: ?Sized> {
    info: PacketTypeInfo,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> PacketType<T> {
    // JAVA: net.minecraft.network.protocol.PacketType#<init> (conceptual)
    pub fn new(id: i32, flow: PacketFlow) -> Self {
        Self {
            info: PacketTypeInfo { id, flow },
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.protocol.PacketType#flow
    pub fn flow(&self) -> PacketFlow {
        self.info.flow
    }

    // JAVA: net.minecraft.network.protocol.PacketType (id accessor)
    pub fn id(&self) -> i32 {
        self.info.id
    }

    // JAVA: net.minecraft.network.protocol.PacketType (info accessor)
    pub fn info(&self) -> PacketTypeInfo {
        self.info
    }
}

/// Base packet trait.
/// JAVA: net.minecraft.network.protocol.Packet
pub trait Packet<L: PacketListener + ?Sized>: Any + Send {
    // JAVA: net.minecraft.network.protocol.Packet#type
    fn packet_type(&self) -> PacketTypeInfo;
}

impl<L: PacketListener + ?Sized + 'static> dyn Packet<L> {
    /// Downcast helper for encoding concrete packet types.
    pub fn downcast_ref<T: Packet<L> + 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}

/// Stream codec for encoding/decoding values to/from a byte buffer.
/// JAVA: net.minecraft.network.codec.StreamCodec
pub trait StreamCodec<B, V> {
    // JAVA: net.minecraft.network.codec.StreamCodec#decode
    fn decode(&self, buf: &mut B) -> V;

    // JAVA: net.minecraft.network.codec.StreamCodec#encode
    fn encode(&self, buf: &mut B, value: &V);
}

// ---------------------------------------------------------------------------
// Minimal IdDispatchCodec builder/codec (self-contained support)
// ---------------------------------------------------------------------------

struct DispatchEntry<B, L: PacketListener + ?Sized> {
    type_info: PacketTypeInfo,
    decode: Box<dyn Fn(&mut B) -> Box<dyn Packet<L>> + Send + Sync>,
    encode: Box<dyn Fn(&mut B, &dyn Packet<L>) + Send + Sync>,
}

impl<B, L: PacketListener + ?Sized> fmt::Debug for DispatchEntry<B, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DispatchEntry")
            .field("type_info", &self.type_info)
            .finish()
    }
}

/// JAVA: net.minecraft.network.codec.IdDispatchCodec.Builder
pub struct IdDispatchCodecBuilder<B, L: PacketListener + ?Sized> {
    entries: Vec<DispatchEntry<B, L>>,
}

impl<B, L: PacketListener + ?Sized> fmt::Debug for IdDispatchCodecBuilder<B, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdDispatchCodecBuilder")
            .field("entries_len", &self.entries.len())
            .finish()
    }
}

impl<B, L: PacketListener + ?Sized + 'static> IdDispatchCodecBuilder<B, L> {
    // JAVA: net.minecraft.network.codec.IdDispatchCodec#builder
    pub fn builder(
        _type_extractor: fn(&dyn Packet<L>) -> PacketTypeInfo,
    ) -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // JAVA: net.minecraft.network.codec.IdDispatchCodec.Builder#add
    pub fn add<T, C>(&mut self, packet_type: PacketType<T>, codec: C) -> &mut Self
    where
        T: Packet<L> + 'static,
        C: StreamCodec<B, T> + Send + Sync + 'static,
    {
        let type_info = packet_type.info();
        let codec = std::sync::Arc::new(codec);

        let decode_codec = std::sync::Arc::clone(&codec);
        let decode = Box::new(move |buf: &mut B| -> Box<dyn Packet<L>> {
            let value: T = decode_codec.decode(buf);
            Box::new(value) as Box<dyn Packet<L>>
        });

        let encode_codec = codec;
        let encode = Box::new(move |buf: &mut B, packet: &dyn Packet<L>| {
            let concrete = packet.downcast_ref::<T>().unwrap_or_else(|| {
                panic!(
                    "Packet type mismatch during encode: expected type id {}, got {:?}",
                    type_info.id,
                    packet.packet_type()
                )
            });
            encode_codec.encode(buf, concrete);
        });

        self.entries.push(DispatchEntry {
            type_info,
            decode,
            encode,
        });
        self
    }

    // JAVA: net.minecraft.network.codec.IdDispatchCodec.Builder#build
    pub fn build(self) -> IdDispatchCodec<B, L> {
        IdDispatchCodec {
            entries: self.entries,
        }
    }
}

/// Built dispatch codec.
/// JAVA: net.minecraft.network.codec.IdDispatchCodec
pub struct IdDispatchCodec<B, L: PacketListener + ?Sized> {
    entries: Vec<DispatchEntry<B, L>>,
}

impl<B, L: PacketListener + ?Sized> fmt::Debug for IdDispatchCodec<B, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdDispatchCodec")
            .field("entries_len", &self.entries.len())
            .finish()
    }
}

impl<B, L: PacketListener + ?Sized> IdDispatchCodec<B, L> {
    fn find_by_id(&self, id: i32) -> Option<&DispatchEntry<B, L>> {
        self.entries.iter().find(|e| e.type_info.id == id)
    }

    fn find_by_type(&self, type_info: PacketTypeInfo) -> Option<&DispatchEntry<B, L>> {
        self.entries.iter().find(|e| e.type_info == type_info)
    }
}

fn write_varint<B: BufMut>(buf: &mut B, mut value: i32) {
    let mut v = value as u32;
    loop {
        let mut byte = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        buf.put_u8(byte);
        if v == 0 {
            break;
        }
    }
}

fn read_varint<B: Buf>(buf: &mut B) -> i32 {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = buf.get_u8();
        result |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            panic!("VarInt too big");
        }
    }
    result
}

impl<B: BufMut + Buf, L: PacketListener + ?Sized> StreamCodec<B, Box<dyn Packet<L>>>
    for IdDispatchCodec<B, L>
{
    // JAVA: net.minecraft.network.codec.IdDispatchCodec#decode
    fn decode(&self, buf: &mut B) -> Box<dyn Packet<L>> {
        let id = read_varint(buf);
        let entry = self
            .find_by_id(id)
            .unwrap_or_else(|| panic!("Unknown packet id: {}", id));
        (entry.decode)(buf)
    }

    // JAVA: net.minecraft.network.codec.IdDispatchCodec#encode
    fn encode(&self, buf: &mut B, value: &Box<dyn Packet<L>>) {
        let type_info = value.packet_type();
        let entry = self.find_by_type(type_info).unwrap_or_else(|| {
            panic!("Unknown packet type: {:?}", type_info)
        });
        write_varint(buf, type_info.id);
        (entry.encode)(buf, value.as_ref());
    }
}

// ---------------------------------------------------------------------------
// ProtocolCodecBuilder
// ---------------------------------------------------------------------------

/// JAVA: net.minecraft.network.protocol.ProtocolCodecBuilder
pub struct ProtocolCodecBuilder<B, L: PacketListener + ?Sized + 'static> {
    dispatch_builder: IdDispatchCodecBuilder<B, L>,
    flow: PacketFlow,
}

impl<B, L: PacketListener + ?Sized + 'static> fmt::Debug for ProtocolCodecBuilder<B, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProtocolCodecBuilder")
            .field("dispatch_builder", &self.dispatch_builder)
            .field("flow", &self.flow)
            .finish()
    }
}

impl<B, L: PacketListener + ?Sized + 'static> ProtocolCodecBuilder<B, L> {
    // JAVA: net.minecraft.network.protocol.ProtocolCodecBuilder#<init>
    pub fn new(flow: PacketFlow) -> Self {
        Self {
            dispatch_builder: IdDispatchCodecBuilder::builder(|p| p.packet_type()),
            flow,
        }
    }

    // JAVA: net.minecraft.network.protocol.ProtocolCodecBuilder#add
    pub fn add<T, C>(&mut self, packet_type: PacketType<T>, codec: C) -> &mut Self
    where
        T: Packet<L> + 'static,
        C: StreamCodec<B, T> + Send + Sync + 'static,
    {
        if packet_type.flow() != self.flow {
            panic!(
                "Invalid packet flow for packet {:?}, expected {}",
                packet_type.info(),
                self.flow.name()
            );
        }
        self.dispatch_builder.add(packet_type, codec);
        self
    }

    // JAVA: net.minecraft.network.protocol.ProtocolCodecBuilder#build
    pub fn build(self) -> impl StreamCodec<B, Box<dyn Packet<L>>>
    where
        B: BufMut + Buf,
    {
        self.dispatch_builder.build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestListener;
    impl PacketListener for TestListener {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PingPacket {
        value: i32,
    }

    impl Packet<TestListener> for PingPacket {
        fn packet_type(&self) -> PacketTypeInfo {
            PacketTypeInfo {
                id: 1,
                flow: PacketFlow::Serverbound,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PongPacket {
        value: i64,
    }

    impl Packet<TestListener> for PongPacket {
        fn packet_type(&self) -> PacketTypeInfo {
            PacketTypeInfo {
                id: 2,
                flow: PacketFlow::Serverbound,
            }
        }
    }

    struct PingCodec;
    impl StreamCodec<BytesMut, PingPacket> for PingCodec {
        fn decode(&self, buf: &mut BytesMut) -> PingPacket {
            PingPacket {
                value: buf.get_i32(),
            }
        }
        fn encode(&self, buf: &mut BytesMut, value: &PingPacket) {
            buf.put_i32(value.value);
        }
    }

    struct PongCodec;
    impl StreamCodec<BytesMut, PongPacket> for PongCodec {
        fn decode(&self, buf: &mut BytesMut) -> PongPacket {
            PongPacket {
                value: buf.get_i64(),
            }
        }
        fn encode(&self, buf: &mut BytesMut, value: &PongPacket) {
            buf.put_i64(value.value);
        }
    }

    fn ping_type() -> PacketType<PingPacket> {
        PacketType::new(1, PacketFlow::Serverbound)
    }

    fn pong_type() -> PacketType<PongPacket> {
        PacketType::new(2, PacketFlow::Serverbound)
    }

    #[test]
    fn build_and_roundtrip_dispatch() {
        let mut builder = ProtocolCodecBuilder::<BytesMut, TestListener>::new(PacketFlow::Serverbound);
        builder.add(ping_type(), PingCodec);
        builder.add(pong_type(), PongCodec);
        let codec = builder.build();

        let mut buf = BytesMut::new();
        let packet: Box<dyn Packet<TestListener>> = Box::new(PingPacket { value: -42 });
        codec.encode(&mut buf, &packet);

        let mut read_buf = buf;
        let decoded = codec.decode(&mut read_buf);
        let ping = decoded.downcast_ref::<PingPacket>().expect("should be PingPacket");
        assert_eq!(ping.value, -42);
    }

    #[test]
    fn roundtrip_second_packet_type() {
        let mut builder = ProtocolCodecBuilder::<BytesMut, TestListener>::new(PacketFlow::Serverbound);
        builder.add(ping_type(), PingCodec);
        builder.add(pong_type(), PongCodec);
        let codec = builder.build();

        let mut buf = BytesMut::new();
        let packet: Box<dyn Packet<TestListener>> = Box::new(PongPacket { value: i64::MIN });
        codec.encode(&mut buf, &packet);

        let decoded = codec.decode(&mut buf);
        let pong = decoded.downcast_ref::<PongPacket>().expect("should be PongPacket");
        assert_eq!(pong.value, i64::MIN);
    }

    #[test]
    #[should_panic(expected = "Invalid packet flow")]
    fn add_wrong_flow_panics() {
        let mut builder = ProtocolCodecBuilder::<BytesMut, TestListener>::new(PacketFlow::Clientbound);
        builder.add(ping_type(), PingCodec);
    }

    #[test]
    fn packet_flow_name_matches_java_enum() {
        assert_eq!(PacketFlow::Clientbound.name(), "CLIENTBOUND");
        assert_eq!(PacketFlow::Serverbound.name(), "SERVERBOUND");
    }

    #[test]
    fn packet_type_exposes_flow_and_id() {
        let pt = PacketType::<PingPacket>::new(7, PacketFlow::Clientbound);
        assert_eq!(pt.flow(), PacketFlow::Clientbound);
        assert_eq!(pt.id(), 7);
        assert_eq!(pt.info(), PacketTypeInfo { id: 7, flow: PacketFlow::Clientbound });
    }

    #[test]
    fn varint_roundtrip_negative_and_edge_values() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, 0);
        write_varint(&mut buf, -1);
        write_varint(&mut buf, i32::MAX);
        write_varint(&mut buf, i32::MIN);

        assert_eq!(read_varint(&mut buf), 0);
        assert_eq!(read_varint(&mut buf), -1);
        assert_eq!(read_varint(&mut buf), i32::MAX);
        assert_eq!(read_varint(&mut buf), i32::MIN);
    }
}
