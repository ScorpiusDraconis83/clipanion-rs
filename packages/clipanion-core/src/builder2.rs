use std::{fmt::Display, iter::once};

use colored::Colorize;
use itertools::Itertools;

use crate::{machine, runner2::{self, DeriveState, RunnerState, ValidateTransition}, shared::{Arg, INITIAL_NODE_ID, SUCCESS_NODE_ID}, CommandUsageOptions, CommandUsageResult, Error};

#[derive(Debug, Clone)]
pub struct Info {
    pub program_name: String,
    pub binary_name: String,
    pub version: String,
    pub about: String,
    pub colorized: bool,
}

#[derive(Debug)]
pub struct Context {
    command_id: usize,
    command_spec: CommandSpec,
}

#[derive(Clone, Debug, Default)]
pub struct State<'a> {
    pub context_id: usize,
    pub node_id: usize,
    pub keyword_count: usize,
    pub path: Vec<&'a str>,
    pub positional_values: Vec<(usize, Vec<&'a str>)>,
    pub option_values: Vec<(usize, Vec<&'a str>)>,
}

impl<'a> State<'a> {
    pub fn values(&self) -> Vec<(usize, Vec<&'a str>)> {
        self.positional_values.clone()
            .into_iter()
            .chain(self.option_values.clone().into_iter())
            .sorted_by_key(|(id, _)| *id)
            .collect()
    }
}

pub trait SelectBestState<'a> {
    fn select_best_state(self) -> Result<State<'a>, Error<'a>>;
}

impl<'a> SelectBestState<'a> for Vec<State<'a>> {
    fn select_best_state(self) -> Result<State<'a>, Error<'a>> {
        let mut all_states = self;

        let highest_keyword_count = all_states.iter()
            .map(|state| state.keyword_count)
            .max()
            .unwrap();

        all_states.retain(|state| {
            state.keyword_count == highest_keyword_count
        });

        let state
            = all_states.pop()
                .ok_or(Error::NotFound(vec![]))?;

        Ok(state)
    }
}

impl<'a> RunnerState for State<'a> {
    fn get_context_id(&self) -> usize {
        self.context_id
    }

    fn set_context_id(&mut self, context_id: usize) {
        self.context_id = context_id;
    }

