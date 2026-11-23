use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};

/// Minimal self-contained stand-in for `net.minecraft.SharedConstants`.
pub mod shared_constants {
    use super::{AtomicBool, Ordering};

    static IS_RUNNING_IN_IDE: AtomicBool = AtomicBool::new(false);

    pub fn is_running_in_ide() -> bool {
        IS_RUNNING_IN_IDE.load(Ordering::Relaxed)
    }

    pub fn set_is_running_in_ide(value: bool) {
        IS_RUNNING_IN_IDE.store(value, Ordering::Relaxed);
    }
}

/// Minimal abstraction for `net.minecraft.server.level.ServerPlayer`.
pub trait ServerPlayerLike: Clone {
    type Subscription: Eq + Hash + Clone;
    type Packet;
    type NameAndId: Clone;

    fn debug_subscriptions(&self) -> Vec<Self::Subscription>;
    fn send_packet(&self, packet: &Self::Packet);
    fn name_and_id(&self) -> Self::NameAndId;
}

/// Minimal abstraction for `MinecraftServer#getPlayerList()`.
pub trait PlayerListLike<P: ServerPlayerLike> {
    fn get_players(&self) -> Vec<P>;
    fn is_op(&self, name_and_id: &P::NameAndId) -> bool;
}

/// Minimal abstraction for `net.minecraft.server.MinecraftServer`.
pub trait MinecraftServerLike<P: ServerPlayerLike> {
    type PlayerList: PlayerListLike<P>;

    fn get_player_list(&self) -> &Self::PlayerList;
    fn is_singleplayer_owner(&self, name_and_id: &P::NameAndId) -> bool;
}

/// Faithful port of `net.minecraft.util.debug.ServerDebugSubscribers`.
/// External Minecraft types are abstracted through traits to keep this module self-contained.
pub struct ServerDebugSubscribers<S, P>
where
    P: ServerPlayerLike,
    S: MinecraftServerLike<P>,
{
    server: S,
    enabled_subscriptions: HashMap<P::Subscription, Vec<P>>,
}

