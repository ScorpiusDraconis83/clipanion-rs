use std::fmt::Debug;

use crate::{shared::{Arg, INITIAL_NODE_ID}, transition::Transition, Machine};

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

pub trait DeriveState<TState> {
    fn derive(&self, state: &mut TState, target_id: usize, arg: &str) -> () where TState: RunnerState;
}

impl<T, TState> DeriveState<TState> for Option<T> where T: DeriveState<TState> {
    fn derive(&self, state: &mut TState, target_id: usize, arg: &str) -> () where TState: RunnerState {
        if let Some(reducer) = self {
            reducer.derive(state, target_id, arg)
        }

        state.set_node_id(target_id);
    }
}

pub struct Runner<'a, TCheck, TReducer, TState> {
    states: Vec<TState>,
    next_states: Vec<TState>,
    machine: &'a Machine<TCheck, TReducer>,

}

impl<'a, TCheck, TReducer, TState> Runner<'a, TCheck, TReducer, TState> {
    pub fn run<'b>(machine: &'a Machine<TCheck, TReducer>, it: &[&'b str]) -> Result<Vec<TState>, ()> where TCheck: ValidateTransition<TState>, TReducer: DeriveState<TState> + Debug, TState: Clone + RunnerState, TState: Default + std::fmt::Debug {
        let mut runner
            = Runner::<'a, TCheck, TReducer, TState>::new(machine);

        runner.update(Arg::StartOfInput);

        for state in runner.states.iter_mut() {
            let node
                = &runner.machine.nodes[state.get_node_id()];

            state.set_context_id(node.context);
        }

        for token in it.iter() {
            runner.update(Arg::User(token.to_string()));
        }

        runner.update(Arg::EndOfInput);
        runner.digest()
    }

    pub fn new(machine: &'a Machine<TCheck, TReducer>) -> Self where TCheck: ValidateTransition<TState>, TReducer: DeriveState<TState> + Debug, TState: Clone + RunnerState + Debug + Default {
        let mut runner = Runner {
            states: vec![],
            next_states: vec![],
            machine,
        };

        let initial_state
            = TState::default();

        runner.next_states.push(initial_state.clone());

        let initial_node
            = runner.machine.nodes.get(INITIAL_NODE_ID)
                .unwrap();

        for shortcut in &initial_node.shortcuts {
            runner.transition_to(&initial_state, shortcut, &Arg::StartOfInput);
        }

        std::mem::swap(
            &mut runner.states,
            &mut runner.next_states,
        );

        runner
    }

    fn transition_to(&mut self, from_state: &TState, transition: &Transition<TReducer>, token: &Arg) -> () where TCheck: ValidateTransition<TState>, TReducer: DeriveState<TState> + Debug, TState: Clone + RunnerState + Debug {
        let mut next_state
            = from_state.clone();

        if let Arg::User(raw) = token {
            transition.reducer.derive(&mut next_state, transition.to, &raw);
        } else {
            next_state.set_node_id(transition.to);
        }

        let target_node
            = &self.machine.nodes[transition.to];

        for shortcut in &target_node.shortcuts {
            self.transition_to(&next_state, shortcut, token);
        }

        self.next_states.push(next_state);
    }

    pub fn update(&mut self, token: Arg) -> () where TCheck: ValidateTransition<TState>, TReducer: DeriveState<TState> + Debug, TState: Clone + RunnerState + Debug {
        let mut states
            = std::mem::take(&mut self.states);

        for state in &states {
            let current_node
                = self.machine.nodes.get(state.get_node_id())
                    .unwrap();

            let transitions
                = current_node.statics.get(&token);

            if let Some(transitions) = transitions {
                for transition in transitions {
                    self.transition_to(state, transition, &token);
                }
            }

            if let Arg::User(raw) = &token {
                for (check, transition) in &current_node.dynamics {
                    if check.check(state, raw) {
                        self.transition_to(state, transition, &token);
                    }
                }
            }
        }

        std::mem::swap(&mut self.states, &mut states);
        std::mem::swap(&mut self.states, &mut self.next_states);

        self.next_states.clear();
    }

    pub fn digest(self) -> Result<Vec<TState>, ()> {
        Ok(self.states)
    }
}
