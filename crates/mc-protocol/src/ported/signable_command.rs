//! Port of `net.minecraft.network.chat.SignableCommand` (1.21.11).

use std::fmt;

/// Minimal abstraction over Brigadier's `StringRange`.
pub trait StringRange {
    fn get<'a>(&self, command: &'a str) -> &'a str;
}

/// Minimal abstraction over Brigadier's `ParsedArgument`.
pub trait ParsedArgument {
    type Range: StringRange;

    fn range(&self) -> &Self::Range;
}

/// Minimal abstraction for an argument command node that may carry a signed argument type.
pub trait SignedArgumentNode: Clone {
    fn name(&self) -> &str;
}

/// Minimal abstraction over Brigadier's `ParsedCommandNode`.
pub trait ParsedCommandNode<N: SignedArgumentNode> {
    /// Returns the node as a signed argument node when it is an `ArgumentCommandNode`
    /// whose type implements `SignedArgument`; otherwise returns `None`.
    fn as_signed_argument_node(&self) -> Option<&N>;
}

/// Minimal abstraction over Brigadier's `CommandContextBuilder`.
pub trait CommandContext<N: SignedArgumentNode> {
    type ParsedNode: ParsedCommandNode<N>;
    type ParsedArg: ParsedArgument;

    fn nodes(&self) -> &[Self::ParsedNode];
    fn get_argument(&self, name: &str) -> Option<&Self::ParsedArg>;
    fn child(&self) -> Option<&Self>;
    /// Identity comparison equivalent to Java's `getRootNode() != other.getRootNode()`.
    fn same_root(&self, other: &Self) -> bool;
}

/// Minimal abstraction over Brigadier's `ParseResults`.
pub trait ParseResults<N: SignedArgumentNode> {
    type Ctx: CommandContext<N>;

    fn reader_string(&self) -> &str;
    fn context(&self) -> &Self::Ctx;
}

/// Port of `net.minecraft.network.chat.SignableCommand.Argument`.
#[derive(Clone, PartialEq, Eq)]
pub struct Argument<N> {
    pub node: N,
    pub value: String,
}

impl<N: SignedArgumentNode> Argument<N> {
    // JAVA: net.minecraft.network.chat.SignableCommand$Argument#name
    pub fn name(&self) -> &str {
        self.node.name()
    }
}

impl<N: fmt::Debug> fmt::Debug for Argument<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Argument")
            .field("node", &self.node)
            .field("value", &self.value)
            .finish()
    }
}

/// Port of `net.minecraft.network.chat.SignableCommand`.
#[derive(Clone, PartialEq, Eq)]
pub struct SignableCommand<N> {
    arguments: Vec<Argument<N>>,
}

impl<N: fmt::Debug> fmt::Debug for SignableCommand<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignableCommand")
            .field("arguments", &self.arguments)
            .finish()
    }
}

impl<N: SignedArgumentNode> SignableCommand<N> {
    /// Record component accessor.
    pub fn arguments(&self) -> &[Argument<N>] {
        &self.arguments
    }

    // JAVA: net.minecraft.network.chat.SignableCommand#hasSignableArguments
    pub fn has_signable_arguments<PR: ParseResults<N>>(results: &PR) -> bool {
        !Self::of(results).arguments.is_empty()
    }

    // JAVA: net.minecraft.network.chat.SignableCommand#of
    pub fn of<PR: ParseResults<N>>(results: &PR) -> Self {
        let input = results.reader_string();
        let context = results.context();
        let mut arguments = Self::collect_arguments(input, context);

        let mut current = context;
        while let Some(child) = current.child() {
            if child.same_root(context) {
                break;
            }
            arguments.extend(Self::collect_arguments(input, child));
            current = child;
        }

        SignableCommand { arguments }
    }

    // JAVA: net.minecraft.network.chat.SignableCommand#collectArguments
    fn collect_arguments<Ctx: CommandContext<N>>(input: &str, ctx: &Ctx) -> Vec<Argument<N>> {
        let mut out = Vec::new();
        for parsed_node in ctx.nodes() {
            if let Some(node) = parsed_node.as_signed_argument_node() {
                let name = node.name();
                if let Some(parsed_arg) = ctx.get_argument(name) {
                    let value = parsed_arg.range().get(input).to_owned();
                    out.push(Argument {
                        node: node.clone(),
                        value,
                    });
                }
            }
        }
        out
    }

