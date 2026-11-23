use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError(pub String);

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DecodeError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    reader_index: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            reader_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readableBytes
    pub fn readable_bytes(&self) -> i32 {
        self.data.len().saturating_sub(self.reader_index) as i32
    }

    fn read_byte(&mut self) -> Result<u8, DecodeError> {
        if self.reader_index >= self.data.len() {
            return Err(DecodeError("Not enough bytes".into()));
        }
        let b = self.data[self.reader_index];
        self.reader_index += 1;
        Ok(b)
    }

    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> Result<i32, DecodeError> {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let b = self.read_byte()? as i32;
            value |= (b & 0x7F).wrapping_shl(position);
            if (b & 0x80) == 0 {
                return Ok(value);
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                return Err(DecodeError("VarInt too big".into()));
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7Fu32) == 0 {
                self.write_byte(val as u8);
                return;
            }
            self.write_byte(((val & 0x7F) as u8) | 0x80);
            val >>= 7;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#skipBytes
    pub fn skip_bytes(&mut self, length: i32) {
        let n = if length < 0 { 0usize } else { length as usize };
        self.reader_index = self.reader_index.saturating_add(n).min(self.data.len());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_boolean(&mut self, value: bool) {
        self.write_byte(if value { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeNullable
    pub fn write_nullable<T, F>(&mut self, value: Option<&T>, writer: F)
    where
        F: FnOnce(&mut Self, &T),
    {
        match value {
            Some(v) => {
                self.write_boolean(true);
                writer(self, v);
            }
            None => {
                self.write_boolean(false);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscardedQueryAnswerPayload;

impl DiscardedQueryAnswerPayload {
    pub const INSTANCE: Self = Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomQueryAnswerPayload {
    Discarded(DiscardedQueryAnswerPayload),
}

impl CustomQueryAnswerPayload {
    // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload#write
    pub fn write(&self, _buf: &mut FriendlyByteBuf) {
        match self {
            CustomQueryAnswerPayload::Discarded(_) => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub struct LoginPacketTypes;

impl LoginPacketTypes {
    pub const SERVERBOUND_CUSTOM_QUERY_ANSWER: PacketType =
        PacketType("minecraft:custom_query_answer");
}

pub struct StreamCodec<V> {
    pub encode: fn(&V, &mut FriendlyByteBuf),
    pub decode: fn(&mut FriendlyByteBuf) -> Result<V, DecodeError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCustomQueryAnswerPacket {
    pub transaction_id: i32,
    pub payload: Option<CustomQueryAnswerPayload>,
}

pub trait ServerLoginPacketListener {
    // JAVA: net.minecraft.network.protocol.login.ServerLoginPacketListener#handleCustomQueryPacket
    fn handle_custom_query_packet(&mut self, packet: &ServerboundCustomQueryAnswerPacket);
}

impl ServerboundCustomQueryAnswerPacket {
    pub const MAX_PAYLOAD_SIZE: i32 = 1_048_576;

    pub const STREAM_CODEC: StreamCodec<Self> = StreamCodec {
        encode: Self::write,
        decode: Self::read,
    };

    // JAVA: net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket#read
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, DecodeError> {
        let transaction_id = buf.read_var_int()?;
        let payload = Self::read_payload(transaction_id, buf)?;
        Ok(Self {
            transaction_id,
            payload: Some(payload),
        })
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket#readPayload
    fn read_payload(
        _transaction_id: i32,
        buf: &mut FriendlyByteBuf,
    ) -> Result<CustomQueryAnswerPayload, DecodeError> {
        Self::read_unknown_payload(buf)
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket#readUnknownPayload
    fn read_unknown_payload(
        buf: &mut FriendlyByteBuf,
    ) -> Result<CustomQueryAnswerPayload, DecodeError> {
        let readable = buf.readable_bytes();
        if readable >= 0 && readable <= Self::MAX_PAYLOAD_SIZE {
            buf.skip_bytes(readable);
            Ok(CustomQueryAnswerPayload::Discarded(
                DiscardedQueryAnswerPayload::INSTANCE,
            ))
        } else {
            Err(DecodeError(
                "Payload may not be larger than 1048576 bytes".to_string(),
            ))
        }
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.transaction_id);
        buf.write_nullable(self.payload.as_ref(), |b, p| p.write(b));
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket#type
    pub fn get_type(&self) -> PacketType {
        LoginPacketTypes::SERVERBOUND_CUSTOM_QUERY_ANSWER
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerLoginPacketListener) {
        listener.handle_custom_query_packet(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_with_discarded_payload() {
        let packet = ServerboundCustomQueryAnswerPacket {
            transaction_id: 42,
            payload: Some(CustomQueryAnswerPayload::Discarded(
                DiscardedQueryAnswerPayload::INSTANCE,
            )),
        };
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundCustomQueryAnswerPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.transaction_id, 42);
        assert_eq!(
            decoded.payload,
            Some(CustomQueryAnswerPayload::Discarded(
                DiscardedQueryAnswerPayload::INSTANCE
            ))
        );
    }

    #[test]
    fn write_null_payload_writes_false_boolean() {
        let packet = ServerboundCustomQueryAnswerPacket {
            transaction_id: 1,
            payload: None,
        };
        let mut buf = FriendlyByteBuf::default();
        packet.write(&mut buf);
        let bytes = buf.into_inner();
        // varint 1 = [0x01], nullable false = [0x00]
        assert_eq!(bytes, vec![0x01, 0x00]);
    }

    #[test]
    fn read_empty_remaining_payload() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_var_int(7);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundCustomQueryAnswerPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.transaction_id, 7);
        assert!(matches!(
            decoded.payload,
            Some(CustomQueryAnswerPayload::Discarded(_))
        ));
    }

    #[test]
    fn read_rejects_oversized_payload() {
        let mut data = Vec::new();
        let mut tmp = FriendlyByteBuf::default();
        tmp.write_var_int(0);
        data.extend_from_slice(&tmp.into_inner());
        data.extend(std::iter::repeat_n(0u8, 1_048_577));
        let mut buf = FriendlyByteBuf::new(data);
        let err = ServerboundCustomQueryAnswerPacket::read(&mut buf).unwrap_err();
        assert_eq!(err.0, "Payload may not be larger than 1048576 bytes");
    }

    #[test]
    fn negative_and_extreme_transaction_ids_round_trip() {
        for id in [i32::MIN, -1, 0, i32::MAX] {
            let packet = ServerboundCustomQueryAnswerPacket {
                transaction_id: id,
                payload: Some(CustomQueryAnswerPayload::Discarded(
                    DiscardedQueryAnswerPayload::INSTANCE,
                )),
            };
            let mut buf = FriendlyByteBuf::default();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ServerboundCustomQueryAnswerPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.transaction_id, id);
        }
    }

    #[test]
    fn read_accepts_max_payload_size_boundary() {
        let mut data = Vec::new();
        let mut tmp = FriendlyByteBuf::default();
        tmp.write_var_int(5);
        data.extend_from_slice(&tmp.into_inner());
        data.extend(std::iter::repeat_n(0xABu8, 1_048_576));
        let mut buf = FriendlyByteBuf::new(data);
        let decoded = ServerboundCustomQueryAnswerPacket::read(&mut buf).unwrap();
        assert_eq!(decoded.transaction_id, 5);
        assert!(matches!(
            decoded.payload,
            Some(CustomQueryAnswerPayload::Discarded(_))
        ));
        assert_eq!(buf.readable_bytes(), 0);
    }
}
