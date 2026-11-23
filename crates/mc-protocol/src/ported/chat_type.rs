use std::io::{self, Cursor, Read, Write};
use std::sync::Arc;

/// Minimal self-contained representation of a Minecraft chat component.
/// In the full port this maps to `net.minecraft.network.chat.Component`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Component {
    Literal(String),
    Translatable {
        key: String,
        args: Vec<Component>,
    },
}

impl Component {
    // JAVA: net.minecraft.network.chat.Component#literal(String)
    pub fn literal(text: impl Into<String>) -> Self {
        Component::Literal(text.into())
    }

    // JAVA: net.minecraft.network.chat.Component#translatable(String, Object...)
    pub fn translatable(key: impl Into<String>, args: Vec<Component>) -> Self {
        Component::Translatable {
            key: key.into(),
            args,
        }
    }

    fn as_plain_text(&self) -> String {
        match self {
            Component::Literal(s) => s.clone(),
            Component::Translatable { key, args } => {
                // Minimal placeholder rendering for self-contained testing:
                // replace "%s" sequentially with args' plain text.
                let mut out = key.clone();
                for arg in args {
                    let arg_text = arg.as_plain_text();
                    if let Some(pos) = out.find("%s") {
                        out.replace_range(pos..pos + 2, &arg_text);
                    } else {
                        out.push(' ');
                        out.push_str(&arg_text);
                    }
                }
                out
            }
        }
    }
}

/// Minimal local stand-in for `net.minecraft.resources.Identifier`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace(String)
    pub fn with_default_namespace(path: impl Into<String>) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.into(),
        }
    }

    pub fn to_string_repr(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

/// Minimal local stand-in for `net.minecraft.resources.ResourceKey<T>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey<T> {
    pub registry: Identifier,
    pub location: Identifier,
    _marker: std::marker::PhantomData<T>,
}

impl<T> ResourceKey<T> {
    // JAVA: net.minecraft.resources.ResourceKey#create(ResourceKey, Identifier)
    pub fn create(registry: Identifier, location: Identifier) -> Self {
        Self {
            registry,
            location,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Minimal local stand-in for registry key constants used by ChatType.
pub struct Registries;

impl Registries {
    // JAVA: net.minecraft.core.registries.Registries#CHAT_TYPE
    pub fn chat_type() -> Identifier {
        Identifier::with_default_namespace("chat_type")
    }
}

/// Minimal local stand-in for `net.minecraft.core.Holder<T>`.
#[derive(Clone, Debug)]
pub struct Holder<T> {
    value: Arc<T>,
}

impl<T> Holder<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(value),
        }
    }

    // JAVA: net.minecraft.core.Holder#value()
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T: PartialEq> PartialEq for Holder<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.as_ref() == other.value.as_ref()
    }
}

impl<T: Eq> Eq for Holder<T> {}

/// Minimal local stand-in for `net.minecraft.core.Registry<T>`.
pub trait Registry<T> {
    fn get_or_throw(&self, key: &ResourceKey<T>) -> Holder<T>;
}

/// Minimal local stand-in for `net.minecraft.core.RegistryAccess`.
pub trait RegistryAccess {
    fn lookup_or_throw_chat_type(&self) -> Box<dyn Registry<ChatType> + '_>;
}

/// Minimal local stand-in for `net.minecraft.world.entity.Entity`.
pub trait Entity {
    fn level_registry_access(&self) -> &dyn RegistryAccess;
    fn get_display_name(&self) -> Component;
}

/// Minimal local stand-in for `net.minecraft.commands.CommandSourceStack`.
pub trait CommandSourceStack {
    fn registry_access(&self) -> &dyn RegistryAccess;
    fn get_display_name(&self) -> Component;
}

/// Minimal local stand-in for `net.minecraft.data.worldgen.BootstrapContext<T>`.
pub trait BootstrapContext<T> {
    fn register(&mut self, key: ResourceKey<T>, value: T);
}

