use std::fmt;

/// Minimal `Either<L, R>` equivalent for Mojang DataFixers usage in `ScoreContents`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn left(&self) -> Option<&L> {
        match self {
            Either::Left(value) => Some(value),
            Either::Right(_) => None,
        }
    }

    pub fn right(&self) -> Option<&R> {
        match self {
            Either::Left(_) => None,
            Either::Right(value) => Some(value),
        }
    }
}

impl<L: fmt::Display, R: fmt::Display> fmt::Display for Either<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(value) => write!(f, "Left[{value}]"),
            Either::Right(value) => write!(f, "Right[{value}]"),
        }
    }
}

/// Minimal selector pattern representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorPattern {
    pattern: String,
}

impl SelectorPattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

impl fmt::Display for SelectorPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.pattern)
    }
}

/// Minimal score holder identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreHolder {
    name: String,
}

impl ScoreHolder {
    pub fn wildcard() -> Self {
        Self {
            name: "*".to_owned(),
        }
    }

    pub fn for_name_only(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_wildcard(&self) -> bool {
        self.name == "*"
    }
}

pub trait AsScoreHolder {
    fn as_score_holder(&self) -> ScoreHolder;
}

impl AsScoreHolder for ScoreHolder {
    fn as_score_holder(&self) -> ScoreHolder {
        self.clone()
    }
}

/// Minimal mutable chat component representation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MutableComponent {
    pub text: String,
}

impl MutableComponent {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
        }
    }
}

/// Minimal component helper matching `Component.empty()`.
pub struct Component;

impl Component {
    pub fn empty() -> MutableComponent {
        MutableComponent::empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StyledFormat;

impl StyledFormat {
    pub const NO_STYLE: StyledFormat = StyledFormat;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
}

impl CommandSyntaxException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CommandSyntaxException {}

pub fn error_not_single_entity() -> CommandSyntaxException {
    CommandSyntaxException::new("ERROR_NOT_SINGLE_ENTITY")
}

pub trait ReadOnlyScoreInfoLike {
    fn format_value(&self, style: StyledFormat) -> MutableComponent;
}

pub trait ObjectiveLike {
    fn number_format_or_default(&self, default: StyledFormat) -> StyledFormat;
}

pub trait ScoreboardLike {
    type Objective: ObjectiveLike;
    type ScoreInfo: ReadOnlyScoreInfoLike;

    fn get_objective(&self, name: &str) -> Option<&Self::Objective>;
    fn get_player_score_info(
        &self,
        holder: &ScoreHolder,
        objective: &Self::Objective,
    ) -> Option<&Self::ScoreInfo>;
}

pub trait MinecraftServerLike {
    type Scoreboard: ScoreboardLike;

    fn get_scoreboard(&self) -> &Self::Scoreboard;
}

pub trait CommandSourceStackLike {
    type Entity: AsScoreHolder;
    type Server: MinecraftServerLike;

    fn find_entities(
        &self,
        pattern: &SelectorPattern,
    ) -> Result<Vec<Self::Entity>, CommandSyntaxException>;
    fn get_server(&self) -> Option<&Self::Server>;
}

/// Port of `net.minecraft.network.chat.contents.ScoreContents`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreContents {
    pub name: Either<SelectorPattern, String>,
    pub objective: String,
}

impl ScoreContents {
    pub fn new(name: Either<SelectorPattern, String>, objective: String) -> Self {
        Self { name, objective }
    }

    // JAVA: net.minecraft.network.chat.contents.ScoreContents#findTargetName
    fn find_target_name<S: CommandSourceStackLike>(
        &self,
        source: &S,
    ) -> Result<ScoreHolder, CommandSyntaxException> {
        if let Some(pattern) = self.name.left() {
            let entities = source.find_entities(pattern)?;
            if !entities.is_empty() {
                if entities.len() != 1 {
                    return Err(error_not_single_entity());
                }
                Ok(entities[0].as_score_holder())
            } else {
                Ok(ScoreHolder::for_name_only(pattern.pattern()))
            }
        } else {
            Ok(ScoreHolder::for_name_only(
                self.name
                    .right()
                    .expect("ScoreContents name must be Left or Right")
                    .clone(),
            ))
        }
    }

    // JAVA: net.minecraft.network.chat.contents.ScoreContents#getScore
    fn get_score<S: CommandSourceStackLike>(
        &self,
        holder: &ScoreHolder,
        source: &S,
    ) -> MutableComponent {
        if let Some(server) = source.get_server() {
            let scoreboard = server.get_scoreboard();
            if let Some(objective) = scoreboard.get_objective(&self.objective) {
                if let Some(info) = scoreboard.get_player_score_info(holder, objective) {
                    return info.format_value(
                        objective.number_format_or_default(StyledFormat::NO_STYLE),
                    );
                }
            }
        }
        Component::empty()
    }

