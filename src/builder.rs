use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{actions::{Check, Reducer}, errors::BuildError, machine::Machine, node::Node, runner::PartialRunState, shared::{Arg, ERROR_NODE_ID, HELP_COMMAND_INDEX, INITIAL_NODE_ID, SUCCESS_NODE_ID}};

pub struct CliBuilder {
    pub commands: Vec<CommandBuilder>,
}

impl CliBuilder {
    pub fn new() -> CliBuilder {
        CliBuilder {
            commands: vec![],
        }
    }

    pub fn add_command(&mut self) -> &mut CommandBuilder {
        let cli_index = self.commands.len();

        self.commands.push(CommandBuilder::new(cli_index));
        self.commands.last_mut().unwrap()
    }

    pub fn compile(&self) -> Machine {
        let mut machine = Machine::new_any_of(self.commands.iter().map(|command| command.compile()));
        machine.simplify_machine();

        machine
    }
}

pub struct Arity {
    leading: Vec<String>,
    optionals: Vec<String>,
    rest: Option<String>,
    trailing: Vec<String>,
    proxy: bool,
}

#[derive(Debug)]
pub struct OptionDefinition {
    pub name_set: Vec<String>,
    pub description: String,
    pub arity: usize,
    pub hidden: bool,
    pub required: bool,
    pub allow_binding: bool,
}

impl Default for OptionDefinition {
    fn default() -> Self {
        OptionDefinition {
            name_set: vec![],
            description: String::new(),
            arity: 0,
            hidden: false,
            required: false,
            allow_binding: true,
        }
    }
}

pub struct CommandBuilder {
    pub cli_index: usize,

    pub paths: Vec<Vec<String>>,

    pub options: BTreeMap<String, OptionDefinition>,
    pub preferred_names: HashMap<String, String>,
    pub required_options: Vec<String>,
    pub valid_bindings: HashSet<String>,


    pub arity: Arity,
}

pub struct CommandUsageOptions {
    pub detailed: bool,
    pub inline_options: bool,
}

pub struct OptionUsage {
    pub preferred_name: String,
    pub name_set: Vec<String>,
    pub definition: String,
    pub description: String,
    pub required: bool,
}

impl CommandBuilder {
    pub fn new(cli_index: usize) -> CommandBuilder {
        CommandBuilder {
            cli_index,

            paths: vec![],

            options: BTreeMap::new(),
            preferred_names: HashMap::new(),
            required_options: vec![],
            valid_bindings: HashSet::new(),

            arity: Arity {
                leading: vec![],
                optionals: vec![],
                rest: None,
                trailing: vec![],
                proxy: false,
            },
        }
    }

    pub fn usage(&self, opts: CommandUsageOptions) -> (String, Vec<OptionUsage>) {
        let mut segments = self.paths.first().cloned().unwrap_or_default();
        let mut detailed_option_list = vec![];

        if opts.detailed {
            for (preferred_name, option) in &self.options {
                if option.hidden {
                    continue;
                }

                let mut args = vec![];
                for t in 0..option.arity {
                    args.push(format!(" #{}", t));
                }

                let definition = format!("{}{}", option.name_set.join(", "), args.join(""));

                if !opts.inline_options && !option.description.is_empty() {
                    detailed_option_list.push(OptionUsage {
                        preferred_name: preferred_name.clone(),
                        name_set: option.name_set.clone(),
                        definition,
                        description: option.description.clone(),
                        required: option.required,
                    });
                } else {
                    segments.push(if option.required {
                        format!("<{}>", definition)
                    } else {
                        format!("[{}]", definition)
                    });
                }
            }
        }

        for name in &self.arity.leading {
            segments.push(format!("<{}>", name));
        }

        for name in &self.arity.optionals {
            segments.push(format!("[{}]", name));
        }

        if self.arity.rest.is_some() {
            segments.push(format!("[{}...]", self.arity.rest.as_ref().unwrap()));
        }

        for name in &self.arity.trailing {
            segments.push(format!("<{}>", name));
        }

        (segments.join(" "), detailed_option_list)
    }