/// Minimal local stand-in for `net.minecraft.network.chat.ChatTypeDecoration`.
/// Captures the translation key and parameter style needed for decoration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatTypeDecoration {
    pub translation_key: String,
    pub style: ChatTypeDecorationStyle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatTypeDecorationStyle {
    /// Sender + content: args [sender, content]
    WithSender,
    /// Incoming direct message: args [sender, content] (same shape, different key)
    IncomingDirectMessage,
    /// Outgoing direct message: args [target, content]
    OutgoingDirectMessage,
    /// Team message: args [team, sender, content] in Java; simplified to [sender, content]
    /// while preserving the translation key distinction.
    TeamMessage,
}

impl ChatTypeDecoration {
    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#withSender(String)
    pub fn with_sender(key: impl Into<String>) -> Self {
        Self {
            translation_key: key.into(),
            style: ChatTypeDecorationStyle::WithSender,
        }
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#incomingDirectMessage(String)
    pub fn incoming_direct_message(key: impl Into<String>) -> Self {
        Self {
            translation_key: key.into(),
            style: ChatTypeDecorationStyle::IncomingDirectMessage,
        }
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#outgoingDirectMessage(String)
    pub fn outgoing_direct_message(key: impl Into<String>) -> Self {
        Self {
            translation_key: key.into(),
            style: ChatTypeDecorationStyle::OutgoingDirectMessage,
        }
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#teamMessage(String)
    pub fn team_message(key: impl Into<String>) -> Self {
        Self {
            translation_key: key.into(),
            style: ChatTypeDecorationStyle::TeamMessage,
        }
    }

    // JAVA: net.minecraft.network.chat.ChatTypeDecoration#decorate(Component, ChatType.Bound)
    pub fn decorate(&self, content: &Component, bound: &Bound) -> Component {
        match self.style {
            ChatTypeDecorationStyle::WithSender
            | ChatTypeDecorationStyle::IncomingDirectMessage => Component::translatable(
                &self.translation_key,
                vec![bound.name.clone(), content.clone()],
            ),
            ChatTypeDecorationStyle::OutgoingDirectMessage => {
                let target = bound
                    .target_name
                    .clone()
                    .unwrap_or_else(|| bound.name.clone());
                Component::translatable(
                    &self.translation_key,
                    vec![target, content.clone()],
                )
            }
            ChatTypeDecorationStyle::TeamMessage => {
                let target = bound
                    .target_name
                    .clone()
                    .unwrap_or_else(|| Component::literal(""));
                Component::translatable(
                    &self.translation_key,
                    vec![target, bound.name.clone(), content.clone()],
                )
            }
        }
    }
}

/// Port of `net.minecraft.network.chat.ChatType`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatType {
    pub chat: ChatTypeDecoration,
    pub narration: ChatTypeDecoration,
}

impl ChatType {
    // JAVA: net.minecraft.network.chat.ChatType#<init>(ChatTypeDecoration, ChatTypeDecoration)
    pub fn new(chat: ChatTypeDecoration, narration: ChatTypeDecoration) -> Self {
        Self { chat, narration }
    }

    // JAVA: net.minecraft.network.chat.ChatType#chat()
    pub fn chat(&self) -> &ChatTypeDecoration {
        &self.chat
    }

    // JAVA: net.minecraft.network.chat.ChatType#narration()
    pub fn narration(&self) -> &ChatTypeDecoration {
        &self.narration
    }

    // JAVA: net.minecraft.network.chat.ChatType#create(String)
    fn create(name: &str) -> ResourceKey<ChatType> {
        ResourceKey::create(Registries::chat_type(), Identifier::with_default_namespace(name))
    }

    // JAVA: net.minecraft.network.chat.ChatType#CHAT
    pub fn chat_key() -> ResourceKey<ChatType> {
        Self::create("chat")
    }

    // JAVA: net.minecraft.network.chat.ChatType#SAY_COMMAND
    pub fn say_command_key() -> ResourceKey<ChatType> {
        Self::create("say_command")
    }

    // JAVA: net.minecraft.network.chat.ChatType#MSG_COMMAND_INCOMING
    pub fn msg_command_incoming_key() -> ResourceKey<ChatType> {
        Self::create("msg_command_incoming")
    }

    // JAVA: net.minecraft.network.chat.ChatType#MSG_COMMAND_OUTGOING
    pub fn msg_command_outgoing_key() -> ResourceKey<ChatType> {
        Self::create("msg_command_outgoing")
    }

    // JAVA: net.minecraft.network.chat.ChatType#TEAM_MSG_COMMAND_INCOMING
    pub fn team_msg_command_incoming_key() -> ResourceKey<ChatType> {
        Self::create("team_msg_command_incoming")
    }

    // JAVA: net.minecraft.network.chat.ChatType#TEAM_MSG_COMMAND_OUTGOING
    pub fn team_msg_command_outgoing_key() -> ResourceKey<ChatType> {
        Self::create("team_msg_command_outgoing")
    }

    // JAVA: net.minecraft.network.chat.ChatType#EMOTE_COMMAND
    pub fn emote_command_key() -> ResourceKey<ChatType> {
        Self::create("emote_command")
    }

    // JAVA: net.minecraft.network.chat.ChatType#DEFAULT_CHAT_DECORATION
    pub fn default_chat_decoration() -> ChatTypeDecoration {
        ChatTypeDecoration::with_sender("chat.type.text")
    }

    // JAVA: net.minecraft.network.chat.ChatType#bootstrap(BootstrapContext)
    pub fn bootstrap<C: BootstrapContext<ChatType>>(ctx: &mut C) {
        ctx.register(
            Self::chat_key(),
            ChatType::new(
                Self::default_chat_decoration(),
                ChatTypeDecoration::with_sender("chat.type.text.narrate"),
            ),
        );
        ctx.register(
            Self::say_command_key(),
            ChatType::new(
                ChatTypeDecoration::with_sender("chat.type.announcement"),
                ChatTypeDecoration::with_sender("chat.type.text.narrate"),
            ),
        );
        ctx.register(
            Self::msg_command_incoming_key(),
            ChatType::new(
                ChatTypeDecoration::incoming_direct_message("commands.message.display.incoming"),
                ChatTypeDecoration::with_sender("chat.type.text.narrate"),
            ),
        );
        ctx.register(
            Self::msg_command_outgoing_key(),
            ChatType::new(
                ChatTypeDecoration::outgoing_direct_message("commands.message.display.outgoing"),
                ChatTypeDecoration::with_sender("chat.type.text.narrate"),
            ),
        );
        ctx.register(
            Self::team_msg_command_incoming_key(),
            ChatType::new(
                ChatTypeDecoration::team_message("chat.type.team.text"),
                ChatTypeDecoration::with_sender("chat.type.text.narrate"),
            ),
        );
        ctx.register(
            Self::team_msg_command_outgoing_key(),
            ChatType::new(
                ChatTypeDecoration::team_message("chat.type.team.sent"),
                ChatTypeDecoration::with_sender("chat.type.text.narrate"),
            ),
        );
        ctx.register(
            Self::emote_command_key(),
            ChatType::new(
                ChatTypeDecoration::with_sender("chat.type.emote"),
                ChatTypeDecoration::with_sender("chat.type.emote"),
            ),
        );
    }

    // JAVA: net.minecraft.network.chat.ChatType#bind(ResourceKey, Entity)
    pub fn bind_entity(key: &ResourceKey<ChatType>, entity: &dyn Entity) -> Bound {
        Self::bind(
            key,
            entity.level_registry_access(),
            entity.get_display_name(),
        )
    }

    // JAVA: net.minecraft.network.chat.ChatType#bind(ResourceKey, CommandSourceStack)
    pub fn bind_command_source(key: &ResourceKey<ChatType>, source: &dyn CommandSourceStack) -> Bound {
        Self::bind(key, source.registry_access(), source.get_display_name())
    }

    // JAVA: net.minecraft.network.chat.ChatType#bind(ResourceKey, RegistryAccess, Component)
    pub fn bind(key: &ResourceKey<ChatType>, registry_access: &dyn RegistryAccess, name: Component) -> Bound {
        let registry = registry_access.lookup_or_throw_chat_type();
        let holder = registry.get_or_throw(key);
        Bound::new(holder, name)
    }
}

/// Port of `net.minecraft.network.chat.ChatType$Bound`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bound {
    pub chat_type: Holder<ChatType>,
    pub name: Component,
    pub target_name: Option<Component>,
}

impl Bound {
    // JAVA: net.minecraft.network.chat.ChatType$Bound#<init>(Holder, Component, Optional)
    pub fn new_with_target(
        chat_type: Holder<ChatType>,
        name: Component,
        target_name: Option<Component>,
    ) -> Self {
        Self {
            chat_type,
            name,
            target_name,
        }
    }

