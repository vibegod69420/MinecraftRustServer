use std::io;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_byte(&mut self) -> io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    pub fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        for shift in (0..32).step_by(7) {
            let b = self.read_byte()?;
            result |= ((b & 0x7F) as i32) << shift;
            if (b & 0x80) == 0 {
                return Ok(result);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VarInt too big",
        ))
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7F) == 0 {
                self.write_byte(val as u8);
                return;
            }
            self.write_byte(((val & 0x7F) as u8) | 0x80);
            val >>= 7;
        }
    }
}

#[derive(Debug)]
pub struct RegistryFriendlyByteBuf {
    inner: FriendlyByteBuf,
}

impl RegistryFriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            inner: FriendlyByteBuf::new(data),
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.inner.into_inner()
    }

    pub fn remaining(&self) -> usize {
        self.inner.remaining()
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        self.inner.read_var_int()
    }

    pub fn write_var_int(&mut self, value: i32) {
        self.inner.write_var_int(value);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    id: i32,
    _marker: PhantomData<T>,
}

impl<T> Holder<T> {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}

pub trait Level {
    type EntityType;
    fn get_entity(&self, entity_id: i32) -> Option<&Self::EntityType>;
}

pub trait ClientGamePacketListener {
    fn handle_remove_mob_effect(&mut self, packet: &ClientboundRemoveMobEffectPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRemoveMobEffectPacket {
    pub entity_id: i32,
    pub effect: Holder<MobEffect>,
}

impl ClientboundRemoveMobEffectPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveMobEffectPacket#<init>
    pub fn new(entity_id: i32, effect: Holder<MobEffect>) -> Self {
        Self { entity_id, effect }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveMobEffectPacket#STREAM_CODEC
    pub fn decode(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let entity_id = buf.read_var_int()?;
        let effect_id = buf.read_var_int()?;
        Ok(Self::new(entity_id, Holder::new(effect_id)))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveMobEffectPacket#STREAM_CODEC
    pub fn encode(&self, buf: &mut RegistryFriendlyByteBuf) -> io::Result<()> {
        buf.write_var_int(self.entity_id);
        buf.write_var_int(self.effect.id());
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveMobEffectPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:remove_mob_effect")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveMobEffectPacket#handle
    pub fn handle<H: ClientGamePacketListener>(&self, handler: &mut H) {
        handler.handle_remove_mob_effect(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveMobEffectPacket#getEntity
    pub fn get_entity<'a, L: Level>(&self, level: &'a L) -> Option<&'a L::EntityType> {
        level.get_entity(self.entity_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let pkt = ClientboundRemoveMobEffectPacket::new(42, Holder::new(7));
        let mut buf = RegistryFriendlyByteBuf::new(Vec::new());
        pkt.encode(&mut buf).unwrap();
        let mut reader = RegistryFriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundRemoveMobEffectPacket::decode(&mut reader).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn negative_and_extreme_ids_roundtrip() {
        let pkt = ClientboundRemoveMobEffectPacket::new(i32::MIN, Holder::new(i32::MAX));
        let mut buf = RegistryFriendlyByteBuf::new(Vec::new());
        pkt.encode(&mut buf).unwrap();
        let mut reader = RegistryFriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundRemoveMobEffectPacket::decode(&mut reader).unwrap();
        assert_eq!(decoded.entity_id, i32::MIN);
        assert_eq!(decoded.effect.id(), i32::MAX);
    }

    #[test]
    fn get_entity_found_and_missing() {
        #[derive(Debug, PartialEq, Eq)]
        struct E(i32);
        struct L {
            entities: Vec<E>,
        }
        impl Level for L {
            type EntityType = E;
            fn get_entity(&self, entity_id: i32) -> Option<&Self::EntityType> {
                self.entities.iter().find(|e| e.0 == entity_id)
            }
        }

        let level = L {
            entities: vec![E(10), E(20)],
        };
        let pkt = ClientboundRemoveMobEffectPacket::new(20, Holder::new(1));
        assert_eq!(pkt.get_entity(&level), Some(&E(20)));

        let missing = ClientboundRemoveMobEffectPacket::new(99, Holder::new(1));
        assert_eq!(missing.get_entity(&level), None);
    }

    #[test]
    fn handle_and_type() {
        let pkt = ClientboundRemoveMobEffectPacket::new(5, Holder::new(2));
        assert_eq!(
            pkt.packet_type(),
            PacketType("minecraft:remove_mob_effect")
        );

        struct H {
            entity_id: i32,
        }
        impl ClientGamePacketListener for H {
            fn handle_remove_mob_effect(&mut self, p: &ClientboundRemoveMobEffectPacket) {
                self.entity_id = p.entity_id;
            }
        }

        let mut handler = H { entity_id: 0 };
        pkt.handle(&mut handler);
        assert_eq!(handler.entity_id, 5);
    }
}
