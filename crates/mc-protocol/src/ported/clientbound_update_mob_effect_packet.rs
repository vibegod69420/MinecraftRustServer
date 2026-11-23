use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    rpos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            rpos: 0,
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, rpos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_byte() as u8;
            result |= ((b & 0x7F) as i32).wrapping_shl(shift);
            if (b & 0x80) == 0 {
                break;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                panic!("VarInt too big");
            }
        }
        result
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7F) == 0 {
                self.write_byte(val as u8 as i8);
                return;
            }
            self.write_byte(((val & 0x7F) | 0x80) as u8 as i8);
            val >>= 7;
        }
    }

    pub fn read_byte(&mut self) -> i8 {
        if self.rpos >= self.data.len() {
            panic!("buffer underflow");
        }
        let b = self.data[self.rpos] as i8;
        self.rpos += 1;
        b
    }

    pub fn write_byte(&mut self, value: i8) {
        self.data.push(value as u8);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    pub id: i32,
    _pd: PhantomData<T>,
}

impl<T> Holder<T> {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            _pd: PhantomData,
        }
    }
}

pub struct MobEffectStreamCodec;

impl MobEffectStreamCodec {
    pub fn decode(buf: &mut RegistryFriendlyByteBuf) -> Holder<MobEffect> {
        Holder::new(buf.read_var_int())
    }