    // JAVA: net.minecraft.network.chat.ChatType$Bound#<init>(Holder, Component)
    pub fn new(chat_type: Holder<ChatType>, name: Component) -> Self {
        Self::new_with_target(chat_type, name, None)
    }

    // JAVA: net.minecraft.network.chat.ChatType$Bound#decorate(Component)
    pub fn decorate(&self, content: &Component) -> Component {
        self.chat_type.value().chat().decorate(content, self)
    }

    // JAVA: net.minecraft.network.chat.ChatType$Bound#decorateNarration(Component)
    pub fn decorate_narration(&self, content: &Component) -> Component {
        self.chat_type.value().narration().decorate(content, self)
    }

    // JAVA: net.minecraft.network.chat.ChatType$Bound#withTargetName(Component)
    pub fn with_target_name(&self, target: Component) -> Self {
        Self::new_with_target(self.chat_type.clone(), self.name.clone(), Some(target))
    }
}

// --- Minimal stream helpers for self-contained codec-shaped tests ---
// These mirror the shape of Java StreamCodec boundaries without pulling in
// unported network codec infrastructure.

fn write_utf(buf: &mut Vec<u8>, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();
    let len = i32::try_from(bytes.len()).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "string too long"))?;
    write_varint(buf, len)?;
    buf.extend_from_slice(bytes);
    Ok(())
}

