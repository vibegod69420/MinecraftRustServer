// === FILE: server_tick_rate_manager.rs ===
//! Faithful port of `net.minecraft.server.ServerTickRateManager` (Minecraft Java 1.21.11).

pub const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;
pub const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000;
pub const MILLISECONDS_PER_SECOND: i64 = 1_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundTickingStatePacket {
    pub tick_rate: f32,
    pub is_frozen: bool,
}

impl ClientboundTickingStatePacket {
    #[must_use]
    pub fn from_manager(manager: &TickRateManager) -> Self {
        Self {
            tick_rate: manager.tickrate(),
            is_frozen: manager.is_frozen(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundTickingStepPacket {
    pub tick_steps: i32,
}

impl ClientboundTickingStepPacket {
    #[must_use]
    pub fn from_manager(manager: &TickRateManager) -> Self {
        Self {
            tick_steps: manager.frozen_ticks_to_run,
        }
    }
}

pub trait MinecraftServer {
    fn broadcast_ticking_state(&mut self, packet: ClientboundTickingStatePacket);
    fn broadcast_ticking_step(&mut self, packet: ClientboundTickingStepPacket);
    fn on_tick_rate_changed(&mut self);
    fn send_sprint_report(&mut self, ticks_per_second: i32, ms_per_tick: &str);
    fn nano_time(&self) -> i64;
}

pub trait ServerPlayerConnection {
    fn send_ticking_state(&mut self, packet: ClientboundTickingStatePacket);
    fn send_ticking_step(&mut self, packet: ClientboundTickingStepPacket);
}

/// Minimal base `TickRateManager` state/behavior used by [`ServerTickRateManager`].
#[derive(Clone, Debug)]
pub struct TickRateManager {
    tickrate: f32,
    nanoseconds_per_tick: i64,
    pub frozen_ticks_to_run: i32,
    pub run_game_elements: bool,
    frozen: bool,
}

impl TickRateManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tickrate: 20.0f32,
            nanoseconds_per_tick: NANOSECONDS_PER_SECOND / 20i64,
            frozen_ticks_to_run: 0i32,
            run_game_elements: true,
            frozen: false,
        }
    }

    pub fn set_tick_rate(&mut self, tick_rate: f32) {
        self.tickrate = if tick_rate > 1.0f32 || tick_rate.is_nan() {
            tick_rate
        } else {
            1.0f32
        };
        self.nanoseconds_per_tick =
            ((NANOSECONDS_PER_SECOND as f64) / (f64::from(self.tickrate))) as i64;
    }

    #[must_use]
    pub fn tickrate(&self) -> f32 {
        self.tickrate
    }

    #[must_use]
    pub fn milliseconds_per_tick(&self) -> f32 {
        (self.nanoseconds_per_tick as f32) / (NANOSECONDS_PER_MILLISECOND as f32)
    }

    #[must_use]
    pub fn nanoseconds_per_tick(&self) -> i64 {
        self.nanoseconds_per_tick
    }

    #[must_use]
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub fn set_frozen(&mut self, frozen: bool) {
        self.frozen = frozen;
    }
}

impl Default for TickRateManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ServerTickRateManager<S> {
    base: TickRateManager,
    remaining_sprint_ticks: i64,
    sprint_tick_start_time: i64,
    sprint_time_spend: i64,
    scheduled_current_sprint_ticks: i64,
    previous_is_frozen: bool,
    server: S,
}

impl<S> ServerTickRateManager<S> {
    // JAVA: net.minecraft.server.ServerTickRateManager#<init>
    pub fn new(server: S) -> Self {
        Self {
            base: TickRateManager::new(),
            remaining_sprint_ticks: 0i64,
            sprint_tick_start_time: 0i64,
            sprint_time_spend: 0i64,
            scheduled_current_sprint_ticks: 0i64,
            previous_is_frozen: false,
            server,
        }
    }

