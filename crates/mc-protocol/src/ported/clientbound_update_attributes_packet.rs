use bytes::{Buf, BufMut};
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum DecodeError {
    UnexpectedEof,
    VarIntTooLong,
    InvalidOperation(i32),
    InvalidUtf8(std::string::FromUtf8Error),
    StringTooLong(usize),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of buffer"),
            Self::VarIntTooLong => write!(f, "varint too long"),
            Self::InvalidOperation(id) => write!(f, "invalid attribute modifier operation: {id}"),
            Self::InvalidUtf8(e) => write!(f, "invalid utf8 string: {e}"),
            Self::StringTooLong(len) => write!(f, "string too long: {len}"),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}

fn read_var_int<B: Buf>(buf: &mut B) -> Result<i32, DecodeError> {
    let mut val: i32 = 0;
    let mut i: i32 = 0;
    while i < 5 {
        if !buf.has_remaining() {
            return Err(DecodeError::UnexpectedEof);
        }
        let b = buf.get_u8() as i32;
        val |= (b & 0x7F) << ((i.wrapping_mul(7)) & 31);
        if (b & 0x80) == 0 {
            return Ok(val);
        }
        i = i.wrapping_add(1);
    }
    Err(DecodeError::VarIntTooLong)
}

fn write_var_int<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value & !0x7F_i32) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

fn read_string<B: Buf>(buf: &mut B, max_len: usize) -> Result<String, DecodeError> {
    let len = read_var_int(buf)? as usize;
    if len > max_len {
        return Err(DecodeError::StringTooLong(len));
    }
    if buf.remaining() < len {
        return Err(DecodeError::UnexpectedEof);
    }
    let mut bytes = vec![0u8; len];
    buf.copy_to_slice(&mut bytes);
    String::from_utf8(bytes).map_err(DecodeError::InvalidUtf8)
}