    pub fn encode(buf: &mut RegistryFriendlyByteBuf, value: &Holder<MobEffect>) {
        buf.write_var_int(value.id);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectInstance {
    effect: Holder<MobEffect>,
    amplifier: i32,
    duration: i32,
    ambient: bool,
    visible: bool,
    show_icon: bool,
}

impl MobEffectInstance {
    pub fn new(
        effect: Holder<MobEffect>,
        amplifier: i32,
        duration: i32,
        ambient: bool,
        visible: bool,
        show_icon: bool,
    ) -> Self {
        Self {
            effect,
            amplifier,
            duration,
            ambient,
            visible,
            show_icon,
        }
    }

    // JAVA: net.minecraft.world.effect.MobEffectInstance#getEffect
    pub fn get_effect(&self) -> Holder<MobEffect> {
        self.effect.clone()
    }

    // JAVA: net.minecraft.world.effect.MobEffectInstance#getAmplifier
    pub fn get_amplifier(&self) -> i32 {
        self.amplifier
    }

    // JAVA: net.minecraft.world.effect.MobEffectInstance#getDuration
    pub fn get_duration(&self) -> i32 {
        self.duration
    }

    // JAVA: net.minecraft.world.effect.MobEffectInstance#isAmbient
    pub fn is_ambient(&self) -> bool {
        self.ambient
    }

    // JAVA: net.minecraft.world.effect.MobEffectInstance#isVisible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    // JAVA: net.minecraft.world.effect.MobEffectInstance#showIcon
    pub fn show_icon(&self) -> bool {
        self.show_icon
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType<T>(PhantomData<T>);

impl<T> PacketType<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateMobEffectPacket {
    entity_id: i32,
    effect: Holder<MobEffect>,
    effect_amplifier: i32,
    effect_duration_ticks: i32,
    flags: i8,
}

pub trait ClientGamePacketListener {
    fn handle_update_mob_effect(&mut self, packet: &ClientboundUpdateMobEffectPacket);
}

pub mod game_packet_types {
    use super::{ClientboundUpdateMobEffectPacket, PacketType};

    pub const CLIENTBOUND_UPDATE_MOB_EFFECT: PacketType<ClientboundUpdateMobEffectPacket> =
        PacketType::new();
}

impl ClientboundUpdateMobEffectPacket {
    const FLAG_AMBIENT: i32 = 1;
    const FLAG_VISIBLE: i32 = 2;
    const FLAG_SHOW_ICON: i32 = 4;
    const FLAG_BLEND: i32 = 8;

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#<init>
    pub fn new(entity_id: i32, effect_instance: &MobEffectInstance, blend: bool) -> Self {
        let effect = effect_instance.get_effect();
        let effect_amplifier = effect_instance.get_amplifier();
        let effect_duration_ticks = effect_instance.get_duration();
        let mut flags: i8 = 0;
        if effect_instance.is_ambient() {
            flags = ((flags as i32) | Self::FLAG_AMBIENT) as i8;
        }
        if effect_instance.is_visible() {
            flags = ((flags as i32) | Self::FLAG_VISIBLE) as i8;
        }
        if effect_instance.show_icon() {
            flags = ((flags as i32) | Self::FLAG_SHOW_ICON) as i8;
        }
        if blend {
            flags = ((flags as i32) | Self::FLAG_BLEND) as i8;
        }
        Self {
            entity_id,
            effect,
            effect_amplifier,
            effect_duration_ticks,
            flags,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#<init>
    pub fn from_buf(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let entity_id = buf.read_var_int();
        let effect = MobEffectStreamCodec::decode(buf);
        let effect_amplifier = buf.read_var_int();
        let effect_duration_ticks = buf.read_var_int();
        let flags = buf.read_byte();
        Self {
            entity_id,
            effect,
            effect_amplifier,
            effect_duration_ticks,
            flags,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        MobEffectStreamCodec::encode(buf, &self.effect);
        buf.write_var_int(self.effect_amplifier);
        buf.write_var_int(self.effect_duration_ticks);
        buf.write_byte(self.flags);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#type
    pub fn packet_type(&self) -> PacketType<Self> {
        game_packet_types::CLIENTBOUND_UPDATE_MOB_EFFECT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_update_mob_effect(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#getEntityId
    pub fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#getEffect
    pub fn get_effect(&self) -> &Holder<MobEffect> {
        &self.effect
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#getEffectAmplifier
    pub fn get_effect_amplifier(&self) -> i32 {
        self.effect_amplifier
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#getEffectDurationTicks
    pub fn get_effect_duration_ticks(&self) -> i32 {
        self.effect_duration_ticks
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#isEffectVisible
    pub fn is_effect_visible(&self) -> bool {
        (self.flags as i32 & Self::FLAG_VISIBLE) != 0
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#isEffectAmbient
    pub fn is_effect_ambient(&self) -> bool {
        (self.flags as i32 & Self::FLAG_AMBIENT) != 0
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#effectShowsIcon
    pub fn effect_shows_icon(&self) -> bool {
        (self.flags as i32 & Self::FLAG_SHOW_ICON) != 0
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket#shouldBlend
    pub fn should_blend(&self) -> bool {
        (self.flags as i32 & Self::FLAG_BLEND) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_instance(ambient: bool, visible: bool, show_icon: bool) -> MobEffectInstance {
        MobEffectInstance::new(Holder::new(5), 2, 200, ambient, visible, show_icon)
    }

    #[test]
    fn test_flags_and_getters() {
        let pkt = ClientboundUpdateMobEffectPacket::new(42, &make_instance(true, true, false), true);
        assert_eq!(pkt.get_entity_id(), 42);
        assert_eq!(pkt.get_effect().id, 5);
        assert_eq!(pkt.get_effect_amplifier(), 2);
        assert_eq!(pkt.get_effect_duration_ticks(), 200);
        assert!(pkt.is_effect_ambient());
        assert!(pkt.is_effect_visible());
        assert!(!pkt.effect_shows_icon());
        assert!(pkt.should_blend());
    }

    #[test]
    fn test_write_read_round_trip() {
        let pkt = ClientboundUpdateMobEffectPacket::new(-7, &make_instance(false, true, true), false);
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundUpdateMobEffectPacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_entity_id(), -7);
        assert_eq!(decoded.get_effect().id, 5);
        assert_eq!(decoded.get_effect_amplifier(), 2);
        assert_eq!(decoded.get_effect_duration_ticks(), 200);
        assert!(!decoded.is_effect_ambient());
        assert!(decoded.is_effect_visible());
        assert!(decoded.effect_shows_icon());
        assert!(!decoded.should_blend());
    }

    #[test]
    fn test_negative_and_large_values() {
        let inst = MobEffectInstance::new(Holder::new(-1), i32::MIN, i32::MAX, false, false, false);
        let pkt = ClientboundUpdateMobEffectPacket::new(i32::MIN, &inst, false);
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ClientboundUpdateMobEffectPacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_entity_id(), i32::MIN);
        assert_eq!(decoded.get_effect().id, -1);
        assert_eq!(decoded.get_effect_amplifier(), i32::MIN);
        assert_eq!(decoded.get_effect_duration_ticks(), i32::MAX);
    }

    #[test]
    fn test_all_flags_off() {
        let pkt = ClientboundUpdateMobEffectPacket::new(0, &make_instance(false, false, false), false);
        assert!(!pkt.is_effect_ambient());
        assert!(!pkt.is_effect_visible());
        assert!(!pkt.effect_shows_icon());
        assert!(!pkt.should_blend());
    }

    #[test]
    fn test_handle_dispatch() {
        struct MockListener {
            called: bool,
            entity_id: i32,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_update_mob_effect(&mut self, packet: &ClientboundUpdateMobEffectPacket) {
                self.called = true;
                self.entity_id = packet.get_entity_id();
            }
        }
        let pkt = ClientboundUpdateMobEffectPacket::new(99, &make_instance(false, false, false), false);
        let mut listener = MockListener {
            called: false,
            entity_id: 0,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.entity_id, 99);
    }
}
