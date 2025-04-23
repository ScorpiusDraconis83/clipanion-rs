use std::iter::once;

use crate::{machine, runner2::{self, DeriveState, RunnerState, ValidateTransition}, shared::{Arg, INITIAL_NODE_ID, SUCCESS_NODE_ID}};

#[derive(Debug)]
pub struct Context {
    command_id: usize,
    command_spec: CommandSpec,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    context_id: usize,
    node_id: usize,
    keyword_count: usize,
    values: Vec<(usize, Vec<String>)>,
}

pub trait SelectBestState<'a> {
    fn select_best_state(self) -> State;
}

impl<'a> SelectBestState<'a> for Vec<State> {
    fn select_best_state(self) -> State {
        let mut all_states = self;

        let highest_keyword_count = all_states.iter()
            .map(|state| state.keyword_count)
            .max()
            .unwrap();

        all_states.retain(|state| {
            state.keyword_count == highest_keyword_count
        });

        all_states.pop().unwrap()
    }
}

impl RunnerState for State {
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

#[derive(Clone, Debug, PartialEq)]
enum Check {
}

impl ValidateTransition<State> for Check {
    fn check(&self, _state: &State, _arg: &str) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Attachment {
    Option,
    Positional,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Reducer {
    IncreaseStaticCount,
    StartValue(Attachment, usize),
    PushValue(Attachment),
}

impl DeriveState<State> for Reducer {
    fn derive(&self, state: &mut State, target_id: usize, token: &str) -> () {
        match self {
            Reducer::IncreaseStaticCount => {
                state.keyword_count += 1;
            },

            Reducer::StartValue(attachment, positional_id) => {
                match attachment {
                    Attachment::Option => {
                        // We insert options at the front. That's because positionals may be
                        // interrupted by options, but not the other way arround. By keeping
                        // options at the front, we can always push values to the last
                        // positional.
                        state.values.insert(0, (*positional_id, vec![]));
                    },

                    Attachment::Positional => {
                        state.values.push((*positional_id, vec![token.to_string()]));
                    },
                }
            },

            Reducer::PushValue(attachment) => {
                match attachment {
                    Attachment::Option => {
                        if let Some((_, ref mut values)) = state.values.first_mut() {
                            values.push(token.to_string());
                        }
                    },

                    Attachment::Positional => {
                        if let Some((_, ref mut values)) = state.values.last_mut() {
                            values.push(token.to_string());
                        }
                    },
                }
            },
        }
    }
}

type Machine
    = machine::Machine<Option<Check>, Option<Reducer>>;

#[derive(Clone, Debug)]
pub enum PositionalSpec {
    Keyword {
        expected: String,
    },

    Dynamic {
        name: String,
        description: String,

        min_len: usize,
        max_len: Option<usize>,
    },
}

impl PositionalSpec {
    fn keyword<T: Into<String>>(value: T) -> Self {
        PositionalSpec::Keyword {
            expected: value.into(),
        }
    }

    fn optional() -> Self {
        PositionalSpec::Dynamic {
            name: "".to_string(),
            description: "".to_string(),

            min_len: 0,
            max_len: Some(1),
        }
    }

    fn required() -> Self {
        PositionalSpec::Dynamic {
            name: "".to_string(),
            description: "".to_string(),
            
            min_len: 1,
            max_len: Some(1),
        }
    }

    fn rest() -> Self {
        PositionalSpec::Dynamic {
            name: "".to_string(),
            description: "".to_string(),

            min_len: 0,
            max_len: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OptionSpec {
    pub primary_name: String,
    pub aliases: Vec<String>,

    pub description: String,

    pub min_len: usize,
    pub max_len: Option<usize>,

    pub allow_binding: bool,
    pub is_hidden: bool,
    pub is_required: bool,
}

impl OptionSpec {
    fn boolean<TName: Into<String>>(name: TName) -> Self {
        OptionSpec {
            primary_name: name.into(),
            aliases: vec![],
            description: "".to_string(),
            
            min_len: 0,
            max_len: Some(0),
            
            allow_binding: false,
            is_hidden: false,
            is_required: true,
        }
    }

    fn parametrized<TName: Into<String>>(name: TName) -> Self {
        OptionSpec {
            primary_name: name.into(),
            aliases: vec![],
            description: "".to_string(),
            
            min_len: 1,
            max_len: Some(1),
            
            allow_binding: false,
            is_hidden: false,
            is_required: true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Component {
    Positional(PositionalSpec),
    Option(OptionSpec),
}

#[derive(Clone, Debug)]
pub struct CommandSpec {
    pub components: Vec<Component>,
}

impl CommandSpec {
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
                    = option.max_len
                        .map(|max_len| max_len > 0)
                        .unwrap_or(true);

                if accepts_arguments {
                    self.enter_inhibit_options();

                    post_option_node_id = self.attach_variadic(
                        post_options_node_id,
                        option.min_len,
                        option.max_len,
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
            None,
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
            None,
            next_node_id,
            Some(reducer),
        );

        self.attach_options(next_node_id)
    }

    fn attach_variadic(&mut self, pre_node_id: usize, min_len: usize, max_len: Option<usize>, start_action: Reducer, subsequent_actions: Reducer) -> usize {
        let mut current_node_id
            = pre_node_id;

        let mut next_action
            = start_action;

        for _ in 0..min_len {
            current_node_id = self.attach_required(current_node_id, next_action);
            next_action = subsequent_actions;
        }

        match max_len {
            Some(max_len) => {
                for _ in min_len..max_len {
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
                    None,
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

                PositionalSpec::Dynamic {min_len, max_len, ..} => {
                    current_node_id = self.attach_variadic(
                        current_node_id,
                        *min_len,
                        *max_len,
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
struct CliBuilder {
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

        machine.simplify_machine();
        machine
    }

    pub fn run(mut self, args: &[&str]) -> (State, CommandSpec) {
        let machine
            = self.compile();

        let state
            = runner2::Runner::run(&machine, args).unwrap()
                .select_best_state();

        let command
            = self.commands.remove(state.context_id);

        (state, command)
    }
}

#[test]
fn it_should_select_the_default_command_when_using_no_arguments() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        components: vec![],
    });

    let (state, _context)
        = cli_builder.run(&[]);

    assert_eq!(state.context_id, 0);
}

#[test]
fn it_should_select_the_default_command_when_using_mandatory_positional_arguments() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        components: vec![
            Component::Positional(PositionalSpec::required()),
            Component::Positional(PositionalSpec::required()),
        ],
    });

    let (state, _context)
        = cli_builder.run(&["foo", "bar"]);

    assert_eq!(state.context_id, 0);
}

#[test]
fn it_should_select_commands_by_their_path() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        components: vec![Component::Positional(PositionalSpec::keyword("foo"))],
    });

    cli_builder.add_command(CommandSpec {
        components: vec![Component::Positional(PositionalSpec::keyword("bar"))],
    });

    let (state, _context)
        = cli_builder.clone().run(&["foo"]);

    assert_eq!(state.context_id, 0);

    let (state, _context)
        = cli_builder.run(&["bar"]);

    assert_eq!(state.context_id, 1);
}

#[test]
fn it_should_favor_paths_over_mandatory_positional_arguments() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        components: vec![Component::Positional(PositionalSpec::required())],
    });

    cli_builder.add_command(CommandSpec {
        components: vec![Component::Positional(PositionalSpec::keyword("foo"))],
    });

    let (state, _context)
        = cli_builder.run(&["foo"]);

    assert_eq!(state.context_id, 1);
}

#[test]
fn it_should_favor_paths_over_optional_positional_arguments() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        components: vec![Component::Positional(PositionalSpec::optional())],
    });

    cli_builder.add_command(CommandSpec {
        components: vec![Component::Positional(PositionalSpec::keyword("foo"))],
    });

    let machine
        = cli_builder.compile();

    let (state, _context)
        = cli_builder.run(&["foo"]);

    assert_eq!(state.context_id, 1);
}

#[test]
fn it_should_aggregate_positional_values() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        components: vec![
            Component::Positional(PositionalSpec::required()),
            Component::Positional(PositionalSpec::required()),
        ],
    });

    let (state, _context)
        = cli_builder.run(&["foo", "bar"]);

    assert_eq!(state.values, vec![
        (0, vec!["foo".to_string()]),
        (1, vec!["bar".to_string()]),
    ]);
}

#[test]
fn it_should_aggregate_positional_values_with_rest() {
    let mut cli_builder
        = CliBuilder::new();

    cli_builder.add_command(CommandSpec {
        components: vec![
            Component::Positional(PositionalSpec::required()),
            Component::Positional(PositionalSpec::rest()),
        ],
    });

    let (state, _context)
        = cli_builder.run(&["foo", "bar", "baz"]);

    assert_eq!(state.values, vec![
        (0, vec!["foo".to_string()]),
        (1, vec!["bar".to_string(), "baz".to_string()]),
    ]);
}