fn write_string<B: BufMut>(buf: &mut B, s: &str) {
    let bytes = s.as_bytes();
    write_var_int(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

fn read_f64<B: Buf>(buf: &mut B) -> Result<f64, DecodeError> {
    if buf.remaining() < 8 {
        return Err(DecodeError::UnexpectedEof);
    }
    let mut bytes = [0u8; 8];
    buf.copy_to_slice(&mut bytes);
    Ok(f64::from_be_bytes(bytes))
}

fn write_f64<B: BufMut>(buf: &mut B, value: f64) {
    buf.put_slice(&value.to_be_bytes());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    pub fn parse(s: &str) -> Self {
        match s.split_once(':') {
            Some((ns, path)) => Self::new(ns, path),
            None => Self::new("minecraft", s),
        }
    }

    pub fn as_str(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribute;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    pub id: i32,
    _marker: PhantomData<T>,
}

impl<T> Holder<T> {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

fn read_attribute_holder<B: Buf>(buf: &mut B) -> Result<Holder<Attribute>, DecodeError> {
    let raw = read_var_int(buf)?;
    Ok(Holder::new(raw.wrapping_sub(1)))
}

fn write_attribute_holder<B: BufMut>(buf: &mut B, holder: &Holder<Attribute>) {
    write_var_int(buf, holder.id.wrapping_add(1));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeModifierOperation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

impl AttributeModifierOperation {
    pub fn from_id(id: i32) -> Result<Self, DecodeError> {
        match id {
            0 => Ok(Self::AddValue),
            1 => Ok(Self::AddMultipliedBase),
            2 => Ok(Self::AddMultipliedTotal),
            _ => Err(DecodeError::InvalidOperation(id)),
        }
    }

    pub fn to_id(self) -> i32 {
        match self {
            Self::AddValue => 0,
            Self::AddMultipliedBase => 1,
            Self::AddMultipliedTotal => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifier {
    id: Identifier,
    amount: f64,
    operation: AttributeModifierOperation,
}

impl AttributeModifier {
    pub fn new(id: Identifier, amount: f64, operation: AttributeModifierOperation) -> Self {
        Self {
            id,
            amount,
            operation,
        }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn operation(&self) -> AttributeModifierOperation {
        self.operation
    }

    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, DecodeError> {
        let id = Identifier::parse(&read_string(buf, 32767)?);
        let amount = read_f64(buf)?;
        let operation = AttributeModifierOperation::from_id(read_var_int(buf)?)?;
        Ok(Self {
            id,
            amount,
            operation,
        })
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_string(buf, &self.id.as_str());
        write_f64(buf, self.amount);
        write_var_int(buf, self.operation.to_id());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSnapshot {
    attribute: Holder<Attribute>,
    base: f64,
    modifiers: Vec<AttributeModifier>,
}

impl AttributeSnapshot {
    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket$AttributeSnapshot#<init>
    pub fn new(
        attribute: Holder<Attribute>,
        base: f64,
        modifiers: Vec<AttributeModifier>,
    ) -> Self {
        Self {
            attribute,
            base,
            modifiers,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket$AttributeSnapshot#attribute
    pub fn attribute(&self) -> &Holder<Attribute> {
        &self.attribute
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket$AttributeSnapshot#base
    pub fn base(&self) -> f64 {
        self.base
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket$AttributeSnapshot#modifiers
    pub fn modifiers(&self) -> &[AttributeModifier] {
        &self.modifiers
    }

    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, DecodeError> {
        let attribute = read_attribute_holder(buf)?;
        let base = read_f64(buf)?;
        let count = read_var_int(buf)? as usize;
        let mut modifiers = Vec::with_capacity(count);
        for _ in 0..count {
            modifiers.push(AttributeModifier::read(buf)?);
        }
        Ok(Self {
            attribute,
            base,
            modifiers,
        })
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_attribute_holder(buf, &self.attribute);
        write_f64(buf, self.base);
        write_var_int(buf, self.modifiers.len() as i32);
        for modifier in &self.modifiers {
            modifier.write(buf);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const GAME_PACKET_TYPES_CLIENTBOUND_UPDATE_ATTRIBUTES: PacketType =
    PacketType("minecraft:update_attributes");

pub trait ClientGamePacketListener {
    fn handle_update_attributes(&mut self, packet: &ClientboundUpdateAttributesPacket);
}

pub trait AttributeInstanceLike {
    fn get_attribute(&self) -> Holder<Attribute>;
    fn get_base_value(&self) -> f64;
    fn get_modifiers(&self) -> Vec<AttributeModifier>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundUpdateAttributesPacket {
    entity_id: i32,
    attributes: Vec<AttributeSnapshot>,
}

impl ClientboundUpdateAttributesPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket#<init>
    pub fn from_instances<I, A>(entity_id: i32, instances: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: AttributeInstanceLike,
    {
        let mut attributes = Vec::new();
        for inst in instances {
            attributes.push(AttributeSnapshot::new(
                inst.get_attribute(),
                inst.get_base_value(),
                inst.get_modifiers(),
            ));
        }
        Self {
            entity_id,
            attributes,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket#<init>
    pub fn new(entity_id: i32, attributes: Vec<AttributeSnapshot>) -> Self {
        Self {
            entity_id,
            attributes,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket#type
    pub fn packet_type(&self) -> PacketType {
        GAME_PACKET_TYPES_CLIENTBOUND_UPDATE_ATTRIBUTES
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_update_attributes(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket#getEntityId
    pub fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket#getValues
    pub fn get_values(&self) -> &[AttributeSnapshot] {
        &self.attributes
    }

    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, DecodeError> {
        let entity_id = read_var_int(buf)?;
        let count = read_var_int(buf)? as usize;
        let mut attributes = Vec::with_capacity(count);
        for _ in 0..count {
            attributes.push(AttributeSnapshot::read(buf)?);
        }
        Ok(Self {
            entity_id,
            attributes,
        })
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_var_int(buf, self.entity_id);
        write_var_int(buf, self.attributes.len() as i32);
        for attr in &self.attributes {
            attr.write(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(packet: &ClientboundUpdateAttributesPacket) -> ClientboundUpdateAttributesPacket {
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        ClientboundUpdateAttributesPacket::read(&mut slice).expect("decode should succeed")
    }

    #[test]
    fn round_trip_packet_with_modifiers() {
        let packet = ClientboundUpdateAttributesPacket::new(
            42,
            vec![AttributeSnapshot::new(
                Holder::new(5),
                20.0,
                vec![
                    AttributeModifier::new(
                        Identifier::new("minecraft", "base_bonus"),
                        1.5,
                        AttributeModifierOperation::AddValue,
                    ),
                    AttributeModifier::new(
                        Identifier::new("minecraft", "mult"),
                        -0.25,
                        AttributeModifierOperation::AddMultipliedBase,
                    ),
                ],
            )],
        );
        let decoded = round_trip(&packet);
        assert_eq!(decoded, packet);
        assert_eq!(decoded.get_entity_id(), 42);
        assert_eq!(decoded.get_values()[0].attribute().id, 5);
        assert_eq!(decoded.get_values()[0].modifiers().len(), 2);
    }

    #[test]
    fn negative_ids_and_empty_collections() {
        let packet = ClientboundUpdateAttributesPacket::new(
            -1,
            vec![AttributeSnapshot::new(Holder::new(-2), -0.0, vec![])],
        );
        let decoded = round_trip(&packet);
        assert_eq!(decoded.get_entity_id(), -1);
        assert_eq!(decoded.get_values()[0].attribute().id, -2);
        assert!(decoded.get_values()[0].base().is_sign_negative());
        assert!(decoded.get_values()[0].modifiers().is_empty());
    }

    #[test]
    fn float_bits_are_preserved() {
        let nan_bits = 0x7FF8_0000_0000_0001_u64;
        let nan = f64::from_bits(nan_bits);
        let packet = ClientboundUpdateAttributesPacket::new(
            0,
            vec![
                AttributeSnapshot::new(Holder::new(0), nan, vec![]),
                AttributeSnapshot::new(
                    Holder::new(1),
                    f64::NEG_INFINITY,
                    vec![AttributeModifier::new(
                        Identifier::new("a", "b"),
                        f64::INFINITY,
                        AttributeModifierOperation::AddMultipliedTotal,
                    )],
                ),
            ],
        );
        let decoded = round_trip(&packet);
        assert_eq!(decoded.get_values()[0].base().to_bits(), nan_bits);
        assert_eq!(decoded.get_values()[1].base(), f64::NEG_INFINITY);
        assert_eq!(
            decoded.get_values()[1].modifiers()[0].amount(),
            f64::INFINITY
        );
    }

    #[test]
    fn holder_id_wrapping_and_extreme_ints() {
        let packet = ClientboundUpdateAttributesPacket::new(
            i32::MIN,
            vec![
                AttributeSnapshot::new(Holder::new(i32::MAX), 1.0, vec![]),
                AttributeSnapshot::new(Holder::new(i32::MIN), 2.0, vec![]),
            ],
        );
        let decoded = round_trip(&packet);
        assert_eq!(decoded.get_entity_id(), i32::MIN);
        assert_eq!(decoded.get_values()[0].attribute().id, i32::MAX);
        assert_eq!(decoded.get_values()[1].attribute().id, i32::MIN);
    }

    #[test]
    fn from_instances_handle_and_type() {
        struct MockInst {
            attr_id: i32,
            base: f64,
        }

        impl AttributeInstanceLike for MockInst {
            fn get_attribute(&self) -> Holder<Attribute> {
                Holder::new(self.attr_id)
            }

            fn get_base_value(&self) -> f64 {
                self.base
            }

            fn get_modifiers(&self) -> Vec<AttributeModifier> {
                vec![AttributeModifier::new(
                    Identifier::new("minecraft", "m"),
                    1.0,
                    AttributeModifierOperation::AddValue,
                )]
            }
        }

        struct MockListener {
            called: bool,
            entity_id: i32,
        }

        impl ClientGamePacketListener for MockListener {
            fn handle_update_attributes(&mut self, packet: &ClientboundUpdateAttributesPacket) {
                self.called = true;
                self.entity_id = packet.get_entity_id();
            }
        }

        let packet = ClientboundUpdateAttributesPacket::from_instances(
            7,
            vec![MockInst {
                attr_id: 3,
                base: 9.5,
            }],
        );
        assert_eq!(
            packet.packet_type(),
            GAME_PACKET_TYPES_CLIENTBOUND_UPDATE_ATTRIBUTES
        );
        assert_eq!(packet.get_entity_id(), 7);
        assert_eq!(packet.get_values().len(), 1);
        assert_eq!(packet.get_values()[0].attribute().id, 3);
        assert_eq!(packet.get_values()[0].base(), 9.5);

        let mut listener = MockListener {
            called: false,
            entity_id: 0,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.entity_id, 7);
    }

    #[test]
    fn invalid_operation_id_errors() {
        let mut buf = Vec::new();
        // entity_id = 1
        write_var_int(&mut buf, 1);
        // attributes count = 1
        write_var_int(&mut buf, 1);
        // attribute holder raw id (id + 1) => id 0
        write_var_int(&mut buf, 1);
        // base double
        write_f64(&mut buf, 1.0);
        // modifiers count = 1
        write_var_int(&mut buf, 1);
        // modifier id string
        write_string(&mut buf, "minecraft:test");
        // modifier amount
        write_f64(&mut buf, 2.0);
        // invalid operation id
        write_var_int(&mut buf, 99);

        let mut slice: &[u8] = &buf;
        let err = ClientboundUpdateAttributesPacket::read(&mut slice).unwrap_err();
        match err {
            DecodeError::InvalidOperation(99) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