impl<S, P> ServerDebugSubscribers<S, P>
where
    P: ServerPlayerLike,
    S: MinecraftServerLike<P>,
{
    // JAVA: net.minecraft.util.debug.ServerDebugSubscribers#<init>
    pub fn new(server: S) -> Self {
        Self {
            server,
            enabled_subscriptions: HashMap::new(),
        }
    }

    // JAVA: net.minecraft.util.debug.ServerDebugSubscribers#getSubscribersFor
    fn get_subscribers_for(&self, subscription: &P::Subscription) -> &[P] {
        self.enabled_subscriptions
            .get(subscription)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    // JAVA: net.minecraft.util.debug.ServerDebugSubscribers#tick
    pub fn tick(&mut self) {
        for players in self.enabled_subscriptions.values_mut() {
            players.clear();
        }

        let players = self.server.get_player_list().get_players();
        for player in &players {
            for sub in player.debug_subscriptions() {
                self.enabled_subscriptions
                    .entry(sub)
                    .or_insert_with(Vec::new)
                    .push(player.clone());
            }
        }

        self.enabled_subscriptions.retain(|_, v| !v.is_empty());
    }

    // JAVA: net.minecraft.util.debug.ServerDebugSubscribers#broadcastToAll
    pub fn broadcast_to_all(&self, subscription: &P::Subscription, packet: &P::Packet) {
        for player in self.get_subscribers_for(subscription) {
            player.send_packet(packet);
        }
    }

    // JAVA: net.minecraft.util.debug.ServerDebugSubscribers#enabledSubscriptions
    pub fn enabled_subscriptions(&self) -> HashSet<P::Subscription> {
        self.enabled_subscriptions.keys().cloned().collect()
    }

    // JAVA: net.minecraft.util.debug.ServerDebugSubscribers#hasAnySubscriberFor
    pub fn has_any_subscriber_for(&self, subscription: &P::Subscription) -> bool {
        !self.get_subscribers_for(subscription).is_empty()
    }

    // JAVA: net.minecraft.util.debug.ServerDebugSubscribers#hasRequiredPermissions
    pub fn has_required_permissions(&self, player: &P) -> bool {
        self.has_required_permissions_with_ide(player, shared_constants::is_running_in_ide())
    }

    /// Testable helper preserving the exact permission logic with an explicit IDE flag.
    pub fn has_required_permissions_with_ide(&self, player: &P, is_running_in_ide: bool) -> bool {
        let name_and_id = player.name_and_id();
        if is_running_in_ide && self.server.is_singleplayer_owner(&name_and_id) {
            true
        } else {
            self.server.get_player_list().is_op(&name_and_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct TestName(String);

    #[derive(Clone, Debug)]
    struct TestPlayer {
        _id: u32,
        subs: Vec<u64>,
        name: TestName,
        sent: Rc<RefCell<Vec<i32>>>,
    }

    impl ServerPlayerLike for TestPlayer {
        type Subscription = u64;
        type Packet = i32;
        type NameAndId = TestName;

        fn debug_subscriptions(&self) -> Vec<Self::Subscription> {
            self.subs.clone()
        }

        fn send_packet(&self, packet: &Self::Packet) {
            self.sent.borrow_mut().push(*packet);
        }

        fn name_and_id(&self) -> Self::NameAndId {
            self.name.clone()
        }
    }

    struct TestPlayerList {
        players: Vec<TestPlayer>,
        ops: HashSet<TestName>,
    }

    impl PlayerListLike<TestPlayer> for TestPlayerList {
        fn get_players(&self) -> Vec<TestPlayer> {
            self.players.clone()
        }

        fn is_op(&self, name_and_id: &TestName) -> bool {
            self.ops.contains(name_and_id)
        }
    }

    struct TestServer {
        list: TestPlayerList,
        owner: Option<TestName>,
    }

    impl MinecraftServerLike<TestPlayer> for TestServer {
        type PlayerList = TestPlayerList;

        fn get_player_list(&self) -> &Self::PlayerList {
            &self.list
        }

        fn is_singleplayer_owner(&self, name_and_id: &TestName) -> bool {
            self.owner.as_ref() == Some(name_and_id)
        }
    }

    #[test]
    fn tick_collects_subscriptions_and_removes_empty() {
        let sent = Rc::new(RefCell::new(Vec::new()));
        let p1 = TestPlayer {
            _id: 1,
            subs: vec![10, 20],
            name: TestName("a".into()),
            sent: sent.clone(),
        };
        let p2 = TestPlayer {
            _id: 2,
            subs: vec![20],
            name: TestName("b".into()),
            sent,
        };
        let server = TestServer {
            list: TestPlayerList {
                players: vec![p1, p2],
                ops: HashSet::new(),
            },
            owner: None,
        };
        let mut dbg = ServerDebugSubscribers::new(server);
        dbg.tick();
        assert!(dbg.has_any_subscriber_for(&10));
        assert!(dbg.has_any_subscriber_for(&20));
        assert!(!dbg.has_any_subscriber_for(&99));
        assert_eq!(dbg.enabled_subscriptions().len(), 2);

        dbg.server.list.players[0].subs = vec![20];
        dbg.server.list.players[1].subs.clear();
        dbg.tick();
        assert!(!dbg.has_any_subscriber_for(&10));
        assert!(dbg.has_any_subscriber_for(&20));
        assert_eq!(dbg.enabled_subscriptions().len(), 1);
    }

    #[test]
    fn broadcast_to_all_sends_only_to_subscribers() {
        let sent1 = Rc::new(RefCell::new(Vec::new()));
        let sent2 = Rc::new(RefCell::new(Vec::new()));
        let p1 = TestPlayer {
            _id: 1,
            subs: vec![7],
            name: TestName("a".into()),
            sent: sent1.clone(),
        };
        let p2 = TestPlayer {
            _id: 2,
            subs: vec![8],
            name: TestName("b".into()),
            sent: sent2.clone(),
        };
        let server = TestServer {
            list: TestPlayerList {
                players: vec![p1, p2],
                ops: HashSet::new(),
            },
            owner: None,
        };
        let mut dbg = ServerDebugSubscribers::new(server);
        dbg.tick();
        dbg.broadcast_to_all(&7, &42);
        dbg.broadcast_to_all(&999, &-1);
        assert_eq!(*sent1.borrow(), vec![42]);
        assert_eq!(*sent2.borrow(), Vec::<i32>::new());
    }

    #[test]
    fn has_required_permissions_respects_ide_owner_and_op() {
        let owner = TestName("owner".into());
        let op = TestName("op".into());
        let other = TestName("other".into());
        let mk = |name: &TestName| TestPlayer {
            _id: 0,
            subs: vec![],
            name: name.clone(),
            sent: Rc::new(RefCell::new(vec![])),
        };
        let server = TestServer {
            list: TestPlayerList {
                players: vec![],
                ops: [op.clone()].into_iter().collect(),
            },
            owner: Some(owner.clone()),
        };
        let dbg = ServerDebugSubscribers::new(server);
        let p_owner = mk(&owner);
        let p_op = mk(&op);
        let p_other = mk(&other);

        assert!(!dbg.has_required_permissions_with_ide(&p_owner, false));
        assert!(dbg.has_required_permissions_with_ide(&p_op, false));
        assert!(!dbg.has_required_permissions_with_ide(&p_other, false));

        assert!(dbg.has_required_permissions_with_ide(&p_owner, true));
        assert!(dbg.has_required_permissions_with_ide(&p_op, true));
        assert!(!dbg.has_required_permissions_with_ide(&p_other, true));
    }

    #[test]
    fn tick_clears_stale_subscribers_when_players_leave() {
        let sent = Rc::new(RefCell::new(Vec::new()));
        let p1 = TestPlayer {
            _id: 1,
            subs: vec![5],
            name: TestName("a".into()),
            sent,
        };
        let server = TestServer {
            list: TestPlayerList {
                players: vec![p1],
                ops: HashSet::new(),
            },
            owner: None,
        };
        let mut dbg = ServerDebugSubscribers::new(server);
        dbg.tick();
        assert!(dbg.has_any_subscriber_for(&5));

        dbg.server.list.players.clear();
        dbg.tick();
        assert!(!dbg.has_any_subscriber_for(&5));
        assert!(dbg.enabled_subscriptions().is_empty());
    }
}