fn read_utf(cursor: &mut Cursor<&[u8]>) -> io::Result<String> {
    let len = read_varint(cursor)?;
    if len < 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "negative string length"));
    }
    let mut bytes = vec![0u8; len as usize];
    cursor.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn write_varint(buf: &mut Vec<u8>, mut value: i32) -> io::Result<()> {
    loop {
        let mut temp = (value as u32 & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            temp |= 0x80;
        }
        buf.write_all(&[temp])?;
        if value == 0 {
            return Ok(());
        }
    }
}

fn read_varint(cursor: &mut Cursor<&[u8]>) -> io::Result<i32> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        let mut b = [0u8; 1];
        cursor.read_exact(&mut b)?;
        let byte = b[0];
        result |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
        }
    }
}

impl ChatTypeDecoration {
    pub fn write_to(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        write_utf(buf, &self.translation_key)?;
        let style_id: i32 = match self.style {
            ChatTypeDecorationStyle::WithSender => 0,
            ChatTypeDecorationStyle::IncomingDirectMessage => 1,
            ChatTypeDecorationStyle::OutgoingDirectMessage => 2,
            ChatTypeDecorationStyle::TeamMessage => 3,
        };
        write_varint(buf, style_id)
    }

    pub fn read_from(cursor: &mut Cursor<&[u8]>) -> io::Result<Self> {
        let key = read_utf(cursor)?;
        let style_id = read_varint(cursor)?;
        let style = match style_id {
            0 => ChatTypeDecorationStyle::WithSender,
            1 => ChatTypeDecorationStyle::IncomingDirectMessage,
            2 => ChatTypeDecorationStyle::OutgoingDirectMessage,
            3 => ChatTypeDecorationStyle::TeamMessage,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unknown decoration style")),
        };
        Ok(Self { translation_key: key, style })
    }
}

impl ChatType {
    pub fn write_to(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        self.chat.write_to(buf)?;
        self.narration.write_to(buf)
    }

    pub fn read_from(cursor: &mut Cursor<&[u8]>) -> io::Result<Self> {
        let chat = ChatTypeDecoration::read_from(cursor)?;
        let narration = ChatTypeDecoration::read_from(cursor)?;
        Ok(Self::new(chat, narration))
    }
}

fn write_component(buf: &mut Vec<u8>, component: &Component) -> io::Result<()> {
    match component {
        Component::Literal(s) => {
            write_varint(buf, 0)?;
            write_utf(buf, s)
        }
        Component::Translatable { key, args } => {
            write_varint(buf, 1)?;
            write_utf(buf, key)?;
            write_varint(buf, i32::try_from(args.len()).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "too many args"))?)?;
            for arg in args {
                write_component(buf, arg)?;
            }
            Ok(())
        }
    }
}

