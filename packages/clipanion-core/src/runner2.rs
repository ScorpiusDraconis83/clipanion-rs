use std::fmt::Debug;

use crate::{shared::{Arg, ERROR_NODE_ID, INITIAL_NODE_ID}, transition::Transition, Machine};

pub trait RunnerState {
    fn get_context_id(&self) -> usize;
    fn set_context_id(&mut self, context_id: usize);

    fn get_node_id(&self) -> usize;
    fn set_node_id(&mut self, node_id: usize);
}

pub trait ValidateTransition<TState> {
    fn check(&self, state: &TState, arg: &str) -> bool;
}

impl<T, TState> ValidateTransition<TState> for Option<T> where T: ValidateTransition<TState> {
    fn check(&self, state: &TState, arg: &str) -> bool {
        self.as_ref().map_or(true, |reducer| reducer.check(state, arg))
    }
}

pub trait DeriveState<'a, TState> {
    fn derive(&self, state: &mut TState, target_id: usize, arg: &'a str) -> () where TState: RunnerState;
}

impl<'a, T, TState> DeriveState<'a, TState> for Option<T> where T: DeriveState<'a, TState> {
    fn derive(&self, state: &mut TState, target_id: usize, arg: &'a str) -> () where TState: RunnerState {
        if let Some(reducer) = self {
            reducer.derive(state, target_id, arg)
        }

        state.set_node_id(target_id);
    }
}

pub struct Runner<'machine, 'cmds, TCheck, TReducer, TState> {
    machine: &'machine Machine<'cmds, TCheck, TReducer>,

    states: Vec<TState>,
    next_states: Vec<TState>,

    // Colors are used to avoid infinite loops.
    node_colors: Vec<usize>,
    current_color: usize,
}

impl<'machine, 'cmds, TCheck, TReducer, TState> Runner<'machine, 'cmds, TCheck, TReducer, TState> {
    pub fn run<'args>(machine: &'machine Machine<'cmds, TCheck, TReducer>, args: &[&'args str]) -> Result<Vec<TState>, ()>
    where
        TCheck: ValidateTransition<TState>,
        TReducer: DeriveState<'args, TState> + Debug,
        TState: Clone + RunnerState,
        TState: Default + std::fmt::Debug
    {
        let mut runner
            = Runner::<'machine, 'cmds, TCheck, TReducer, TState>::new(machine);

        runner.update(Arg::StartOfInput);

        for state in runner.states.iter_mut() {
            let node
                = &runner.machine.nodes[state.get_node_id()];

            state.set_context_id(node.context);
        }

        for arg in args {
            runner.update(Arg::User(arg));
        }

        runner.update(Arg::EndOfInput);
        runner.digest()
    }

    pub fn new<'args>(machine: &'machine Machine<'cmds, TCheck, TReducer>) -> Self
    where
        TCheck: ValidateTransition<TState>,
        TReducer: DeriveState<'args, TState> + Debug,
        TState: Clone + RunnerState + Debug + Default
    {
        let mut runner = Runner {
            states: vec![],
            next_states: vec![],
            machine,
            node_colors: vec![0; machine.nodes.len()],
            current_color: 0,
        };

        let initial_state
            = TState::default();

        runner.next_states.push(initial_state.clone());

        let initial_node
            = runner.machine.nodes.get(INITIAL_NODE_ID)
                .unwrap();

        for shortcut in &initial_node.shortcuts {
            runner.transition_to(&initial_state, shortcut, Arg::StartOfInput);
        }

        std::mem::swap(
            &mut runner.states,
            &mut runner.next_states,
        );

        runner
    }

    fn transition_to<'args>(&mut self, from_state: &TState, transition: &Transition<TReducer>, token: Arg<'args>) -> () where TCheck: ValidateTransition<TState>, TReducer: DeriveState<'args, TState> + Debug, TState: Clone + RunnerState + Debug {
        self.current_color = self.current_color.wrapping_add(1);
        self.transition_to_color(from_state, transition, token, self.current_color);
    }

    fn transition_to_color<'args>(&mut self, from_state: &TState, transition: &Transition<TReducer>, token: Arg<'args>, color: usize) -> () where TCheck: ValidateTransition<TState>, TReducer: DeriveState<'args, TState> + Debug, TState: Clone + RunnerState + Debug {
        let mut next_state
            = from_state.clone();

        if let Arg::User(raw) = token {
            transition.reducer.derive(&mut next_state, transition.to, &raw);
        } else {
            next_state.set_node_id(transition.to);
        }

        self.node_colors[transition.to] = color;

        let target_node
            = &self.machine.nodes[transition.to];

        for shortcut in &target_node.shortcuts {
            if self.node_colors[shortcut.to] != color {
                self.transition_to_color(&next_state, shortcut, token, color);
            }
        }

        self.next_states.push(next_state);
    }

    pub fn update<'args>(&mut self, token: Arg<'args>) -> () where TCheck: ValidateTransition<TState>, TReducer: DeriveState<'args, TState> + Debug, TState: Clone + RunnerState + Debug {
        let mut states
            = std::mem::take(&mut self.states);

        for state in &states {
            let current_node
                = self.machine.nodes.get(state.get_node_id())
                    .unwrap();

            let transitions
                = current_node.statics.get(&token);

            let mut transitioned
                = false;

            if let Some(transitions) = transitions {
                for transition in transitions {
                    self.transition_to(state, transition, token);
                    transitioned = true;
                }
            }

            if let Arg::User(raw) = &token {
                for (check, transition) in &current_node.dynamics {
                    if check.check(state, raw) {
                        self.transition_to(state, transition, token);
                        transitioned = true;
                    }
                }
            }

            if !transitioned {
                let mut next_state = state.clone();
                next_state.set_node_id(ERROR_NODE_ID);
                self.next_states.push(next_state);
            }
        }

        self.next_states.retain(|state| {
            state.get_node_id() != ERROR_NODE_ID
        });

        if self.next_states.is_empty() {
            println!("no next states due to {:?}", token);
        }

        std::mem::swap(&mut self.states, &mut states);
        std::mem::swap(&mut self.states, &mut self.next_states);

        self.next_states.clear();
    }

    pub fn digest(self) -> Result<Vec<TState>, ()> {
        Ok(self.states)
    }
}