    // JAVA: net.minecraft.network.chat.SignableCommand#getArgument
    pub fn get_argument(&self, name: &str) -> Option<&Argument<N>> {
        for arg in &self.arguments {
            if name == arg.name() {
                return Some(arg);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestNode {
        name: String,
        signed: bool,
    }

    impl SignedArgumentNode for TestNode {
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[derive(Clone, Debug)]
    struct TestRange {
        start: usize,
        end: usize,
    }

    impl StringRange for TestRange {
        fn get<'a>(&self, command: &'a str) -> &'a str {
            &command[self.start..self.end]
        }
    }

    #[derive(Clone, Debug)]
    struct TestParsedArg {
        range: TestRange,
    }

    impl ParsedArgument for TestParsedArg {
        type Range = TestRange;

        fn range(&self) -> &Self::Range {
            &self.range
        }
    }

    #[derive(Clone, Debug)]
    struct TestParsedNode {
        node: TestNode,
    }

    impl ParsedCommandNode<TestNode> for TestParsedNode {
        fn as_signed_argument_node(&self) -> Option<&TestNode> {
            if self.node.signed {
                Some(&self.node)
            } else {
                None
            }
        }
    }

    struct TestContext {
        nodes: Vec<TestParsedNode>,
        arguments: HashMap<String, TestParsedArg>,
        child: Option<Box<TestContext>>,
        root_id: usize,
    }

    impl CommandContext<TestNode> for TestContext {
        type ParsedNode = TestParsedNode;
        type ParsedArg = TestParsedArg;

        fn nodes(&self) -> &[Self::ParsedNode] {
            &self.nodes
        }

        fn get_argument(&self, name: &str) -> Option<&Self::ParsedArg> {
            self.arguments.get(name)
        }

        fn child(&self) -> Option<&Self> {
            self.child.as_deref()
        }

        fn same_root(&self, other: &Self) -> bool {
            self.root_id == other.root_id
        }
    }

    struct TestParseResults {
        input: String,
        context: TestContext,
    }

    impl ParseResults<TestNode> for TestParseResults {
        type Ctx = TestContext;

        fn reader_string(&self) -> &str {
            &self.input
        }

        fn context(&self) -> &Self::Ctx {
            &self.context
        }
    }

    fn node(name: &str, signed: bool) -> TestParsedNode {
        TestParsedNode {
            node: TestNode {
                name: name.to_string(),
                signed,
            },
        }
    }

    fn arg(start: usize, end: usize) -> TestParsedArg {
        TestParsedArg {
            range: TestRange { start, end },
        }
    }

    #[test]
    fn no_signable_arguments() {
        let results = TestParseResults {
            input: "cmd foo".to_string(),
            context: TestContext {
                nodes: vec![node("foo", false)],
                arguments: HashMap::new(),
                child: None,
                root_id: 1,
            },
        };

        assert!(!SignableCommand::has_signable_arguments(&results));
        let cmd = SignableCommand::of(&results);
        assert!(cmd.arguments().is_empty());
        assert!(cmd.get_argument("foo").is_none());
    }

    #[test]
    fn single_signable_argument_extracts_value() {
        let mut arguments = HashMap::new();
        arguments.insert("msg".to_string(), arg(4, 9));
        let results = TestParseResults {
            input: "say hello".to_string(),
            context: TestContext {
                nodes: vec![node("msg", true)],
                arguments,
                child: None,
                root_id: 1,
            },
        };

        assert!(SignableCommand::has_signable_arguments(&results));
        let cmd = SignableCommand::of(&results);
        assert_eq!(cmd.arguments().len(), 1);
        let found = cmd.get_argument("msg").expect("argument present");
        assert_eq!(found.value, "hello");
        assert_eq!(found.name(), "msg");
    }

    #[test]
    fn signed_node_without_parsed_argument_is_ignored() {
        let results = TestParseResults {
            input: "cmd".to_string(),
            context: TestContext {
                nodes: vec![node("missing", true)],
                arguments: HashMap::new(),
                child: None,
                root_id: 1,
            },
        };

        let cmd = SignableCommand::of(&results);
        assert!(cmd.arguments().is_empty());
        assert!(!SignableCommand::has_signable_arguments(&results));
    }

    #[test]
    fn collects_from_child_contexts_until_same_root() {
        let mut root_args = HashMap::new();
        root_args.insert("a".to_string(), arg(0, 1));
        let mut child_args = HashMap::new();
        child_args.insert("b".to_string(), arg(2, 3));
        let mut grandchild_args = HashMap::new();
        grandchild_args.insert("c".to_string(), arg(4, 5));

        let grandchild = TestContext {
            nodes: vec![node("c", true)],
            arguments: grandchild_args,
            child: None,
            root_id: 2, // same root as original context -> traversal stops before collecting
        };
        let child = TestContext {
            nodes: vec![node("b", true)],
            arguments: child_args,
            child: Some(Box::new(grandchild)),
            root_id: 1,
        };
        let root = TestContext {
            nodes: vec![node("a", true)],
            arguments: root_args,
            child: Some(Box::new(child)),
            root_id: 2,
        };

        let results = TestParseResults {
            input: "x y z".to_string(),
            context: root,
        };
        let cmd = SignableCommand::of(&results);
        assert_eq!(cmd.arguments().len(), 2);
        assert_eq!(cmd.arguments()[0].name(), "a");
        assert_eq!(cmd.arguments()[0].value, "x");
        assert_eq!(cmd.arguments()[1].name(), "b");
        assert_eq!(cmd.arguments()[1].value, "y");
        assert!(cmd.get_argument("c").is_none());
    }

    #[test]
    fn get_argument_finds_by_name_and_preserves_order() {
        let mut arguments = HashMap::new();
        arguments.insert("x".to_string(), arg(0, 1));
        arguments.insert("y".to_string(), arg(2, 4));
        arguments.insert("z".to_string(), arg(5, 6));
        let results = TestParseResults {
            input: "a bb c".to_string(),
            context: TestContext {
                nodes: vec![node("x", true), node("skip", false), node("y", true), node("z", true)],
                arguments,
                child: None,
                root_id: 0,
            },
        };

        let cmd = SignableCommand::of(&results);
        assert_eq!(cmd.arguments().len(), 3);
        assert_eq!(cmd.get_argument("y").unwrap().value, "bb");
        assert_eq!(cmd.get_argument("x").unwrap().value, "a");
        assert_eq!(cmd.get_argument("z").unwrap().value, "c");
        assert!(cmd.get_argument("missing").is_none());
    }

    #[test]
    fn empty_command_has_no_signable_arguments() {
        let results = TestParseResults {
            input: String::new(),
            context: TestContext {
                nodes: Vec::new(),
                arguments: HashMap::new(),
                child: None,
                root_id: 42,
            },
        };

        let cmd = SignableCommand::of(&results);
        assert!(cmd.arguments().is_empty());
        assert!(!SignableCommand::has_signable_arguments(&results));
        assert!(cmd.get_argument("").is_none());
    }
}
