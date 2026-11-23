use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketType {
    ClientboundDebugSample,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            read_pos: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_varint(&mut self) -> io::Result<i32> {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_u8()? as i32;
            let shift = j.wrapping_mul(7) & 31;
            i |= (b0 & 127) << shift;
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if (b0 & 128) == 0 {
                break;
            }
        }
        Ok(i)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_varint(&mut self, mut value: i32) -> io::Result<()> {
        loop {
            if (value & !0x7F) == 0 {
                self.data.push(value as u8);
                return Ok(());
            }
            self.data.push(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong
    pub fn read_long(&mut self) -> io::Result<i64> {
        if self.data.len().saturating_sub(self.read_pos) < 8 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 8]);
        self.read_pos += 8;
        Ok(i64::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
    pub fn write_long(&mut self, value: i64) -> io::Result<()> {
        self.data.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLongArray
    pub fn read_long_array(&mut self) -> io::Result<Vec<i64>> {
        let len = self.read_varint()?;
        if len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative long array length",
            ));
        }
        let len = len as usize;
        let mut arr = Vec::with_capacity(len);
        for _ in 0..len {
            arr.push(self.read_long()?);
        }
        Ok(arr)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLongArray
    pub fn write_long_array(&mut self, array: &[i64]) -> io::Result<()> {
        self.write_varint(array.len() as i32)?;
        for &v in array {
            self.write_long(v)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteDebugSampleType {
    TickTime,
}

impl RemoteDebugSampleType {
    // JAVA: net.minecraft.util.debugchart.RemoteDebugSampleType#values
    pub fn from_ordinal(ordinal: i32) -> io::Result<Self> {
        match ordinal {
            0 => Ok(Self::TickTime),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown RemoteDebugSampleType ordinal",
            )),
        }
    }

    // JAVA: net.minecraft.util.debugchart.RemoteDebugSampleType#ordinal
    pub fn ordinal(self) -> i32 {
        match self {
            Self::TickTime => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundDebugSamplePacket {
    pub sample: Vec<i64>,
    pub debug_sample_type: RemoteDebugSampleType,
}

pub trait ClientGamePacketListener {
    fn handle_debug_sample(&mut self, packet: &ClientboundDebugSamplePacket);
}

impl ClientboundDebugSamplePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugSamplePacket#<init>
    pub fn new(sample: Vec<i64>, debug_sample_type: RemoteDebugSampleType) -> Self {
        Self {
            sample,
            debug_sample_type,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugSamplePacket#sample
    pub fn sample(&self) -> &[i64] {
        &self.sample
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugSamplePacket#debugSampleType
    pub fn debug_sample_type(&self) -> RemoteDebugSampleType {
        self.debug_sample_type
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugSamplePacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let sample = buf.read_long_array()?;
        let debug_sample_type = RemoteDebugSampleType::from_ordinal(buf.read_varint()?)?;
        Ok(Self::new(sample, debug_sample_type))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugSamplePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_long_array(&self.sample)?;
        buf.write_varint(self.debug_sample_type.ordinal())?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugSamplePacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ClientboundDebugSample
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundDebugSamplePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_debug_sample(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let pkt = ClientboundDebugSamplePacket::new(
            vec![1, -2, i64::MAX, i64::MIN],
            RemoteDebugSampleType::TickTime,
        );
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundDebugSamplePacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn empty_sample_round_trip() {
        let pkt = ClientboundDebugSamplePacket::new(vec![], RemoteDebugSampleType::TickTime);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundDebugSamplePacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert!(decoded.sample().is_empty());
    }

    #[test]
    fn unknown_enum_ordinal_errors() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_long_array(&[42]).unwrap();
        buf.write_varint(99).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let err = ClientboundDebugSamplePacket::read(&mut read_buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn type_and_handle() {
        let pkt = ClientboundDebugSamplePacket::new(vec![7], RemoteDebugSampleType::TickTime);
        assert_eq!(pkt.packet_type(), PacketType::ClientboundDebugSample);

        struct Listener(Option<ClientboundDebugSamplePacket>);
        impl ClientGamePacketListener for Listener {
            fn handle_debug_sample(&mut self, packet: &ClientboundDebugSamplePacket) {
                self.0 = Some(packet.clone());
            }
        }
        let mut listener = Listener(None);
        pkt.handle(&mut listener);
        assert_eq!(listener.0, Some(pkt));
    }
}
