use std::collections::HashSet;

use crate::{actions::{apply_check, apply_reducer}, errors::Error, machine::{Machine, MachineContext}, shared::{Arg, ERROR_NODE_ID, HELP_COMMAND_INDEX, INITIAL_NODE_ID}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Path {
        segment_index: usize,
    },

    Positional {
        segment_index: usize,
    },

    Option {
        segment_index: usize,
        slice: Option<(usize, usize)>,
        option: String,
    },

    Assign {
        segment_index: usize,
        slice: (usize, usize),
    },

    Value {
        segment_index: usize,
        slice: Option<(usize, usize)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptionValue {
    None,
    Array(Vec<String>),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Positional {
    Required(String),
    Optional(String),
    Rest(String),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RunState {
    pub candidate_index: usize,
    pub required_options: Vec<String>,
    pub error_message: Option<Error>,
    pub ignore_options: bool,
    pub is_help: bool,
    pub options: Vec<(String, OptionValue)>,
    pub path: Vec<String>,
    pub positionals: Vec<Positional>,
    pub remainder: Option<String>,
    pub selected_index: Option<usize>,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunBranch {
    node_id: usize,
    state: RunState,
}

impl RunBranch {
    pub fn apply_transition(&self, transition: &crate::transition::Transition, context: &MachineContext, segment: &Arg, segment_index: usize) -> RunBranch {
        RunBranch {
            node_id: transition.to,
            state: apply_reducer(&transition.reducer, context, &self.state, segment, segment_index),
        }
    }
}

fn trim_smaller_branches(branches: &mut Vec<RunBranch>) {
    let max_path_size = branches.iter()
        .map(|b| b.state.path.len())
        .max()
        .unwrap();

    branches.retain(|b| b.state.path.len() == max_path_size);
}

fn select_best_state(_input: &Vec<String>, mut states: Vec<RunState>) -> Result<RunState, Error> {
    states.retain(|s| {
        s.selected_index.is_some()
    });

    if states.is_empty() {
        panic!("No terminal states found");
    }

    states.retain(|s| {
        s.selected_index == Some(HELP_COMMAND_INDEX) || s.required_options.iter().all(|o| s.options.iter().any(|(name, _)| name == o))
    });

    if states.is_empty() {
        return Err(Error::InternalError);
    }

    let max_path_size = states.iter()
        .map(|s| s.path.len())
        .max()
        .unwrap();

    states.retain(|s| {
        s.path.len() == max_path_size
    });

    fn get_fill_score(state: &RunState) -> usize {
        let option_scope = state.options.len();

        let positional_score = state.positionals.iter()
            .filter(|mode| match mode { Positional::Required(_) => true, _ => false })
            .count();

        option_scope + positional_score
    }

    let best_fill_score = states.iter()
        .map(|s| get_fill_score(s))
        .max()
        .unwrap();

    states.retain(|s| {
        get_fill_score(s) == best_fill_score
    });

    let mut aggregated_states
        = aggregate_help_states(states.into_iter());

    if aggregated_states.len() > 1 {
        let candidate_commands = aggregated_states.iter()
            .map(|s| s.selected_index.unwrap())
            .collect::<Vec<_>>();

        return Err(Error::AmbiguousSyntax(candidate_commands));
    }

    Ok(std::mem::take(aggregated_states.first_mut().unwrap()))
}

fn find_common_prefix<'t, I>(mut it: I) -> Vec<String> where I: Iterator<Item = &'t Vec<String>> {
    let mut common_prefix
        = it.next().unwrap().clone();

    while let Some(path) = it.next() {
        if path.len() < common_prefix.len() {
            common_prefix.resize(path.len(), Default::default());
        }

        let diff = common_prefix.iter()
            .zip(path.iter())
            .position(|(a, b)| a != b);

        if let Some(diff) = diff {
            common_prefix.resize(diff, Default::default());
        }
    }

    common_prefix
}

fn aggregate_help_states<I>(it: I) -> Vec<RunState> where I: Iterator<Item = RunState> {
    let (helps, mut not_helps)
        = it.partition::<Vec<_>, _>(|s| s.selected_index == Some(HELP_COMMAND_INDEX));

    if !helps.is_empty() {
        let options = helps.iter()
            .flat_map(|s| s.options.iter())
            .cloned()
            .collect();

        not_helps.push(RunState {
            selected_index: Some(HELP_COMMAND_INDEX),
            path: find_common_prefix(helps.iter().map(|s| &s.path)),
            options,
            ..Default::default()
        });
    }

    not_helps
}

fn extract_error_from_branches(_input: &Vec<String>, mut branches: Vec<RunBranch>, is_next: bool) -> Error {
    if is_next {
        if let Some(lead) = branches.pop() {
            if let Some(Error::CommandError(usize, command_error)) = lead.state.error_message {
                if branches.iter().all(|b| match &b.state.error_message {
                    Some(Error::CommandError(_, command_error)) => command_error == command_error,
                    _ => false,
                }) {
                    return Error::CommandError(usize, command_error);
                }
            }
        }
    }

    let candidate_indices = branches.iter()
        .filter(|b| b.node_id != ERROR_NODE_ID)
        .map(|b| b.state.candidate_index)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    Error::NotFound(candidate_indices)
}

fn run_machine_internal(machine: &Machine, input: &Vec<String>, partial: bool) -> Result<Vec<RunBranch>, Error> {
    let mut args = vec![Arg::StartOfInput];

    args.extend(input.iter().map(|s| {
        Arg::User(s.to_string())
    }));

    args.push(match partial {
        true => Arg::EndOfPartialInput,
        false => Arg::EndOfInput,
    });

    let mut branches = vec![RunBranch {
        node_id: INITIAL_NODE_ID,
        state: RunState::default(),
    }];

    for (t, arg) in args.iter().enumerate() {
        let is_eoi = arg == &Arg::EndOfInput || arg == &Arg::EndOfPartialInput;
        let mut next_branches = vec![];

        for branch in &branches {
            if branch.node_id == ERROR_NODE_ID {
                next_branches.push(branch.clone());
                continue;
            }

            let node = &machine.nodes[branch.node_id];
            let context = &machine.contexts[node.context];

            let has_exact_match = node.statics.contains_key(arg);
            if !partial || t < args.len() - 1 || has_exact_match {
                if has_exact_match {
                    for transition in &node.statics[arg] {
                        next_branches.push(branch.apply_transition(transition, context, arg, t.wrapping_sub(1)));
                    }
                }
            } else {
                for candidate in machine.nodes[branch.node_id].statics.keys() {
                    if !candidate.starts_with(arg) {
                        continue;
                    }

                    for transition in &node.statics[candidate] {
                        next_branches.push(branch.apply_transition(transition, context, arg, t - 1));
                    }
                }
            }

            if !is_eoi {
                for (check, transition) in &node.dynamics {
                    if apply_check(check, context, &branch.state, &arg, t - 1) {
                        next_branches.push(branch.apply_transition(transition, context, arg, t - 1));
                    }
                }
            }
        }

        if next_branches.is_empty() && is_eoi && input.len() == 1 {
            return Ok(vec![RunBranch {
                node_id: INITIAL_NODE_ID,
                state: RunState {
                    selected_index: Some(HELP_COMMAND_INDEX),
                    ..RunState::default()
                },
            }]);
        }

        if next_branches.is_empty() {
            return Err(extract_error_from_branches(input, branches, false));
        }

        if next_branches.iter().all(|b| b.node_id == ERROR_NODE_ID) {
            return Err(extract_error_from_branches(input, next_branches, true));
        }

        branches = next_branches;
        trim_smaller_branches(&mut branches);
    }

    Ok(branches)
}

pub fn run_machine(machine: &Machine, input: &Vec<String>) -> Result<RunState, Error> {
    let branches = run_machine_internal(machine, input, false)?;

    let states = branches.into_iter()
        .map(|b| b.state)
        .collect();

    select_best_state(input, states)
}

pub fn run_partial_machine(machine: &Machine, input: &Vec<String>) -> Result<RunState, Error> {
    let branches = run_machine_internal(machine, input, true)?;

    let states = branches.into_iter()
        .map(|b| b.state)
        .collect();

    select_best_state(input, states)
}