fn read_component(cursor: &mut Cursor<&[u8]>) -> io::Result<Component> {
    let kind = read_varint(cursor)?;
    match kind {
        0 => Ok(Component::Literal(read_utf(cursor)?)),
        1 => {
            let key = read_utf(cursor)?;
            let argc = read_varint(cursor)?;
            if argc < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "negative arg count"));
            }
            let mut args = Vec::with_capacity(argc as usize);
            for _ in 0..argc {
                args.push(read_component(cursor)?);
            }
            Ok(Component::Translatable { key, args })
        }
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "unknown component kind")),
    }
}

impl Bound {
    pub fn write_to(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        // Holder<ChatType> represented as inline ChatType for self-contained codec.
        self.chat_type.value().write_to(buf)?;
        write_component(buf, &self.name)?;
        match &self.target_name {
            Some(t) => {
                buf.write_all(&[1])?;
                write_component(buf, t)?;
            }
            None => {
                buf.write_all(&[0])?;
            }
        }
        Ok(())
    }

    pub fn read_from(cursor: &mut Cursor<&[u8]>) -> io::Result<Self> {
        let chat_type = ChatType::read_from(cursor)?;
        let name = read_component(cursor)?;
        let mut has_target = [0u8; 1];
        cursor.read_exact(&mut has_target)?;
        let target_name = if has_target[0] != 0 {
            Some(read_component(cursor)?)
        } else {
            None
        };
        Ok(Self::new_with_target(Holder::new(chat_type), name, target_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestRegistry {
        map: HashMap<String, ChatType>,
    }

    impl Registry<ChatType> for TestRegistry {
        fn get_or_throw(&self, key: &ResourceKey<ChatType>) -> Holder<ChatType> {
            let k = key.location.to_string_repr();
            Holder::new(self.map.get(&k).cloned().expect("missing chat type"))
        }
    }

    struct TestRegistryAccess {
        registry: TestRegistry,
    }

    impl RegistryAccess for TestRegistryAccess {
        fn lookup_or_throw_chat_type(&self) -> Box<dyn Registry<ChatType> + '_> {
            Box::new(RegistryRef(&self.registry))
        }
    }

    struct RegistryRef<'a>(&'a TestRegistry);
    impl<'a> Registry<ChatType> for RegistryRef<'a> {
        fn get_or_throw(&self, key: &ResourceKey<ChatType>) -> Holder<ChatType> {
            self.0.get_or_throw(key)
        }
    }

    struct TestBootstrap {
        map: HashMap<String, ChatType>,
    }

    impl BootstrapContext<ChatType> for TestBootstrap {
        fn register(&mut self, key: ResourceKey<ChatType>, value: ChatType) {
            self.map.insert(key.location.to_string_repr(), value);
        }
    }

    fn bootstrap_access() -> TestRegistryAccess {
        let mut ctx = TestBootstrap { map: HashMap::new() };
        ChatType::bootstrap(&mut ctx);
        TestRegistryAccess {
            registry: TestRegistry { map: ctx.map },
        }
    }

    #[test]
    fn bootstrap_registers_all_keys_and_decorations() {
        let access = bootstrap_access();
        let chat = access.registry.get_or_throw(&ChatType::chat_key());
        assert_eq!(chat.value().chat().translation_key, "chat.type.text");
        assert_eq!(chat.value().narration().translation_key, "chat.type.text.narrate");

        let say = access.registry.get_or_throw(&ChatType::say_command_key());
        assert_eq!(say.value().chat().translation_key, "chat.type.announcement");

        let msg_in = access.registry.get_or_throw(&ChatType::msg_command_incoming_key());
        assert_eq!(msg_in.value().chat().translation_key, "commands.message.display.incoming");
        assert_eq!(msg_in.value().chat().style, ChatTypeDecorationStyle::IncomingDirectMessage);

        let msg_out = access.registry.get_or_throw(&ChatType::msg_command_outgoing_key());
        assert_eq!(msg_out.value().chat().translation_key, "commands.message.display.outgoing");
        assert_eq!(msg_out.value().chat().style, ChatTypeDecorationStyle::OutgoingDirectMessage);

        let team_in = access.registry.get_or_throw(&ChatType::team_msg_command_incoming_key());
        assert_eq!(team_in.value().chat().translation_key, "chat.type.team.text");
        assert_eq!(team_in.value().chat().style, ChatTypeDecorationStyle::TeamMessage);

        let emote = access.registry.get_or_throw(&ChatType::emote_command_key());
        assert_eq!(emote.value().chat().translation_key, "chat.type.emote");
        assert_eq!(emote.value().narration().translation_key, "chat.type.emote");
    }

    #[test]
    fn bind_and_decorate_with_sender() {
        let access = bootstrap_access();
        let bound = ChatType::bind(
            &ChatType::chat_key(),
            &access,
            Component::literal("Alice"),
        );
        let decorated = bound.decorate(&Component::literal("Hello"));
        assert_eq!(
            decorated,
            Component::translatable("chat.type.text", vec![
                Component::literal("Alice"),
                Component::literal("Hello"),
            ])
        );
        let narration = bound.decorate_narration(&Component::literal("Hello"));
        assert_eq!(
            narration,
            Component::translatable("chat.type.text.narrate", vec![
                Component::literal("Alice"),
                Component::literal("Hello"),
            ])
        );
    }

    #[test]
    fn bind_with_target_name_for_outgoing_and_team() {
        let access = bootstrap_access();
        let outgoing = ChatType::bind(
            &ChatType::msg_command_outgoing_key(),
            &access,
            Component::literal("Alice"),
        )
        .with_target_name(Component::literal("Bob"));
        let decorated = outgoing.decorate(&Component::literal("psst"));
        assert_eq!(
            decorated,
            Component::translatable("commands.message.display.outgoing", vec![
                Component::literal("Bob"),
                Component::literal("psst"),
            ])
        );

        let team = ChatType::bind(
            &ChatType::team_msg_command_incoming_key(),
            &access,
            Component::literal("Alice"),
        )
        .with_target_name(Component::literal("RedTeam"));
        let team_decorated = team.decorate(&Component::literal("hi"));
        assert_eq!(
            team_decorated,
            Component::translatable("chat.type.team.text", vec![
                Component::literal("RedTeam"),
                Component::literal("Alice"),
                Component::literal("hi"),
            ])
        );
    }

    #[test]
    fn bound_codec_round_trip() {
        let access = bootstrap_access();
        let bound = ChatType::bind(
            &ChatType::say_command_key(),
            &access,
            Component::literal("Server"),
        )
        .with_target_name(Component::literal("ignored-target"));

        let mut buf = Vec::new();
        bound.write_to(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf.as_slice());
        let decoded = Bound::read_from(&mut cursor).unwrap();

        assert_eq!(decoded.name, bound.name);
        assert_eq!(decoded.target_name, bound.target_name);
        assert_eq!(decoded.chat_type.value().chat().translation_key, "chat.type.announcement");
        assert_eq!(
            decoded.decorate(&Component::literal("msg")),
            bound.decorate(&Component::literal("msg"))
        );
    }

    #[test]
    fn varint_round_trip_edge_cases() {
        let cases: &[i32] = &[0, 1, 127, 128, 255, -1, i32::MIN, i32::MAX];
        for &v in cases {
            let mut buf = Vec::new();
            write_varint(&mut buf, v).unwrap();
            let mut cursor = Cursor::new(buf.as_slice());
            let decoded = read_varint(&mut cursor).unwrap();
            assert_eq!(decoded, v, "varint round-trip failed for {v}");
        }
    }

    #[test]
    fn resource_key_and_identifier_format() {
        let key = ChatType::chat_key();
        assert_eq!(key.registry.to_string_repr(), "minecraft:chat_type");
        assert_eq!(key.location.to_string_repr(), "minecraft:chat");
        let custom = ResourceKey::<ChatType>::create(
            Registries::chat_type(),
            Identifier::with_default_namespace("custom"),
        );
        assert_eq!(custom.location.path, "custom");
        assert_eq!(custom.location.namespace, "minecraft");
    }
}
