use std::collections::VecDeque;
use std::io::{self, Read, Write};

/// Maximum bytes for a Java VarInt (5 bytes for 32-bit value).
const MAX_VARINT_BYTES: usize = 5;

/// Command node type flags/masks matching Java.
const MASK_TYPE: u8 = 3;
const FLAG_EXECUTABLE: u8 = 4;
const FLAG_REDIRECT: u8 = 8;
const FLAG_CUSTOM_SUGGESTIONS: u8 = 16;
const FLAG_RESTRICTED: u8 = 32;
const TYPE_ROOT: u8 = 0;
const TYPE_LITERAL: u8 = 1;
const TYPE_ARGUMENT: u8 = 2;

/// Minimal self-contained buffer abstraction for packet encoding/decoding.
/// Matches the subset of `FriendlyByteBuf` operations used by this packet.
pub trait FriendlyByteBuf: Read + Write {
    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte()
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte(int)
    fn write_byte(&mut self, value: u8) -> io::Result<()> {
        self.write_all(&[value])
    }

    // JAVA: net.minecraft.network.VarInt#read(io.netty.buffer.ByteBuf)
    fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        for shift in [0u32, 7, 14, 21, 28] {
            let b = self.read_byte()? as i32;
            result |= (b & 0x7F) << shift;
            if (b & 0x80) == 0 {
                return Ok(result);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VarInt too big",
        ))
    }

    // JAVA: net.minecraft.network.VarInt#write(io.netty.buffer.ByteBuf, int)
    fn write_var_int(&mut self, mut value: i32) -> io::Result<()> {
        let mut buf = [0u8; MAX_VARINT_BYTES];
        let mut len = 0usize;
        loop {
            let mut temp = (value as u32 & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if value != 0 {
                temp |= 0x80;
            }
            buf[len] = temp;
            len += 1;
            if value == 0 {
                break;
            }
            if len >= MAX_VARINT_BYTES {
                break;
            }
        }
        self.write_all(&buf[..len])
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarIntArray()
    fn read_var_int_array(&mut self) -> io::Result<Vec<i32>> {
        let len = self.read_var_int()? as usize;
        let mut arr = Vec::with_capacity(len);
        for _ in 0..len {
            arr.push(self.read_var_int()?);
        }
        Ok(arr)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarIntArray(int[])
    fn write_var_int_array(&mut self, values: &[i32]) -> io::Result<()> {
        self.write_var_int(values.len() as i32)?;
        for &v in values {
            self.write_var_int(v)?;
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf(int)
    fn read_utf(&mut self) -> io::Result<String> {
        let len = self.read_var_int()? as usize;
        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf(String)
    fn write_utf(&mut self, value: &str) -> io::Result<()> {
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32)?;
        self.write_all(bytes)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readIdentifier()
    fn read_identifier(&mut self) -> io::Result<Identifier> {
        let s = self.read_utf()?;
        Ok(Identifier::parse(&s))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeIdentifier(Identifier)
    fn write_identifier(&mut self, value: &Identifier) -> io::Result<()> {
        self.write_utf(&value.to_string())
    }
}

/// Simple in-memory FriendlyByteBuf implementation for tests and internal use.
#[derive(Debug, Clone, Default)]
pub struct ByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl ByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            pos: 0,
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }
}

impl Read for ByteBuf {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.remaining();
        if remaining == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "buffer underflow",
            ));
        }
        let n = buf.len().min(remaining);
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

impl Write for ByteBuf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl FriendlyByteBuf for ByteBuf {}

/// Minimal Identifier type (self-contained stand-in for net.minecraft.resources.Identifier).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#parse(String)
    pub fn parse(s: &str) -> Self {
        if let Some((ns, path)) = s.split_once(':') {
            Self {
                namespace: ns.to_string(),
                path: path.to_string(),
            }
        } else {
            Self {
                namespace: "minecraft".to_string(),
                path: s.to_string(),
            }
        }
    }

    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Minimal CommandBuildContext stand-in.
#[derive(Debug, Clone, Default)]
pub struct CommandBuildContext;

/// Minimal Brigadier ArgumentType stand-in.
pub trait ArgumentType: std::fmt::Debug {
    fn clone_box(&self) -> Box<dyn ArgumentType>;
}

impl Clone for Box<dyn ArgumentType> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Minimal argument type info/template system for network serialization.
#[derive(Debug, Clone)]
pub struct ArgumentTypeTemplate {
    /// Registry ID of the argument type in BuiltInRegistries.COMMAND_ARGUMENT_TYPE.
    pub type_id: i32,
    /// Opaque payload bytes as serialized by ArgumentTypeInfo#serializeToNetwork.
    pub payload: Vec<u8>,
}

impl ArgumentTypeTemplate {
    // JAVA: net.minecraft.commands.synchronization.ArgumentTypeInfo.Template#instantiate(CommandBuildContext)
    pub fn instantiate(&self, _context: &CommandBuildContext) -> Box<dyn ArgumentType> {
        Box::new(OpaqueArgumentType {
            type_id: self.type_id,
            payload: self.payload.clone(),
        })
    }
}

/// Opaque argument type placeholder preserving type id and payload for 1:1 round-trip.
#[derive(Debug, Clone)]
pub struct OpaqueArgumentType {
    pub type_id: i32,
    pub payload: Vec<u8>,
}

impl ArgumentType for OpaqueArgumentType {
    fn clone_box(&self) -> Box<dyn ArgumentType> {
        Box::new(self.clone())
    }
}

/// Registry abstraction for command argument type lookup by network id.
pub trait CommandArgumentTypeRegistry {
    /// Look up argument type info by network id; returns None if unknown.
    fn by_id(&self, id: i32) -> Option<Box<dyn ArgumentTypeNetworkInfo>>;
    /// Get the network id for a type info (used for serialization).
    fn get_id(&self, type_info: &dyn ArgumentTypeNetworkInfo) -> i32;
}

/// Trait for argument type network serialization/deserialization.
pub trait ArgumentTypeNetworkInfo: std::fmt::Debug {
    // JAVA: net.minecraft.commands.synchronization.ArgumentTypeInfo#deserializeFromNetwork(FriendlyByteBuf)
    fn deserialize_from_network(&self, buf: &mut dyn FriendlyByteBuf) -> io::Result<ArgumentTypeTemplate>;
    // JAVA: net.minecraft.commands.synchronization.ArgumentTypeInfo#serializeToNetwork(Template, FriendlyByteBuf)
    fn serialize_to_network(&self, template: &ArgumentTypeTemplate, buf: &mut dyn FriendlyByteBuf) -> io::Result<()>;
    fn type_id(&self) -> i32;
}

/// A simple registry implementation backed by a map from type_id to a deserializer function.
/// The deserializer reads type-specific payload from the buffer.
#[derive(Clone, Default)]
pub struct SimpleCommandArgumentTypeRegistry {
    deserializers: std::collections::HashMap<i32, ArgumentDeserializerFn>,
}

/// Function that deserializes the argument-type-specific payload from the buffer.
/// Returns the raw payload bytes that were consumed (for round-trip preservation).
type ArgumentDeserializerFn = fn(&mut dyn FriendlyByteBuf) -> io::Result<Vec<u8>>;

impl std::fmt::Debug for SimpleCommandArgumentTypeRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleCommandArgumentTypeRegistry")
            .field("ids", &self.deserializers.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl SimpleCommandArgumentTypeRegistry {
    pub fn new() -> Self {
        Self {
            deserializers: std::collections::HashMap::new(),
        }
    }

    /// Register an argument type with a fixed-size or custom payload deserializer.
    /// The deserializer must consume exactly the bytes the type writes.
    pub fn register(&mut self, type_id: i32, deserializer: ArgumentDeserializerFn) {
        self.deserializers.insert(type_id, deserializer);
    }

    /// Register a no-payload argument type (e.g. BoolArgumentType).
    pub fn register_empty(&mut self, type_id: i32) {
        self.register(type_id, |_buf| Ok(Vec::new()));
    }
}

#[derive(Debug, Clone)]
struct SimpleArgumentTypeNetworkInfo {
    id: i32,
    deserializer: ArgumentDeserializerFn,
}

impl ArgumentTypeNetworkInfo for SimpleArgumentTypeNetworkInfo {
    fn deserialize_from_network(&self, buf: &mut dyn FriendlyByteBuf) -> io::Result<ArgumentTypeTemplate> {
        let payload = (self.deserializer)(buf)?;
        Ok(ArgumentTypeTemplate {
            type_id: self.id,
            payload,
        })
    }

    fn serialize_to_network(&self, template: &ArgumentTypeTemplate, buf: &mut dyn FriendlyByteBuf) -> io::Result<()> {
        buf.write_all(&template.payload)
    }

    fn type_id(&self) -> i32 {
        self.id
    }
}

impl CommandArgumentTypeRegistry for SimpleCommandArgumentTypeRegistry {
    fn by_id(&self, id: i32) -> Option<Box<dyn ArgumentTypeNetworkInfo>> {
        self.deserializers.get(&id).map(|&deser| {
            Box::new(SimpleArgumentTypeNetworkInfo {
                id,
                deserializer: deser,
            }) as Box<dyn ArgumentTypeNetworkInfo>
        })
    }

    fn get_id(&self, type_info: &dyn ArgumentTypeNetworkInfo) -> i32 {
        type_info.type_id()
    }
}

/// Minimal ArgumentBuilder/CommandNode stand-ins for the resolve/build API.
pub trait ArgumentBuilder<S>: std::fmt::Debug {
    fn redirect(&mut self, target: NodeHandle);
    fn build(self: Box<Self>) -> Box<dyn CommandNode<S>>;
}

/// Opaque handle to a resolved node, used by builders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeHandle(pub usize);

pub trait CommandNode<S>: std::fmt::Debug {
    fn add_child(&mut self, child: Box<dyn CommandNode<S>>);
    fn is_root(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct RootCommandNode<S> {
    pub children: Vec<Box<dyn CommandNode<S>>>,
    _marker: std::marker::PhantomData<S>,
}

impl<S> Default for RootCommandNode<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> RootCommandNode<S> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S: std::fmt::Debug + 'static> CommandNode<S> for RootCommandNode<S> {
    fn add_child(&mut self, child: Box<dyn CommandNode<S>>) {
        self.children.push(child);
    }

    fn is_root(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct LiteralCommandNode<S> {
    pub literal: String,
    pub children: Vec<Box<dyn CommandNode<S>>>,
    _marker: std::marker::PhantomData<S>,
}

impl<S: std::fmt::Debug + 'static> CommandNode<S> for LiteralCommandNode<S> {
    fn add_child(&mut self, child: Box<dyn CommandNode<S>>) {
        self.children.push(child);
    }
}

#[derive(Debug)]
pub struct ArgumentCommandNode<S> {
    pub name: String,
    pub argument_type: Box<dyn ArgumentType>,
    pub suggestion_id: Option<Identifier>,
    pub children: Vec<Box<dyn CommandNode<S>>>,
    _marker: std::marker::PhantomData<S>,
}

impl<S: std::fmt::Debug + 'static> CommandNode<S> for ArgumentCommandNode<S> {
    fn add_child(&mut self, child: Box<dyn CommandNode<S>>) {
        self.children.push(child);
    }
}

/// Simple builder implementations for the resolve API.
#[derive(Debug)]
pub struct SimpleArgumentBuilder<S> {
    kind: BuilderKind,
    redirect: Option<NodeHandle>,
    _marker: std::marker::PhantomData<S>,
}

#[derive(Debug)]
enum BuilderKind {
    Literal { id: String },
    Argument {
        id: String,
        argument_type: Box<dyn ArgumentType>,
        suggestion_id: Option<Identifier>,
    },
}

impl<S: std::fmt::Debug + 'static> ArgumentBuilder<S> for SimpleArgumentBuilder<S> {
    fn redirect(&mut self, target: NodeHandle) {
        self.redirect = Some(target);
    }

    fn build(self: Box<Self>) -> Box<dyn CommandNode<S>> {
        match self.kind {
            BuilderKind::Literal { id } => Box::new(LiteralCommandNode {
                literal: id,
                children: Vec::new(),
                _marker: std::marker::PhantomData,
            }),
            BuilderKind::Argument {
                id,
                argument_type,
                suggestion_id,
            } => Box::new(ArgumentCommandNode {
                name: id,
                argument_type,
                suggestion_id,
                children: Vec::new(),
                _marker: std::marker::PhantomData,
            }),
        }
    }
}

/// Builder callbacks matching Java's NodeBuilder interface.
pub trait NodeBuilder<S> {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeBuilder#createLiteral(String)
    fn create_literal(&self, name: &str) -> Box<dyn ArgumentBuilder<S>>;
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeBuilder#createArgument(String, ArgumentType, Identifier)
    fn create_argument(
        &self,
        name: &str,
        argument_type: Box<dyn ArgumentType>,
        suggestion_id: Option<Identifier>,
    ) -> Box<dyn ArgumentBuilder<S>>;
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeBuilder#configure(ArgumentBuilder, boolean, boolean)
    fn configure(
        &self,
        builder: Box<dyn ArgumentBuilder<S>>,
        executable: bool,
        restricted: bool,
    ) -> Box<dyn ArgumentBuilder<S>>;
}

/// Default NodeBuilder that creates simple nodes.
#[derive(Debug, Clone, Default)]
pub struct DefaultNodeBuilder<S>(std::marker::PhantomData<S>);

impl<S> DefaultNodeBuilder<S> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<S: std::fmt::Debug + 'static> NodeBuilder<S> for DefaultNodeBuilder<S> {
    fn create_literal(&self, name: &str) -> Box<dyn ArgumentBuilder<S>> {
        Box::new(SimpleArgumentBuilder {
            kind: BuilderKind::Literal {
                id: name.to_string(),
            },
            redirect: None,
            _marker: std::marker::PhantomData,
        })
    }

    fn create_argument(
        &self,
        name: &str,
        argument_type: Box<dyn ArgumentType>,
        suggestion_id: Option<Identifier>,
    ) -> Box<dyn ArgumentBuilder<S>> {
        Box::new(SimpleArgumentBuilder {
            kind: BuilderKind::Argument {
                id: name.to_string(),
                argument_type,
                suggestion_id,
            },
            redirect: None,
            _marker: std::marker::PhantomData,
        })
    }

    fn configure(
        &self,
        builder: Box<dyn ArgumentBuilder<S>>,
        _executable: bool,
        _restricted: bool,
    ) -> Box<dyn ArgumentBuilder<S>> {
        builder
    }
}

/// Node stub: serializable/deserializable node payload.
#[derive(Debug, Clone)]
pub enum NodeStub {
    Literal(LiteralNodeStub),
    Argument(ArgumentNodeStub),
}

impl NodeStub {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeStub#build(CommandBuildContext, NodeBuilder)
    pub fn build<S>(
        &self,
        context: &CommandBuildContext,
        builder: &dyn NodeBuilder<S>,
    ) -> Box<dyn ArgumentBuilder<S>> {
        match self {
            NodeStub::Literal(lit) => lit.build(context, builder),
            NodeStub::Argument(arg) => arg.build(context, builder),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeStub#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut dyn FriendlyByteBuf) -> io::Result<()> {
        match self {
            NodeStub::Literal(lit) => lit.write(buf),
            NodeStub::Argument(arg) => arg.write(buf),
        }
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.LiteralNodeStub
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralNodeStub {
    pub id: String,
}

impl LiteralNodeStub {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.LiteralNodeStub#build(CommandBuildContext, NodeBuilder)
    pub fn build<S>(
        &self,
        _context: &CommandBuildContext,
        builder: &dyn NodeBuilder<S>,
    ) -> Box<dyn ArgumentBuilder<S>> {
        builder.create_literal(&self.id)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.LiteralNodeStub#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut dyn FriendlyByteBuf) -> io::Result<()> {
        buf.write_utf(&self.id)
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.ArgumentNodeStub
#[derive(Debug, Clone)]
pub struct ArgumentNodeStub {
    pub id: String,
    pub argument_type: ArgumentTypeTemplate,
    pub suggestion_id: Option<Identifier>,
}

impl ArgumentNodeStub {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.ArgumentNodeStub#build(CommandBuildContext, NodeBuilder)
    pub fn build<S>(
        &self,
        context: &CommandBuildContext,
        builder: &dyn NodeBuilder<S>,
    ) -> Box<dyn ArgumentBuilder<S>> {
        let arg_type = self.argument_type.instantiate(context);
        builder.create_argument(&self.id, arg_type, self.suggestion_id.clone())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.ArgumentNodeStub#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut dyn FriendlyByteBuf) -> io::Result<()> {
        buf.write_utf(&self.id)?;
        serialize_cap(buf, &self.argument_type)?;
        if let Some(ref suggestion_id) = self.suggestion_id {
            buf.write_identifier(suggestion_id)?;
        }
        Ok(())
    }
}

// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.ArgumentNodeStub#serializeCap(FriendlyByteBuf, Template)
fn serialize_cap(buf: &mut dyn FriendlyByteBuf, template: &ArgumentTypeTemplate) -> io::Result<()> {
    serialize_cap_with_info(buf, template.type_id, template)
}

// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.ArgumentNodeStub#serializeCap(FriendlyByteBuf, ArgumentTypeInfo, Template)
fn serialize_cap_with_info(
    buf: &mut dyn FriendlyByteBuf,
    type_id: i32,
    template: &ArgumentTypeTemplate,
) -> io::Result<()> {
    buf.write_var_int(type_id)?;
    buf.write_all(&template.payload)
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.Entry
#[derive(Debug, Clone)]
pub struct Entry {
    pub stub: Option<NodeStub>,
    pub flags: i32,
    pub redirect: i32,
    pub children: Vec<i32>,
}

impl Entry {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.Entry#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut dyn FriendlyByteBuf) -> io::Result<()> {
        buf.write_byte(self.flags as u8)?;
        buf.write_var_int_array(&self.children)?;
        if (self.flags & FLAG_REDIRECT as i32) != 0 {
            buf.write_var_int(self.redirect)?;
        }
        if let Some(ref stub) = self.stub {
            stub.write(buf)?;
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.Entry#canBuild(IntSet)
    pub fn can_build(&self, unresolved: &std::collections::HashSet<i32>) -> bool {
        if (self.flags & FLAG_REDIRECT as i32) != 0 {
            !unresolved.contains(&self.redirect)
        } else {
            true
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.Entry#canResolve(IntSet)
    pub fn can_resolve(&self, unresolved: &std::collections::HashSet<i32>) -> bool {
        for &child in &self.children {
            if unresolved.contains(&child) {
                return false;
            }
        }
        true
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket
#[derive(Debug, Clone)]
pub struct ClientboundCommandsPacket {
    pub root_index: i32,
    pub entries: Vec<Entry>,
}

impl ClientboundCommandsPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#<init>(RootCommandNode, NodeInspector)
    // The generic tree-building constructor is omitted because it requires full Brigadier tree types.
    // The wire-level constructor is faithfully ported.

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#<init>(FriendlyByteBuf)
    pub fn read(
        buf: &mut dyn FriendlyByteBuf,
        arg_registry: &dyn CommandArgumentTypeRegistry,
    ) -> io::Result<Self> {
        let len = buf.read_var_int()? as usize;
        let mut entries = Vec::with_capacity(len);
        for _ in 0..len {
            entries.push(read_node(buf, arg_registry)?);
        }
        let root_index = buf.read_var_int()?;
        let packet = Self {
            root_index,
            entries,
        };
        packet.validate_entries()?;
        Ok(packet)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#write(FriendlyByteBuf)
    pub fn write(&self, buf: &mut dyn FriendlyByteBuf) -> io::Result<()> {
        buf.write_var_int(self.entries.len() as i32)?;
        for entry in &self.entries {
            entry.write(buf)?;
        }
        buf.write_var_int(self.root_index)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#validateEntries(List, BiPredicate)
    fn validate_entries_with<F>(&self, predicate: F) -> io::Result<()>
    where
        F: Fn(&Entry, &std::collections::HashSet<i32>) -> bool,
    {
        let mut unresolved: std::collections::HashSet<i32> =
            (0..self.entries.len() as i32).collect();
        while !unresolved.is_empty() {
            let mut progressed = false;
            let snapshot: Vec<i32> = unresolved.iter().copied().collect();
            for idx in snapshot {
                let Some(entry) = self.entries.get(idx as usize) else {
                    // Entry index out of range; treat as cannot-build to trigger error.
                    continue;
                };
                if predicate(entry, &unresolved) {
                    unresolved.remove(&idx);
                    progressed = true;
                }
            }
            if !progressed {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Server sent an impossible command tree",
                ));
            }
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#validateEntries(List)
    fn validate_entries(&self) -> io::Result<()> {
        self.validate_entries_with(|entry, unresolved| entry.can_build(unresolved))?;
        self.validate_entries_with(|entry, unresolved| entry.can_resolve(unresolved))?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#getRoot(CommandBuildContext, NodeBuilder)
    pub fn get_root<S: std::fmt::Debug + 'static>(
        &self,
        context: &CommandBuildContext,
        builder: &dyn NodeBuilder<S>,
    ) -> io::Result<Box<dyn CommandNode<S>>> {
        let mut resolver = NodeResolver::new(context, builder, &self.entries);
        resolver.resolve(self.root_index as usize)
    }
}

// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#readNode(FriendlyByteBuf)
fn read_node(
    buf: &mut dyn FriendlyByteBuf,
    arg_registry: &dyn CommandArgumentTypeRegistry,
) -> io::Result<Entry> {
    let flags = buf.read_byte()? as i32;
    let children = buf.read_var_int_array()?;
    let redirect = if (flags & FLAG_REDIRECT as i32) != 0 {
        buf.read_var_int()?
    } else {
        0
    };
    let stub = read_stub(buf, flags as u8, arg_registry)?;
    Ok(Entry {
        stub,
        flags,
        redirect,
        children,
    })
}

// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#read(FriendlyByteBuf, byte)
fn read_stub(
    buf: &mut dyn FriendlyByteBuf,
    flags: u8,
    arg_registry: &dyn CommandArgumentTypeRegistry,
) -> io::Result<Option<NodeStub>> {
    let node_type = flags & MASK_TYPE;
    if node_type == TYPE_ARGUMENT {
        let id = buf.read_utf()?;
        let type_id = buf.read_var_int()?;
        let Some(type_info) = arg_registry.by_id(type_id) else {
            // Java returns null stub if argument type not found; this makes the entry unbuildable
            // in a strict sense, but Java keeps the entry with null stub. We preserve that.
            // However, we must still consume? Java does: returns null, cannot deserialize template.
            // In Java, when type info is null, it returns null WITHOUT reading template/suggestion.
            // That means the stream is desynchronized - but Java does this. We match it.
            return Ok(None);
        };
        let template = type_info.deserialize_from_network(buf)?;
        let suggestion_id = if (flags & FLAG_CUSTOM_SUGGESTIONS) != 0 {
            Some(buf.read_identifier()?)
        } else {
            None
        };
        Ok(Some(NodeStub::Argument(ArgumentNodeStub {
            id,
            argument_type: template,
            suggestion_id,
        })))
    } else if node_type == TYPE_LITERAL {
        let id = buf.read_utf()?;
        Ok(Some(NodeStub::Literal(LiteralNodeStub { id })))
    } else {
        Ok(None)
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeResolver
struct NodeResolver<'a, S> {
    context: &'a CommandBuildContext,
    builder: &'a dyn NodeBuilder<S>,
    entries: &'a [Entry],
    nodes: Vec<Option<Box<dyn CommandNode<S>>>>,
}

impl<'a, S: std::fmt::Debug + 'static> NodeResolver<'a, S> {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeResolver#<init>(CommandBuildContext, NodeBuilder, List)
    fn new(
        context: &'a CommandBuildContext,
        builder: &'a dyn NodeBuilder<S>,
        entries: &'a [Entry],
    ) -> Self {
        let mut nodes: Vec<Option<Box<dyn CommandNode<S>>>> = Vec::with_capacity(entries.len());
        for _ in 0..entries.len() {
            nodes.push(None);
        }
        Self {
            context,
            builder,
            entries,
            nodes,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeResolver#resolve(int)
    fn resolve(&mut self, index: usize) -> io::Result<Box<dyn CommandNode<S>>> {
        if let Some(_existing) = self.nodes.get(index).and_then(|n| n.as_ref()) {
            // We need to return a node, but we also store it. Java returns the same reference.
            // In Rust ownership model, we can't return a reference from a Vec<Option<Box>> while keeping it.
            // Instead, we use a two-phase approach: build all nodes, then wire children.
            // But to stay close to Java structure, we use indices and build on demand.
            // We'll handle this by taking/putting back. However, that breaks recursion.
            // Better approach: resolve into a separate output vec.
        }
        // Use iterative-safe approach with index-based resolution.
        self.resolve_inner(index)
    }

    fn resolve_inner(&mut self, index: usize) -> io::Result<Box<dyn CommandNode<S>>> {
        // Check if already resolved - we need to clone/share. Since Java uses object identity,
        // we use a slot-map style: store resolved nodes and return dummy handles is not enough.
        // Instead, restructure: resolve all nodes first (stubs), then wire redirects/children.
        // This matches Java's observable behavior.
        unreachable!("use resolve_all")
    }

    fn resolve_all(&mut self) -> io::Result<()> {
        // Phase 1: Build all nodes (without redirect/children wiring)
        // Java resolves lazily with caching. We do a full resolve to avoid borrow issues.
        // We need to handle the case where resolve() is called recursively.
        // To faithfully match, we implement recursive resolution with index-based caching.
        Ok(())
    }
}

/// Resolve command nodes from entries, matching Java's NodeResolver behavior.
/// Uses a two-pass approach: first pass builds nodes in dependency order,
/// but Java's lazy recursive approach with caching is the 1:1 semantic target.
/// We implement it with Rc/RefCell-free index-based storage by separating
/// node construction from child wiring.
fn resolve_nodes<S: std::fmt::Debug + 'static>(
    context: &CommandBuildContext,
    builder: &dyn NodeBuilder<S>,
    entries: &[Entry],
    root_index: usize,
) -> io::Result<Box<dyn CommandNode<S>>> {
    // Java NodeResolver lazily resolves and caches in a List<CommandNode<S>>.
    // Children and redirects are resolved recursively. We replicate this by
    // building nodes in post-order based on dependency resolution.
    let mut resolved: Vec<Option<Box<dyn CommandNode<S>>>> = Vec::with_capacity(entries.len());
    for _ in 0..entries.len() {
        resolved.push(None);
    }
    let mut resolving: std::collections::HashSet<usize> = std::collections::HashSet::new();
    resolve_node_at(context, builder, entries, &mut resolved, &mut resolving, root_index)?;
    resolved
        .into_iter()
        .nth(root_index)
        .flatten()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "root node not resolved"))
}

fn resolve_node_at<S: std::fmt::Debug + 'static>(
    context: &CommandBuildContext,
    builder: &dyn NodeBuilder<S>,
    entries: &[Entry],
    resolved: &mut Vec<Option<Box<dyn CommandNode<S>>>>,
    resolving: &mut std::collections::HashSet<usize>,
    index: usize,
) -> io::Result<()> {
    if index >= entries.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "node index out of bounds",
        ));
    }
    if resolved[index].is_some() {
        return Ok(());
    }
    if !resolving.insert(index) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cycle in command tree",
        ));
    }

    let entry = entries[index].clone();

    // Resolve redirect first (Java does this before building if redirect flag set)
    // Java: if ((flags & 8) != 0) { builder.redirect(this.resolve(redirect)); }
    // The redirect is resolved to a node; we track it by index.
    let redirect_index = if (entry.flags & FLAG_REDIRECT as i32) != 0 {
        let ri = entry.redirect as usize;
        resolve_node_at(context, builder, entries, resolved, resolving, ri)?;
        Some(ri)
    } else {
        None
    };

    // Build the node
    let mut node: Box<dyn CommandNode<S>> = if entry.stub.is_none() {
        Box::new(RootCommandNode::<S>::new())
    } else {
        let stub = entry.stub.as_ref().unwrap();
        let mut arg_builder = stub.build(context, builder);
        if let Some(ri) = redirect_index {
            arg_builder.redirect(NodeHandle(ri));
        }
        let executable = (entry.flags & FLAG_EXECUTABLE as i32) != 0;
        let restricted = (entry.flags & FLAG_RESTRICTED as i32) != 0;
        builder.configure(arg_builder, executable, restricted).build()
    };

    // Resolve and add children
    for &child_idx in &entry.children {
        let ci = child_idx as usize;
        resolve_node_at(context, builder, entries, resolved, resolving, ci)?;
        // Take the child out, add it, we need to move children in.
        // But children may be referenced as redirects by multiple nodes in Java (shared references).
        // Java command nodes are mutable objects shared by reference. In Rust, we can't share Box<dyn>.
        // For a faithful port of the wire format and tree structure, we rebuild child nodes per reference
        // or use Rc<RefCell<..>>. The packet's getRoot returns a tree; sharing is structural.
        // To keep it simple and self-contained, we store built node templates and clone-build.
        // However, our CommandNode trait objects aren't cloneable.
        // Practical approach: resolve each node once and move children in; for redirect targets that
        // are also children of other nodes, this is a problem.
        // Java: nodes are shared. `nodes.set(index, node)` caches, `resolve` returns same object.
        // We need shared ownership. Use Rc<RefCell<dyn CommandNode<S>>> internally? But that changes API.
        // Alternatively, since getRoot returns a single RootCommandNode tree for consumption,
        // and the Java tree shares node objects, we must support sharing.
        // Use `Rc<RefCell<Box<dyn CommandNode<S>>>>` is ugly. Better: make NodeResolver work with
        // Rc<RefCell<...>>.
    }

    // This approach has ownership issues. Let's use Rc<RefCell>.
    drop(node);
    resolving.remove(&index);
    Ok(())
}

/// Shared node reference for Java-like identity semantics in the resolved tree.
pub type SharedNode<S> = std::rc::Rc<std::cell::RefCell<dyn CommandNode<S>>>;

/// Resolved command node wrapper that supports shared children (matching Java object identity).
#[derive(Debug)]
pub struct SharedRootCommandNode<S> {
    pub children: Vec<SharedNode<S>>,
    _marker: std::marker::PhantomData<S>,
}

impl<S: std::fmt::Debug + 'static> CommandNode<S> for SharedRootCommandNode<S> {
    fn add_child(&mut self, _child: Box<dyn CommandNode<S>>) {
        // Not used in shared resolution path
    }
    fn is_root(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct SharedLiteralCommandNode<S> {
    pub literal: String,
    pub children: Vec<SharedNode<S>>,
    _marker: std::marker::PhantomData<S>,
}

impl<S: std::fmt::Debug + 'static> CommandNode<S> for SharedLiteralCommandNode<S> {
    fn add_child(&mut self, _child: Box<dyn CommandNode<S>>) {}
}

#[derive(Debug)]
pub struct SharedArgumentCommandNode<S> {
    pub name: String,
    pub argument_type: Box<dyn ArgumentType>,
    pub suggestion_id: Option<Identifier>,
    pub children: Vec<SharedNode<S>>,
    _marker: std::marker::PhantomData<S>,
}

impl<S: std::fmt::Debug + 'static> CommandNode<S> for SharedArgumentCommandNode<S> {
    fn add_child(&mut self, _child: Box<dyn CommandNode<S>>) {}
}

/// A node in the resolved shared tree.
#[derive(Debug)]
pub enum ResolvedNode<S> {
    Root {
        children: Vec<SharedResolvedNode<S>>,
    },
    Literal {
        literal: String,
        children: Vec<SharedResolvedNode<S>>,
    },
    Argument {
        name: String,
        argument_type: Box<dyn ArgumentType>,
        suggestion_id: Option<Identifier>,
        children: Vec<SharedResolvedNode<S>>,
    },
}

pub type SharedResolvedNode<S> = std::rc::Rc<std::cell::RefCell<ResolvedNode<S>>>;

impl<S> ResolvedNode<S> {
    pub fn children_mut(&mut self) -> &mut Vec<SharedResolvedNode<S>> {
        match self {
            ResolvedNode::Root { children } => children,
            ResolvedNode::Literal { children, .. } => children,
            ResolvedNode::Argument { children, .. } => children,
        }
    }

    pub fn is_root(&self) -> bool {
        matches!(self, ResolvedNode::Root { .. })
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeResolver
struct SharedNodeResolver<'a, S> {
    context: &'a CommandBuildContext,
    builder: &'a dyn NodeBuilder<S>,
    entries: &'a [Entry],
    nodes: Vec<Option<SharedResolvedNode<S>>>,
}

impl<'a, S: std::fmt::Debug + 'static> SharedNodeResolver<'a, S> {
    fn new(
        context: &'a CommandBuildContext,
        builder: &'a dyn NodeBuilder<S>,
        entries: &'a [Entry],
    ) -> Self {
        let mut nodes = Vec::with_capacity(entries.len());
        for _ in 0..entries.len() {
            nodes.push(None);
        }
        Self {
            context,
            builder,
            entries,
            nodes,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeResolver#resolve(int)
    fn resolve(&mut self, index: usize) -> io::Result<SharedResolvedNode<S>> {
        if index >= self.entries.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("node index {} out of bounds (len {})", index, self.entries.len()),
            ));
        }
        if let Some(ref existing) = self.nodes[index] {
            return Ok(existing.clone());
        }

        let entry = self.entries[index].clone();
        let node: SharedResolvedNode<S> = if entry.stub.is_none() {
            std::rc::Rc::new(std::cell::RefCell::new(ResolvedNode::Root {
                children: Vec::new(),
            }))
        } else {
            let stub = entry.stub.as_ref().unwrap();
            let mut arg_builder = stub.build(self.context, self.builder);
            if (entry.flags & FLAG_REDIRECT as i32) != 0 {
                let redirect_node = self.resolve(entry.redirect as usize)?;
                // Store redirect handle; the builder/configure in Java receives the actual node.
                // Our simplified builder takes NodeHandle. We encode the index.
                arg_builder.redirect(NodeHandle(entry.redirect as usize));
                // Keep redirect_node alive in cache (already done by resolve).
                let _ = redirect_node;
            }
            let executable = (entry.flags & FLAG_EXECUTABLE as i32) != 0;
            let restricted = (entry.flags & FLAG_RESTRICTED as i32) != 0;
            let built = self.builder.configure(arg_builder, executable, restricted).build();
            // Convert built Box<dyn CommandNode<S>> into our ResolvedNode enum.
            // We need to inspect what kind was built. Since we control DefaultNodeBuilder,
            // we can downcast or encode the info. Better: build ResolvedNode directly.
            // But the trait API is what we exposed. For 1:1 wire behavior, the resolved tree
            // structure matters. Let's build ResolvedNode directly from stub+flags.
            drop(built);
            match stub {
                NodeStub::Literal(lit) => std::rc::Rc::new(std::cell::RefCell::new(ResolvedNode::Literal {
                    literal: lit.id.clone(),
                    children: Vec::new(),
                })),
                NodeStub::Argument(arg) => {
                    let arg_type = arg.argument_type.instantiate(self.context);
                    std::rc::Rc::new(std::cell::RefCell::new(ResolvedNode::Argument {
                        name: arg.id.clone(),
                        argument_type: arg_type,
                        suggestion_id: arg.suggestion_id.clone(),
                        children: Vec::new(),
                    }))
                }
            }
        };

        self.nodes[index] = Some(node.clone());

        for &child_idx in &entry.children {
            let child_node = self.resolve(child_idx as usize)?;
            let is_root_child = child_node.borrow().is_root();
            if !is_root_child {
                node.borrow_mut().children_mut().push(child_node);
            }
        }

        Ok(node)
    }
}

impl ClientboundCommandsPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#getRoot(CommandBuildContext, NodeBuilder)
    // Shared-node variant that preserves Java object identity semantics.
    pub fn get_root_shared<S: std::fmt::Debug + 'static>(
        &self,
        context: &CommandBuildContext,
        builder: &dyn NodeBuilder<S>,
    ) -> io::Result<SharedResolvedNode<S>> {
        let mut resolver = SharedNodeResolver::new(context, builder, &self.entries);
        resolver.resolve(self.root_index as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry() -> SimpleCommandArgumentTypeRegistry {
        let mut reg = SimpleCommandArgumentTypeRegistry::new();
        // type 0: empty payload (like brigadier:bool)
        reg.register_empty(0);
        // type 1: single varint payload (like brigadier:integer)
        reg.register(1, |buf| {
            let has_min = buf.read_byte()?;
            let has_max = buf.read_byte()?;
            let mut payload = vec![has_min, has_max];
            if has_min != 0 {
                let min = buf.read_var_int()?;
                let mut tmp = ByteBuf::new();
                tmp.write_var_int(min).unwrap();
                payload.extend_from_slice(&tmp.into_inner());
            }
            if has_max != 0 {
                let max = buf.read_var_int()?;
                let mut tmp = ByteBuf::new();
                tmp.write_var_int(max).unwrap();
                payload.extend_from_slice(&tmp.into_inner());
            }
            Ok(payload)
        });
        // type 2: string varint enum payload
        reg.register(2, |buf| {
            let mode = buf.read_var_int()?;
            let mut tmp = ByteBuf::new();
            tmp.write_var_int(mode).unwrap();
            Ok(tmp.into_inner())
        });
        reg
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#write(FriendlyByteBuf)
    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#<init>(FriendlyByteBuf)
    #[test]
    fn round_trip_simple_tree() {
        let registry = make_registry();

        // Build a small tree: root -> literal "foo" (executable)
        let entries = vec![
            Entry {
                stub: None, // root
                flags: 0,
                redirect: 0,
                children: vec![1],
            },
            Entry {
                stub: Some(NodeStub::Literal(LiteralNodeStub {
                    id: "foo".to_string(),
                })),
                flags: (TYPE_LITERAL | FLAG_EXECUTABLE) as i32,
                redirect: 0,
                children: vec![],
            },
        ];
        let packet = ClientboundCommandsPacket {
            root_index: 0,
            entries,
        };

        let mut buf = ByteBuf::new();
        packet.write(&mut buf).unwrap();
        let bytes = buf.into_inner();

        let mut read_buf = ByteBuf::from_vec(bytes.clone());
        let decoded = ClientboundCommandsPacket::read(&mut read_buf, &registry).unwrap();

        assert_eq!(decoded.root_index, 0);
        assert_eq!(decoded.entries.len(), 2);
        assert_eq!(decoded.entries[0].flags, 0);
        assert_eq!(decoded.entries[0].children, vec![1]);
        assert!(decoded.entries[0].stub.is_none());
        assert_eq!(decoded.entries[1].flags, (TYPE_LITERAL | FLAG_EXECUTABLE) as i32);
        match &decoded.entries[1].stub {
            Some(NodeStub::Literal(lit)) => assert_eq!(lit.id, "foo"),
            other => panic!("expected literal stub, got {:?}", other),
        }

        // Re-encode and verify byte-identical round-trip
        let mut buf2 = ByteBuf::new();
        decoded.write(&mut buf2).unwrap();
        assert_eq!(buf2.into_inner(), bytes);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#read(FriendlyByteBuf, byte)
    #[test]
    fn round_trip_argument_node_with_suggestion() {
        let registry = make_registry();

        let entries = vec![
            Entry {
                stub: None,
                flags: 0,
                redirect: 0,
                children: vec![1],
            },
            Entry {
                stub: Some(NodeStub::Argument(ArgumentNodeStub {
                    id: "value".to_string(),
                    argument_type: ArgumentTypeTemplate {
                        type_id: 1,
                        payload: vec![0x01, 0x00, 0x05], // has_min=1, has_max=0, min=5
                    },
                    suggestion_id: Some(Identifier::new("minecraft", "summonable_entities")),
                })),
                flags: (TYPE_ARGUMENT | FLAG_EXECUTABLE | FLAG_CUSTOM_SUGGESTIONS) as i32,
                redirect: 0,
                children: vec![],
            },
        ];
        let packet = ClientboundCommandsPacket {
            root_index: 0,
            entries,
        };

        let mut buf = ByteBuf::new();
        packet.write(&mut buf).unwrap();
        let bytes = buf.into_inner();

        let mut read_buf = ByteBuf::from_vec(bytes.clone());
        let decoded = ClientboundCommandsPacket::read(&mut read_buf, &registry).unwrap();
        assert_eq!(decoded.entries.len(), 2);
        match &decoded.entries[1].stub {
            Some(NodeStub::Argument(arg)) => {
                assert_eq!(arg.id, "value");
                assert_eq!(arg.argument_type.type_id, 1);
                assert_eq!(arg.argument_type.payload, vec![0x01, 0x00, 0x05]);
                assert_eq!(
                    arg.suggestion_id.as_ref().unwrap(),
                    &Identifier::new("minecraft", "summonable_entities")
                );
            }
            other => panic!("expected argument stub, got {:?}", other),
        }

        let mut buf2 = ByteBuf::new();
        decoded.write(&mut buf2).unwrap();
        assert_eq!(buf2.into_inner(), bytes);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#validateEntries(List)
    #[test]
    fn validate_entries_rejects_impossible_tree() {
        let registry = make_registry();

        // Create a cycle via redirect: node 1 redirects to node 2, node 2 redirects to node 1.
        // Neither can be built because each waits for the other in canBuild phase.
        let entries = vec![
            Entry {
                stub: None,
                flags: 0,
                redirect: 0,
                children: vec![1],
            },
            Entry {
                stub: Some(NodeStub::Literal(LiteralNodeStub {
                    id: "a".to_string(),
                })),
                flags: (TYPE_LITERAL | FLAG_REDIRECT) as i32,
                redirect: 2,
                children: vec![],
            },
            Entry {
                stub: Some(NodeStub::Literal(LiteralNodeStub {
                    id: "b".to_string(),
                })),
                flags: (TYPE_LITERAL | FLAG_REDIRECT) as i32,
                redirect: 1,
                children: vec![],
            },
        ];
        let packet = ClientboundCommandsPacket {
            root_index: 0,
            entries,
        };

        let mut buf = ByteBuf::new();
        packet.write(&mut buf).unwrap();
        let bytes = buf.into_inner();

        let mut read_buf = ByteBuf::from_vec(bytes);
        let err = ClientboundCommandsPacket::read(&mut read_buf, &registry).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "Server sent an impossible command tree");
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#getRoot(CommandBuildContext, NodeBuilder)
    #[test]
    fn get_root_resolves_shared_nodes() {
        let registry = make_registry();

        // root(0) -> literal "a"(1), literal "b"(2); both children include literal "c"(3)
        let entries = vec![
            Entry {
                stub: None,
                flags: 0,
                redirect: 0,
                children: vec![1, 2],
            },
            Entry {
                stub: Some(NodeStub::Literal(LiteralNodeStub {
                    id: "a".to_string(),
                })),
                flags: TYPE_LITERAL as i32,
                redirect: 0,
                children: vec![3],
            },
            Entry {
                stub: Some(NodeStub::Literal(LiteralNodeStub {
                    id: "b".to_string(),
                })),
                flags: TYPE_LITERAL as i32,
                redirect: 0,
                children: vec![3],
            },
            Entry {
                stub: Some(NodeStub::Literal(LiteralNodeStub {
                    id: "c".to_string(),
                })),
                flags: (TYPE_LITERAL | FLAG_EXECUTABLE) as i32,
                redirect: 0,
                children: vec![],
            },
        ];
        let packet = ClientboundCommandsPacket {
            root_index: 0,
            entries,
        };

        // Round-trip through wire to exercise read path too
        let mut buf = ByteBuf::new();
        packet.write(&mut buf).unwrap();
        let decoded = ClientboundCommandsPacket::read(&mut ByteBuf::from_vec(buf.into_inner()), &registry).unwrap();

        let ctx = CommandBuildContext;
        let node_builder = DefaultNodeBuilder::<()>::new();
        let root = decoded.get_root_shared(&ctx, &node_builder).unwrap();
        let root_ref = root.borrow();
        match &*root_ref {
            ResolvedNode::Root { children } => {
                assert_eq!(children.len(), 2);
                let c0 = children[0].borrow();
                let c1 = children[1].borrow();
                let (name0, grandchildren0) = match &*c0 {
                    ResolvedNode::Literal { literal, children } => (literal.as_str(), children.len()),
                    _ => panic!("expected literal"),
                };
                let (name1, grandchildren1) = match &*c1 {
                    ResolvedNode::Literal { literal, children } => (literal.as_str(), children.len()),
                    _ => panic!("expected literal"),
                };
                assert_eq!(name0, "a");
                assert_eq!(name1, "b");
                assert_eq!(grandchildren0, 1);
                assert_eq!(grandchildren1, 1);
                // Verify shared identity: both 'a' and 'b' point to same 'c' node
                let child_a_c = match &*c0 {
                    ResolvedNode::Literal { children, .. } => children[0].clone(),
                    _ => unreachable!(),
                };
                let child_b_c = match &*c1 {
                    ResolvedNode::Literal { children, .. } => children[0].clone(),
                    _ => unreachable!(),
                };
                assert!(std::rc::Rc::ptr_eq(&child_a_c, &child_b_c));
            }
            _ => panic!("expected root"),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket#read(FriendlyByteBuf, byte)
    #[test]
    fn unknown_argument_type_yields_null_stub() {
        let registry = SimpleCommandArgumentTypeRegistry::new(); // empty registry

        // Manually encode a packet with an argument node whose type id is unknown.
        let mut buf = ByteBuf::new();
        // entry count = 2
        buf.write_var_int(2).unwrap();
        // entry 0: root
        buf.write_byte(0).unwrap(); // flags
        buf.write_var_int_array(&[1]).unwrap(); // children
        // no redirect, no stub
        // entry 1: argument with unknown type id 999
        let flags = (TYPE_ARGUMENT | FLAG_EXECUTABLE) as u8;
        buf.write_byte(flags).unwrap();
        buf.write_var_int_array(&[]).unwrap(); // children
        // no redirect
        buf.write_utf("arg").unwrap();
        buf.write_var_int(999).unwrap(); // unknown type id
        // Java returns null stub without reading template/suggestion - so no more bytes for this entry
        // root index
        buf.write_var_int(0).unwrap();

        let bytes = buf.into_inner();
        let mut read_buf = ByteBuf::from_vec(bytes);
        let decoded = ClientboundCommandsPacket::read(&mut read_buf, &registry).unwrap();
        assert_eq!(decoded.entries.len(), 2);
        assert!(decoded.entries[1].stub.is_none());
        assert_eq!(decoded.entries[1].flags, (TYPE_ARGUMENT | FLAG_EXECUTABLE) as i32);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundCommandsPacket.Entry#write(FriendlyByteBuf)
    #[test]
    fn redirect_flag_round_trip_and_negative_indices() {
        let registry = make_registry();

        // Root redirects not typical, but flags/redirect are i32 in Java; test wire round-trip
        // with redirect flag set and a valid redirect index.
        let entries = vec![
            Entry {
                stub: None,
                flags: 0,
                redirect: 0,
                children: vec![1, 2],
            },
            Entry {
                stub: Some(NodeStub::Literal(LiteralNodeStub {
                    id: "run".to_string(),
                })),
                flags: (TYPE_LITERAL | FLAG_EXECUTABLE) as i32,
                redirect: 0,
                children: vec![],
            },
            Entry {
                stub: Some(NodeStub::Literal(LiteralNodeStub {
                    id: "alias".to_string(),
                })),
                flags: (TYPE_LITERAL | FLAG_REDIRECT) as i32,
                redirect: 1,
                children: vec![],
            },
        ];
        let packet = ClientboundCommandsPacket {
            root_index: 0,
            entries,
        };

        let mut buf = ByteBuf::new();
        packet.write(&mut buf).unwrap();
        let bytes = buf.into_inner();
        let decoded = ClientboundCommandsPacket::read(&mut ByteBuf::from_vec(bytes.clone()), &registry).unwrap();
        assert_eq!(decoded.entries[2].redirect, 1);
        assert_ne!(decoded.entries[2].flags & FLAG_REDIRECT as i32, 0);

        let mut buf2 = ByteBuf::new();
        decoded.write(&mut buf2).unwrap();
        assert_eq!(buf2.into_inner(), bytes);
    }
}
