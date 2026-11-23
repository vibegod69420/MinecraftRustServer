use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptException;

impl fmt::Display for CryptException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CryptException")
    }
}

impl std::error::Error for CryptException {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace(String)
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType {
    pub flow: PacketFlow,
    pub id: Identifier,
}

impl PacketType {
    // JAVA: net.minecraft.network.protocol.PacketType#<init>(PacketFlow, Identifier)
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self { flow, id }
    }
}

#[derive(Debug, Clone)]
pub struct FriendlyByteBuf {
    buf: Vec<u8>,
    rpos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>()
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            rpos: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>(byte[])
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { buf: bytes, rpos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.buf[self.rpos];
        self.rpos += 1;
        b
    }

    fn write_byte(&mut self, b: u8) {
        self.buf.push(b);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
    pub fn read_var_int(&mut self) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b = self.read_byte() as i32;
            i |= (b & 127) << (j * 7);
            j += 1;
            if j > 5 {
                panic!("VarInt too big");
            }
            if (b & 128) != 128 {
                return i;
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F) == 0 {
                self.write_byte(value as u8);
                return;
            }
            self.write_byte(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByteArray()
    pub fn read_byte_array(&mut self) -> Vec<u8> {
        let len = self.read_var_int() as usize;
        let bytes = self.buf[self.rpos..self.rpos + len].to_vec();
        self.rpos += len;
        bytes
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByteArray(byte[])
    pub fn write_byte_array(&mut self, data: &[u8]) {
        self.write_var_int(data.len() as i32);
        self.buf.extend_from_slice(data);
    }
}

impl Default for FriendlyByteBuf {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundKeyPacket {
    pub keybytes: Vec<u8>,
    pub encrypted_challenge: Vec<u8>,
}

impl ServerboundKeyPacket {
    // JAVA: net.minecraft.network.protocol.login.ServerboundKeyPacket#<init>(SecretKey, PublicKey, byte[])
    pub fn new_encrypted<F>(
        secret_key_encoded: &[u8],
        public_key: &[u8],
        challenge: &[u8],
        encrypt: F,
    ) -> Result<Self, CryptException>
    where
        F: Fn(&[u8], &[u8]) -> Result<Vec<u8>, CryptException>,
    {
        let keybytes = encrypt(public_key, secret_key_encoded)?;
        let encrypted_challenge = encrypt(public_key, challenge)?;
        Ok(Self {
            keybytes,
            encrypted_challenge,
        })
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundKeyPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let keybytes = buf.read_byte_array();
        let encrypted_challenge = buf.read_byte_array();
        Self {
            keybytes,
            encrypted_challenge,
        }
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundKeyPacket#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_byte_array(&self.keybytes);
        buf.write_byte_array(&self.encrypted_challenge);
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundKeyPacket#type()
    pub fn get_type(&self) -> PacketType {
        PacketType::new(
            PacketFlow::Serverbound,
            Identifier::with_default_namespace("key"),
        )
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundKeyPacket#handle(ServerLoginPacketListener)
    pub fn handle<L: ServerLoginPacketListener>(&self, listener: &mut L) {
        listener.handle_key(self);
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundKeyPacket#getSecretKey(PrivateKey)
    pub fn get_secret_key<F>(
        &self,
        private_key: &[u8],
        decrypt: F,
    ) -> Result<Vec<u8>, CryptException>
    where
        F: FnOnce(&[u8], &[u8]) -> Result<Vec<u8>, CryptException>,
    {
        decrypt(private_key, &self.keybytes)
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundKeyPacket#isChallengeValid(byte[], PrivateKey)
    pub fn is_challenge_valid<F>(
        &self,
        challenge: &[u8],
        private_key: &[u8],
        decrypt: F,
    ) -> bool
    where
        F: FnOnce(&[u8], &[u8]) -> Result<Vec<u8>, CryptException>,
    {
        match decrypt(private_key, &self.encrypted_challenge) {
            Ok(decrypted) => decrypted.as_slice() == challenge,
            Err(_) => false,
        }
    }
}

pub trait ServerLoginPacketListener {
    // JAVA: net.minecraft.network.protocol.login.ServerLoginPacketListener#handleKey(ServerboundKeyPacket)
    fn handle_key(&mut self, packet: &ServerboundKeyPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_key_packet() {
        let packet = ServerboundKeyPacket {
            keybytes: vec![1, 2, 3],
            encrypted_challenge: vec![4, 5, 6, 7],
        };
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ServerboundKeyPacket::read(&mut read_buf);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn new_encrypted_uses_encryptor() {
        let packet = ServerboundKeyPacket::new_encrypted(&[1], &[2], &[3], |key, data| {
            let mut out = key.to_vec();
            out.extend_from_slice(data);
            Ok(out)
        })
        .unwrap();
        assert_eq!(packet.keybytes, vec![2, 1]);
        assert_eq!(packet.encrypted_challenge, vec![2, 3]);
    }

    #[test]
    fn is_challenge_valid_success_and_mismatch() {
        let packet = ServerboundKeyPacket {
            keybytes: vec![10],
            encrypted_challenge: vec![99],
        };
        let ok = packet.is_challenge_valid(&[1, 2, 3], &[0], |_key, _data| Ok(vec![1, 2, 3]));
        assert!(ok);
        let bad = packet.is_challenge_valid(&[1, 2, 3], &[0], |_key, _data| Ok(vec![9, 9, 9]));
        assert!(!bad);
    }

    #[test]
    fn is_challenge_valid_decrypt_error() {
        let packet = ServerboundKeyPacket {
            keybytes: vec![10],
            encrypted_challenge: vec![99],
        };
        let ok = packet.is_challenge_valid(&[1, 2, 3], &[0], |_key, _data| Err(CryptException));
        assert!(!ok);
    }

    #[test]
    fn get_secret_key_with_decrypt() {
        let packet = ServerboundKeyPacket {
            keybytes: vec![42, 43],
            encrypted_challenge: vec![],
        };
        let secret = packet
            .get_secret_key(&[7], |key, data| {
                assert_eq!(key, &[7]);
                assert_eq!(data, &[42, 43]);
                Ok(vec![100, 101])
            })
            .unwrap();
        assert_eq!(secret, vec![100, 101]);
    }

    #[test]
    fn packet_type_is_serverbound_key() {
        let packet = ServerboundKeyPacket {
            keybytes: vec![],
            encrypted_challenge: vec![],
        };
        let t = packet.get_type();
        assert_eq!(t.flow, PacketFlow::Serverbound);
        assert_eq!(t.id, Identifier::with_default_namespace("key"));
    }
}