    pub fn make_default(&mut self) -> &mut Self {
        self.paths.push(vec![]);
        self
    }

    pub fn add_path(&mut self, path: Vec<&str>) -> &mut Self {
        self.paths.push(path.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn add_positional(&mut self, required: bool, name: &str) -> Result<&mut Self, BuildError> {
        if !required {
            if self.arity.rest.is_some() {
                return Err(BuildError::OptionalParametersAfterRest);
            } else if self.arity.trailing.len() > 0 {
                return Err(BuildError::OptionalParametersAfterTrailingPositionals);
            } else {
                self.arity.optionals.push(name.to_string());
            }
        } else {
            if self.arity.optionals.len() > 0 || self.arity.rest.is_some() {
                self.arity.trailing.push(name.to_string());
            } else {
                self.arity.leading.push(name.to_string());
            }
        }

        Ok(self)
    }

    pub fn add_rest(&mut self, name: &str) -> Result<&mut Self, BuildError> {
        if self.arity.rest.is_some() {
            return Err(BuildError::MultipleRestParameters);
        } else if self.arity.trailing.len() > 0 {
            return Err(BuildError::RestAfterTrailingPositionals);
        } else {
            self.arity.rest = Some(name.to_string());
        }

        Ok(self)
    }

    pub fn add_proxy(&mut self, name: &str) -> Result<&mut Self, BuildError> {
        self.add_rest(name)?;
        self.arity.proxy = true;

        Ok(self)
    }

    pub fn add_option(&mut self, option: OptionDefinition) -> Result<&mut Self, BuildError> {
        if !option.allow_binding && option.arity > 1 {
            return Err(BuildError::ArityTooHighForNonBindingOption);
        }

        let preferred_name = option.name_set.iter()
            .max_by_key(|s| s.len()).unwrap()
            .clone();

        for name in &option.name_set {
            self.preferred_names.insert(name.to_string(), preferred_name.clone());
        }

        if option.allow_binding {
            self.valid_bindings.insert(preferred_name.clone());
        }

        if option.required {
            self.required_options.push(preferred_name.clone());
        }

        self.options.insert(preferred_name, option);

        Ok(self)
    }

    pub fn compile(&self) -> Machine {
        let mut machine = Machine::new();

        let context = &mut machine.contexts[0];

        context.command_index = self.cli_index;
        context.preferred_names = self.preferred_names.clone();
        context.valid_bindings = self.valid_bindings.clone();

        context.command_usage = self.usage(CommandUsageOptions {
            detailed: false,
            inline_options: true,
        }).0;

        let first_node_id = machine.inject_node(Node::new());

        machine.register_static(INITIAL_NODE_ID, Arg::StartOfInput, first_node_id, Reducer::SetRequiredOptions(self.required_options.clone()));

        let positional_argument = match self.arity.proxy {
            true => Check::Always,
            false => Check::IsNotOptionLike,
        };

        for path in &self.paths {
            let mut last_path_node_id = first_node_id;

            // We allow options to be specified before the path. Note that we
            // only do this when there is a path, otherwise there would be
            // some redundancy with the options attached later.
            if path.len() > 0 {
                let option_node_id = machine.inject_node(Node::new());
                machine.register_shortcut(last_path_node_id, option_node_id, Reducer::None);
                self.register_options(&mut machine, option_node_id);
                last_path_node_id = option_node_id;
            }

            for t in 0..path.len() {
                let next_path_node_id = machine.inject_node(Node::new());
                machine.register_static(last_path_node_id, Arg::User(path[t].clone()), next_path_node_id, Reducer::PushPath);
                last_path_node_id = next_path_node_id;

                if t + 1 < path.len() {
                    // Allow to pass `-h` (without anything after it) after each part of a path.
                    // Note that we do not do this for the last part, otherwise there would be
                    // some redundancy with the `useHelp` attached later.
                    let help_node_id = machine.inject_node(Node::new());
                    machine.register_dynamic(last_path_node_id, Check::IsHelp, help_node_id, Reducer::UseHelp);
                    machine.register_static(help_node_id, Arg::EndOfInput, SUCCESS_NODE_ID, Reducer::None);
                }
            }

            if self.arity.leading.len() > 0 || !self.arity.proxy {
                let help_node_id = machine.inject_node(Node::new());
                machine.register_dynamic(last_path_node_id, Check::IsHelp, help_node_id, Reducer::UseHelp);
                machine.register_dynamic(help_node_id, Check::Always, help_node_id, Reducer::PushOptional);
                machine.register_static(help_node_id, Arg::EndOfInput, SUCCESS_NODE_ID, Reducer::None);

                self.register_options(&mut machine, last_path_node_id);
            }

            if self.arity.leading.len() > 0 {
                machine.register_static(last_path_node_id, Arg::EndOfInput, ERROR_NODE_ID, Reducer::SetError("Not enough positional arguments".to_string()));
                machine.register_static(last_path_node_id, Arg::EndOfPartialInput, SUCCESS_NODE_ID, Reducer::SetSelectedIndex);
            }

            let mut last_leading_node_id = last_path_node_id;
            for t in 0..self.arity.leading.len() {
                let next_leading_node_id = machine.inject_node(Node::new());

                if !self.arity.proxy || t + 1 != self.arity.leading.len() {
                    self.register_options(&mut machine, next_leading_node_id);
                }

                if self.arity.trailing.len() > 0 || t + 1 != self.arity.leading.len() {
                    machine.register_static(next_leading_node_id, Arg::EndOfInput, ERROR_NODE_ID, Reducer::SetError("Not enough positional arguments".to_string()));
                    machine.register_static(next_leading_node_id, Arg::EndOfPartialInput, SUCCESS_NODE_ID, Reducer::SetSelectedIndex);
                }

                machine.register_dynamic(last_leading_node_id, Check::IsNotOptionLike, next_leading_node_id, Reducer::PushPositional);
                last_leading_node_id = next_leading_node_id;
            }

            let mut last_extra_node_id = last_leading_node_id;
            if self.arity.rest.is_some() || self.arity.optionals.len() > 0 {
                let extra_shortcut_node_id = machine.inject_node(Node::new());
                machine.register_shortcut(last_leading_node_id, extra_shortcut_node_id, Reducer::None);

                if self.arity.rest.is_some() {
                    let extra_node_id = machine.inject_node(Node::new());

                    if !self.arity.proxy {
                        self.register_options(&mut machine, extra_node_id);
                    }

                    machine.register_dynamic(last_leading_node_id, positional_argument.clone(), extra_node_id, Reducer::PushRest);
                    machine.register_dynamic(extra_node_id, positional_argument.clone(), extra_node_id, Reducer::PushRest);
                    machine.register_shortcut(extra_node_id, extra_shortcut_node_id, Reducer::None);
                } else {
                    for _ in 0..self.arity.optionals.len() {
                        let extra_node_id = machine.inject_node(Node::new());

                        if !self.arity.proxy {
                            self.register_options(&mut machine, extra_node_id);
                        }

                        machine.register_dynamic(last_extra_node_id, positional_argument.clone(), extra_node_id, Reducer::PushOptional);
                        machine.register_shortcut(extra_node_id, extra_shortcut_node_id, Reducer::None);
                        last_extra_node_id = extra_node_id;
                    }
                }

                last_extra_node_id = extra_shortcut_node_id;
            }

            if self.arity.trailing.len() > 0 {
                machine.register_static(last_extra_node_id, Arg::EndOfInput, ERROR_NODE_ID, Reducer::SetError("Not enough positional arguments".to_string()));
                machine.register_static(last_extra_node_id, Arg::EndOfPartialInput, SUCCESS_NODE_ID, Reducer::SetSelectedIndex);
            }

            let mut last_trailing_node_id = last_extra_node_id;
            for t in 0..self.arity.trailing.len() {
                let next_trailing_node_id = machine.inject_node(Node::new());

                if !self.arity.proxy {
                    self.register_options(&mut machine, next_trailing_node_id);
                }

                if t + 1 < self.arity.trailing.len() {
                    machine.register_static(next_trailing_node_id, Arg::EndOfInput, ERROR_NODE_ID, Reducer::SetError("Not enough positional arguments".to_string()));
                    machine.register_static(next_trailing_node_id, Arg::EndOfPartialInput, SUCCESS_NODE_ID, Reducer::SetSelectedIndex);
                }

                machine.register_dynamic(last_trailing_node_id, Check::IsNotOptionLike, next_trailing_node_id, Reducer::PushPositional);
                last_trailing_node_id = next_trailing_node_id;
            }

            machine.register_dynamic(last_trailing_node_id, positional_argument.clone(), ERROR_NODE_ID, Reducer::SetError("Extraneous positional argument".to_string()));
            machine.register_static(last_trailing_node_id, Arg::EndOfInput, SUCCESS_NODE_ID, Reducer::SetSelectedIndex);
            machine.register_static(last_trailing_node_id, Arg::EndOfPartialInput, SUCCESS_NODE_ID, Reducer::SetSelectedIndex);
        }

        machine
    }

    fn register_options(&self, machine: &mut Machine, node_id: usize) {
        machine.register_dynamic(node_id, Check::IsExactOption("--".to_string()), node_id, Reducer::InhibateOptions);
        machine.register_dynamic(node_id, Check::IsBatchOption, node_id, Reducer::PushBatch);
        machine.register_dynamic(node_id, Check::IsBoundOption, node_id, Reducer::PushBound);
        machine.register_dynamic(node_id, Check::IsUnsupportedOption, ERROR_NODE_ID, Reducer::SetError("Unsupported option name".to_string()));
        machine.register_dynamic(node_id, Check::IsInvalidOption, ERROR_NODE_ID, Reducer::SetError("Invalid option name".to_string()));

        for (preferred_name, option) in &self.options {
            if option.arity == 0 {
                for name in &option.name_set {
                    machine.register_dynamic(node_id, Check::IsExactOption(name.to_string()), node_id, Reducer::PushTrue(preferred_name.clone()));

                    if name.starts_with("--") && !name.starts_with("--no-") {
                        machine.register_dynamic(node_id, Check::IsNegatedOption(name.to_string()), node_id, Reducer::PushFalse(preferred_name.clone()));
                    }
                }
            } else {
                // We inject a new node at the end of the state machine
                let mut last_node_id = machine.inject_node(Node::new());

                // We register transitions from the starting node to this new node
                for name in &option.name_set {
                    machine.register_dynamic(node_id, Check::IsExactOption(name.to_string()), last_node_id, Reducer::PushNone(preferred_name.clone()));
                }

                // For each argument, we inject a new node at the end and we
                // register a transition from the current node to this new node
                for _ in 0..option.arity {
                    let next_node_id = machine.inject_node(Node::new());

                    // We can provide better errors when another option or EndOfInput is encountered
                    machine.register_static(last_node_id, Arg::EndOfInput, ERROR_NODE_ID, Reducer::SetOptionArityError);
                    machine.register_static(last_node_id, Arg::EndOfPartialInput, ERROR_NODE_ID, Reducer::SetOptionArityError);
                    machine.register_dynamic(last_node_id, Check::IsOptionLike, ERROR_NODE_ID, Reducer::SetOptionArityError);

                    // If the option has a single argument, no need to store it in an array
                    let action = match option.arity {
                        1 => Reducer::ResetStringValue,
                        _ => Reducer::AppendStringValue,
                    };

                    machine.register_dynamic(last_node_id, Check::IsNotOptionLike, next_node_id, action);

                    last_node_id = next_node_id;
                }

                // In the end, we register a shortcut from
                // the last node back to the starting node
                machine.register_shortcut(last_node_id, node_id, Reducer::None);
            }
        }
    }
}
