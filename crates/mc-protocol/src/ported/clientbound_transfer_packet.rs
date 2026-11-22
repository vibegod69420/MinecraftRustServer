use std::io;

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    reader_index: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            reader_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.reader_index)
    }

    fn read_exact(&mut self, dst: &mut [u8]) -> io::Result<()> {
        if self.remaining() < dst.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_exact"));
        }
        let start = self.reader_index;
        let end = start + dst.len();
        dst.copy_from_slice(&self.data[start..end]);
        self.reader_index = end;
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte
    pub fn read_byte(&mut self) -> io::Result<u8> {
        let mut b = [0u8; 1];
        self.read_exact(&mut b)?;
        Ok(b[0])
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let b = self.read_byte()? as i32;
            value |= (b & 0x7f) << position;
            if (b & 0x80) == 0 {
                break;
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
            }
        }
        Ok(value)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> io::Result<()> {
        loop {
            if (value as u32 & !0x7fu32) == 0 {
                self.data.push(value as u8);
                return Ok(());
            }
            self.data.push(((value as u32 & 0x7f) as u8) | 0x80);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
    pub fn read_utf(&mut self) -> io::Result<String> {
        self.read_utf_max(32767)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
    pub fn read_utf_max(&mut self, max_length: usize) -> io::Result<String> {
        let max_bytes = max_length.saturating_mul(3);
        let byte_len = self.read_var_int()?;
        if byte_len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Negative length",
            ));
        }
        let byte_len = byte_len as usize;
        if byte_len > max_bytes {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "String too long"));
        }
        if self.remaining() < byte_len {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_utf"));
        }
        let start = self.reader_index;
        let end = start + byte_len;
        let bytes = self.data[start..end].to_vec();
        self.reader_index = end;
        let s = String::from_utf8(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if s.chars().count() > max_length {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "String too long"));
        }
        Ok(s)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
    pub fn write_utf(&mut self, value: &str) -> io::Result<()> {
        self.write_utf_max(value, 32767)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
    pub fn write_utf_max(&mut self, value: &str, max_length: usize) -> io::Result<()> {
        if value.chars().count() > max_length {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "String too long"));
        }
        let bytes = value.as_bytes();
        if bytes.len() > max_length.saturating_mul(3) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "String too long"));
        }
        self.write_var_int(bytes.len() as i32)?;
        self.data.extend_from_slice(bytes);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub const CLIENTBOUND_TRANSFER: PacketType = PacketType {
    id: "minecraft:common/transfer",
};

pub trait ClientCommonPacketListener {
    fn handle_transfer(&mut self, packet: &ClientboundTransferPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTransferPacket {
    pub host: String,
    pub port: i32,
}

impl ClientboundTransferPacket {
    // JAVA: net.minecraft.network.protocol.common.ClientboundTransferPacket#<init>
    pub fn new(host: String, port: i32) -> Self {
        Self { host, port }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundTransferPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let host = buf.read_utf()?;
        let port = buf.read_var_int()?;
        Ok(Self { host, port })
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundTransferPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_utf(&self.host)?;
        buf.write_var_int(self.port)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundTransferPacket#type
    pub fn type_(&self) -> PacketType {
        CLIENTBOUND_TRANSFER
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundTransferPacket#handle
    pub fn handle<L: ClientCommonPacketListener>(&self, listener: &mut L) {
        listener.handle_transfer(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_transfer_packet() {
        let pkt = ClientboundTransferPacket::new("example.com".to_string(), 25565);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundTransferPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(pkt.type_(), CLIENTBOUND_TRANSFER);
    }

    #[test]
    fn transfer_packet_negative_port() {
        let pkt = ClientboundTransferPacket::new("localhost".to_string(), -1);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundTransferPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.port, -1);
    }

    #[test]
    fn transfer_packet_port_extremes() {
        for port in [i32::MIN, i32::MAX, 0] {
            let pkt = ClientboundTransferPacket::new("host".to_string(), port);
            let mut buf = FriendlyByteBuf::new();
            pkt.write(&mut buf).unwrap();
            let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
            let decoded = ClientboundTransferPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.port, port);
        }
    }

    #[test]
    fn transfer_packet_handle_dispatch() {
        struct TestListener {
            called: bool,
            host: String,
            port: i32,
        }
        impl ClientCommonPacketListener for TestListener {
            fn handle_transfer(&mut self, packet: &ClientboundTransferPacket) {
                self.called = true;
                self.host = packet.host.clone();
                self.port = packet.port;
            }
        }
        let pkt = ClientboundTransferPacket::new("mc.example".to_string(), 12345);
        let mut listener = TestListener {
            called: false,
            host: String::new(),
            port: 0,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.host, "mc.example");
        assert_eq!(listener.port, 12345);
    }

    #[test]
    fn varint_negative_one_wire_format() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1).unwrap();
        let bytes = buf.into_inner();
        assert_eq!(bytes, vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        let mut r = FriendlyByteBuf::from_bytes(bytes);
        assert_eq!(r.read_var_int().unwrap(), -1);
    }
}
