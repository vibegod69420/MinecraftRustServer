// === FILE: connection.rs ===
use std::collections::VecDeque;
use std::fmt;
use std::sync::Mutex;

pub const AVERAGE_PACKETS_SMOOTHING: f32 = 0.75_f32;
pub const CURRENT_PROTOCOL_VERSION: i32 = 774;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketFlow {
    Serverbound,
    Clientbound,
}

impl PacketFlow {
    pub fn get_opposite(self) -> Self {
        match self {
            PacketFlow::Serverbound => PacketFlow::Clientbound,
            PacketFlow::Clientbound => PacketFlow::Serverbound,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    Handshaking,
    Play,
    Status,
    Login,
    Configuration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientIntent {
    Status,
    Login,
    Transfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub key: String,
    pub args: Vec<String>,
}

impl Component {
    pub fn translatable(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            args: Vec::new(),
        }
    }

    pub fn translatable1(key: impl Into<String>, arg: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            args: vec![arg.into()],
        }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.args.is_empty() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "{}{:?}", self.key, self.args)
        }
    }
}

#[derive(Debug, Clone)]
pub struct DisconnectionDetails {
    pub reason: Component,
    pub report: Option<String>,
}

impl DisconnectionDetails {
    pub fn new(reason: Component) -> Self {
        Self {
            reason,
            report: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocketAddress {
    Inet { host: String, port: u16 },
    Local(String),
    Other(String),
}

impl fmt::Display for SocketAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SocketAddress::Inet { host, port } => write!(f, "{host}:{port}"),
            SocketAddress::Local(s) | SocketAddress::Other(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundlerInfo;

#[derive(Debug, Clone)]
pub struct ProtocolInfoData {
    pub id: ConnectionProtocol,
    pub flow: PacketFlow,
    pub bundler_info: Option<BundlerInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct LocalSampleLogger;

#[derive(Debug)]
pub struct BandwidthDebugMonitor {
    _logger: LocalSampleLogger,
}

impl BandwidthDebugMonitor {
    pub fn new(logger: LocalSampleLogger) -> Self {
        Self { _logger: logger }
    }

    pub fn tick(&mut self) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketHandleError {
    RunningOnDifferentThread,
    RejectedExecution,
    ClassCast(String),
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExceptionCause {
    SkipPacket(Option<String>),
    Timeout,
    ClosedChannel,
    Other(String),
}

pub type BoxPacket = Box<dyn Packet>;

pub trait Packet: fmt::Debug + Send {
    fn handle_packet(&self, listener: &mut dyn PacketListener) -> Result<(), PacketHandleError>;
}

pub trait PacketListener: Send {
    fn should_handle_message(&self, _packet: &dyn Packet) -> bool {
        true
    }

    fn flow(&self) -> PacketFlow;

    fn protocol(&self) -> ConnectionProtocol;

    fn create_disconnection_info(
        &self,
        reason: Component,
        _cause: Option<&str>,
    ) -> DisconnectionDetails {
        DisconnectionDetails::new(reason)
    }

    fn on_disconnect(&mut self, _details: &DisconnectionDetails) {}

    fn tick(&mut self) {}

    fn is_tickable(&self) -> bool {
        false
    }
}

pub enum PacketSendAction {
    Callback(Box<dyn FnOnce(bool) + Send>),
    Disconnect(DisconnectionDetails),
}

pub struct PacketSendListener;

impl PacketSendListener {
    pub fn then_run(f: impl FnOnce() + Send + 'static) -> Box<dyn FnOnce(bool) + Send> {
        Box::new(move |_success| f())
    }

    pub fn then_disconnect(details: DisconnectionDetails) -> PacketSendAction {
        PacketSendAction::Disconnect(details)
    }
}

pub trait Channel: Send {
    fn is_open(&self) -> bool;
    fn remote_address(&self) -> Option<SocketAddress>;
    fn close(&mut self);
    fn close_await_uninterruptibly(&mut self);
    fn write(&mut self, packet: BoxPacket, listener: Option<Box<dyn FnOnce(bool) + Send>>);
    fn write_and_flush(
        &mut self,
        packet: BoxPacket,
        listener: Option<Box<dyn FnOnce(bool) + Send>>,
    );
    fn flush(&mut self);
    fn event_loop_in_event_loop(&self) -> bool;
    fn event_loop_execute(&mut self, task: Box<dyn FnOnce() + Send + 'static>);
    fn config_set_auto_read(&mut self, auto_read: bool);
    fn is_local_channel(&self) -> bool;
    fn is_local_server_channel(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ClientIntentionPacket {
    pub protocol_version: i32,
    pub host: String,
    pub port: i32,
    pub intent: ClientIntent,
}

impl Packet for ClientIntentionPacket {
    fn handle_packet(&self, _listener: &mut dyn PacketListener) -> Result<(), PacketHandleError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ClientboundLoginDisconnectPacket {
    pub reason: Component,
}

impl Packet for ClientboundLoginDisconnectPacket {
    fn handle_packet(&self, _listener: &mut dyn PacketListener) -> Result<(), PacketHandleError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ClientboundDisconnectPacket {
    pub reason: Component,
}

impl Packet for ClientboundDisconnectPacket {
    fn handle_packet(&self, _listener: &mut dyn PacketListener) -> Result<(), PacketHandleError> {
        Ok(())
    }
}

pub struct SharedConstants;

impl SharedConstants {
    pub fn protocol_version() -> i32 {
        CURRENT_PROTOCOL_VERSION
    }
}

#[inline]
fn mth_lerp_f32(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

type PendingAction = Box<dyn FnOnce(&mut Connection) + Send>;

pub struct Connection {
    receiving: PacketFlow,
    send_login_disconnect: bool,
    pending_actions: Mutex<VecDeque<PendingAction>>,
    channel: Option<Box<dyn Channel>>,
    address: Option<SocketAddress>,
    disconnect_listener: Option<Box<dyn PacketListener>>,
    packet_listener: Option<Box<dyn PacketListener>>,
    disconnection_details: Option<DisconnectionDetails>,
    encrypted: bool,
    disconnection_handled: bool,
    received_packets: i32,
    sent_packets: i32,
    average_received_packets: f32,
    average_sent_packets: f32,
    tick_count: i32,
    handling_fault: bool,
    delayed_disconnect: Option<DisconnectionDetails>,
    bandwidth_debug_monitor: Option<BandwidthDebugMonitor>,
    compression_threshold: Option<i32>,
    compression_validate_decompressed: bool,
}

impl Connection {
    // JAVA: net.minecraft.network.Connection#<init>
    pub fn new(receiving: PacketFlow) -> Self {
        Self {
            receiving,
            send_login_disconnect: true,
            pending_actions: Mutex::new(VecDeque::new()),
            channel: None,
            address: None,
            disconnect_listener: None,
            packet_listener: None,
            disconnection_details: None,
            encrypted: false,
            disconnection_handled: false,
            received_packets: 0,
            sent_packets: 0,
            average_received_packets: 0.0_f32,
            average_sent_packets: 0.0_f32,
            tick_count: 0,
            handling_fault: false,
            delayed_disconnect: None,
            bandwidth_debug_monitor: None,
            compression_threshold: None,
            compression_validate_decompressed: false,
        }
    }

    // JAVA: net.minecraft.network.Connection#channelActive
    pub fn channel_active(&mut self, channel: Box<dyn Channel>) {
        self.address = channel.remote_address();
        self.channel = Some(channel);
        if let Some(details) = self.delayed_disconnect.take() {
            self.disconnect_with_details(details);
        }
    }

    // JAVA: net.minecraft.network.Connection#channelInactive
    pub fn channel_inactive(&mut self) {
        self.disconnect(Component::translatable("disconnect.endOfStream"));
    }

    // JAVA: net.minecraft.network.Connection#exceptionCaught
    pub fn exception_caught(&mut self, cause: ExceptionCause) {
        if let ExceptionCause::SkipPacket(_) = cause {
            return;
        }

        let first_fault = !self.handling_fault;
        self.handling_fault = true;
        if !self.is_connected() {
            return;
        }

        if let ExceptionCause::Timeout = cause {
            self.disconnect(Component::translatable("disconnect.timeout"));
            return;
        }

        let msg = match &cause {
            ExceptionCause::Other(m) => m.clone(),
            ExceptionCause::ClosedChannel => "ClosedChannelException".to_string(),
            ExceptionCause::SkipPacket(_) | ExceptionCause::Timeout => unreachable!(),
        };

        let reason = Component::translatable1(
            "disconnect.genericReason",
            format!("Internal Exception: {msg}"),
        );

        let disconnection_details = if let Some(listener) = self.packet_listener.as_ref() {
            listener.create_disconnection_info(reason.clone(), Some(&msg))
        } else {
            DisconnectionDetails::new(reason.clone())
        };

        if first_fault {
            if self.get_sending() == PacketFlow::Clientbound {
                let disconnect_packet: BoxPacket = if self.send_login_disconnect {
                    Box::new(ClientboundLoginDisconnectPacket {
                        reason: reason.clone(),
                    })
                } else {
                    Box::new(ClientboundDisconnectPacket {
                        reason: reason.clone(),
                    })
                };
                self.send_with_listener_and_flush(
                    disconnect_packet,
                    Some(PacketSendAction::Disconnect(disconnection_details)),
                    true,
                );
            } else {
                self.disconnect_with_details(disconnection_details);
            }
            self.set_read_only();
        } else {
            self.disconnect_with_details(disconnection_details);
        }
    }

    // JAVA: net.minecraft.network.Connection#channelRead0
    pub fn channel_read0(&mut self, packet: &dyn Packet) {
        if !self.is_connected() {
            return;
        }

        let handle_result = {
            let listener = match self.packet_listener.as_mut() {
                Some(l) => l,
                None => panic!("Received a packet before the packet listener was initialized"),
            };
            if !listener.should_handle_message(packet) {
                None
            } else {
                Some(generics_ftw(packet, &mut **listener))
            }
        };

        if let Some(result) = handle_result {
            match result {
                Ok(()) | Err(PacketHandleError::RunningOnDifferentThread) => {}
                Err(PacketHandleError::RejectedExecution) => {
                    self.disconnect(Component::translatable(
                        "multiplayer.disconnect.server_shutdown",
                    ));
                }
                Err(PacketHandleError::ClassCast(_)) => {
                    self.disconnect(Component::translatable(
                        "multiplayer.disconnect.invalid_packet",
                    ));
                }
                Err(PacketHandleError::Other(_)) => {}
            }
            self.received_packets = self.received_packets.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.network.Connection#validateListener
    fn validate_listener(&self, protocol_info: &ProtocolInfoData, listener: &dyn PacketListener) {
        let flow = listener.flow();
        if flow != self.receiving {
            panic!(
                "Trying to set listener for wrong side: connection is {:?}, but listener is {:?}",
                self.receiving, flow
            );
        }
        let protocol = listener.protocol();
        if protocol_info.id != protocol {
            panic!(
                "Listener protocol ({:?}) does not match requested one {:?}",
                protocol, protocol_info
            );
        }
    }

    // JAVA: net.minecraft.network.Connection#syncAfterConfigurationChange
    fn sync_after_configuration_change() {
        // Netty ChannelFuture.syncUninterruptibly(); no-op in self-contained port.
    }

    // JAVA: net.minecraft.network.Connection#setupInboundProtocol
    pub fn setup_inbound_protocol(
        &mut self,
        protocol_info: &ProtocolInfoData,
        listener: Box<dyn PacketListener>,
    ) {
        self.validate_listener(protocol_info, listener.as_ref());
        if protocol_info.flow != self.get_receiving() {
            panic!("Invalid inbound protocol: {:?}", protocol_info.id);
        }
        self.packet_listener = Some(listener);
        self.disconnect_listener = None;
        let _ = protocol_info.bundler_info;
        Self::sync_after_configuration_change();
    }

    // JAVA: net.minecraft.network.Connection#setupOutboundProtocol
    pub fn setup_outbound_protocol(&mut self, protocol_info: &ProtocolInfoData) {
        if protocol_info.flow != self.get_sending() {
            panic!("Invalid outbound protocol: {:?}", protocol_info.id);
        }
        let _ = protocol_info.bundler_info;
        let is_login = protocol_info.id == ConnectionProtocol::Login;
        Self::sync_after_configuration_change();
        self.send_login_disconnect = is_login;
    }

    // JAVA: net.minecraft.network.Connection#setListenerForServerboundHandshake
    pub fn set_listener_for_serverbound_handshake(&mut self, listener: Box<dyn PacketListener>) {
        if self.packet_listener.is_some() {
            panic!("Listener already set");
        }
        if self.receiving == PacketFlow::Serverbound
            && listener.flow() == PacketFlow::Serverbound
            && listener.protocol() == ConnectionProtocol::Handshaking
        {
            self.packet_listener = Some(listener);
        } else {
            panic!("Invalid initial listener");
        }
    }

    // JAVA: net.minecraft.network.Connection#initiateServerboundStatusConnection
    pub fn initiate_serverbound_status_connection(
        &mut self,
        host: String,
        port: i32,
        listener: Box<dyn PacketListener>,
    ) {
        let serverbound = ProtocolInfoData {
            id: ConnectionProtocol::Status,
            flow: PacketFlow::Serverbound,
            bundler_info: None,
        };
        let clientbound = ProtocolInfoData {
            id: ConnectionProtocol::Status,
            flow: PacketFlow::Clientbound,
            bundler_info: None,
        };
        self.initiate_serverbound_connection(
            host,
            port,
            serverbound,
            clientbound,
            listener,
            ClientIntent::Status,
        );
    }

    // JAVA: net.minecraft.network.Connection#initiateServerboundPlayConnection
    pub fn initiate_serverbound_play_connection(
        &mut self,
        host: String,
        port: i32,
        listener: Box<dyn PacketListener>,
    ) {
        let serverbound = ProtocolInfoData {
            id: ConnectionProtocol::Login,
            flow: PacketFlow::Serverbound,
            bundler_info: None,
        };
        let clientbound = ProtocolInfoData {
            id: ConnectionProtocol::Login,
            flow: PacketFlow::Clientbound,
            bundler_info: None,
        };
        self.initiate_serverbound_connection(
            host,
            port,
            serverbound,
            clientbound,
            listener,
            ClientIntent::Login,
        );
    }

    // JAVA: net.minecraft.network.Connection#initiateServerboundPlayConnection
    pub fn initiate_serverbound_play_connection_with_transfer(
        &mut self,
        host: String,
        port: i32,
        serverbound: ProtocolInfoData,
        clientbound: ProtocolInfoData,
        listener: Box<dyn PacketListener>,
        transfer: bool,
    ) {
        let intent = if transfer {
            ClientIntent::Transfer
        } else {
            ClientIntent::Login
        };
        self.initiate_serverbound_connection(
            host,
            port,
            serverbound,
            clientbound,
            listener,
            intent,
        );
    }

    // JAVA: net.minecraft.network.Connection#initiateServerboundConnection
    fn initiate_serverbound_connection(
        &mut self,
        host: String,
        port: i32,
        serverbound: ProtocolInfoData,
        clientbound: ProtocolInfoData,
        listener: Box<dyn PacketListener>,
        intent: ClientIntent,
    ) {
        if serverbound.id != clientbound.id {
            panic!("Mismatched initial protocols");
        }

        self.disconnect_listener = Some(listener);
        let clientbound_clone = clientbound.clone();
        let serverbound_clone = serverbound;
        self.run_once_connected(Box::new(move |conn: &mut Connection| {
            let listener = conn.disconnect_listener.take();
            if let Some(l) = listener {
                conn.setup_inbound_protocol(&clientbound_clone, l);
            }
            let intention = ClientIntentionPacket {
                protocol_version: SharedConstants::protocol_version(),
                host,
                port,
                intent,
            };
            conn.send(Box::new(intention));
            conn.setup_outbound_protocol(&serverbound_clone);
        }));
    }

    // JAVA: net.minecraft.network.Connection#send
    pub fn send(&mut self, packet: BoxPacket) {
        self.send_with_listener_and_flush(packet, None, true);
    }

    // JAVA: net.minecraft.network.Connection#send
    pub fn send_with_listener(&mut self, packet: BoxPacket, listener: Option<PacketSendAction>) {
        self.send_with_listener_and_flush(packet, listener, true);
    }

    // JAVA: net.minecraft.network.Connection#send
    pub fn send_with_listener_and_flush(
        &mut self,
        packet: BoxPacket,
        listener: Option<PacketSendAction>,
        flush: bool,
    ) {
        if self.is_connected() {
            self.flush_queue();
            self.send_packet(packet, listener, flush);
        } else if let Ok(mut q) = self.pending_actions.lock() {
            q.push_back(Box::new(move |conn: &mut Connection| {
                conn.send_packet(packet, listener, flush);
            }));
        }
    }

    // JAVA: net.minecraft.network.Connection#runOnceConnected
    pub fn run_once_connected(&mut self, action: PendingAction) {
        if self.is_connected() {
            self.flush_queue();
            action(self);
        } else if let Ok(mut q) = self.pending_actions.lock() {
            q.push_back(action);
        }
    }

    // JAVA: net.minecraft.network.Connection#sendPacket
    fn send_packet(&mut self, packet: BoxPacket, listener: Option<PacketSendAction>, flush: bool) {
        self.sent_packets = self.sent_packets.wrapping_add(1);
        let in_event_loop = self
            .channel
            .as_ref()
            .map(|c| c.event_loop_in_event_loop())
            .unwrap_or(false);
        if in_event_loop {
            self.do_send_packet(packet, listener, flush);
        } else if let Ok(mut q) = self.pending_actions.lock() {
            q.push_back(Box::new(move |conn: &mut Connection| {
                conn.do_send_packet(packet, listener, flush);
            }));
        }
    }

    // JAVA: net.minecraft.network.Connection#doSendPacket
    fn do_send_packet(
        &mut self,
        packet: BoxPacket,
        listener: Option<PacketSendAction>,
        flush: bool,
    ) {
        let (cb_listener, disconnect_after) = match listener {
            Some(PacketSendAction::Callback(f)) => (Some(f), None),
            Some(PacketSendAction::Disconnect(d)) => (None, Some(d)),
            None => (None, None),
        };
        if let Some(ch) = self.channel.as_mut() {
            if flush {
                ch.write_and_flush(packet, cb_listener);
            } else {
                ch.write(packet, cb_listener);
            }
        }
        if let Some(details) = disconnect_after {
            self.disconnect_with_details(details);
        }
    }

    // JAVA: net.minecraft.network.Connection#flushChannel
    pub fn flush_channel(&mut self) {
        if self.is_connected() {
            self.flush();
        } else if let Ok(mut q) = self.pending_actions.lock() {
            q.push_back(Box::new(|conn: &mut Connection| conn.flush()));
        }
    }

    // JAVA: net.minecraft.network.Connection#flush
    fn flush(&mut self) {
        if let Some(ch) = self.channel.as_mut() {
            ch.flush();
        }
    }

    // JAVA: net.minecraft.network.Connection#flushQueue
    fn flush_queue(&mut self) {
        let connected = self
            .channel
            .as_ref()
            .map(|c| c.is_open())
            .unwrap_or(false);
        if !connected {
            return;
        }
        loop {
            let action = match self.pending_actions.lock() {
                Ok(mut q) => q.pop_front(),
                Err(_) => return,
            };
            match action {
                Some(a) => a(self),
                None => break,
            }
        }
    }

    // JAVA: net.minecraft.network.Connection#tick
    pub fn tick(&mut self) {
        self.flush_queue();

        if let Some(listener) = self.packet_listener.as_mut() {
            if listener.is_tickable() {
                listener.tick();
            }
        }

        if !self.is_connected() && !self.disconnection_handled {
            self.handle_disconnection();
        }

        if let Some(ch) = self.channel.as_mut() {
            ch.flush();
        }

        let old_tick = self.tick_count;
        self.tick_count = self.tick_count.wrapping_add(1);
        if old_tick % 20 == 0 {
            self.tick_second();
        }

        if let Some(monitor) = self.bandwidth_debug_monitor.as_mut() {
            monitor.tick();
        }
    }

    // JAVA: net.minecraft.network.Connection#tickSecond
    fn tick_second(&mut self) {
        self.average_sent_packets = mth_lerp_f32(
            AVERAGE_PACKETS_SMOOTHING,
            self.sent_packets as f32,
            self.average_sent_packets,
        );
        self.average_received_packets = mth_lerp_f32(
            AVERAGE_PACKETS_SMOOTHING,
            self.received_packets as f32,
            self.average_received_packets,
        );
        self.sent_packets = 0;
        self.received_packets = 0;
    }

    // JAVA: net.minecraft.network.Connection#getRemoteAddress
    pub fn get_remote_address(&self) -> Option<&SocketAddress> {
        self.address.as_ref()
    }

    // JAVA: net.minecraft.network.Connection#getLoggableAddress
    pub fn get_loggable_address(&self, log_ips: bool) -> String {
        match &self.address {
            None => "local".to_string(),
            Some(addr) => {
                if log_ips {
                    addr.to_string()
                } else {
                    "IP hidden".to_string()
                }
            }
        }
    }

    // JAVA: net.minecraft.network.Connection#disconnect
    pub fn disconnect(&mut self, reason: Component) {
        self.disconnect_with_details(DisconnectionDetails::new(reason));
    }

    // JAVA: net.minecraft.network.Connection#disconnect
    pub fn disconnect_with_details(&mut self, details: DisconnectionDetails) {
        if self.channel.is_none() {
            self.delayed_disconnect = Some(details.clone());
        }

        if self.is_connected() {
            if let Some(ch) = self.channel.as_mut() {
                ch.close_await_uninterruptibly();
            }
            self.disconnection_details = Some(details);
        }
    }

    // JAVA: net.minecraft.network.Connection#isMemoryConnection
    pub fn is_memory_connection(&self) -> bool {
        self.channel
            .as_ref()
            .map(|c| c.is_local_channel() || c.is_local_server_channel())
            .unwrap_or(false)
    }

    // JAVA: net.minecraft.network.Connection#getReceiving
    pub fn get_receiving(&self) -> PacketFlow {
        self.receiving
    }

    // JAVA: net.minecraft.network.Connection#getSending
    pub fn get_sending(&self) -> PacketFlow {
        self.receiving.get_opposite()
    }

    // JAVA: net.minecraft.network.Connection#connectToServer
    pub fn connect_to_server(addr: SocketAddress, channel: Box<dyn Channel>) -> Self {
        let mut connection = Connection::new(PacketFlow::Clientbound);
        connection.address = Some(addr);
        connection.channel = Some(channel);
        connection
    }

    // JAVA: net.minecraft.network.Connection#connect
    pub fn connect(_addr: &SocketAddress, _channel: &mut dyn Channel) {
        // Netty bootstrap; no-op in self-contained port.
    }

    // JAVA: net.minecraft.network.Connection#outboundHandlerName
    fn outbound_handler_name(configuring: bool) -> &'static str {
        if configuring {
            "encoder"
        } else {
            "outbound_config"
        }
    }

    // JAVA: net.minecraft.network.Connection#inboundHandlerName
    fn inbound_handler_name(configuring: bool) -> &'static str {
        if configuring {
            "decoder"
        } else {
            "inbound_config"
        }
    }

    // JAVA: net.minecraft.network.Connection#configurePacketHandler
    pub fn configure_packet_handler(&mut self) {
        // Netty pipeline setup; no-op in self-contained port.
    }

    // JAVA: net.minecraft.network.Connection#configureSerialization
    pub fn configure_serialization(_flow: PacketFlow, _memory: bool) {
        // Netty pipeline setup; no-op in self-contained port.
    }

    // JAVA: net.minecraft.network.Connection#createFrameEncoder
    fn create_frame_encoder(memory: bool) -> &'static str {
        if memory {
            "LocalFrameEncoder"
        } else {
            "Varint21LengthFieldPrepender"
        }
    }

    // JAVA: net.minecraft.network.Connection#createFrameDecoder
    fn create_frame_decoder(memory: bool) -> &'static str {
        if memory {
            "LocalFrameDecoder"
        } else {
            "Varint21FrameDecoder"
        }
    }

    // JAVA: net.minecraft.network.Connection#configureInMemoryPipeline
    pub fn configure_in_memory_pipeline(flow: PacketFlow) {
        Self::configure_serialization(flow, true);
    }

    // JAVA: net.minecraft.network.Connection#connectToLocalServer
    pub fn connect_to_local_server(channel: Box<dyn Channel>) -> Self {
        let mut connection = Connection::new(PacketFlow::Clientbound);
        connection.channel = Some(channel);
        connection
    }

    // JAVA: net.minecraft.network.Connection#setEncryptionKey
    pub fn set_encryption_key<D, E>(&mut self, _decrypting: D, _encrypting: E) {
        self.encrypted = true;
    }

    // JAVA: net.minecraft.network.Connection#isEncrypted
    pub fn is_encrypted(&self) -> bool {
        self.encrypted
    }

    // JAVA: net.minecraft.network.Connection#isConnected
    pub fn is_connected(&self) -> bool {
        self.channel.as_ref().map(|c| c.is_open()).unwrap_or(false)
    }

    // JAVA: net.minecraft.network.Connection#isConnecting
    pub fn is_connecting(&self) -> bool {
        self.channel.is_none()
    }

    // JAVA: net.minecraft.network.Connection#getPacketListener
    pub fn get_packet_listener(&self) -> Option<&dyn PacketListener> {
        self.packet_listener.as_deref()
    }

    // JAVA: net.minecraft.network.Connection#getDisconnectionDetails
    pub fn get_disconnection_details(&self) -> Option<&DisconnectionDetails> {
        self.disconnection_details.as_ref()
    }

    // JAVA: net.minecraft.network.Connection#setReadOnly
    pub fn set_read_only(&mut self) {
        if let Some(ch) = self.channel.as_mut() {
            ch.config_set_auto_read(false);
        }
    }

    // JAVA: net.minecraft.network.Connection#setupCompression
    pub fn setup_compression(&mut self, threshold: i32, validate_decompressed: bool) {
        if threshold >= 0 {
            self.compression_threshold = Some(threshold);
            self.compression_validate_decompressed = validate_decompressed;
        } else {
            self.compression_threshold = None;
            self.compression_validate_decompressed = false;
        }
    }

    // JAVA: net.minecraft.network.Connection#handleDisconnection
    pub fn handle_disconnection(&mut self) {
        let channel_open = self.channel.as_ref().map(|c| c.is_open()).unwrap_or(false);
        if self.channel.is_some() && !channel_open {
            if self.disconnection_handled {
                return;
            }
            self.disconnection_handled = true;
            let details = self.disconnection_details.clone().unwrap_or_else(|| {
                DisconnectionDetails::new(Component::translatable(
                    "multiplayer.disconnect.generic",
                ))
            });

            if let Some(listener) = self.packet_listener.as_mut() {
                listener.on_disconnect(&details);
            } else if let Some(listener) = self.disconnect_listener.as_mut() {
                listener.on_disconnect(&details);
            }
        }
    }

    // JAVA: net.minecraft.network.Connection#getAverageReceivedPackets
    pub fn get_average_received_packets(&self) -> f32 {
        self.average_received_packets
    }

    // JAVA: net.minecraft.network.Connection#getAverageSentPackets
    pub fn get_average_sent_packets(&self) -> f32 {
        self.average_sent_packets
    }

    // JAVA: net.minecraft.network.Connection#setBandwidthLogger
    pub fn set_bandwidth_logger(&mut self, logger: LocalSampleLogger) {
        self.bandwidth_debug_monitor = Some(BandwidthDebugMonitor::new(logger));
    }
}

// JAVA: net.minecraft.network.Connection#genericsFtw
fn generics_ftw(
    packet: &dyn Packet,
    listener: &mut dyn PacketListener,
) -> Result<(), PacketHandleError> {
    packet.handle_packet(listener)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockChannel {
        state: Arc<Mutex<MockState>>,
    }

    struct MockState {
        open: bool,
        local: bool,
        local_server: bool,
        addr: Option<SocketAddress>,
        auto_read: bool,
        writes: usize,
        flushed: usize,
        closed: usize,
    }

    impl MockChannel {
        fn new(open: bool) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    open,
                    local: false,
                    local_server: false,
                    addr: Some(SocketAddress::Inet {
                        host: "127.0.0.1".to_string(),
                        port: 25565,
                    }),
                    auto_read: true,
                    writes: 0,
                    flushed: 0,
                    closed: 0,
                })),
            }
        }
    }

    impl Channel for MockChannel {
        fn is_open(&self) -> bool {
            self.state.lock().unwrap().open
        }
        fn remote_address(&self) -> Option<SocketAddress> {
            self.state.lock().unwrap().addr.clone()
        }
        fn close(&mut self) {
            let mut st = self.state.lock().unwrap();
            st.open = false;
            st.closed = st.closed.wrapping_add(1);
        }
        fn close_await_uninterruptibly(&mut self) {
            let mut st = self.state.lock().unwrap();
            st.open = false;
            st.closed = st.closed.wrapping_add(1);
        }
        fn write(&mut self, _packet: BoxPacket, _listener: Option<Box<dyn FnOnce(bool) + Send>>) {
            let mut st = self.state.lock().unwrap();
            st.writes = st.writes.wrapping_add(1);
        }
        fn write_and_flush(
            &mut self,
            _packet: BoxPacket,
            _listener: Option<Box<dyn FnOnce(bool) + Send>>,
        ) {
            let mut st = self.state.lock().unwrap();
            st.writes = st.writes.wrapping_add(1);
            st.flushed = st.flushed.wrapping_add(1);
        }
        fn flush(&mut self) {
            let mut st = self.state.lock().unwrap();
            st.flushed = st.flushed.wrapping_add(1);
        }
        fn event_loop_in_event_loop(&self) -> bool {
            true
        }
        fn event_loop_execute(&mut self, task: Box<dyn FnOnce() + Send + 'static>) {
            task();
        }
        fn config_set_auto_read(&mut self, auto_read: bool) {
            self.state.lock().unwrap().auto_read = auto_read;
        }
        fn is_local_channel(&self) -> bool {
            self.state.lock().unwrap().local
        }
        fn is_local_server_channel(&self) -> bool {
            self.state.lock().unwrap().local_server
        }
    }

    struct TickListener {
        flow: PacketFlow,
        protocol: ConnectionProtocol,
        ticked: Arc<Mutex<i32>>,
        disconnects: Arc<Mutex<i32>>,
    }

    impl PacketListener for TickListener {
        fn flow(&self) -> PacketFlow {
            self.flow
        }
        fn protocol(&self) -> ConnectionProtocol {
            self.protocol
        }
        fn tick(&mut self) {
            let mut t = self.ticked.lock().unwrap();
            *t = t.wrapping_add(1);
        }
        fn is_tickable(&self) -> bool {
            true
        }
        fn on_disconnect(&mut self, _details: &DisconnectionDetails) {
            let mut d = self.disconnects.lock().unwrap();
            *d = d.wrapping_add(1);
        }
    }

    #[test]
    fn tick_second_averages_with_java_float_order_and_bits() {
        let mut conn = Connection::new(PacketFlow::Clientbound);
        conn.sent_packets = 20;
        conn.received_packets = 40;
        conn.average_sent_packets = 0.0_f32;
        conn.average_received_packets = 0.0_f32;

        conn.tick_second();

        let expected_sent = mth_lerp_f32(AVERAGE_PACKETS_SMOOTHING, 20.0_f32, 0.0_f32);
        let expected_recv = mth_lerp_f32(AVERAGE_PACKETS_SMOOTHING, 40.0_f32, 0.0_f32);
        assert_eq!(conn.get_average_sent_packets().to_bits(), expected_sent.to_bits());
        assert_eq!(
            conn.get_average_received_packets().to_bits(),
            expected_recv.to_bits()
        );
        assert_eq!(conn.sent_packets, 0);
        assert_eq!(conn.received_packets, 0);

        // Second pass: smoothing toward previous average
        conn.sent_packets = 0;
        conn.average_sent_packets = 5.0_f32;
        conn.tick_second();
        let expected2 = mth_lerp_f32(AVERAGE_PACKETS_SMOOTHING, 0.0_f32, 5.0_f32);
        assert_eq!(conn.get_average_sent_packets().to_bits(), expected2.to_bits());
    }

    #[test]
    fn tick_wraps_count_and_handles_negative_modulo() {
        let mut conn = Connection::new(PacketFlow::Clientbound);
        conn.disconnection_handled = true;

        conn.tick_count = i32::MAX;
        conn.sent_packets = 10;
        conn.average_sent_packets = 1.0_f32;
        conn.tick();
        assert_eq!(conn.tick_count, i32::MIN);
        assert_eq!(conn.sent_packets, 10); // i32::MAX % 20 == 7, no tick_second

        conn.tick_count = -20_i32;
        conn.sent_packets = 6;
        conn.average_sent_packets = 0.0_f32;
        conn.tick();
        assert_eq!(conn.tick_count, -19_i32);
        let expected = mth_lerp_f32(AVERAGE_PACKETS_SMOOTHING, 6.0_f32, 0.0_f32);
        assert_eq!(conn.get_average_sent_packets().to_bits(), expected.to_bits());
    }

    #[test]
    fn packet_counters_wrap_and_negative_values_affect_float_average() {
        let mut conn = Connection::new(PacketFlow::Clientbound);
        conn.disconnection_handled = true;
        conn.received_packets = i32::MAX;
        conn.sent_packets = i32::MAX;

        conn.received_packets = conn.received_packets.wrapping_add(1);
        conn.sent_packets = conn.sent_packets.wrapping_add(1);
        assert_eq!(conn.received_packets, i32::MIN);
        assert_eq!(conn.sent_packets, i32::MIN);

        conn.average_received_packets = 0.0_f32;
        conn.average_sent_packets = 0.0_f32;
        conn.tick_second();

        let expected = mth_lerp_f32(AVERAGE_PACKETS_SMOOTHING, i32::MIN as f32, 0.0_f32);
        assert_eq!(conn.get_average_received_packets().to_bits(), expected.to_bits());
        assert_eq!(conn.get_average_sent_packets().to_bits(), expected.to_bits());
    }

    #[test]
    fn compression_threshold_negative_disables_and_zero_enables() {
        let mut conn = Connection::new(PacketFlow::Clientbound);
        assert_eq!(conn.compression_threshold, None);

        conn.setup_compression(256, true);
        assert_eq!(conn.compression_threshold, Some(256));
        assert!(conn.compression_validate_decompressed);

        conn.setup_compression(-1, true);
        assert_eq!(conn.compression_threshold, None);
        assert!(!conn.compression_validate_decompressed);

        conn.setup_compression(0, false);
        assert_eq!(conn.compression_threshold, Some(0));
        assert!(!conn.compression_validate_decompressed);
    }

    #[test]
    fn loggable_address_and_disconnect_delay_without_channel() {
        let mut conn = Connection::new(PacketFlow::Clientbound);
        assert_eq!(conn.get_loggable_address(true), "local");
        assert_eq!(conn.get_loggable_address(false), "local");
        assert!(conn.is_connecting());
        assert!(!conn.is_connected());

        conn.disconnect(Component::translatable("disconnect.test"));
        assert!(conn.delayed_disconnect.is_some());
        assert_eq!(
            conn.delayed_disconnect.as_ref().unwrap().reason.key,
            "disconnect.test"
        );
        assert!(conn.disconnection_details.is_none());

        conn.address = Some(SocketAddress::Inet {
            host: "192.168.1.1".to_string(),
            port: 25565,
        });
        assert_eq!(conn.get_loggable_address(true), "192.168.1.1:25565");
        assert_eq!(conn.get_loggable_address(false), "IP hidden");
    }

    #[test]
    fn channel_active_applies_delayed_disconnect_and_tick_updates_listener() {
        let mut conn = Connection::new(PacketFlow::Clientbound);
        conn.disconnect(Component::translatable("disconnect.later"));

        let ch = MockChannel::new(true);
        let state = ch.state.clone();
        conn.channel_active(Box::new(ch));

        assert!(conn.disconnection_details.is_some());
        assert_eq!(
            conn.disconnection_details.as_ref().unwrap().reason.key,
            "disconnect.later"
        );
        assert!(conn.delayed_disconnect.is_none());
        {
            let st = state.lock().unwrap();
            assert!(!st.open);
            assert_eq!(st.closed, 1);
        }

        let mut conn2 = Connection::new(PacketFlow::Clientbound);
        let ch2 = MockChannel::new(true);
        conn2.channel_active(Box::new(ch2));
        let ticked = Arc::new(Mutex::new(0_i32));
        let disconnects = Arc::new(Mutex::new(0_i32));
        conn2.packet_listener = Some(Box::new(TickListener {
            flow: PacketFlow::Clientbound,
            protocol: ConnectionProtocol::Play,
            ticked: ticked.clone(),
            disconnects: disconnects.clone(),
        }));
        conn2.tick_count = 19; // next tick increments to 20? no: old=19, becomes 20, 19%20!=0
        conn2.tick();
        assert_eq!(*ticked.lock().unwrap(), 1);
        assert_eq!(conn2.tick_count, 20);

        conn2.sent_packets = 4;
        conn2.average_sent_packets = 0.0_f32;
        conn2.tick(); // old=20, becomes 21, 20%20==0 -> tick_second
        assert_eq!(*ticked.lock().unwrap(), 2);
        let expected = mth_lerp_f32(AVERAGE_PACKETS_SMOOTHING, 4.0_f32, 0.0_f32);
        assert_eq!(conn2.get_average_sent_packets().to_bits(), expected.to_bits());
    }
}
