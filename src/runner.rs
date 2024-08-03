use partially::Partial;

use crate::{actions::{apply_check, apply_reducer}, errors::Error, machine::Machine, shared::{Arg, ERROR_NODE_ID, HELP_COMMAND_INDEX, INITIAL_NODE_ID}};

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

#[derive(Debug, Default, Clone, PartialEq, Eq, Partial)]
#[partially(derive(Debug, Default, Clone, PartialEq, Eq))]
pub struct RunState {
    pub candidate_usage: String,
    pub required_options: Vec<Vec<String>>,
    pub error_message: String,
    pub ignore_options: bool,
    pub options: Vec<(String, OptionValue)>,
    pub path: Vec<String>,
    pub positionals: Vec<Positional>,
    pub remainder: Option<String>,
    pub selected_index: Option<isize>,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunBranch {
    node_id: usize,
    state: RunState,
}

impl RunBranch {
    pub fn apply_transition(&self, transition: &crate::transition::Transition, segment: &Arg, segment_index: usize) -> RunBranch {
        RunBranch {
            node_id: transition.to,
            state: apply_reducer(&transition.reducer, &self.state, segment, segment_index),
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

fn select_best_state(input: &Vec<&str>, mut states: Vec<RunState>) -> Result<RunState, Error> {
    states.retain(|s| {
        s.selected_index.is_some()
    });

    if states.is_empty() {
        panic!("No terminal states found");
    }

    states.retain(|s| s.selected_index == Some(HELP_COMMAND_INDEX) || s.required_options.iter().all(|names| {
        names.iter().any(|name| s.options.iter().any(|(k, _)| k == name))
    }));

    if states.is_empty() {
        return Err(Error::UnknownSyntax("Internal error".to_string()));
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
        println!("{:?} {}", s, get_fill_score(s));
        get_fill_score(s) == best_fill_score
    });

    let mut aggregated_states
        = aggregate_help_states(states.into_iter());

    if aggregated_states.len() > 1 {
        return Err(Error::AmbiguousSyntax);
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

fn extract_error_from_branches(input: &Vec<&str>, branches: &Vec<RunBranch>, is_next: bool) -> Error {
    if branches.len() == 0 {
        return Error::UnknownSyntax("Command not found, but we're not sure what's the alternative.".to_string());
    }

    let lead = branches.first().unwrap();

    if is_next && branches.iter().all(|b| b.state.error_message == lead.state.error_message) {
        return Error::UnknownSyntax(format!("{}\n\n{}", &lead.state.error_message, branches.iter().map(|b| format!("$ {}", b.state.candidate_usage)).collect::<Vec<_>>().join("\n")));
    }

    if branches.len() == 1 {
        return Error::UnknownSyntax(format!("Command not found; did you mean:\n\n$ {}", lead.state.candidate_usage));
    }

    return Error::UnknownSyntax(format!("Command not found; did you mean one of:\n\n{}", branches.iter().map(|b| format!("$ {}", b.state.candidate_usage)).collect::<Vec<_>>().join("\n")));
}

fn run_machine_internal(machine: &Machine, input: &Vec<&str>, partial: bool) -> Result<Vec<RunBranch>, Error> {
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

            let has_exact_match = machine.nodes[branch.node_id].statics.contains_key(arg);
            if !partial || t < args.len() - 1 || has_exact_match {
                if has_exact_match {
                    for transition in &machine.nodes[branch.node_id].statics[arg] {
                        next_branches.push(branch.apply_transition(transition, arg, t.wrapping_sub(1)));
                    }
                }
            } else {
                for candidate in machine.nodes[branch.node_id].statics.keys() {
                    if !candidate.starts_with(arg) {
                        continue;
                    }

                    for transition in &machine.nodes[branch.node_id].statics[candidate] {
                        next_branches.push(branch.apply_transition(transition, arg, t - 1));
                    }
                }
            }

            if !is_eoi {
                for (check, transition) in &machine.nodes[branch.node_id].dynamics {
                    if apply_check(check, &branch.state, &arg, t - 1) {
                        next_branches.push(branch.apply_transition(transition, arg, t - 1));
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
            return Err(extract_error_from_branches(input, &branches, false));
        }

        if next_branches.iter().all(|b| b.node_id == ERROR_NODE_ID) {
            return Err(extract_error_from_branches(input, &next_branches, true));
        }

        branches = next_branches;
        trim_smaller_branches(&mut branches);
    }

    Ok(branches)
}

pub fn run_machine(machine: &Machine, input: &Vec<&str>) -> Result<RunState, Error> {
    let branches = run_machine_internal(machine, input, false)?;

    let states = branches.into_iter()
        .map(|b| b.state)
        .collect();

    select_best_state(input, states)
}

pub fn run_partial_machine(machine: &Machine, input: &Vec<&str>) -> Result<RunState, Error> {
    let branches = run_machine_internal(machine, input, true)?;

    let states = branches.into_iter()
        .map(|b| b.state)
        .collect();

    select_best_state(input, states)
}
