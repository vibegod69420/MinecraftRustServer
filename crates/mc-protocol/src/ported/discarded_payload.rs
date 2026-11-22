#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn minecraft(path: impl Into<String>) -> Self {
        Self::new("minecraft", path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomPacketPayloadType {
    pub id: Identifier,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_index: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            read_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn readable_bytes(&self) -> usize {
        self.data.len().saturating_sub(self.read_index)
    }

    pub fn skip_bytes(&mut self, count: usize) {
        let new_index = self.read_index.saturating_add(count);
        self.read_index = new_index.min(self.data.len());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscardedPayload {
    pub id: Identifier,
}

impl DiscardedPayload {
    // JAVA: net.minecraft.network.protocol.common.custom.DiscardedPayload#type
    pub fn payload_type(&self) -> CustomPacketPayloadType {
        CustomPacketPayloadType {
            id: self.id.clone(),
        }
    }
}

pub struct DiscardedPayloadCodec {
    id: Identifier,
    max_bytes: i32,
}

impl DiscardedPayloadCodec {
    pub fn encode(
        &self,
        _payload: &DiscardedPayload,
        _buf: &mut FriendlyByteBuf,
    ) -> Result<(), String> {
        Ok(())
    }

    pub fn decode(&self, buf: &mut FriendlyByteBuf) -> Result<DiscardedPayload, String> {
        let readable = buf.readable_bytes() as i32;
        if readable >= 0 && readable <= self.max_bytes {
            buf.skip_bytes(readable as usize);
            Ok(DiscardedPayload {
                id: self.id.clone(),
            })
        } else {
            Err(format!(
                "Payload may not be larger than {} bytes",
                self.max_bytes
            ))
        }
    }
}

// JAVA: net.minecraft.network.protocol.common.custom.DiscardedPayload#codec
pub fn codec(id: Identifier, max_bytes: i32) -> DiscardedPayloadCodec {
    DiscardedPayloadCodec { id, max_bytes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discard_within_limit_skips_all_bytes() {
        let id = Identifier::minecraft("test");
        let c = codec(id.clone(), 10);
        let mut buf = FriendlyByteBuf::from_bytes(vec![1, 2, 3, 4, 5]);
        let payload = c.decode(&mut buf).unwrap();
        assert_eq!(payload.id, id);
        assert_eq!(buf.readable_bytes(), 0);
    }

    #[test]
    fn discard_exact_limit_is_allowed() {
        let id = Identifier::minecraft("test");
        let c = codec(id.clone(), 3);
        let mut buf = FriendlyByteBuf::from_bytes(vec![9, 8, 7]);
        let payload = c.decode(&mut buf).unwrap();
        assert_eq!(payload.id, id);
    }

    #[test]
    fn discard_over_limit_errors() {
        let id = Identifier::minecraft("test");
        let c = codec(id, 2);
        let mut buf = FriendlyByteBuf::from_bytes(vec![1, 2, 3]);
        let err = c.decode(&mut buf).unwrap_err();
        assert_eq!(err, "Payload may not be larger than 2 bytes");
    }

    #[test]
    fn discard_negative_max_rejects_bytes() {
        let id = Identifier::minecraft("test");
        let c = codec(id, -1);
        let mut non_empty = FriendlyByteBuf::from_bytes(vec![1]);
        assert!(c.decode(&mut non_empty).is_err());

        let c_zero = codec(Identifier::minecraft("test"), -1);
        let mut empty = FriendlyByteBuf::from_bytes(vec![]);
        let err = c_zero.decode(&mut empty).unwrap_err();
        assert_eq!(err, "Payload may not be larger than -1 bytes");
    }

    #[test]
    fn discard_type_uses_payload_id() {
        let id = Identifier::new("custom", "my_payload");
        let payload = DiscardedPayload { id: id.clone() };
        let ty = payload.payload_type();
        assert_eq!(ty.id, id);
    }

    #[test]
    fn discard_encode_is_noop() {
        let id = Identifier::minecraft("x");
        let payload = DiscardedPayload { id: id.clone() };
        let c = codec(id, 100);
        let mut buf = FriendlyByteBuf::new();
        c.encode(&payload, &mut buf).unwrap();
        assert_eq!(buf.into_inner(), Vec::<u8>::new());
    }
}