    // JAVA: net.minecraft.network.chat.contents.ScoreContents#resolve
    pub fn resolve<S: CommandSourceStackLike>(
        &self,
        source: Option<&S>,
        entity: Option<&S::Entity>,
        _seed: i32,
    ) -> Result<MutableComponent, CommandSyntaxException> {
        let Some(source) = source else {
            return Ok(Component::empty());
        };

        let holder = self.find_target_name(source)?;
        let resolved_holder = if let Some(entity) = entity {
            if holder == ScoreHolder::wildcard() {
                entity.as_score_holder()
            } else {
                holder
            }
        } else {
            holder
        };
        Ok(self.get_score(&resolved_holder, source))
    }
}

impl fmt::Display for ScoreContents {
    // JAVA: net.minecraft.network.chat.contents.ScoreContents#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "score{{name='{}', objective='{}'}}",
            self.name, self.objective
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MockEntity {
        holder: ScoreHolder,
    }

    impl MockEntity {
        fn new(name: &str) -> Self {
            Self {
                holder: ScoreHolder::for_name_only(name),
            }
        }
    }

    impl AsScoreHolder for MockEntity {
        fn as_score_holder(&self) -> ScoreHolder {
            self.holder.clone()
        }
    }

    struct MockScoreInfo {
        text: String,
    }

    impl ReadOnlyScoreInfoLike for MockScoreInfo {
        fn format_value(&self, _style: StyledFormat) -> MutableComponent {
            MutableComponent {
                text: self.text.clone(),
            }
        }
    }

    struct MockObjective {
        style: StyledFormat,
    }

    impl ObjectiveLike for MockObjective {
        fn number_format_or_default(&self, _default: StyledFormat) -> StyledFormat {
            self.style
        }
    }

    struct MockScoreboard {
        objective_name: String,
        objective: MockObjective,
        info_holder: String,
        info: Option<MockScoreInfo>,
    }

    impl ScoreboardLike for MockScoreboard {
        type Objective = MockObjective;
        type ScoreInfo = MockScoreInfo;

        fn get_objective(&self, name: &str) -> Option<&Self::Objective> {
            if name == self.objective_name {
                Some(&self.objective)
            } else {
                None
            }
        }

        fn get_player_score_info(
            &self,
            holder: &ScoreHolder,
            _objective: &Self::Objective,
        ) -> Option<&Self::ScoreInfo> {
            if holder.name() == self.info_holder {
                self.info.as_ref()
            } else {
                None
            }
        }
    }

    struct MockServer {
        scoreboard: MockScoreboard,
    }

    impl MinecraftServerLike for MockServer {
        type Scoreboard = MockScoreboard;

        fn get_scoreboard(&self) -> &Self::Scoreboard {
            &self.scoreboard
        }
    }

    struct MockSource {
        entities: Vec<MockEntity>,
        entities_error: Option<CommandSyntaxException>,
        server: Option<MockServer>,
    }

    impl CommandSourceStackLike for MockSource {
        type Entity = MockEntity;
        type Server = MockServer;

        fn find_entities(
            &self,
            _pattern: &SelectorPattern,
        ) -> Result<Vec<Self::Entity>, CommandSyntaxException> {
            if let Some(err) = &self.entities_error {
                return Err(err.clone());
            }
            Ok(self.entities.clone())
        }

        fn get_server(&self) -> Option<&Self::Server> {
            self.server.as_ref()
        }
    }

    #[test]
    fn resolve_null_source_returns_empty() {
        let contents = ScoreContents::new(Either::Right("player".to_owned()), "obj".to_owned());
        let result = contents.resolve::<MockSource>(None, None, 0).unwrap();
        assert_eq!(result, Component::empty());
    }

    #[test]
    fn resolve_literal_name_returns_formatted_score() {
        let contents = ScoreContents::new(Either::Right("Alice".to_owned()), "kills".to_owned());
        let source = MockSource {
            entities: vec![],
            entities_error: None,
            server: Some(MockServer {
                scoreboard: MockScoreboard {
                    objective_name: "kills".to_owned(),
                    objective: MockObjective {
                        style: StyledFormat::NO_STYLE,
                    },
                    info_holder: "Alice".to_owned(),
                    info: Some(MockScoreInfo {
                        text: "42".to_owned(),
                    }),
                },
            }),
        };
        let result = contents.resolve(Some(&source), None, 0).unwrap();
        assert_eq!(result.text, "42");
    }

