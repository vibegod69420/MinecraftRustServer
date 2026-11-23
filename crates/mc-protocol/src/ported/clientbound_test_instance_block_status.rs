use bytes::{Buf, BufMut, BytesMut};
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn write(&self, buf: &mut BytesMut) {
        buf.put_i32(self.x);
        buf.put_i32(self.y);
        buf.put_i32(self.z);
    }

    fn read(buf: &mut Cursor<&[u8]>) -> Option<Self> {
        if buf.remaining() < 12 {
            return None;
        }
        Some(Self {
            x: buf.get_i32(),
            y: buf.get_i32(),
            z: buf.get_i32(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    fn write(&self, buf: &mut BytesMut) {
        write_utf(buf, &self.text);
    }

    fn read(buf: &mut Cursor<&[u8]>) -> Option<Self> {
        Some(Self {
            text: read_utf(buf)?,
        })
    }
}

pub trait ClientGamePacketListener {
    fn handle_test_instance_block_status(&mut self, packet: &ClientboundTestInstanceBlockStatus);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTestInstanceBlockStatus {
    status: Component,
    size: Option<Vec3i>,
}

impl ClientboundTestInstanceBlockStatus {
    // JAVA: net.minecraft.network.protocol.game.ClientboundTestInstanceBlockStatus#<init>
    pub fn new(status: Component, size: Option<Vec3i>) -> Self {
        Self { status, size }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTestInstanceBlockStatus#status
    pub fn status(&self) -> &Component {
        &self.status
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTestInstanceBlockStatus#size
    pub fn size(&self) -> Option<&Vec3i> {
        self.size.as_ref()
    }

    pub fn write(&self, buf: &mut BytesMut) {
        self.status.write(buf);
        match &self.size {
            Some(size) => {
                buf.put_u8(1);
                size.write(buf);
            }
            None => buf.put_u8(0),
        }
    }

    pub fn read(buf: &mut Cursor<&[u8]>) -> Option<Self> {
        let status = Component::read(buf)?;
        if !buf.has_remaining() {
            return None;
        }
        let has_size = buf.get_u8() != 0;
        let size = if has_size {
            Some(Vec3i::read(buf)?)
        } else {
            None
        };
        Some(Self { status, size })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTestInstanceBlockStatus#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:test_instance_block_status"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTestInstanceBlockStatus#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_test_instance_block_status(self);
    }
}

fn write_utf(buf: &mut BytesMut, s: &str) {
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

fn read_utf(buf: &mut Cursor<&[u8]>) -> Option<String> {
    let len = read_varint(buf)?;
    if len < 0 {
        return None;
    }
    let len = len as usize;
    if buf.remaining() < len {
        return None;
    }
    let mut bytes = vec![0u8; len];
    buf.copy_to_slice(&mut bytes);
    String::from_utf8(bytes).ok()
}

fn write_varint(buf: &mut BytesMut, value: i32) {
    let mut val = value as u32;
    loop {
        let mut temp = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            temp |= 0x80;
        }
        buf.put_u8(temp);
        if val == 0 {
            break;
        }
    }
}

fn read_varint(buf: &mut Cursor<&[u8]>) -> Option<i32> {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return None;
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as u32) << shift;
        if (b & 0x80) == 0 {
            return Some(result as i32);
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn round_trip_with_size() {
        let pkt = ClientboundTestInstanceBlockStatus::new(
            Component {
                text: "ok".to_string(),
            },
            Some(Vec3i::new(1, 2, 3)),
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded = ClientboundTestInstanceBlockStatus::read(&mut cur).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn round_trip_without_size() {
        let pkt = ClientboundTestInstanceBlockStatus::new(
            Component {
                text: String::new(),
            },
            None,
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded = ClientboundTestInstanceBlockStatus::read(&mut cur).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn round_trip_negative_and_extreme_coords() {
        let pkt = ClientboundTestInstanceBlockStatus::new(
            Component {
                text: "status".to_string(),
            },
            Some(Vec3i::new(-1, i32::MIN, i32::MAX)),
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded = ClientboundTestInstanceBlockStatus::read(&mut cur).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.size().unwrap().y, i32::MIN);
        assert_eq!(decoded.size().unwrap().z, i32::MAX);
    }

    #[test]
    fn packet_type_and_handle() {
        struct Listener {
            called: bool,
        }
        impl ClientGamePacketListener for Listener {
            fn handle_test_instance_block_status(
                &mut self,
                _packet: &ClientboundTestInstanceBlockStatus,
            ) {
                self.called = true;
            }
        }

        let pkt = ClientboundTestInstanceBlockStatus::new(
            Component {
                text: "x".to_string(),
            },
            None,
        );
        assert_eq!(pkt.packet_type(), "minecraft:test_instance_block_status");
        let mut listener = Listener { called: false };
        pkt.handle(&mut listener);
        assert!(listener.called);
    }
}
