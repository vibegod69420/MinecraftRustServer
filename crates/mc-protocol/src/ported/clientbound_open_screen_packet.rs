use bytes::{Buf, BufMut};

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int(buf: &mut impl Buf) -> i32 {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32) << (shift & 31);
        if (b & 0x80) == 0 {
            return result;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return result;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int(buf: &mut impl BufMut, value: i32) {
    let mut v = value as u32;
    loop {
        if (v & !0x7Fu32) == 0 {
            buf.put_u8(v as u8);
            return;
        }
        buf.put_u8(((v & 0x7F) as u8) | 0x80);
        v >>= 7;
    }
}

/// Stub for net.minecraft.network.chat.Component.
/// Real wire format uses ComponentSerialization.TRUSTED_STREAM_CODEC (network NBT).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub encoded: Vec<u8>,
}

impl Component {
    // JAVA: net.minecraft.network.chat.ComponentSerialization#TRUSTED_STREAM_CODEC (stub decode)
    pub fn read(buf: &mut impl Buf) -> Self {
        let len = read_var_int(buf).max(0) as usize;
        let mut encoded = vec![0u8; len];
        buf.copy_to_slice(&mut encoded);
        Self { encoded }
    }

    // JAVA: net.minecraft.network.chat.ComponentSerialization#TRUSTED_STREAM_CODEC (stub encode)
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.encoded.len() as i32);
        buf.put_slice(&self.encoded);
    }
}

/// Stub for net.minecraft.world.inventory.MenuType<?> registry reference.
/// Real wire format uses ByteBufCodecs.registry(Registries.MENU).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuType {
    pub registry_id: i32,
}

impl MenuType {
    // JAVA: net.minecraft.network.codec.ByteBufCodecs#registry (stub decode)
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            registry_id: read_var_int(buf),
        }
    }

    // JAVA: net.minecraft.network.codec.ByteBufCodecs#registry (stub encode)
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.registry_id);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundOpenScreenPacket {
    container_id: i32,
    menu_type: MenuType,
    title: Component,
}

impl ClientboundOpenScreenPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenScreenPacket#<init>(int,MenuType,Component)
    pub fn new(container_id: i32, menu_type: MenuType, title: Component) -> Self {
        Self {
            container_id,
            menu_type,
            title,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenScreenPacket#<init>(RegistryFriendlyByteBuf) (composed from STREAM_CODEC)
    pub fn read(buf: &mut impl Buf) -> Self {
        let container_id = read_var_int(buf);
        let menu_type = MenuType::read(buf);
        let title = Component::read(buf);
        Self {
            container_id,
            menu_type,
            title,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenScreenPacket#write (composed from STREAM_CODEC)
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.container_id);
        self.menu_type.write(buf);
        self.title.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenScreenPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:open_screen"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenScreenPacket#handle
    pub fn handle<H: ClientGamePacketListener>(&self, handler: &mut H) {
        handler.handle_open_screen(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenScreenPacket#getContainerId
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenScreenPacket#getType
    pub fn get_menu_type(&self) -> MenuType {
        self.menu_type
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenScreenPacket#getTitle
    pub fn get_title(&self) -> &Component {
        &self.title
    }
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleOpenScreen
    fn handle_open_screen(&mut self, packet: &ClientboundOpenScreenPacket);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_basic() {
        let pkt = ClientboundOpenScreenPacket::new(
            7,
            MenuType { registry_id: 13 },
            Component {
                encoded: vec![1, 2, 3],
            },
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ClientboundOpenScreenPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_container_id(), 7);
        assert_eq!(decoded.get_menu_type().registry_id, 13);
        assert_eq!(decoded.get_title().encoded, vec![1, 2, 3]);
    }

    #[test]
    fn negative_ids_and_empty_title() {
        let pkt = ClientboundOpenScreenPacket::new(
            -1,
            MenuType { registry_id: -5 },
            Component { encoded: vec![] },
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ClientboundOpenScreenPacket::read(&mut slice);
        assert_eq!(decoded.get_container_id(), -1);
        assert_eq!(decoded.get_menu_type().registry_id, -5);
        assert!(decoded.get_title().encoded.is_empty());
    }

    #[test]
    fn extreme_varint_container_id() {
        let pkt = ClientboundOpenScreenPacket::new(
            i32::MAX,
            MenuType {
                registry_id: i32::MIN,
            },
            Component {
                encoded: vec![0xFF],
            },
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ClientboundOpenScreenPacket::read(&mut slice);
        assert_eq!(decoded.get_container_id(), i32::MAX);
        assert_eq!(decoded.get_menu_type().registry_id, i32::MIN);
    }

    #[test]
    fn handle_dispatches() {
        struct H {
            called: bool,
            container_id: i32,
        }
        impl ClientGamePacketListener for H {
            fn handle_open_screen(&mut self, packet: &ClientboundOpenScreenPacket) {
                self.called = true;
                self.container_id = packet.get_container_id();
            }
        }
        let pkt = ClientboundOpenScreenPacket::new(
            3,
            MenuType { registry_id: 1 },
            Component { encoded: vec![9] },
        );
        let mut h = H {
            called: false,
            container_id: 0,
        };
        pkt.handle(&mut h);
        assert!(h.called);
        assert_eq!(h.container_id, 3);
        assert_eq!(pkt.packet_type(), "minecraft:open_screen");
    }
}
