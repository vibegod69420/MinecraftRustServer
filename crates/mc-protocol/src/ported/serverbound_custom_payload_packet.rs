pub const MAX_PAYLOAD_SIZE: i32 = 32767;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ResourceLocation {
    // JAVA: net.minecraft.resources.ResourceLocation#<init>
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomPacketPayload {
    Brand { brand: String },
    Discarded { id: ResourceLocation, data: Vec<u8> },
}

impl CustomPacketPayload {
    // JAVA: net.minecraft.network.protocol.common.custom.CustomPacketPayload#type
    pub fn payload_type(&self) -> ResourceLocation {
        match self {
            CustomPacketPayload::Brand { .. } => ResourceLocation::new("minecraft", "brand"),
            CustomPacketPayload::Discarded { id, .. } => id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ServerCommonPacketListener {
    fn handle_custom_payload(&mut self, packet: &ServerboundCustomPayloadPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCustomPayloadPacket {
    pub payload: CustomPacketPayload,
}

impl ServerboundCustomPayloadPacket {
    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket#<init>
    pub fn new(payload: CustomPacketPayload) -> Self {
        Self { payload }
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket#payload
    pub fn payload(&self) -> &CustomPacketPayload {
        &self.payload
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:custom_payload")
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerCommonPacketListener) {
        listener.handle_custom_payload(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_payload_size_constant() {
        assert_eq!(MAX_PAYLOAD_SIZE, 32_767_i32);
    }

    #[test]
    fn brand_payload_fields_and_type() {
        let pkt = ServerboundCustomPayloadPacket::new(CustomPacketPayload::Brand {
            brand: "vanilla".to_string(),
        });
        match pkt.payload() {
            CustomPacketPayload::Brand { brand } => assert_eq!(brand, "vanilla"),
            _ => panic!("expected brand payload"),
        }
        assert_eq!(pkt.get_type(), PacketType("minecraft:custom_payload"));
        assert_eq!(
            pkt.payload().payload_type(),
            ResourceLocation::new("minecraft", "brand")
        );
    }

    #[test]
    fn discarded_payload_preserves_id_and_data() {
        let data = vec![0_u8, 127_u8, 255_u8];
        let pkt = ServerboundCustomPayloadPacket::new(CustomPacketPayload::Discarded {
            id: ResourceLocation::new("mymod", "channel"),
            data: data.clone(),
        });
        match pkt.payload() {
            CustomPacketPayload::Discarded { id, data: d } => {
                assert_eq!(id.namespace, "mymod");
                assert_eq!(id.path, "channel");
                assert_eq!(d, &data);
            }
            _ => panic!("expected discarded payload"),
        }
    }

    #[test]
    fn discarded_empty_data_and_negative_like_bytes() {
        let pkt = ServerboundCustomPayloadPacket::new(CustomPacketPayload::Discarded {
            id: ResourceLocation::new("minecraft", "empty"),
            data: Vec::new(),
        });
        match pkt.payload() {
            CustomPacketPayload::Discarded { data, .. } => assert!(data.is_empty()),
            _ => panic!("expected discarded payload"),
        }
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct MockListener {
            seen_brand: Option<String>,
        }
        impl ServerCommonPacketListener for MockListener {
            fn handle_custom_payload(&mut self, packet: &ServerboundCustomPayloadPacket) {
                if let CustomPacketPayload::Brand { brand } = packet.payload() {
                    self.seen_brand = Some(brand.clone());
                }
            }
        }

        let pkt = ServerboundCustomPayloadPacket::new(CustomPacketPayload::Brand {
            brand: "test".into(),
        });
        let mut listener = MockListener { seen_brand: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen_brand.as_deref(), Some("test"));
    }
}
