use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    UnexpectedEof,
    VarIntTooBig,
    VarLongTooBig,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of buffer"),
            Self::VarIntTooBig => write!(f, "VarInt is too big"),
            Self::VarLongTooBig => write!(f, "VarLong is too big"),
        }
    }
}

impl std::error::Error for DecodeError {}

pub struct FriendlyByteBuf<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> FriendlyByteBuf<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_byte(&mut self) -> Result<u8, DecodeError> {
        let b = *self.data.get(self.pos).ok_or(DecodeError::UnexpectedEof)?;
        self.pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readDouble
    pub fn read_double(&mut self) -> Result<f64, DecodeError> {
        if self.pos + 8 > self.data.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(f64::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> Result<i32, DecodeError> {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_byte()?;
            value |= ((current_byte & 0x7F) as i32).wrapping_shl(position);
            if (current_byte & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                return Err(DecodeError::VarIntTooBig);
            }
        }
        Ok(value)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarLong
    pub fn read_var_long(&mut self) -> Result<i64, DecodeError> {
        let mut value: i64 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_byte()?;
            value |= ((current_byte & 0x7F) as i64).wrapping_shl(position);
            if (current_byte & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 64 {
                return Err(DecodeError::VarLongTooBig);
            }
        }
        Ok(value)
    }
}

pub struct FriendlyByteBufWriter {
    buf: Vec<u8>,
}

impl FriendlyByteBufWriter {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeDouble
    pub fn write_double(&mut self, value: f64) {
        self.buf.extend_from_slice(&value.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F_i32) == 0 {
                self.buf.push(value as u8);
                return;
            }
            self.buf.push(((value & 0x7F) as u8) | 0x80);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarLong
    pub fn write_var_long(&mut self, mut value: i64) {
        loop {
            if (value & !0x7F_i64) == 0 {
                self.buf.push(value as u8);
                return;
            }
            self.buf.push(((value & 0x7F) as u8) | 0x80);
            value = ((value as u64) >> 7) as i64;
        }
    }
}

impl Default for FriendlyByteBufWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundInitializeBorderPacket {
    new_center_x: f64,
    new_center_z: f64,
    old_size: f64,
    new_size: f64,
    lerp_time: i64,
    new_absolute_max_size: i32,
    warning_blocks: i32,
    warning_time: i32,
}

impl ClientboundInitializeBorderPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#<init>(WorldBorder)
    pub fn new(
        new_center_x: f64,
        new_center_z: f64,
        old_size: f64,
        new_size: f64,
        lerp_time: i64,
        new_absolute_max_size: i32,
        warning_blocks: i32,
        warning_time: i32,
    ) -> Self {
        Self {
            new_center_x,
            new_center_z,
            old_size,
            new_size,
            lerp_time,
            new_absolute_max_size,
            warning_blocks,
            warning_time,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf<'_>) -> Result<Self, DecodeError> {
        let new_center_x = buf.read_double()?;
        let new_center_z = buf.read_double()?;
        let old_size = buf.read_double()?;
        let new_size = buf.read_double()?;
        let lerp_time = buf.read_var_long()?;
        let new_absolute_max_size = buf.read_var_int()?;
        let warning_blocks = buf.read_var_int()?;
        let warning_time = buf.read_var_int()?;
        Ok(Self {
            new_center_x,
            new_center_z,
            old_size,
            new_size,
            lerp_time,
            new_absolute_max_size,
            warning_blocks,
            warning_time,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut FriendlyByteBufWriter) {
        buf.write_double(self.new_center_x);
        buf.write_double(self.new_center_z);
        buf.write_double(self.old_size);
        buf.write_double(self.new_size);
        buf.write_var_long(self.lerp_time);
        buf.write_var_int(self.new_absolute_max_size);
        buf.write_var_int(self.warning_blocks);
        buf.write_var_int(self.warning_time);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#getNewCenterX
    pub fn get_new_center_x(&self) -> f64 {
        self.new_center_x
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#getNewCenterZ
    pub fn get_new_center_z(&self) -> f64 {
        self.new_center_z
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#getNewSize
    pub fn get_new_size(&self) -> f64 {
        self.new_size
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#getOldSize
    pub fn get_old_size(&self) -> f64 {
        self.old_size
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#getLerpTime
    pub fn get_lerp_time(&self) -> i64 {
        self.lerp_time
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#getNewAbsoluteMaxSize
    pub fn get_new_absolute_max_size(&self) -> i32 {
        self.new_absolute_max_size
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#getWarningTime
    pub fn get_warning_time(&self) -> i32 {
        self.warning_time
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket#getWarningBlocks
    pub fn get_warning_blocks(&self) -> i32 {
        self.warning_blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(pkt: &ClientboundInitializeBorderPacket) -> ClientboundInitializeBorderPacket {
        let mut w = FriendlyByteBufWriter::new();
        pkt.write(&mut w);
        let bytes = w.into_inner();
        let mut r = FriendlyByteBuf::new(&bytes);
        ClientboundInitializeBorderPacket::read(&mut r).unwrap()
    }

    #[test]
    fn roundtrip_basic() {
        let pkt = ClientboundInitializeBorderPacket::new(
            1.5, -2.5, 100.0, 200.0, 5000, 29_999_984, 5, 15,
        );
        assert_eq!(roundtrip(&pkt), pkt);
    }

    #[test]
    fn roundtrip_negative_and_special_floats() {
        let pkt = ClientboundInitializeBorderPacket::new(
            -0.0,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NAN,
            -1,
            -1,
            i32::MIN,
            i32::MAX,
        );
        let decoded = roundtrip(&pkt);
        assert_eq!(decoded.get_new_center_x().to_bits(), (-0.0f64).to_bits());
        assert!(decoded.get_new_center_z().is_infinite() && decoded.get_new_center_z().is_sign_negative());
        assert!(decoded.get_old_size().is_infinite() && decoded.get_old_size().is_sign_positive());
        assert!(decoded.get_new_size().is_nan());
        assert_eq!(decoded.get_lerp_time(), -1);
        assert_eq!(decoded.get_new_absolute_max_size(), -1);
        assert_eq!(decoded.get_warning_blocks(), i32::MIN);
        assert_eq!(decoded.get_warning_time(), i32::MAX);
    }

    #[test]
    fn varint_encoding_edge_cases() {
        let mut w = FriendlyByteBufWriter::new();
        w.write_var_int(0);
        w.write_var_int(-1);
        w.write_var_int(i32::MAX);
        w.write_var_int(i32::MIN);
        let bytes = w.into_inner();
        let mut r = FriendlyByteBuf::new(&bytes);
        assert_eq!(r.read_var_int().unwrap(), 0);
        assert_eq!(r.read_var_int().unwrap(), -1);
        assert_eq!(r.read_var_int().unwrap(), i32::MAX);
        assert_eq!(r.read_var_int().unwrap(), i32::MIN);
    }

    #[test]
    fn varlong_encoding_edge_cases() {
        let mut w = FriendlyByteBufWriter::new();
        w.write_var_long(0);
        w.write_var_long(-1);
        w.write_var_long(i64::MAX);
        w.write_var_long(i64::MIN);
        let bytes = w.into_inner();
        let mut r = FriendlyByteBuf::new(&bytes);
        assert_eq!(r.read_var_long().unwrap(), 0);
        assert_eq!(r.read_var_long().unwrap(), -1);
        assert_eq!(r.read_var_long().unwrap(), i64::MAX);
        assert_eq!(r.read_var_long().unwrap(), i64::MIN);
    }

    #[test]
    fn decode_unexpected_eof() {
        let mut r = FriendlyByteBuf::new(&[0x00]);
        assert!(matches!(
            ClientboundInitializeBorderPacket::read(&mut r),
            Err(DecodeError::UnexpectedEof)
        ));
    }
}
