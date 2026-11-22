use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most: i64,
    pub least: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_PLAYER_INFO_REMOVE_TYPE: PacketType =
    PacketType("minecraft:clientbound_player_info_remove");

pub trait ClientGamePacketListener {
    fn handle_player_info_remove(&mut self, packet: &ClientboundPlayerInfoRemovePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    buf: Vec<u8>,
    rpos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            rpos: 0,
        }
    }

    pub fn from_vec(buf: Vec<u8>) -> Self {
        Self { buf, rpos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.rpos)
    }

    fn ensure(&self, n: usize) -> io::Result<()> {
        if self.remaining() < n {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "buffer underflow"))
        } else {
            Ok(())
        }
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.ensure(1)?;
        let v = self.buf[self.rpos];
        self.rpos += 1;
        Ok(v)
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn read_i64(&mut self) -> io::Result<i64> {
        self.ensure(8)?;
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.buf[self.rpos..self.rpos + 8]);
        self.rpos += 8;
        Ok(i64::from_be_bytes(bytes))
    }

    pub fn write_i64(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_u8()?;
            value |= ((current_byte & 0x7F) as i32) << position;
            if (current_byte & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
        }
        Ok(value)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            let mut temp = (value as u32 & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if (value as u32) != 0 {
                temp |= 0x80;
            }
            self.write_u8(temp);
            if (value as u32) == 0 {
                break;
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUUID()
    pub fn read_uuid(&mut self) -> io::Result<Uuid> {
        let most = self.read_i64()?;
        let least = self.read_i64()?;
        Ok(Uuid { most, least })
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUUID(UUID)
    pub fn write_uuid(&mut self, uuid: Uuid) {
        self.write_i64(uuid.most);
        self.write_i64(uuid.least);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readList(StreamCodec)
    pub fn read_list<T, F>(&mut self, mut reader: F) -> io::Result<Vec<T>>
    where
        F: FnMut(&mut Self) -> io::Result<T>,
    {
        let len = self.read_var_int()? as usize;
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(reader(self)?);
        }
        Ok(list)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeCollection(Collection,StreamCodec)
    pub fn write_collection<T, F>(&mut self, values: &[T], mut writer: F)
    where
        F: FnMut(&mut Self, &T),
    {
        self.write_var_int(values.len() as i32);
        for value in values {
            writer(self, value);
        }
    }
}

impl Default for FriendlyByteBuf {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerInfoRemovePacket {
    pub profile_ids: Vec<Uuid>,
}

impl ClientboundPlayerInfoRemovePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoRemovePacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let profile_ids = buf.read_list(|b| b.read_uuid())?;
        Ok(Self { profile_ids })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoRemovePacket#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.profile_ids, |b, uuid| b.write_uuid(*uuid));
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoRemovePacket#type()
    pub fn type_(&self) -> PacketType {
        CLIENTBOUND_PLAYER_INFO_REMOVE_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerInfoRemovePacket#handle(ClientGamePacketListener)
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_player_info_remove(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_empty() {
        let packet = ClientboundPlayerInfoRemovePacket {
            profile_ids: vec![],
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundPlayerInfoRemovePacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn round_trip_multiple_uuids() {
        let packet = ClientboundPlayerInfoRemovePacket {
            profile_ids: vec![
                Uuid {
                    most: 0x0123_4567_89ab_cdef_u64 as i64,
                    least: -1i64,
                },
                Uuid {
                    most: i64::MIN,
                    least: i64::MAX,
                },
            ],
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundPlayerInfoRemovePacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn uuid_wire_format_two_i64() {
        let uuid = Uuid { most: 1, least: 2 };
        let mut buf = FriendlyByteBuf::new();
        buf.write_uuid(uuid);
        let bytes = buf.into_inner();
        assert_eq!(bytes.len(), 16);
        assert_eq!(&bytes[0..8], &1i64.to_be_bytes());
        assert_eq!(&bytes[8..16], &2i64.to_be_bytes());
        let mut read_buf = FriendlyByteBuf::from_vec(bytes);
        assert_eq!(read_buf.read_uuid().unwrap(), uuid);
    }

    #[test]
    fn var_int_round_trip_negative() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        assert_eq!(read_buf.read_var_int().unwrap(), -1);
    }

    #[test]
    fn handle_calls_listener() {
        struct Listener {
            called: bool,
            count: usize,
        }
        impl ClientGamePacketListener for Listener {
            fn handle_player_info_remove(&mut self, packet: &ClientboundPlayerInfoRemovePacket) {
                self.called = true;
                self.count = packet.profile_ids.len();
            }
        }

        let packet = ClientboundPlayerInfoRemovePacket {
            profile_ids: vec![Uuid { most: 5, least: 6 }],
        };
        let mut listener = Listener {
            called: false,
            count: 0,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.count, 1);
        assert_eq!(packet.type_(), CLIENTBOUND_PLAYER_INFO_REMOVE_TYPE);
    }
}