    fn get_node_id(&self) -> usize {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: usize) {
        self.node_id = node_id;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check {
    IsOptionLike,
    IsNotOptionLike,
}

impl<'a> ValidateTransition<State<'a>> for Check {
    fn check(&self, _state: &State<'a>, arg: &str) -> bool {
        match self {
            Check::IsOptionLike => {
                arg.starts_with("-")
            },
            
            Check::IsNotOptionLike => {
                !arg.starts_with("-")
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attachment {
    Option,
    Positional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reducer {
    IncreaseStaticCount,
    StartValue(Attachment, usize),
    PushValue(Attachment),
}

impl<'a> DeriveState<State<'a>> for Reducer {
    fn derive(&self, state: &mut State<'a>, _target_id: usize, token: &str) -> () {
        match self {
            Reducer::IncreaseStaticCount => {
                state.keyword_count += 1;
            },

            Reducer::StartValue(attachment, positional_id) => {
                match attachment {
                    Attachment::Option => {
                        state.option_values.push((*positional_id, vec![]));
                    },

                    Attachment::Positional => {
                        state.positional_values.push((*positional_id, vec![token.to_string()]));
                    },
                }
            },

            Reducer::PushValue(attachment) => {
                match attachment {
                    Attachment::Option => {
                        if let Some((_, ref mut values)) = state.option_values.last_mut() {
                            values.push(token);
                        }
                    },

                    Attachment::Positional => {
                        if let Some((_, ref mut values)) = state.positional_values.last_mut() {
                            values.push(token);
                        }
                    },
                }
            },
        }
    }
}

type Machine
    = machine::Machine<Option<Check>, Option<Reducer>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PositionalSpec {
    Keyword {
        expected: String,
    },

    Dynamic {
        name: String,
        description: String,

        min_len: usize,
        extra_len: Option<usize>,
    },
}

fn format_range(f: &mut std::fmt::Formatter<'_>, name: &str, min_len: usize, extra_len: Option<usize>) -> std::fmt::Result {
    if min_len > 0 {
        write!(f, "<{}>", name)?;

        for i in 1..min_len {
            write!(f, " <{}{}>", name, i + 1)?;
        }
    }

    let spacing
        = if min_len > 0 {" "} else {""};

    if extra_len != Some(0) {
        if let Some(extra_len) = extra_len {
            write!(f, "{}[…{}{}]", spacing, name, extra_len)?;
        } else {
            write!(f, "{}[…{}N]", spacing, name)?;
        }
    }

    Ok(())
}

fn format_collection<T: Display>(f: &mut std::fmt::Formatter<'_>, components: impl IntoIterator<Item = T>, separator: &str) -> std::fmt::Result {
    let mut first = true;

    for component in components {
        if !first {
            write!(f, "{}", separator)?;
        }

        write!(f, "{}", component)?;
        first = false;
    }

    Ok(())
}

impl std::fmt::Display for PositionalSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionalSpec::Keyword {expected} => {
                write!(f, "{}", expected)
            },

            PositionalSpec::Dynamic {name, min_len, extra_len, ..} => {
                format_range(f, name, *min_len, *extra_len)
            },
        }
    }
}

impl PositionalSpec {
    pub fn keyword<T: Into<String>>(value: T) -> Self {
        PositionalSpec::Keyword {
            expected: value.into(),
        }
    }

    pub fn optional() -> Self {
        PositionalSpec::Dynamic {
            name: "".to_string(),
            description: "".to_string(),

            min_len: 0,
            extra_len: Some(1),
        }
    }

    pub fn required() -> Self {
        PositionalSpec::Dynamic {
            name: "".to_string(),
            description: "".to_string(),
            
            min_len: 1,
            extra_len: Some(0),
        }
    }

    pub fn rest() -> Self {
        PositionalSpec::Dynamic {
            name: "".to_string(),
            description: "".to_string(),

            min_len: 0,
            extra_len: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionSpec {
    pub primary_name: String,
    pub aliases: Vec<String>,

    pub description: String,

    pub min_len: usize,
    pub extra_len: Option<usize>,

    pub allow_binding: bool,
    pub is_hidden: bool,
    pub is_required: bool,
}

impl OptionSpec {
    pub fn boolean<TName: Into<String>>(name: TName) -> Self {
        OptionSpec {
            primary_name: name.into(),
            aliases: vec![],
            description: "".to_string(),
            
            min_len: 0,
            extra_len: Some(0),
            
            allow_binding: false,
            is_hidden: false,
            is_required: true,
        }
    }

    pub fn parametrized<TName: Into<String>>(name: TName) -> Self {
        OptionSpec {
            primary_name: name.into(),
            aliases: vec![],
            description: "".to_string(),
            
            min_len: 1,
            extra_len: Some(0),
            
            allow_binding: false,
            is_hidden: false,
            is_required: true,
        }
    }
}

impl std::fmt::Display for OptionSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_required {
            write!(f, "<{}", self.primary_name)?;
        } else {
            write!(f, "[{}", self.primary_name)?;
        }

        for alias in &self.aliases {
            write!(f, ",{}", alias)?;
        }

        if self.min_len > 0 || self.extra_len != Some(0) {
            write!(f, " ")?;
            format_range(f, "arg", self.min_len, self.extra_len)?;
        }

        if self.is_required {
            write!(f, ">")
        } else {
            write!(f, "]")
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Component {
    Positional(PositionalSpec),
    Option(OptionSpec),
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Component::Positional(spec)
                => write!(f, "{}", spec),

            Component::Option(spec)
                => write!(f, "{}", spec),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandSpec {
    pub paths: Vec<Vec<String>>,
    pub components: Vec<Component>,
}

impl std::fmt::Display for CommandSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_collection(f, self.components.iter(), " ")?;
        Ok(())
    }
}

impl CommandSpec {
    pub fn usage(&self) -> CommandUsageResult {
        CommandUsageResult::new(self.clone())
    }

    pub fn build(&self, command_id: usize) -> Machine {
        CommandBuilderContext::new(&self, command_id).build()
    }
}

pub struct CommandBuilderContext<'a> {
    machine: Machine,
    spec: &'a CommandSpec,
    inhibit_options: usize,
}

impl<'a> CommandBuilderContext<'a> {
    fn new(spec: &'a CommandSpec, command_id: usize) -> Self {
        CommandBuilderContext {
            machine: Machine::new(command_id),
            spec,
            inhibit_options: 0,
        }
    }

    fn enter_inhibit_options(&mut self) {
        self.inhibit_options += 1;
    }

    fn exit_inhibit_options(&mut self) {
        self.inhibit_options -= 1;
    }

    fn attach_options(&mut self, pre_options_node_id: usize) -> usize {
        if self.inhibit_options > 0 {
            return pre_options_node_id;
        }

        let post_options_node_id
            = self.machine.create_node();

        self.machine.register_shortcut(
            pre_options_node_id,
            post_options_node_id,
        );

        let options = self.spec.components.iter()
            .enumerate()
            .filter_map(|(i, component)| match component {
                Component::Option(option) => Some((i, option)),
                _ => None,
            });

        for (option_id, option) in options {
            let all_names
                = once(&option.primary_name)
                    .chain(option.aliases.iter());

            for name in all_names {
                let mut post_option_node_id
                    = self.machine.create_node();

                self.machine.register_static(
                    pre_options_node_id,
                    Arg::User(name.to_string()),
                    post_option_node_id,
                    Some(Reducer::StartValue(Attachment::Option, option_id)),
                );

                let accepts_arguments
                    = option.min_len > 0 || option.extra_len != Some(0);

                if accepts_arguments {
                    self.enter_inhibit_options();

                    post_option_node_id = self.attach_variadic(
                        post_option_node_id,
                        option.min_len,
                        option.extra_len,
                        Reducer::PushValue(Attachment::Option),
                        Reducer::PushValue(Attachment::Option),
                    );

                    self.exit_inhibit_options();
                }

                self.machine.register_shortcut(
                    post_option_node_id,
                    pre_options_node_id,
                );
            }
        }

        post_options_node_id
    }

    fn attach_optional(&mut self, pre_node_id: usize, reducer: Reducer) -> usize {
        let next_node_id
            = self.machine.create_node();

        self.machine.register_dynamic(
            pre_node_id,
            Some(Check::IsNotOptionLike),
            next_node_id,
            Some(reducer),
        );

        self.machine.register_shortcut(
            pre_node_id,
            next_node_id,
        );

        self.attach_options(next_node_id)
    }

    fn attach_required(&mut self, pre_node_id: usize, reducer: Reducer) -> usize {
        let next_node_id
            = self.machine.create_node();

        self.machine.register_dynamic(
            pre_node_id,
            Some(Check::IsNotOptionLike),
            next_node_id,
            Some(reducer),
        );

        self.attach_options(next_node_id)
    }

    fn attach_variadic(&mut self, pre_node_id: usize, min_len: usize, extra_len: Option<usize>, start_action: Reducer, subsequent_actions: Reducer) -> usize {
        let mut current_node_id
            = pre_node_id;

        let mut next_action
            = start_action;

        for _ in 0..min_len {
            current_node_id = self.attach_required(current_node_id, next_action);
            next_action = subsequent_actions;
        }

        match extra_len {
            Some(extra_len) => {
                for _ in 0..extra_len {
                    current_node_id = self.attach_optional(current_node_id, next_action);
                    next_action = subsequent_actions;
                }
            },

            None => {
                if next_action == start_action && next_action != subsequent_actions {
                    current_node_id = self.attach_optional(current_node_id, next_action);
                    next_action = subsequent_actions;
                }

                let next_node_id
                    = self.machine.create_node();

                self.machine.register_shortcut(
                    current_node_id,
                    next_node_id,
                );

                current_node_id
                    = self.attach_options(next_node_id);

                self.machine.register_dynamic(
                    next_node_id,
                    Some(Check::IsNotOptionLike),
                    next_node_id,
                    Some(next_action),
                );
            },
        }

        current_node_id
    }

    fn build(mut self) -> Machine {
        let first_node_id
            = self.machine.create_node();

        self.machine.register_static(
            INITIAL_NODE_ID,
            Arg::StartOfInput,
            first_node_id,
            None,
        );

        let mut current_node_id
            = self.attach_options(first_node_id);

        let positionals
            = self.spec.components.iter()
                .enumerate()
                .filter_map(|(i, component)| match component {
                    Component::Positional(spec) => Some((i, spec)),
                    _ => None,
                });

        for (id, spec) in positionals {
            match spec {
                PositionalSpec::Keyword {expected} => {
                    let next_node_id
                        = self.machine.create_node();

                    self.machine.register_static(
                        current_node_id,
                        Arg::User(expected.to_string()),
                        next_node_id,
                        Some(Reducer::IncreaseStaticCount),
                    );

                    current_node_id
                        = self.attach_options(next_node_id);
                },

                PositionalSpec::Dynamic {min_len, extra_len, ..} => {
                    current_node_id = self.attach_variadic(
                        current_node_id,
                        *min_len,
                        *extra_len,
                        Reducer::StartValue(Attachment::Positional, id),
                        Reducer::PushValue(Attachment::Positional),
                    );
                },
            }
        }

        self.machine.register_static(
            current_node_id,
            Arg::EndOfInput,
            SUCCESS_NODE_ID,
            None,
        );

        self.machine
    }
}

#[derive(Clone)]
pub struct CliBuilder {
    commands: Vec<CommandSpec>,
}

impl CliBuilder {
    pub fn new() -> Self {
        CliBuilder {
            commands: vec![],
        }
    }

    pub fn add_command(&mut self, spec: CommandSpec) -> &mut Self {
        self.commands.push(spec);
        self
    }

    pub fn compile(&self) -> Machine {
        let command_machines = self.commands.iter()
            .enumerate()
            .map(|(command_id, command)| command.build(command_id))
            .collect::<Vec<_>>();

        let mut machine
            = Machine::new_any_of(command_machines);

        //machine.simplify_machine();
        machine
    }

    pub fn run<'a>(&'a self, args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<(State, &'a CommandSpec), Error<'a>> {
        let machine
            = self.compile();

        let state
            = runner2::Runner::run(&machine, args).unwrap()
                .select_best_state()?;

        let command
            = &self.commands[state.context_id];

        Ok((state, command))
    }
}

#[test]
fn it_should_select_the_default_command_when_using_no_arguments() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![],
    });

    let (state, _context)
        = cli_builder.run(&[""; 0]).unwrap();

    assert_eq!(state.context_id, 0);
}

#[test]
fn it_should_select_the_default_command_when_using_mandatory_positional_arguments() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![
            Component::Positional(PositionalSpec::required()),
            Component::Positional(PositionalSpec::required()),
        ],
    });

    let (state, _context)
        = cli_builder.run(&["foo", "bar"]).unwrap();

    assert_eq!(state.context_id, 0);
}

#[test]
fn it_should_select_commands_by_their_path() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![Component::Positional(PositionalSpec::keyword("foo"))],
    });

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![Component::Positional(PositionalSpec::keyword("bar"))],
    });

    let (state, _context)
        = cli_builder.run(&["foo"]).unwrap();

    assert_eq!(state.context_id, 0);

    let (state, _context)
        = cli_builder.run(&["bar"]).unwrap();

    assert_eq!(state.context_id, 1);
}