    #[must_use]
    pub fn base(&self) -> &TickRateManager {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut TickRateManager {
        &mut self.base
    }

    #[must_use]
    pub fn server(&self) -> &S {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut S {
        &mut self.server
    }

    #[must_use]
    pub fn remaining_sprint_ticks(&self) -> i64 {
        self.remaining_sprint_ticks
    }

    #[must_use]
    pub fn scheduled_current_sprint_ticks(&self) -> i64 {
        self.scheduled_current_sprint_ticks
    }

    #[must_use]
    pub fn sprint_time_spend(&self) -> i64 {
        self.sprint_time_spend
    }

    #[must_use]
    pub fn previous_is_frozen(&self) -> bool {
        self.previous_is_frozen
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#isSprinting
    pub fn is_sprinting(&self) -> bool {
        self.scheduled_current_sprint_ticks > 0i64
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#updateJoiningPlayer
    pub fn update_joining_player<P: ServerPlayerConnection>(&self, player: &mut P) {
        let state_packet = ClientboundTickingStatePacket::from_manager(&self.base);
        let step_packet = ClientboundTickingStepPacket::from_manager(&self.base);
        player.send_ticking_state(state_packet);
        player.send_ticking_step(step_packet);
    }
}

impl<S: MinecraftServer> ServerTickRateManager<S> {
    // JAVA: net.minecraft.server.ServerTickRateManager#setFrozen
    pub fn set_frozen(&mut self, frozen: bool) {
        self.base.set_frozen(frozen);
        self.update_state_to_clients();
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#updateStateToClients
    fn update_state_to_clients(&mut self) {
        let packet = ClientboundTickingStatePacket::from_manager(&self.base);
        self.server.broadcast_ticking_state(packet);
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#updateStepTicks
    fn update_step_ticks(&mut self) {
        let packet = ClientboundTickingStepPacket::from_manager(&self.base);
        self.server.broadcast_ticking_step(packet);
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#stepGameIfPaused
    pub fn step_game_if_paused(&mut self, ticks: i32) -> bool {
        if !self.base.is_frozen() {
            return false;
        }
        self.base.frozen_ticks_to_run = ticks;
        self.update_step_ticks();
        true
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#stopStepping
    pub fn stop_stepping(&mut self) -> bool {
        if self.base.frozen_ticks_to_run > 0 {
            self.base.frozen_ticks_to_run = 0;
            self.update_step_ticks();
            return true;
        }
        false
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#stopSprinting
    pub fn stop_sprinting(&mut self) -> bool {
        if self.remaining_sprint_ticks > 0i64 {
            self.finish_tick_sprint();
            return true;
        }
        false
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#requestGameToSprint
    pub fn request_game_to_sprint(&mut self, ticks: i32) -> bool {
        let already_sprinting = self.remaining_sprint_ticks > 0i64;
        self.sprint_time_spend = 0i64;
        self.scheduled_current_sprint_ticks = i64::from(ticks);
        self.remaining_sprint_ticks = i64::from(ticks);
        self.previous_is_frozen = self.base.is_frozen();
        self.set_frozen(false);
        already_sprinting
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#finishTickSprint
    fn finish_tick_sprint(&mut self) {
        let delta = self
            .scheduled_current_sprint_ticks
            .wrapping_sub(self.remaining_sprint_ticks);
        let ms_spent = 1.0f64.max(self.sprint_time_spend as f64)
            / (NANOSECONDS_PER_MILLISECOND as f64);
        let ticks_per_second =
            ((MILLISECONDS_PER_SECOND.wrapping_mul(delta)) as f64 / ms_spent) as i32;
        let ms_per_tick = if delta == 0i64 {
            f64::from(self.base.milliseconds_per_tick())
        } else {
            ms_spent / (delta as f64)
        };
        let ms_per_tick_str = format!("{ms_per_tick:.2}");
        self.scheduled_current_sprint_ticks = 0i64;
        self.sprint_time_spend = 0i64;
        self.server
            .send_sprint_report(ticks_per_second, &ms_per_tick_str);
        self.remaining_sprint_ticks = 0i64;
        self.set_frozen(self.previous_is_frozen);
        self.server.on_tick_rate_changed();
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#checkShouldSprintThisTick
    pub fn check_should_sprint_this_tick(&mut self) -> bool {
        if !self.base.run_game_elements {
            return false;
        }
        if self.remaining_sprint_ticks > 0i64 {
            self.sprint_tick_start_time = self.server.nano_time();
            self.remaining_sprint_ticks = self.remaining_sprint_ticks.wrapping_sub(1i64);
            return true;
        }
        self.finish_tick_sprint();
        false
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#endTickWork
    pub fn end_tick_work(&mut self) {
        let now = self.server.nano_time();
        let elapsed = now.wrapping_sub(self.sprint_tick_start_time);
        self.sprint_time_spend = self.sprint_time_spend.wrapping_add(elapsed);
    }

    // JAVA: net.minecraft.server.ServerTickRateManager#setTickRate
    pub fn set_tick_rate(&mut self, tick_rate: f32) {
        self.base.set_tick_rate(tick_rate);
        self.server.on_tick_rate_changed();
        self.update_state_to_clients();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockServer {
        state_broadcasts: Vec<ClientboundTickingStatePacket>,
        step_broadcasts: Vec<ClientboundTickingStepPacket>,
        tick_rate_changed: usize,
        sprint_reports: Vec<(i32, String)>,
        nano_time: i64,
    }

    impl MinecraftServer for MockServer {
        fn broadcast_ticking_state(&mut self, packet: ClientboundTickingStatePacket) {
            self.state_broadcasts.push(packet);
        }

        fn broadcast_ticking_step(&mut self, packet: ClientboundTickingStepPacket) {
            self.step_broadcasts.push(packet);
        }

        fn on_tick_rate_changed(&mut self) {
            self.tick_rate_changed = self.tick_rate_changed.wrapping_add(1);
        }

        fn send_sprint_report(&mut self, ticks_per_second: i32, ms_per_tick: &str) {
            self.sprint_reports
                .push((ticks_per_second, ms_per_tick.to_string()));
        }

        fn nano_time(&self) -> i64 {
            self.nano_time
        }
    }

    struct MockPlayer {
        states: Vec<ClientboundTickingStatePacket>,
        steps: Vec<ClientboundTickingStepPacket>,
    }

    impl MockPlayer {
        fn new() -> Self {
            Self {
                states: Vec::new(),
                steps: Vec::new(),
            }
        }
    }

    impl ServerPlayerConnection for MockPlayer {
        fn send_ticking_state(&mut self, packet: ClientboundTickingStatePacket) {
            self.states.push(packet);
        }

        fn send_ticking_step(&mut self, packet: ClientboundTickingStepPacket) {
            self.steps.push(packet);
        }
    }

    #[test]
    fn set_frozen_and_stepping() {
        let mut mgr = ServerTickRateManager::new(MockServer::default());
        assert!(!mgr.base().is_frozen());
        mgr.set_frozen(true);
        assert!(mgr.base().is_frozen());
        assert_eq!(
            mgr.server().state_broadcasts.last().copied(),
            Some(ClientboundTickingStatePacket {
                tick_rate: 20.0,
                is_frozen: true,
            })
        );

        let mut unfrozen = ServerTickRateManager::new(MockServer::default());
        assert!(!unfrozen.step_game_if_paused(5));
        assert!(mgr.step_game_if_paused(3));
        assert_eq!(mgr.base().frozen_ticks_to_run, 3);
        assert_eq!(
            mgr.server().step_broadcasts.last().copied(),
            Some(ClientboundTickingStepPacket { tick_steps: 3 })
        );
        assert!(mgr.stop_stepping());
        assert_eq!(mgr.base().frozen_ticks_to_run, 0);
        assert!(!mgr.stop_stepping());
    }

    #[test]
    fn sprint_lifecycle_report_and_restore() {
        let mut mgr = ServerTickRateManager::new(MockServer::default());
        mgr.set_frozen(true);
        mgr.server_mut().state_broadcasts.clear();
        mgr.server_mut().tick_rate_changed = 0;

        assert!(!mgr.request_game_to_sprint(3));
        assert!(mgr.is_sprinting());
        assert!(!mgr.base().is_frozen());
        assert!(mgr.previous_is_frozen());

        for _ in 0..3 {
            mgr.server_mut().nano_time = mgr.server().nano_time.wrapping_add(5_000_000);
            assert!(mgr.check_should_sprint_this_tick());
            mgr.server_mut().nano_time = mgr.server().nano_time.wrapping_add(5_000_000);
            mgr.end_tick_work();
        }
        assert_eq!(mgr.remaining_sprint_ticks(), 0);
        assert!(mgr.is_sprinting());

        assert!(!mgr.check_should_sprint_this_tick());
        assert!(!mgr.is_sprinting());
        assert!(mgr.base().is_frozen());
        assert_eq!(mgr.server().sprint_reports.len(), 1);
        assert_eq!(mgr.server().sprint_reports[0].0, 200);
        assert_eq!(mgr.server().sprint_reports[0].1, "5.00");
        assert!(mgr.server().tick_rate_changed >= 1);
    }

    #[test]
    fn set_tick_rate_clamp_and_float_bits() {
        let mut mgr = ServerTickRateManager::new(MockServer::default());
        mgr.set_tick_rate(0.0);
        assert_eq!(mgr.base().tickrate(), 1.0);
        assert_eq!(mgr.base().nanoseconds_per_tick(), NANOSECONDS_PER_SECOND);
        mgr.set_tick_rate(-5.0);
        assert_eq!(mgr.base().tickrate(), 1.0);

        mgr.set_tick_rate(40.0);
        assert_eq!(mgr.base().tickrate().to_bits(), 40.0f32.to_bits());
        assert_eq!(mgr.base().nanoseconds_per_tick(), 25_000_000);
        assert_eq!(
            mgr.base().milliseconds_per_tick().to_bits(),
            25.0f32.to_bits()
        );
        assert!(mgr.server().tick_rate_changed >= 1);
        assert!(!mgr.server().state_broadcasts.is_empty());
    }

    #[test]
    fn wrapping_overflow_and_negative_inputs() {
        let mut mgr = ServerTickRateManager::new(MockServer::default());
        assert!(!mgr.request_game_to_sprint(-5));
        assert!(!mgr.is_sprinting());
        assert_eq!(mgr.remaining_sprint_ticks(), -5i64);
        assert_eq!(mgr.scheduled_current_sprint_ticks(), -5i64);

        let mut mgr2 = ServerTickRateManager::new(MockServer::default());
        mgr2.request_game_to_sprint(2);
        mgr2.server_mut().nano_time = i64::MAX;
        assert!(mgr2.check_should_sprint_this_tick());
        mgr2.server_mut().nano_time = i64::MIN;
        mgr2.end_tick_work();
        let expected_elapsed = i64::MIN.wrapping_sub(i64::MAX);
        assert_eq!(mgr2.sprint_time_spend(), expected_elapsed);
    }

    #[test]
    fn update_joining_player_stop_sprinting_and_zero_delta() {
        let mut mgr = ServerTickRateManager::new(MockServer::default());
        mgr.base_mut().frozen_ticks_to_run = 7;
        mgr.set_frozen(true);

        let mut player = MockPlayer::new();
        mgr.update_joining_player(&mut player);
        assert_eq!(
            player.states,
            vec![ClientboundTickingStatePacket {
                tick_rate: 20.0,
                is_frozen: true,
            }]
        );
        assert_eq!(
            player.steps,
            vec![ClientboundTickingStepPacket { tick_steps: 7 }]
        );

        let mut mgr2 = ServerTickRateManager::new(MockServer::default());
        assert!(!mgr2.stop_sprinting());
        mgr2.request_game_to_sprint(10);
        assert!(mgr2.stop_sprinting());
        assert!(!mgr2.is_sprinting());
        assert_eq!(mgr2.remaining_sprint_ticks(), 0);
        assert_eq!(mgr2.server().sprint_reports.len(), 1);

        let mut mgr3 = ServerTickRateManager::new(MockServer::default());
        assert!(!mgr3.check_should_sprint_this_tick());
        assert_eq!(mgr3.server().sprint_reports[0], (0, "50.00".to_string()));
    }
}
