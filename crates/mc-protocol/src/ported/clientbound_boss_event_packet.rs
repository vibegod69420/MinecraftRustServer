#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

/// Minimal local stand-in for Minecraft Component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component(pub String);

pub trait FriendlyEnum: Copy {
    fn ordinal(self) -> i32;
    fn from_ordinal(ord: i32) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

impl FriendlyEnum for BossBarColor {
    fn ordinal(self) -> i32 {
        match self {
            Self::Pink => 0,
            Self::Blue => 1,
            Self::Red => 2,
            Self::Green => 3,
            Self::Yellow => 4,
            Self::Purple => 5,
            Self::White => 6,
        }
    }

    fn from_ordinal(ord: i32) -> Self {
        match ord {
            0 => Self::Pink,
            1 => Self::Blue,
            2 => Self::Red,
            3 => Self::Green,
            4 => Self::Yellow,
            5 => Self::Purple,
            6 => Self::White,
            _ => Self::Pink,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarOverlay {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

impl FriendlyEnum for BossBarOverlay {
    fn ordinal(self) -> i32 {
        match self {
            Self::Progress => 0,
            Self::Notched6 => 1,
            Self::Notched10 => 2,
            Self::Notched12 => 3,
            Self::Notched20 => 4,
        }
    }

    fn from_ordinal(ord: i32) -> Self {
        match ord {
            0 => Self::Progress,
            1 => Self::Notched6,
            2 => Self::Notched10,
            3 => Self::Notched12,
            4 => Self::Notched20,
            _ => Self::Progress,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Add,
    Remove,
    UpdateProgress,
    UpdateName,
    UpdateStyle,
    UpdateProperties,
}

impl FriendlyEnum for OperationType {
    fn ordinal(self) -> i32 {
        match self {
            Self::Add => 0,
            Self::Remove => 1,
            Self::UpdateProgress => 2,
            Self::UpdateName => 3,
            Self::UpdateStyle => 4,
            Self::UpdateProperties => 5,
        }
    }

    fn from_ordinal(ord: i32) -> Self {
        match ord {
            0 => Self::Add,
            1 => Self::Remove,
            2 => Self::UpdateProgress,
            3 => Self::UpdateName,
            4 => Self::UpdateStyle,
            5 => Self::UpdateProperties,
            _ => Self::Remove,
        }
    }
}

/// Minimal local stand-in for RegistryFriendlyByteBuf.
#[derive(Debug, Clone, Default)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    reader_index: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            reader_index: 0,
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self {
            data,
            reader_index: 0,
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn write_byte(&mut self, value: i32) {
        self.data.push(value as u8);
    }

    pub fn read_unsigned_byte(&mut self) -> i32 {
        let b = self.data[self.reader_index];
        self.reader_index = self.reader_index.wrapping_add(1);
        (b as i32) & 0xFF
    }

    pub fn write_float(&mut self, value: f32) {
        self.data
            .extend_from_slice(&value.to_bits().to_be_bytes());
    }

    pub fn read_float(&mut self) -> f32 {
        let mut bytes = [0u8; 4];
        let start = self.reader_index;
        let end = self.reader_index.wrapping_add(4);
        bytes.copy_from_slice(&self.data[start..end]);
        self.reader_index = end;
        f32::from_bits(u32::from_be_bytes(bytes))
    }

    pub fn write_uuid(&mut self, uuid: Uuid) {
        self.data
            .extend_from_slice(&uuid.most_significant_bits.to_be_bytes());
        self.data
            .extend_from_slice(&uuid.least_significant_bits.to_be_bytes());
    }

    pub fn read_uuid(&mut self) -> Uuid {
        let mut most_bytes = [0u8; 8];
        let start = self.reader_index;
        let end = self.reader_index.wrapping_add(8);
        most_bytes.copy_from_slice(&self.data[start..end]);
        self.reader_index = end;

        let mut least_bytes = [0u8; 8];
        let start = self.reader_index;
        let end = self.reader_index.wrapping_add(8);
        least_bytes.copy_from_slice(&self.data[start..end]);
        self.reader_index = end;

        Uuid {
            most_significant_bits: i64::from_be_bytes(most_bytes),
            least_significant_bits: i64::from_be_bytes(least_bytes),
        }
    }

    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F) == 0 {
                self.data.push(value as u8);
                return;
            }
            self.data.push(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: i32 = 0;
        loop {
            let current_byte = self.read_unsigned_byte();
            value |= (current_byte & 0x7F).wrapping_shl(position as u32);
            if (current_byte & 0x80) == 0 {
                break;
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                panic!("VarInt too big");
            }
        }
        value
    }

    pub fn write_enum<E: FriendlyEnum>(&mut self, value: E) {
        self.write_var_int(value.ordinal());
    }

    pub fn read_enum<E: FriendlyEnum>(&mut self) -> E {
        E::from_ordinal(self.read_var_int())
    }

    /// Minimal stand-in for ComponentSerialization.TRUSTED_STREAM_CODEC encode.
    pub fn write_component(&mut self, component: &Component) {
        let bytes = component.0.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    /// Minimal stand-in for ComponentSerialization.TRUSTED_STREAM_CODEC decode.
    pub fn read_component(&mut self) -> Component {
        let len = self.read_var_int() as usize;
        let start = self.reader_index;
        let end = self.reader_index.wrapping_add(len);
        let bytes = self.data[start..end].to_vec();
        self.reader_index = end;
        Component(String::from_utf8(bytes).unwrap_or_default())
    }
}

/// Minimal local stand-in for BossEvent.
#[derive(Debug, Clone)]
pub struct BossEvent {
    pub id: Uuid,
    pub name: Component,
    pub progress: f32,
    pub color: BossBarColor,
    pub overlay: BossBarOverlay,
    pub darken_screen: bool,
    pub play_boss_music: bool,
    pub create_world_fog: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_BOSS_EVENT: PacketType = PacketType("clientbound_boss_event");
}

pub trait Handler {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$Handler#add
    fn add(
        &mut self,
        _id: Uuid,
        _name: Component,
        _progress: f32,
        _color: BossBarColor,
        _overlay: BossBarOverlay,
        _darken_screen: bool,
        _play_music: bool,
        _create_world_fog: bool,
    ) {
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$Handler#remove
    fn remove(&mut self, _id: Uuid) {}

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$Handler#updateProgress
    fn update_progress(&mut self, _id: Uuid, _progress: f32) {}

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$Handler#updateName
    fn update_name(&mut self, _id: Uuid, _name: Component) {}

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$Handler#updateStyle
    fn update_style(&mut self, _id: Uuid, _color: BossBarColor, _overlay: BossBarOverlay) {}

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$Handler#updateProperties
    fn update_properties(
        &mut self,
        _id: Uuid,
        _darken_screen: bool,
        _play_music: bool,
        _create_world_fog: bool,
    ) {
    }
}

pub trait ClientGamePacketListener {
    fn handle_boss_update(&mut self, packet: &ClientboundBossEventPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddOperation {
    pub name: Component,
    pub progress: f32,
    pub color: BossBarColor,
    pub overlay: BossBarOverlay,
    pub darken_screen: bool,
    pub play_music: bool,
    pub create_world_fog: bool,
}

impl AddOperation {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$AddOperation#<init>(BossEvent)
    pub fn from_boss_event(event: &BossEvent) -> Self {
        Self {
            name: event.name.clone(),
            progress: event.progress,
            color: event.color,
            overlay: event.overlay,
            darken_screen: event.darken_screen,
            play_music: event.play_boss_music,
            create_world_fog: event.create_world_fog,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$AddOperation#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let name = buf.read_component();
        let progress = buf.read_float();
        let color = buf.read_enum::<BossBarColor>();
        let overlay = buf.read_enum::<BossBarOverlay>();
        let flags = buf.read_unsigned_byte();
        let darken_screen = (flags & 1) > 0;
        let play_music = (flags & 2) > 0;
        let create_world_fog = (flags & 4) > 0;
        Self {
            name,
            progress,
            color,
            overlay,
            darken_screen,
            play_music,
            create_world_fog,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$AddOperation#getType
    pub fn get_type(&self) -> OperationType {
        OperationType::Add
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$AddOperation#dispatch
    pub fn dispatch(&self, id: Uuid, handler: &mut dyn Handler) {
        handler.add(
            id,
            self.name.clone(),
            self.progress,
            self.color,
            self.overlay,
            self.darken_screen,
            self.play_music,
            self.create_world_fog,
        );
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$AddOperation#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_component(&self.name);
        buf.write_float(self.progress);
        buf.write_enum(self.color);
        buf.write_enum(self.overlay);
        buf.write_byte(ClientboundBossEventPacket::encode_properties(
            self.darken_screen,
            self.play_music,
            self.create_world_fog,
        ));
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateProgressOperation {
    pub progress: f32,
}

impl UpdateProgressOperation {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateProgressOperation#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        Self {
            progress: buf.read_float(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateProgressOperation#getType
    pub fn get_type(&self) -> OperationType {
        OperationType::UpdateProgress
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateProgressOperation#dispatch
    pub fn dispatch(&self, id: Uuid, handler: &mut dyn Handler) {
        handler.update_progress(id, self.progress);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateProgressOperation#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_float(self.progress);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateNameOperation {
    pub name: Component,
}

impl UpdateNameOperation {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateNameOperation#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        Self {
            name: buf.read_component(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateNameOperation#getType
    pub fn get_type(&self) -> OperationType {
        OperationType::UpdateName
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateNameOperation#dispatch
    pub fn dispatch(&self, id: Uuid, handler: &mut dyn Handler) {
        handler.update_name(id, self.name.clone());
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateNameOperation#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_component(&self.name);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStyleOperation {
    pub color: BossBarColor,
    pub overlay: BossBarOverlay,
}

impl UpdateStyleOperation {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateStyleOperation#<init>(BossBarColor,BossBarOverlay)
    pub fn new(color: BossBarColor, overlay: BossBarOverlay) -> Self {
        Self { color, overlay }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateStyleOperation#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let color = buf.read_enum::<BossBarColor>();
        let overlay = buf.read_enum::<BossBarOverlay>();
        Self { color, overlay }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateStyleOperation#getType
    pub fn get_type(&self) -> OperationType {
        OperationType::UpdateStyle
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateStyleOperation#dispatch
    pub fn dispatch(&self, id: Uuid, handler: &mut dyn Handler) {
        handler.update_style(id, self.color, self.overlay);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdateStyleOperation#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_enum(self.color);
        buf.write_enum(self.overlay);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatePropertiesOperation {
    pub darken_screen: bool,
    pub play_music: bool,
    pub create_world_fog: bool,
}

impl UpdatePropertiesOperation {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdatePropertiesOperation#<init>(boolean,boolean,boolean)
    pub fn new(darken_screen: bool, play_music: bool, create_world_fog: bool) -> Self {
        Self {
            darken_screen,
            play_music,
            create_world_fog,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdatePropertiesOperation#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let flags = buf.read_unsigned_byte();
        let darken_screen = (flags & 1) > 0;
        let play_music = (flags & 2) > 0;
        let create_world_fog = (flags & 4) > 0;
        Self {
            darken_screen,
            play_music,
            create_world_fog,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdatePropertiesOperation#getType
    pub fn get_type(&self) -> OperationType {
        OperationType::UpdateProperties
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdatePropertiesOperation#dispatch
    pub fn dispatch(&self, id: Uuid, handler: &mut dyn Handler) {
        handler.update_properties(id, self.darken_screen, self.play_music, self.create_world_fog);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$UpdatePropertiesOperation#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_byte(ClientboundBossEventPacket::encode_properties(
            self.darken_screen,
            self.play_music,
            self.create_world_fog,
        ));
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Add(AddOperation),
    Remove,
    UpdateProgress(UpdateProgressOperation),
    UpdateName(UpdateNameOperation),
    UpdateStyle(UpdateStyleOperation),
    UpdateProperties(UpdatePropertiesOperation),
}

impl Operation {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$Operation#getType
    pub fn get_type(&self) -> OperationType {
        match self {
            Self::Add(op) => op.get_type(),
            Self::Remove => OperationType::Remove,
            Self::UpdateProgress(op) => op.get_type(),
            Self::UpdateName(op) => op.get_type(),
            Self::UpdateStyle(op) => op.get_type(),
            Self::UpdateProperties(op) => op.get_type(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$Operation#dispatch
    pub fn dispatch(&self, id: Uuid, handler: &mut dyn Handler) {
        match self {
            Self::Add(op) => op.dispatch(id, handler),
            Self::Remove => handler.remove(id),
            Self::UpdateProgress(op) => op.dispatch(id, handler),
            Self::UpdateName(op) => op.dispatch(id, handler),
            Self::UpdateStyle(op) => op.dispatch(id, handler),
            Self::UpdateProperties(op) => op.dispatch(id, handler),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$Operation#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        match self {
            Self::Add(op) => op.write(buf),
            Self::Remove => {}
            Self::UpdateProgress(op) => op.write(buf),
            Self::UpdateName(op) => op.write(buf),
            Self::UpdateStyle(op) => op.write(buf),
            Self::UpdateProperties(op) => op.write(buf),
        }
    }
}

impl OperationType {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket$OperationType#reader
    pub fn read_operation(&self, buf: &mut RegistryFriendlyByteBuf) -> Operation {
        match self {
            Self::Add => Operation::Add(AddOperation::read(buf)),
            Self::Remove => Operation::Remove,
            Self::UpdateProgress => Operation::UpdateProgress(UpdateProgressOperation::read(buf)),
            Self::UpdateName => Operation::UpdateName(UpdateNameOperation::read(buf)),
            Self::UpdateStyle => Operation::UpdateStyle(UpdateStyleOperation::read(buf)),
            Self::UpdateProperties => {
                Operation::UpdateProperties(UpdatePropertiesOperation::read(buf))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundBossEventPacket {
    pub id: Uuid,
    pub operation: Operation,
}

impl ClientboundBossEventPacket {
    pub const REMOVE_OPERATION: Operation = Operation::Remove;

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#<init>(UUID,Operation)
    pub fn new(id: Uuid, operation: Operation) -> Self {
        Self { id, operation }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let id = buf.read_uuid();
        let operation_type = buf.read_enum::<OperationType>();
        let operation = operation_type.read_operation(buf);
        Self { id, operation }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#createAddPacket
    pub fn create_add_packet(event: &BossEvent) -> Self {
        Self {
            id: event.id,
            operation: Operation::Add(AddOperation::from_boss_event(event)),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#createRemovePacket
    pub fn create_remove_packet(id: Uuid) -> Self {
        Self {
            id,
            operation: Self::REMOVE_OPERATION,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#createUpdateProgressPacket
    pub fn create_update_progress_packet(event: &BossEvent) -> Self {
        Self {
            id: event.id,
            operation: Operation::UpdateProgress(UpdateProgressOperation {
                progress: event.progress,
            }),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#createUpdateNamePacket
    pub fn create_update_name_packet(event: &BossEvent) -> Self {
        Self {
            id: event.id,
            operation: Operation::UpdateName(UpdateNameOperation {
                name: event.name.clone(),
            }),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#createUpdateStylePacket
    pub fn create_update_style_packet(event: &BossEvent) -> Self {
        Self {
            id: event.id,
            operation: Operation::UpdateStyle(UpdateStyleOperation {
                color: event.color,
                overlay: event.overlay,
            }),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#createUpdatePropertiesPacket
    pub fn create_update_properties_packet(event: &BossEvent) -> Self {
        Self {
            id: event.id,
            operation: Operation::UpdateProperties(UpdatePropertiesOperation {
                darken_screen: event.darken_screen,
                play_music: event.play_boss_music,
                create_world_fog: event.create_world_fog,
            }),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_uuid(self.id);
        buf.write_enum(self.operation.get_type());
        self.operation.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#encodeProperties
    pub fn encode_properties(darken_screen: bool, play_music: bool, create_world_fog: bool) -> i32 {
        let mut flags: i32 = 0;
        if darken_screen {
            flags |= 1;
        }
        if play_music {
            flags |= 2;
        }
        if create_world_fog {
            flags |= 4;
        }
        flags
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#type
    pub fn type_(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_BOSS_EVENT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_boss_update(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBossEventPacket#dispatch
    pub fn dispatch(&self, handler: &mut dyn Handler) {
        self.operation.dispatch(self.id, handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHandler {
        added: Vec<(
            Uuid,
            Component,
            f32,
            BossBarColor,
            BossBarOverlay,
            bool,
            bool,
            bool,
        )>,
        removed: Vec<Uuid>,
        progress_updates: Vec<(Uuid, f32)>,
        name_updates: Vec<(Uuid, Component)>,
        style_updates: Vec<(Uuid, BossBarColor, BossBarOverlay)>,
        property_updates: Vec<(Uuid, bool, bool, bool)>,
    }

    impl MockHandler {
        fn new() -> Self {
            Self {
                added: Vec::new(),
                removed: Vec::new(),
                progress_updates: Vec::new(),
                name_updates: Vec::new(),
                style_updates: Vec::new(),
                property_updates: Vec::new(),
            }
        }
    }

    impl Handler for MockHandler {
        fn add(
            &mut self,
            id: Uuid,
            name: Component,
            progress: f32,
            color: BossBarColor,
            overlay: BossBarOverlay,
            darken_screen: bool,
            play_music: bool,
            create_world_fog: bool,
        ) {
            self.added.push((
                id,
                name,
                progress,
                color,
                overlay,
                darken_screen,
                play_music,
                create_world_fog,
            ));
        }

        fn remove(&mut self, id: Uuid) {
            self.removed.push(id);
        }

        fn update_progress(&mut self, id: Uuid, progress: f32) {
            self.progress_updates.push((id, progress));
        }

        fn update_name(&mut self, id: Uuid, name: Component) {
            self.name_updates.push((id, name));
        }

        fn update_style(&mut self, id: Uuid, color: BossBarColor, overlay: BossBarOverlay) {
            self.style_updates.push((id, color, overlay));
        }

        fn update_properties(
            &mut self,
            id: Uuid,
            darken_screen: bool,
            play_music: bool,
            create_world_fog: bool,
        ) {
            self.property_updates
                .push((id, darken_screen, play_music, create_world_fog));
        }
    }

    struct MockListener {
        handled: usize,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_boss_update(&mut self, _packet: &ClientboundBossEventPacket) {
            self.handled = self.handled.wrapping_add(1);
        }
    }

    fn sample_event() -> BossEvent {
        BossEvent {
            id: Uuid {
                most_significant_bits: 0x0123_4567_89AB_CDEFu64 as i64,
                least_significant_bits: 0xFEDC_BA98_7654_3210u64 as i64,
            },
            name: Component("Dragon".to_string()),
            progress: 0.75f32,
            color: BossBarColor::Purple,
            overlay: BossBarOverlay::Notched10,
            darken_screen: true,
            play_boss_music: false,
            create_world_fog: true,
        }
    }

    #[test]
    fn test_encode_properties_flags() {
        assert_eq!(
            ClientboundBossEventPacket::encode_properties(false, false, false),
            0
        );
        assert_eq!(
            ClientboundBossEventPacket::encode_properties(true, false, false),
            1
        );
        assert_eq!(
            ClientboundBossEventPacket::encode_properties(false, true, false),
            2
        );
        assert_eq!(
            ClientboundBossEventPacket::encode_properties(false, false, true),
            4
        );
        assert_eq!(
            ClientboundBossEventPacket::encode_properties(true, true, true),
            7
        );
        assert_eq!(
            ClientboundBossEventPacket::encode_properties(true, false, true),
            5
        );
    }

    #[test]
    fn test_add_operation_round_trip_and_float_bits() {
        let id = Uuid {
            most_significant_bits: -1i64,
            least_significant_bits: i64::MIN,
        };
        let progress_bits: u32 = 0xBF80_0000; // -1.0f
        let event = BossEvent {
            id,
            name: Component("Boss".to_string()),
            progress: f32::from_bits(progress_bits),
            color: BossBarColor::Purple,
            overlay: BossBarOverlay::Notched12,
            darken_screen: true,
            play_boss_music: false,
            create_world_fog: true,
        };

        let packet = ClientboundBossEventPacket::create_add_packet(&event);
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded =
            ClientboundBossEventPacket::read(&mut RegistryFriendlyByteBuf::from_vec(buf.into_vec()));

        assert_eq!(decoded.id, id);
        match decoded.operation {
            Operation::Add(op) => {
                assert_eq!(op.name, Component("Boss".to_string()));
                assert_eq!(op.progress.to_bits(), progress_bits);
                assert_eq!(op.color, BossBarColor::Purple);
                assert_eq!(op.overlay, BossBarOverlay::Notched12);
                assert!(op.darken_screen);
                assert!(!op.play_music);
                assert!(op.create_world_fog);
            }
            _ => panic!("expected Add operation"),
        }
    }

    #[test]
    fn test_update_operations_round_trip() {
        let event = sample_event();
        let id = event.id;

        let packet = ClientboundBossEventPacket::create_update_progress_packet(&BossEvent {
            progress: f32::INFINITY,
            ..event.clone()
        });
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded =
            ClientboundBossEventPacket::read(&mut RegistryFriendlyByteBuf::from_vec(buf.into_vec()));
        match decoded.operation {
            Operation::UpdateProgress(op) => {
                assert_eq!(op.progress.to_bits(), f32::INFINITY.to_bits())
            }
            _ => panic!("expected UpdateProgress"),
        }

        let packet = ClientboundBossEventPacket::create_update_name_packet(&BossEvent {
            name: Component("Renamed".to_string()),
            ..event.clone()
        });
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded =
            ClientboundBossEventPacket::read(&mut RegistryFriendlyByteBuf::from_vec(buf.into_vec()));
        match decoded.operation {
            Operation::UpdateName(op) => assert_eq!(op.name, Component("Renamed".to_string())),
            _ => panic!("expected UpdateName"),
        }

        let packet = ClientboundBossEventPacket::create_update_style_packet(&BossEvent {
            color: BossBarColor::Green,
            overlay: BossBarOverlay::Notched20,
            ..event.clone()
        });
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded =
            ClientboundBossEventPacket::read(&mut RegistryFriendlyByteBuf::from_vec(buf.into_vec()));
        match decoded.operation {
            Operation::UpdateStyle(op) => {
                assert_eq!(op.color, BossBarColor::Green);
                assert_eq!(op.overlay, BossBarOverlay::Notched20);
            }
            _ => panic!("expected UpdateStyle"),
        }

        let packet = ClientboundBossEventPacket::create_update_properties_packet(&BossEvent {
            darken_screen: false,
            play_boss_music: true,
            create_world_fog: true,
            ..event.clone()
        });
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded =
            ClientboundBossEventPacket::read(&mut RegistryFriendlyByteBuf::from_vec(buf.into_vec()));
        match decoded.operation {
            Operation::UpdateProperties(op) => {
                assert!(!op.darken_screen);
                assert!(op.play_music);
                assert!(op.create_world_fog);
            }
            _ => panic!("expected UpdateProperties"),
        }

        let packet = ClientboundBossEventPacket::create_remove_packet(id);
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded =
            ClientboundBossEventPacket::read(&mut RegistryFriendlyByteBuf::from_vec(buf.into_vec()));
        assert_eq!(decoded.id, id);
        assert_eq!(decoded.operation, Operation::Remove);
    }

    #[test]
    fn test_dispatch_and_handle_routes_correctly() {
        let id = Uuid {
            most_significant_bits: 1,
            least_significant_bits: 2,
        };
        let mut handler = MockHandler::new();

        let packet = ClientboundBossEventPacket::new(
            id,
            Operation::UpdateProgress(UpdateProgressOperation { progress: 0.5f32 }),
        );
        packet.dispatch(&mut handler);
        assert_eq!(handler.progress_updates.len(), 1);
        assert_eq!(handler.progress_updates[0].0, id);
        assert_eq!(handler.progress_updates[0].1.to_bits(), 0.5f32.to_bits());

        let packet = ClientboundBossEventPacket::create_remove_packet(id);
        packet.dispatch(&mut handler);
        assert_eq!(handler.removed, vec![id]);

        let packet = ClientboundBossEventPacket::new(
            id,
            Operation::UpdateName(UpdateNameOperation {
                name: Component("N".to_string()),
            }),
        );
        packet.dispatch(&mut handler);
        assert_eq!(handler.name_updates.len(), 1);
        assert_eq!(handler.name_updates[0].1, Component("N".to_string()));

        let mut listener = MockListener { handled: 0 };
        packet.handle(&mut listener);
        assert_eq!(listener.handled, 1);
        assert_eq!(packet.type_().0, "clientbound_boss_event");
    }

    #[test]
    fn test_flags_masking_and_negative_uuid() {
        let id = Uuid {
            most_significant_bits: i64::MIN,
            least_significant_bits: -1i64,
        };

        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_uuid(id);
        buf.write_enum(OperationType::UpdateProperties);
        buf.write_byte(0xFFu8 as i32);
        let decoded = ClientboundBossEventPacket::read(&mut buf);
        assert_eq!(decoded.id, id);
        match decoded.operation {
            Operation::UpdateProperties(op) => {
                assert!(op.darken_screen);
                assert!(op.play_music);
                assert!(op.create_world_fog);
            }
            _ => panic!("expected UpdateProperties"),
        }

        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_uuid(id);
        buf.write_enum(OperationType::Add);
        buf.write_component(&Component("X".to_string()));
        buf.write_float(0.25f32);
        buf.write_enum(BossBarColor::Red);
        buf.write_enum(BossBarOverlay::Notched6);
        buf.write_byte(0xFAu8 as i32); // 11111010 -> darken=false, music=true, fog=false
        let decoded = ClientboundBossEventPacket::read(&mut buf);
        match decoded.operation {
            Operation::Add(op) => {
                assert!(!op.darken_screen);
                assert!(op.play_music);
                assert!(!op.create_world_fog);
                assert_eq!(op.progress.to_bits(), 0.25f32.to_bits());
                assert_eq!(op.color, BossBarColor::Red);
                assert_eq!(op.overlay, BossBarOverlay::Notched6);
            }
            _ => panic!("expected Add"),
        }
    }
}
