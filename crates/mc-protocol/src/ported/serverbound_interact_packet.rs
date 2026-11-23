//! Port of `net.minecraft.network.protocol.game.ServerboundInteractPacket` (Java 1.21.11).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionHand {
    MainHand,
    OffHand,
}

pub trait FriendlyEnum: Sized {
    fn from_id(id: i32) -> Self;
    fn to_id(&self) -> i32;
}

impl FriendlyEnum for InteractionHand {
    fn from_id(id: i32) -> Self {
        match id {
            0 => Self::MainHand,
            1 => Self::OffHand,
            _ => panic!("Invalid InteractionHand id: {id}"),
        }
    }

    fn to_id(&self) -> i32 {
        match self {
            Self::MainHand => 0,
            Self::OffHand => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            pos: 0,
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.data[self.pos];
        self.pos = self.pos.wrapping_add(1);
        b
    }

    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_byte();
            value |= ((current_byte & 0x7F) as i32).wrapping_shl(position & 31);
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

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7Fu32) == 0 {
                self.write_byte(val as u8);
                return;
            }
            self.write_byte(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
    }

    pub fn read_boolean(&mut self) -> bool {
        self.read_byte() != 0
    }

    pub fn write_boolean(&mut self, value: bool) {
        self.write_byte(if value { 1 } else { 0 });
    }

    pub fn read_float(&mut self) -> f32 {
        let b0 = self.read_byte();
        let b1 = self.read_byte();
        let b2 = self.read_byte();
        let b3 = self.read_byte();
        f32::from_be_bytes([b0, b1, b2, b3])
    }

    pub fn write_float(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_enum<T: FriendlyEnum>(&mut self) -> T {
        T::from_id(self.read_var_int())
    }

    pub fn write_enum<T: FriendlyEnum>(&mut self, value: &T) {
        self.write_var_int(value.to_id());
    }
}

pub trait Entity {
    fn get_id(&self) -> i32;
}

pub trait ServerLevel {
    type EntityType: Entity;
    fn get_entity_or_part(&self, id: i32) -> Option<Self::EntityType>;
}

pub trait ServerPlayer {
    fn is_within_attack_range(&self, aabb: &Aabb, distance: f64) -> bool;
    fn is_within_entity_interaction_range(&self, aabb: &Aabb, distance: f64) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub const GAME_PACKET_TYPES_SERVERBOUND_INTERACT: PacketType = PacketType;

pub trait Handler {
    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$Handler#onInteraction
    fn on_interaction(&mut self, hand: InteractionHand);
    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$Handler#onInteraction
    fn on_interaction_at(&mut self, hand: InteractionHand, location: Vec3);
    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$Handler#onAttack
    fn on_attack(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Interact,
    Attack,
    InteractAt,
}

impl FriendlyEnum for ActionType {
    fn from_id(id: i32) -> Self {
        match id {
            0 => Self::Interact,
            1 => Self::Attack,
            2 => Self::InteractAt,
            _ => panic!("Invalid ActionType id: {id}"),
        }
    }

