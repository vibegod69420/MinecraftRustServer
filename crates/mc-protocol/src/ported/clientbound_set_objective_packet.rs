use std::io;

pub const METHOD_ADD: i32 = 0;
pub const METHOD_REMOVE: i32 = 1;
pub const METHOD_CHANGE: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderType {
    Integer,
    Hearts,
}

impl RenderType {
    fn from_ordinal(id: i32) -> Self {
        match id {
            0 => Self::Integer,
            1 => Self::Hearts,
            _ => panic!("invalid RenderType ordinal: {id}"),
        }
    }

    fn to_ordinal(self) -> i32 {
        match self {
            Self::Integer => 0,
            Self::Hearts => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
        }
    }
}

/// Minimal self-contained placeholder for `ComponentSerialization.TRUSTED_STREAM_CODEC`.
mod component_serialization {
    use super::{Component, RegistryFriendlyByteBuf};

    pub fn trusted_stream_decode(buf: &mut RegistryFriendlyByteBuf) -> Component {
        Component {
            text: buf.read_utf(),
        }
    }

    pub fn trusted_stream_encode(buf: &mut RegistryFriendlyByteBuf, value: &Component) {
        buf.write_utf(&value.text);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFormat {
    /// Opaque payload for self-contained round-tripping.
    pub data: Vec<u8>,
}

/// Minimal self-contained placeholder for `NumberFormatTypes.OPTIONAL_STREAM_CODEC`.
mod number_format_types {
    use super::{NumberFormat, RegistryFriendlyByteBuf};

    pub fn optional_stream_decode(buf: &mut RegistryFriendlyByteBuf) -> Option<NumberFormat> {
        let present = buf.read_byte() != 0;
        if present {
            let len = buf.read_var_int() as usize;
            let data = buf.read_bytes(len);
            Some(NumberFormat { data })
        } else {
            None
        }
    }

    pub fn optional_stream_encode(
        buf: &mut RegistryFriendlyByteBuf,
        value: &Option<NumberFormat>,
    ) {
        match value {
            Some(format) => {
                buf.write_byte(1);
                buf.write_var_int(format.data.len() as i32);
                buf.write_raw_bytes(&format.data);
            }
            None => {
                buf.write_byte(0);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    ClientboundSetObjective,
}

/// Minimal local substitute for `net.minecraft.world.scores.Objective`.
pub trait ObjectiveLike {
    fn get_name(&self) -> &str;
    fn get_display_name(&self) -> &Component;
    fn get_render_type(&self) -> RenderType;
    fn number_format(&self) -> Option<&NumberFormat>;
}

/// Minimal local substitute for `ClientGamePacketListener`.
pub trait ClientGamePacketListener {
    fn handle_add_objective(&mut self, packet: &ClientboundSetObjectivePacket);
}

/// Minimal self-contained buffer substituting `RegistryFriendlyByteBuf`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn write_byte(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    pub fn read_byte(&mut self) -> i8 {
        let b = self.data[self.read_pos] as i8;
        self.read_pos = self.read_pos.wrapping_add(1);
        b
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7Fu32) == 0 {
                self.data.push(val as u8);
                return;
            }
            self.data.push(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_byte() as u8;
            result |= ((b & 0x7F) as u32) << shift;
            if (b & 0x80) == 0 {
                break;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                panic!("VarInt too big");
            }
        }
        result as i32
    }

    pub fn write_utf(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    pub fn read_utf(&mut self) -> String {
        let len = self.read_var_int() as usize;
        let bytes = self.read_bytes(len);
        String::from_utf8(bytes).expect("invalid utf8 in read_utf")
    }

    pub fn write_enum(&mut self, value: RenderType) {
        self.write_var_int(value.to_ordinal());
    }

    pub fn read_enum(&mut self) -> RenderType {
        RenderType::from_ordinal(self.read_var_int())
    }

    fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        let end = self.read_pos.wrapping_add(len);
        let bytes = self.data[self.read_pos..end].to_vec();
        self.read_pos = end;
        bytes
    }

    fn write_raw_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetObjectivePacket {
    objective_name: String,
    display_name: Component,
    render_type: RenderType,
    number_format: Option<NumberFormat>,
    method: i32,
}

impl ClientboundSetObjectivePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#<init>(Objective,int)
    pub fn new(objective: &impl ObjectiveLike, method: i32) -> Self {
        Self {
            objective_name: objective.get_name().to_string(),
            display_name: objective.get_display_name().clone(),
            render_type: objective.get_render_type(),
            number_format: objective.number_format().cloned(),
            method,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let objective_name = buf.read_utf();
        let method = buf.read_byte() as i32;
        if method != 0 && method != 2 {
            Self {
                objective_name,
                display_name: Component::empty(),
                render_type: RenderType::Integer,
                number_format: None,
                method,
            }
        } else {
            let display_name = component_serialization::trusted_stream_decode(buf);
            let render_type = buf.read_enum();
            let number_format = number_format_types::optional_stream_decode(buf);
            Self {
                objective_name,
                display_name,
                render_type,
                number_format,
                method,
            }
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_utf(&self.objective_name);
        buf.write_byte(self.method as i8);
        if self.method == 0 || self.method == 2 {
            component_serialization::trusted_stream_encode(buf, &self.display_name);
            buf.write_enum(self.render_type);
            number_format_types::optional_stream_encode(buf, &self.number_format);
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType::ClientboundSetObjective
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_add_objective(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#getObjectiveName
    pub fn get_objective_name(&self) -> &str {
        &self.objective_name
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#getDisplayName
    pub fn get_display_name(&self) -> &Component {
        &self.display_name
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#getMethod
    pub fn get_method(&self) -> i32 {
        self.method
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#getRenderType
    pub fn get_render_type(&self) -> RenderType {
        self.render_type
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetObjectivePacket#getNumberFormat
    pub fn get_number_format(&self) -> Option<&NumberFormat> {
        self.number_format.as_ref()
    }
}

impl io::Read for RegistryFriendlyByteBuf {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let remaining = self.data.len().saturating_sub(self.read_pos);
        let n = remaining.min(out.len());
        let end = self.read_pos.wrapping_add(n);
        out[..n].copy_from_slice(&self.data[self.read_pos..end]);
        self.read_pos = end;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestObjective {
        name: String,
        display_name: Component,
        render_type: RenderType,
        number_format: Option<NumberFormat>,
    }

    impl ObjectiveLike for TestObjective {
        fn get_name(&self) -> &str {
            &self.name
        }

        fn get_display_name(&self) -> &Component {
            &self.display_name
        }

        fn get_render_type(&self) -> RenderType {
            self.render_type
        }

        fn number_format(&self) -> Option<&NumberFormat> {
            self.number_format.as_ref()
        }
    }

    struct TestListener {
        called: bool,
        method: i32,
        name: String,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_add_objective(&mut self, packet: &ClientboundSetObjectivePacket) {
            self.called = true;
            self.method = packet.get_method();
            self.name = packet.get_objective_name().to_string();
        }
    }

    #[test]
    fn test_add_method_round_trip() {
        let objective = TestObjective {
            name: "obj".to_string(),
            display_name: Component {
                text: "Score".to_string(),
            },
            render_type: RenderType::Hearts,
            number_format: Some(NumberFormat {
                data: vec![1, 2, 3],
            }),
        };
        let packet = ClientboundSetObjectivePacket::new(&objective, METHOD_ADD);
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);

        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundSetObjectivePacket::read(&mut read_buf);

        assert_eq!(decoded.get_objective_name(), "obj");
        assert_eq!(decoded.get_display_name().text, "Score");
        assert_eq!(decoded.get_method(), METHOD_ADD);
        assert_eq!(decoded.get_render_type(), RenderType::Hearts);
        assert_eq!(
            decoded.get_number_format().unwrap().data,
            vec![1, 2, 3]
        );
        assert_eq!(decoded.get_type(), PacketType::ClientboundSetObjective);
    }

    #[test]
    fn test_remove_method_uses_defaults_and_skips_payload() {
        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_utf("remove_me");
        buf.write_byte(METHOD_REMOVE as i8);
        // No display/render/number_format payload for REMOVE.

        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundSetObjectivePacket::read(&mut read_buf);

        assert_eq!(decoded.get_objective_name(), "remove_me");
        assert_eq!(decoded.get_method(), METHOD_REMOVE);
        assert_eq!(decoded.get_display_name(), &Component::empty());
        assert_eq!(decoded.get_render_type(), RenderType::Integer);
        assert_eq!(decoded.get_number_format(), None);
    }

    #[test]
    fn test_change_method_round_trip_and_handle() {
        let objective = TestObjective {
            name: "changed".to_string(),
            display_name: Component {
                text: "Changed".to_string(),
            },
            render_type: RenderType::Integer,
            number_format: None,
        };
        let packet = ClientboundSetObjectivePacket::new(&objective, METHOD_CHANGE);
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut decoded = ClientboundSetObjectivePacket::read(&mut RegistryFriendlyByteBuf::from_vec(
            buf.into_vec(),
        ));

        let mut listener = TestListener {
            called: false,
            method: -1,
            name: String::new(),
        };
        decoded.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.method, METHOD_CHANGE);
        assert_eq!(listener.name, "changed");
        assert_eq!(decoded.get_number_format(), None);
    }

    #[test]
    fn test_negative_method_byte_sign_extension_and_unknown_method() {
        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_utf("neg");
        buf.write_byte(-1i8); // 0xFF, sign-extended to -1 in Java int field
        let decoded = ClientboundSetObjectivePacket::read(&mut RegistryFriendlyByteBuf::from_vec(
            buf.into_vec(),
        ));
        assert_eq!(decoded.get_method(), -1);
        assert_eq!(decoded.get_display_name(), &Component::empty());
        assert_eq!(decoded.get_render_type(), RenderType::Integer);
        assert_eq!(decoded.get_number_format(), None);

        // Write path should preserve low byte semantics for negative/overflowing ints.
        let obj = TestObjective {
            name: "ovf".to_string(),
            display_name: Component {
                text: "x".to_string(),
            },
            render_type: RenderType::Hearts,
            number_format: Some(NumberFormat { data: vec![9] }),
        };
        let packet = ClientboundSetObjectivePacket::new(&obj, 256i32.wrapping_add(1)); // 257 -> byte 1
        let mut out = RegistryFriendlyByteBuf::new();
        packet.write(&mut out);
        let bytes = out.into_vec();
        // objective name "ovf" -> varint(3) + 3 bytes, then method byte
        assert_eq!(bytes[4], 1u8);
    }

    #[test]
    fn test_varint_and_float_bits_edge_cases() {
        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_var_int(-1);
        buf.write_var_int(i32::MIN);
        buf.write_var_int(i32::MAX);
        let mut read_buf = RegistryFriendlyByteBuf::from_vec(buf.into_vec());
        assert_eq!(read_buf.read_var_int(), -1);
        assert_eq!(read_buf.read_var_int(), i32::MIN);
        assert_eq!(read_buf.read_var_int(), i32::MAX);

        let nan_bits = f32::NAN.to_bits();
        let neg_zero_bits = (-0.0f32).to_bits();
        assert_eq!(f32::from_bits(nan_bits).to_bits(), nan_bits);
        assert_eq!(f32::from_bits(neg_zero_bits).to_bits(), neg_zero_bits);

        let f64_bits = f64::from_bits(0x7FF8_0000_0000_0001u64).to_bits();
        assert_eq!(f64::from_bits(f64_bits).to_bits(), f64_bits);
    }
}
