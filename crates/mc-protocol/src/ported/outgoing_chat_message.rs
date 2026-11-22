#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Component {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChatTypeBound {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerChatMessage {
    pub decorated: Component,
    pub system: bool,
    pub fully_filtered: bool,
}

impl PlayerChatMessage {
    // JAVA: net.minecraft.network.chat.PlayerChatMessage#isSystem
    pub fn is_system(&self) -> bool {
        self.system
    }

    // JAVA: net.minecraft.network.chat.PlayerChatMessage#decoratedContent
    pub fn decorated_content(&self) -> Component {
        self.decorated.clone()
    }

    // JAVA: net.minecraft.network.chat.PlayerChatMessage#filter
    pub fn filter(&self, should_filter: bool) -> Self {
        let mut filtered = self.clone();
        if should_filter {
            filtered.fully_filtered = true;
        }
        filtered
    }

    // JAVA: net.minecraft.network.chat.PlayerChatMessage#isFullyFiltered
    pub fn is_fully_filtered(&self) -> bool {
        self.fully_filtered
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServerGamePacketListenerImpl {
    pub disguised_sent: Vec<(Component, ChatTypeBound)>,
    pub player_sent: Vec<(PlayerChatMessage, ChatTypeBound)>,
}

impl ServerGamePacketListenerImpl {
    // JAVA: net.minecraft.server.network.ServerGamePacketListenerImpl#sendDisguisedChatMessage
    pub fn send_disguised_chat_message(&mut self, content: Component, bound: ChatTypeBound) {
        self.disguised_sent.push((content, bound));
    }

    // JAVA: net.minecraft.server.network.ServerGamePacketListenerImpl#sendPlayerChatMessage
    pub fn send_player_chat_message(&mut self, message: PlayerChatMessage, bound: ChatTypeBound) {
        self.player_sent.push((message, bound));
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServerPlayer {
    pub connection: ServerGamePacketListenerImpl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disguised {
    pub content: Component,
}

impl Disguised {
    // JAVA: net.minecraft.network.chat.OutgoingChatMessage$Disguised#sendToPlayer
    pub fn send_to_player(&self, player: &mut ServerPlayer, _filter: bool, bound: ChatTypeBound) {
        player
            .connection
            .send_disguised_chat_message(self.content.clone(), bound);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub message: PlayerChatMessage,
}

impl Player {
    // JAVA: net.minecraft.network.chat.OutgoingChatMessage$Player#content
    pub fn content(&self) -> Component {
        self.message.decorated_content()
    }

    // JAVA: net.minecraft.network.chat.OutgoingChatMessage$Player#sendToPlayer
    pub fn send_to_player(&self, player: &mut ServerPlayer, filter: bool, bound: ChatTypeBound) {
        let filtered = self.message.filter(filter);
        if !filtered.is_fully_filtered() {
            player
                .connection
                .send_player_chat_message(filtered, bound);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutgoingChatMessage {
    Disguised(Disguised),
    Player(Player),
}

impl OutgoingChatMessage {
    // JAVA: net.minecraft.network.chat.OutgoingChatMessage#content
    pub fn content(&self) -> Component {
        match self {
            Self::Disguised(d) => d.content.clone(),
            Self::Player(p) => p.content(),
        }
    }

    // JAVA: net.minecraft.network.chat.OutgoingChatMessage#sendToPlayer
    pub fn send_to_player(&self, player: &mut ServerPlayer, filter: bool, bound: ChatTypeBound) {
        match self {
            Self::Disguised(d) => d.send_to_player(player, filter, bound),
            Self::Player(p) => p.send_to_player(player, filter, bound),
        }
    }

    // JAVA: net.minecraft.network.chat.OutgoingChatMessage#create
    pub fn create(message: PlayerChatMessage) -> Self {
        if message.is_system() {
            Self::Disguised(Disguised {
                content: message.decorated_content(),
            })
        } else {
            Self::Player(Player { message })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message(text: &str, system: bool, fully_filtered: bool) -> PlayerChatMessage {
        PlayerChatMessage {
            decorated: Component {
                text: text.to_string(),
            },
            system,
            fully_filtered,
        }
    }

    fn make_bound(id: &str) -> ChatTypeBound {
        ChatTypeBound { id: id.to_string() }
    }

    #[test]
    fn create_system_message_returns_disguised() {
        let msg = make_message("sys", true, false);
        let out = OutgoingChatMessage::create(msg);
        match out {
            OutgoingChatMessage::Disguised(d) => assert_eq!(d.content.text, "sys"),
            other => panic!("expected Disguised, got {other:?}"),
        }
    }

    #[test]
    fn create_non_system_message_returns_player() {
        let msg = make_message("hello", false, false);
        let out = OutgoingChatMessage::create(msg.clone());
        match out {
            OutgoingChatMessage::Player(p) => assert_eq!(p.message, msg),
            other => panic!("expected Player, got {other:?}"),
        }
    }

    #[test]
    fn disguised_send_ignores_filter_flag() {
        let out = OutgoingChatMessage::Disguised(Disguised {
            content: Component {
                text: "d".to_string(),
            },
        });
        let mut player = ServerPlayer::default();
        out.send_to_player(&mut player, true, make_bound("chat"));
        assert_eq!(player.connection.disguised_sent.len(), 1);
        assert_eq!(player.connection.disguised_sent[0].0.text, "d");
        assert_eq!(player.connection.disguised_sent[0].1.id, "chat");
        assert!(player.connection.player_sent.is_empty());
    }

    #[test]
    fn player_send_when_not_filtered() {
        let msg = make_message("hi", false, false);
        let out = OutgoingChatMessage::Player(Player { message: msg });
        let mut player = ServerPlayer::default();
        out.send_to_player(&mut player, false, make_bound("chat"));
        assert_eq!(player.connection.player_sent.len(), 1);
        assert_eq!(player.connection.player_sent[0].0.decorated.text, "hi");
        assert!(!player.connection.player_sent[0].0.is_fully_filtered());
        assert!(player.connection.disguised_sent.is_empty());
    }

    #[test]
    fn player_send_skips_when_filter_makes_fully_filtered() {
        let msg = make_message("bad", false, false);
        let out = OutgoingChatMessage::Player(Player { message: msg });
        let mut player = ServerPlayer::default();
        out.send_to_player(&mut player, true, make_bound("chat"));
        assert!(player.connection.player_sent.is_empty());
        assert!(player.connection.disguised_sent.is_empty());
    }

    #[test]
    fn content_and_already_filtered_player_are_handled() {
        let disguised = OutgoingChatMessage::Disguised(Disguised {
            content: Component {
                text: String::new(),
            },
        });
        assert_eq!(disguised.content().text, "");

        let msg = make_message("filtered", false, true);
        let player_out = OutgoingChatMessage::Player(Player {
            message: msg.clone(),
        });
        assert_eq!(player_out.content().text, "filtered");

        let mut player = ServerPlayer::default();
        player_out.send_to_player(&mut player, false, make_bound("bound"));
        assert!(player.connection.player_sent.is_empty());
        assert!(player.connection.disguised_sent.is_empty());
    }
}