    fn to_id(&self) -> i32 {
        match self {
            Self::Interact => 0,
            Self::Attack => 1,
            Self::InteractAt => 2,
        }
    }
}

impl ActionType {
    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$ActionType#reader
    pub fn reader(self, buf: &mut FriendlyByteBuf) -> Action {
        match self {
            ActionType::Interact => Action::Interact(InteractionAction::read(buf)),
            ActionType::Attack => Action::Attack(AttackAction),
            ActionType::InteractAt => Action::InteractAt(InteractionAtLocationAction::read(buf)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackAction;

impl AttackAction {
    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$1#getType
    pub fn get_type(&self) -> ActionType {
        ActionType::Attack
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$1#dispatch
    pub fn dispatch<H: Handler>(&self, handler: &mut H) {
        handler.on_attack();
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$1#write
    pub fn write(&self, _buf: &mut FriendlyByteBuf) {}
}

pub const ATTACK_ACTION: AttackAction = AttackAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionAction {
    pub hand: InteractionHand,
}

impl InteractionAction {
    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAction#<init>
    pub fn new(hand: InteractionHand) -> Self {
        Self { hand }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAction#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let hand = buf.read_enum();
        Self { hand }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAction#getType
    pub fn get_type(&self) -> ActionType {
        ActionType::Interact
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAction#dispatch
    pub fn dispatch<H: Handler>(&self, handler: &mut H) {
        handler.on_interaction(self.hand);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAction#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_enum(&self.hand);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InteractionAtLocationAction {
    pub hand: InteractionHand,
    pub location: Vec3,
}

impl InteractionAtLocationAction {
    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAtLocationAction#<init>
    pub fn new(hand: InteractionHand, location: Vec3) -> Self {
        Self { hand, location }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAtLocationAction#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let location = Vec3::new(
            buf.read_float() as f64,
            buf.read_float() as f64,
            buf.read_float() as f64,
        );
        let hand = buf.read_enum();
        Self { hand, location }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAtLocationAction#getType
    pub fn get_type(&self) -> ActionType {
        ActionType::InteractAt
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAtLocationAction#dispatch
    pub fn dispatch<H: Handler>(&self, handler: &mut H) {
        handler.on_interaction_at(self.hand, self.location);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$InteractionAtLocationAction#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_float(self.location.x as f32);
        buf.write_float(self.location.y as f32);
        buf.write_float(self.location.z as f32);
        buf.write_enum(&self.hand);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Interact(InteractionAction),
    Attack(AttackAction),
    InteractAt(InteractionAtLocationAction),
}

impl Action {
    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$Action#getType
    pub fn get_type(&self) -> ActionType {
        match self {
            Action::Interact(a) => a.get_type(),
            Action::Attack(a) => a.get_type(),
            Action::InteractAt(a) => a.get_type(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$Action#dispatch
    pub fn dispatch<H: Handler>(&self, handler: &mut H) {
        match self {
            Action::Interact(a) => a.dispatch(handler),
            Action::Attack(a) => a.dispatch(handler),
            Action::InteractAt(a) => a.dispatch(handler),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket$Action#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        match self {
            Action::Interact(a) => a.write(buf),
            Action::Attack(a) => a.write(buf),
            Action::InteractAt(a) => a.write(buf),
        }
    }
}

pub trait ServerGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ServerGamePacketListener#handleInteract
    fn handle_interact(&mut self, packet: &ServerboundInteractPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundInteractPacket {
    entity_id: i32,
    action: Action,
    using_secondary_action: bool,
}

impl ServerboundInteractPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#<init>
    fn new(entity_id: i32, using_secondary_action: bool, action: Action) -> Self {
        Self {
            entity_id,
            action,
            using_secondary_action,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#createAttackPacket
    pub fn create_attack_packet<E: Entity>(entity: &E, using_secondary_action: bool) -> Self {
        Self::new(entity.get_id(), using_secondary_action, Action::Attack(ATTACK_ACTION))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#createInteractionPacket
    pub fn create_interaction_packet<E: Entity>(
        entity: &E,
        using_secondary_action: bool,
        hand: InteractionHand,
    ) -> Self {
        Self::new(
            entity.get_id(),
            using_secondary_action,
            Action::Interact(InteractionAction::new(hand)),
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#createInteractionPacket
    pub fn create_interaction_packet_at<E: Entity>(
        entity: &E,
        using_secondary_action: bool,
        hand: InteractionHand,
        location: Vec3,
    ) -> Self {
        Self::new(
            entity.get_id(),
            using_secondary_action,
            Action::InteractAt(InteractionAtLocationAction::new(hand, location)),
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let entity_id = buf.read_var_int();
        let action_type: ActionType = buf.read_enum();
        let action = action_type.reader(buf);
        let using_secondary_action = buf.read_boolean();
        Self {
            entity_id,
            action,
            using_secondary_action,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        buf.write_enum(&self.action.get_type());
        self.action.write(buf);
        buf.write_boolean(self.using_secondary_action);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#type
    pub fn type_(&self) -> PacketType {
        GAME_PACKET_TYPES_SERVERBOUND_INTERACT
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_interact(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#getTarget
    pub fn get_target<L: ServerLevel>(&self, level: &L) -> Option<L::EntityType> {
        level.get_entity_or_part(self.entity_id)
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#isUsingSecondaryAction
    pub fn is_using_secondary_action(&self) -> bool {
        self.using_secondary_action
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#isWithinRange
    pub fn is_within_range<P: ServerPlayer>(&self, player: &P, aabb: &Aabb, distance: f64) -> bool {
        if self.action.get_type() == ActionType::Attack {
            player.is_within_attack_range(aabb, distance)
        } else {
            player.is_within_entity_interaction_range(aabb, distance)
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundInteractPacket#dispatch
    pub fn dispatch<H: Handler>(&self, handler: &mut H) {
        self.action.dispatch(handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct TestEntity(i32);

    impl Entity for TestEntity {
        fn get_id(&self) -> i32 {
            self.0
        }
    }

    struct TestHandler {
        attacked: bool,
        interacted: Option<InteractionHand>,
        interacted_at: Option<(InteractionHand, Vec3)>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                attacked: false,
                interacted: None,
                interacted_at: None,
            }
        }
    }

    impl Handler for TestHandler {
        fn on_interaction(&mut self, hand: InteractionHand) {
            self.interacted = Some(hand);
        }

        fn on_interaction_at(&mut self, hand: InteractionHand, location: Vec3) {
            self.interacted_at = Some((hand, location));
        }

        fn on_attack(&mut self) {
            self.attacked = true;
        }
    }

    #[test]
    fn roundtrip_attack_and_negative_entity_id() {
        let entity = TestEntity(-1);
        let packet = ServerboundInteractPacket::create_attack_packet(&entity, true);
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ServerboundInteractPacket::read(&mut read_buf);
        assert_eq!(decoded.entity_id, -1);
        assert!(decoded.is_using_secondary_action());
        assert_eq!(decoded.action.get_type(), ActionType::Attack);
        assert_eq!(decoded.type_(), GAME_PACKET_TYPES_SERVERBOUND_INTERACT);
    }

    #[test]
    fn roundtrip_interact_hand() {
        let entity = TestEntity(42);
        let packet =
            ServerboundInteractPacket::create_interaction_packet(&entity, false, InteractionHand::OffHand);
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded = ServerboundInteractPacket::read(&mut FriendlyByteBuf::from_vec(buf.into_vec()));
        match decoded.action {
            Action::Interact(a) => assert_eq!(a.hand, InteractionHand::OffHand),
            _ => panic!("expected interact action"),
        }
        assert!(!decoded.is_using_secondary_action());
    }

    #[test]
    fn roundtrip_interact_at_float_bits() {
        let entity = TestEntity(7);
        let x_bits: u32 = 0xBF80_0001; // -1.0000001f32
        let y_bits: u32 = (-0.0f32).to_bits();
        let z_bits: u32 = f32::INFINITY.to_bits();
        let loc = Vec3::new(
            f32::from_bits(x_bits) as f64,
            f32::from_bits(y_bits) as f64,
            f32::from_bits(z_bits) as f64,
        );
        let packet = ServerboundInteractPacket::create_interaction_packet_at(
            &entity,
            false,
            InteractionHand::MainHand,
            loc,
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded = ServerboundInteractPacket::read(&mut FriendlyByteBuf::from_vec(buf.into_vec()));
        match decoded.action {
            Action::InteractAt(a) => {
                assert_eq!((a.location.x as f32).to_bits(), x_bits);
                assert_eq!((a.location.y as f32).to_bits(), y_bits);
                assert_eq!((a.location.z as f32).to_bits(), z_bits);
                assert_eq!(a.hand, InteractionHand::MainHand);
            }
            _ => panic!("expected interact_at action"),
        }
    }

    #[test]
    fn dispatch_routes_to_correct_handler() {
        let entity = TestEntity(1);

        let mut h = TestHandler::new();
        ServerboundInteractPacket::create_attack_packet(&entity, false).dispatch(&mut h);
        assert!(h.attacked);

        let mut h2 = TestHandler::new();
        ServerboundInteractPacket::create_interaction_packet(&entity, false, InteractionHand::MainHand)
            .dispatch(&mut h2);
        assert_eq!(h2.interacted, Some(InteractionHand::MainHand));

        let mut h3 = TestHandler::new();
        let loc = Vec3::new(1.0, 2.0, 3.0);
        ServerboundInteractPacket::create_interaction_packet_at(
            &entity,
            false,
            InteractionHand::OffHand,
            loc,
        )
        .dispatch(&mut h3);
        assert_eq!(h3.interacted_at, Some((InteractionHand::OffHand, loc)));
    }

    #[test]
    fn is_within_range_uses_attack_or_interaction() {
        struct P {
            attack: bool,
            interact: bool,
        }
        impl ServerPlayer for P {
            fn is_within_attack_range(&self, _aabb: &Aabb, _distance: f64) -> bool {
                self.attack
            }
            fn is_within_entity_interaction_range(&self, _aabb: &Aabb, _distance: f64) -> bool {
                self.interact
            }
        }

        let aabb = Aabb {
            min_x: 0.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 1.0,
            max_y: 1.0,
            max_z: 1.0,
        };
        let player = P {
            attack: true,
            interact: false,
        };
        let entity = TestEntity(5);
        let attack = ServerboundInteractPacket::create_attack_packet(&entity, false);
        let interact =
            ServerboundInteractPacket::create_interaction_packet(&entity, false, InteractionHand::MainHand);

        assert!(attack.is_within_range(&player, &aabb, 3.0));
        assert!(!interact.is_within_range(&player, &aabb, 3.0));
    }

    #[test]
    fn varint_negative_and_max_roundtrip() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(i32::MIN);
        buf.write_var_int(i32::MAX);
        buf.write_var_int(-1);
        buf.write_var_int(0);
        let mut r = FriendlyByteBuf::from_vec(buf.into_vec());
        assert_eq!(r.read_var_int(), i32::MIN);
        assert_eq!(r.read_var_int(), i32::MAX);
        assert_eq!(r.read_var_int(), -1);
        assert_eq!(r.read_var_int(), 0);
    }
}
