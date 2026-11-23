use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    pub message: String,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecodeError {}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_varint(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(DecodeError {
                message: "VarInt too short".into(),
            });
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32) << (shift & 31);
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
        if shift >= 35 {
            return Err(DecodeError {
                message: "VarInt too big".into(),
            });
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_varint(buf: &mut BytesMut, value: i32) {
    let mut val = value as u32;
    loop {
        if (val & !0x7F) == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8(((val & 0x7F) as u8) | 0x80);
        val >>= 7;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
fn read_boolean(buf: &mut &[u8]) -> Result<bool, DecodeError> {
    if !buf.has_remaining() {
        return Err(DecodeError {
            message: "Boolean too short".into(),
        });
    }
    Ok(buf.get_u8() != 0)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
fn write_boolean(buf: &mut BytesMut, value: bool) {
    buf.put_u8(if value { 1 } else { 0 });
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readContainerId
fn read_container_id(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    read_varint(buf)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeContainerId
fn write_container_id(buf: &mut BytesMut, value: i32) {
    write_varint(buf, value);
}

/// Self-contained stub for `net.minecraft.world.item.trading.MerchantOffers`.
/// Stores an opaque encoded payload so the packet can round-trip without
/// depending on unported ItemStack/ItemCost codecs.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MerchantOffers {
    pub payload: Vec<u8>,
}

impl MerchantOffers {
    // JAVA: net.minecraft.world.item.trading.MerchantOffers#copy
    pub fn copy(&self) -> Self {
        self.clone()
    }

    // JAVA: net.minecraft.world.item.trading.MerchantOffers#<streamDecode>
    pub fn decode(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let len = read_varint(buf)?;
        if len < 0 {
            return Err(DecodeError {
                message: "MerchantOffers length negative".into(),
            });
        }
        let len = len as usize;
        if buf.remaining() < len {
            return Err(DecodeError {
                message: "MerchantOffers payload too short".into(),
            });
        }
        let payload = buf[..len].to_vec();
        buf.advance(len);
        Ok(Self { payload })
    }

    // JAVA: net.minecraft.world.item.trading.MerchantOffers#<streamEncode>
    pub fn encode(&self, buf: &mut BytesMut) {
        write_varint(buf, self.payload.len() as i32);
        buf.put_slice(&self.payload);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_MERCHANT_OFFERS_TYPE: PacketType =
    PacketType("minecraft:clientbound_merchant_offers");

pub trait ClientGamePacketListener {
    fn handle_merchant_offers(&mut self, packet: &ClientboundMerchantOffersPacket);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundMerchantOffersPacket {
    container_id: i32,
    offers: MerchantOffers,
    villager_level: i32,
    villager_xp: i32,
    show_progress: bool,
    can_restock: bool,
}

impl ClientboundMerchantOffersPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#<init>
    pub fn new(
        container_id: i32,
        offers: MerchantOffers,
        villager_level: i32,
        villager_xp: i32,
        show_progress: bool,
        can_restock: bool,
    ) -> Self {
        Self {
            container_id,
            offers: offers.copy(),
            villager_level,
            villager_xp,
            show_progress,
            can_restock,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let container_id = read_container_id(buf)?;
        let offers = MerchantOffers::decode(buf)?;
        let villager_level = read_varint(buf)?;
        let villager_xp = read_varint(buf)?;
        let show_progress = read_boolean(buf)?;
        let can_restock = read_boolean(buf)?;
        Ok(Self {
            container_id,
            offers,
            villager_level,
            villager_xp,
            show_progress,
            can_restock,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#write
    pub fn write(&self, buf: &mut BytesMut) {
        write_container_id(buf, self.container_id);
        self.offers.encode(buf);
        write_varint(buf, self.villager_level);
        write_varint(buf, self.villager_xp);
        write_boolean(buf, self.show_progress);
        write_boolean(buf, self.can_restock);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_MERCHANT_OFFERS_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_merchant_offers(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#getContainerId
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#getOffers
    pub fn get_offers(&self) -> &MerchantOffers {
        &self.offers
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#getVillagerLevel
    pub fn get_villager_level(&self) -> i32 {
        self.villager_level
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#getVillagerXp
    pub fn get_villager_xp(&self) -> i32 {
        self.villager_xp
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#showProgress
    pub fn show_progress(&self) -> bool {
        self.show_progress
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket#canRestock
    pub fn can_restock(&self) -> bool {
        self.can_restock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(
        packet: &ClientboundMerchantOffersPacket,
    ) -> ClientboundMerchantOffersPacket {
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf[..];
        let decoded = ClientboundMerchantOffersPacket::read(&mut slice).unwrap();
        assert!(slice.is_empty());
        decoded
    }

    #[test]
    fn round_trip_basic() {
        let offers = MerchantOffers {
            payload: vec![1, 2, 3, 4],
        };
        let packet = ClientboundMerchantOffersPacket::new(
            42,
            offers.clone(),
            5,
            100,
            true,
            false,
        );
        let decoded = round_trip(&packet);
        assert_eq!(decoded.get_container_id(), 42);
        assert_eq!(decoded.get_offers(), &offers);
        assert_eq!(decoded.get_villager_level(), 5);
        assert_eq!(decoded.get_villager_xp(), 100);
        assert!(decoded.show_progress());
        assert!(!decoded.can_restock());
    }

    #[test]
    fn constructor_copies_offers() {
        let offers = MerchantOffers {
            payload: vec![9, 8, 7],
        };
        let packet = ClientboundMerchantOffersPacket::new(
            1,
            offers.clone(),
            1,
            0,
            false,
            false,
        );
        assert_eq!(packet.get_offers(), &offers);
        assert_eq!(offers.payload, vec![9, 8, 7]);
    }

    #[test]
    fn negative_and_extreme_ints() {
        let offers = MerchantOffers { payload: vec![] };
        let packet = ClientboundMerchantOffersPacket::new(
            -1,
            offers,
            i32::MIN,
            i32::MAX,
            false,
            true,
        );
        let decoded = round_trip(&packet);
        assert_eq!(decoded.get_container_id(), -1);
        assert_eq!(decoded.get_villager_level(), i32::MIN);
        assert_eq!(decoded.get_villager_xp(), i32::MAX);
        assert!(!decoded.show_progress());
        assert!(decoded.can_restock());
    }

    #[test]
    fn empty_offers_round_trip() {
        let packet = ClientboundMerchantOffersPacket::new(
            0,
            MerchantOffers::default(),
            0,
            0,
            false,
            false,
        );
        let decoded = round_trip(&packet);
        assert!(decoded.get_offers().payload.is_empty());
    }

    #[test]
    fn packet_type_and_handle() {
        let packet = ClientboundMerchantOffersPacket::new(
            7,
            MerchantOffers::default(),
            1,
            2,
            true,
            true,
        );
        assert_eq!(packet.packet_type(), CLIENTBOUND_MERCHANT_OFFERS_TYPE);

        struct MockListener(Option<i32>);
        impl ClientGamePacketListener for MockListener {
            fn handle_merchant_offers(
                &mut self,
                packet: &ClientboundMerchantOffersPacket,
            ) {
                self.0 = Some(packet.get_container_id());
            }
        }

        let mut listener = MockListener(None);
        packet.handle(&mut listener);
        assert_eq!(listener.0, Some(7));
    }
}
