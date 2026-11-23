use std::marker::PhantomData;
use std::sync::LazyLock;

pub const MAX_PAYLOAD_SIZE: i32 = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType<T> {
    pub flow: PacketFlow,
    pub id: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T> PacketType<T> {
    // JAVA: net.minecraft.network.protocol.PacketType#<init>
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self {
            flow,
            id,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendlyByteBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryFriendlyByteBuf;

#[derive(Debug)]
pub struct StreamCodec<B, V> {
    _buf: PhantomData<fn() -> B>,
    _val: PhantomData<fn() -> V>,
}

impl<B, V> StreamCodec<B, V> {
    // JAVA: net.minecraft.network.codec.StreamCodec (stub constructor)
    pub fn create() -> Self {
        Self {
            _buf: PhantomData,
            _val: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.codec.StreamCodec#map
    pub fn map<V2, F1, F2>(self, _to: F1, _from: F2) -> StreamCodec<B, V2>
    where
        F1: Fn(V) -> V2,
        F2: Fn(&V2) -> V,
    {
        StreamCodec {
            _buf: PhantomData,
            _val: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandPayload {
    pub brand: String,
}

impl BrandPayload {
    // JAVA: net.minecraft.network.protocol.common.custom.BrandPayload#type (TYPE identifier)
    pub fn type_id() -> Identifier {
        Identifier::with_default_namespace("brand")
    }

    // JAVA: net.minecraft.network.protocol.common.custom.BrandPayload#STREAM_CODEC (stub accessor)
    pub fn stream_codec<B>() -> StreamCodec<B, CustomPacketPayload> {
        StreamCodec::create()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscardedPayload {
    pub id: Identifier,
    pub data: Vec<u8>,
}

impl DiscardedPayload {
    // JAVA: net.minecraft.network.protocol.common.custom.DiscardedPayload#codec
    pub fn codec<B>(_id: &Identifier, _max_payload_size: i32) -> StreamCodec<B, CustomPacketPayload> {
        StreamCodec::create()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomPacketPayload {
    Brand(BrandPayload),
    Discarded(DiscardedPayload),
}

pub struct TypeAndCodec<B> {
    pub type_id: Identifier,
    pub codec: StreamCodec<B, CustomPacketPayload>,
    _buf: PhantomData<fn() -> B>,
}

impl<B> TypeAndCodec<B> {
    // JAVA: net.minecraft.network.protocol.common.custom.CustomPacketPayload.TypeAndCodec#<init>
    pub fn new(type_id: Identifier, codec: StreamCodec<B, CustomPacketPayload>) -> Self {
        Self {
            type_id,
            codec,
            _buf: PhantomData,
        }
    }
}

impl CustomPacketPayload {
    // JAVA: net.minecraft.network.protocol.common.custom.CustomPacketPayload#codec
    pub fn codec<B, F>(
        _fallback: F,
        _types: Vec<TypeAndCodec<B>>,
    ) -> StreamCodec<B, CustomPacketPayload>
    where
        F: Fn(&Identifier) -> StreamCodec<B, CustomPacketPayload>,
    {
        StreamCodec::create()
    }
}

pub trait ClientCommonPacketListener {
    fn handle_custom_payload(&mut self, packet: &ClientboundCustomPayloadPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomPayloadPacket {
    pub payload: CustomPacketPayload,
}

impl ClientboundCustomPayloadPacket {
    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket#<init>
    pub fn new(payload: CustomPacketPayload) -> Self {
        Self { payload }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket#payload
    pub fn payload(&self) -> &CustomPacketPayload {
        &self.payload
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket#type
    pub fn packet_type(&self) -> PacketType<ClientboundCustomPayloadPacket> {
        PacketType::new(
            PacketFlow::Clientbound,
            Identifier::with_default_namespace("custom_payload"),
        )
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket#handle
    pub fn handle<L: ClientCommonPacketListener>(&self, listener: &mut L) {
        listener.handle_custom_payload(self);
    }
}

pub static GAMEPLAY_STREAM_CODEC: LazyLock<
    StreamCodec<RegistryFriendlyByteBuf, ClientboundCustomPayloadPacket>,
> = LazyLock::new(|| {
    CustomPacketPayload::codec::<RegistryFriendlyByteBuf, _>(
        |id: &Identifier| DiscardedPayload::codec(id, MAX_PAYLOAD_SIZE),
        vec![TypeAndCodec::<RegistryFriendlyByteBuf>::new(
            BrandPayload::type_id(),
            BrandPayload::stream_codec(),
        )],
    )
    .map(
        ClientboundCustomPayloadPacket::new,
        |p: &ClientboundCustomPayloadPacket| p.payload.clone(),
    )
});

pub static CONFIG_STREAM_CODEC: LazyLock<
    StreamCodec<FriendlyByteBuf, ClientboundCustomPayloadPacket>,
> = LazyLock::new(|| {
    CustomPacketPayload::codec::<FriendlyByteBuf, _>(
        |id: &Identifier| DiscardedPayload::codec(id, MAX_PAYLOAD_SIZE),
        vec![TypeAndCodec::<FriendlyByteBuf>::new(
            BrandPayload::type_id(),
            BrandPayload::stream_codec(),
        )],
    )
    .map(
        ClientboundCustomPayloadPacket::new,
        |p: &ClientboundCustomPayloadPacket| p.payload.clone(),
    )
});

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        handled: bool,
        saw_brand: Option<String>,
    }

    impl ClientCommonPacketListener for MockListener {
        fn handle_custom_payload(&mut self, packet: &ClientboundCustomPayloadPacket) {
            self.handled = true;
            if let CustomPacketPayload::Brand(b) = packet.payload() {
                self.saw_brand = Some(b.brand.clone());
            }
        }
    }

    #[test]
    fn packet_type_is_clientbound_custom_payload() {
        let pkt = ClientboundCustomPayloadPacket::new(CustomPacketPayload::Discarded(
            DiscardedPayload {
                id: Identifier::with_default_namespace("test"),
                data: vec![1, 2, 3],
            },
        ));
        let pt = pkt.packet_type();
        assert_eq!(pt.flow, PacketFlow::Clientbound);
        assert_eq!(pt.id, Identifier::with_default_namespace("custom_payload"));
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let pkt = ClientboundCustomPayloadPacket::new(CustomPacketPayload::Brand(BrandPayload {
            brand: "vanilla".to_string(),
        }));
        let mut listener = MockListener {
            handled: false,
            saw_brand: None,
        };
        pkt.handle(&mut listener);
        assert!(listener.handled);
        assert_eq!(listener.saw_brand.as_deref(), Some("vanilla"));
    }

    #[test]
    fn max_payload_size_constant() {
        assert_eq!(MAX_PAYLOAD_SIZE, 1_048_576_i32);
        assert_eq!(MAX_PAYLOAD_SIZE, 1024 * 1024);
    }

    #[test]
    fn payload_accessor_and_constructor() {
        let payload = CustomPacketPayload::Discarded(DiscardedPayload {
            id: Identifier::with_default_namespace("unknown"),
            data: Vec::new(),
        });
        let pkt = ClientboundCustomPayloadPacket::new(payload.clone());
        assert_eq!(pkt.payload(), &payload);
    }

    #[test]
    fn codec_statics_initialize() {
        let _gameplay = &*GAMEPLAY_STREAM_CODEC;
        let _config = &*CONFIG_STREAM_CODEC;
    }
}