#[test]
fn it_should_favor_paths_over_mandatory_positional_arguments() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![Component::Positional(PositionalSpec::required())],
    });

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![Component::Positional(PositionalSpec::keyword("foo"))],
    });

    let (state, _context)
        = cli_builder.run(&["foo"]).unwrap();

    assert_eq!(state.context_id, 1);
}

#[test]
fn it_should_favor_paths_over_optional_positional_arguments() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![Component::Positional(PositionalSpec::optional())],
    });

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![Component::Positional(PositionalSpec::keyword("foo"))],
    });

    let machine
        = cli_builder.compile();

    let (state, _context)
        = cli_builder.run(&["foo"]).unwrap();

    assert_eq!(state.context_id, 1);
}

#[test]
fn it_should_aggregate_positional_values() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![
            Component::Positional(PositionalSpec::required()),
            Component::Positional(PositionalSpec::required()),
        ],
    });

    let (state, _context)
        = cli_builder.run(&["foo", "bar"]).unwrap();

    assert_eq!(state.values(), vec![
        (0, vec!["foo".to_string()]),
        (1, vec!["bar".to_string()]),
    ]);
}

#[test]
fn it_should_aggregate_positional_values_with_rest() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        paths: vec![],
        components: vec![
            Component::Positional(PositionalSpec::required()),
            Component::Positional(PositionalSpec::rest()),
        ],
    });

    println!("{:?}", cli_builder.compile());

    let (state, _context)
        = cli_builder.run(&["foo", "bar", "baz"]).unwrap();

    assert_eq!(state.values(), vec![
        (0, vec!["foo".to_string()]),
        (1, vec!["bar".to_string(), "baz".to_string()]),
    ]);
}