    #[test]
    fn selector_multiple_entities_errors_and_single_entity_resolves() {
        let contents = ScoreContents::new(
            Either::Left(SelectorPattern::new("@p")),
            "hp".to_owned(),
        );

        let multi_source = MockSource {
            entities: vec![MockEntity::new("A"), MockEntity::new("B")],
            entities_error: None,
            server: None,
        };
        let err = contents.resolve(Some(&multi_source), None, 0).unwrap_err();
        assert_eq!(err, error_not_single_entity());

        let single_source = MockSource {
            entities: vec![MockEntity::new("Steve")],
            entities_error: None,
            server: Some(MockServer {
                scoreboard: MockScoreboard {
                    objective_name: "hp".to_owned(),
                    objective: MockObjective {
                        style: StyledFormat::NO_STYLE,
                    },
                    info_holder: "Steve".to_owned(),
                    info: Some(MockScoreInfo {
                        text: "20".to_owned(),
                    }),
                },
            }),
        };
        let result = contents.resolve(Some(&single_source), None, 0).unwrap();
        assert_eq!(result.text, "20");
    }

    #[test]
    fn empty_selector_uses_pattern_and_wildcard_substitutes_entity() {
        let contents = ScoreContents::new(
            Either::Left(SelectorPattern::new("*")),
            "obj".to_owned(),
        );
        let source = MockSource {
            entities: vec![],
            entities_error: None,
            server: Some(MockServer {
                scoreboard: MockScoreboard {
                    objective_name: "obj".to_owned(),
                    objective: MockObjective {
                        style: StyledFormat::NO_STYLE,
                    },
                    info_holder: "Bob".to_owned(),
                    info: Some(MockScoreInfo {
                        text: "7".to_owned(),
                    }),
                },
            }),
        };
        let entity = MockEntity::new("Bob");
        let result = contents.resolve(Some(&source), Some(&entity), 0).unwrap();
        assert_eq!(result.text, "7");
    }

    #[test]
    fn missing_objective_returns_empty() {
        let contents = ScoreContents::new(Either::Right("P".to_owned()), "missing".to_owned());
        let source = MockSource {
            entities: vec![],
            entities_error: None,
            server: Some(MockServer {
                scoreboard: MockScoreboard {
                    objective_name: "other".to_owned(),
                    objective: MockObjective {
                        style: StyledFormat::NO_STYLE,
                    },
                    info_holder: "P".to_owned(),
                    info: Some(MockScoreInfo {
                        text: "1".to_owned(),
                    }),
                },
            }),
        };
        let result = contents.resolve(Some(&source), None, 0).unwrap();
        assert_eq!(result, Component::empty());
    }

    #[test]
    fn to_string_and_edge_value_score_text() {
        let contents = ScoreContents::new(Either::Right("target".to_owned()), "obj".to_owned());
        assert_eq!(
            contents.to_string(),
            "score{name='Right[target]', objective='obj'}"
        );

        let edge_values = [i32::MIN, -1, 0, i32::MAX];
        for value in edge_values {
            let source = MockSource {
                entities: vec![],
                entities_error: None,
                server: Some(MockServer {
                    scoreboard: MockScoreboard {
                        objective_name: "obj".to_owned(),
                        objective: MockObjective {
                            style: StyledFormat::NO_STYLE,
                        },
                        info_holder: "target".to_owned(),
                        info: Some(MockScoreInfo {
                            text: value.to_string(),
                        }),
                    },
                }),
            };
            let result = contents.resolve(Some(&source), None, 0).unwrap();
            assert_eq!(result.text, value.to_string());
        }

        let nan_bits = f32::NAN.to_bits();
        let neg_zero_bits = (-0.0f32).to_bits();
        let float_bits_source = MockSource {
            entities: vec![],
            entities_error: None,
            server: Some(MockServer {
                scoreboard: MockScoreboard {
                    objective_name: "obj".to_owned(),
                    objective: MockObjective {
                        style: StyledFormat::NO_STYLE,
                    },
                    info_holder: "target".to_owned(),
                    info: Some(MockScoreInfo {
                        text: format!("{nan_bits}:{neg_zero_bits}"),
                    }),
                },
            }),
        };
        let result = contents
            .resolve(Some(&float_bits_source), None, 0)
            .unwrap();
        assert_eq!(result.text, format!("{nan_bits}:{neg_zero_bits}"));
        assert_eq!(neg_zero_bits, 0x8000_0000u32);
        assert_ne!(nan_bits, 0);
    }
}
