use std::fmt::Debug;

use crate::{shared::{Arg, UserArg, ERROR_NODE_ID, INITIAL_NODE_ID}, transition::Transition, Machine};

pub trait RunnerState {
    fn get_context_id(&self) -> usize;
    fn set_context_id(&mut self, context_id: usize);

    fn get_node_id(&self) -> usize;
    fn set_node_id(&mut self, node_id: usize);

    fn get_keyword_count(&self) -> usize;
}

pub trait ValidateTransition<'args, TState> {
    fn check(&self, state: &TState, arg: &'args str) -> bool;
}

impl<'args, T, TState> ValidateTransition<'args, TState> for Option<T> where T: ValidateTransition<'args, TState> {
    fn check(&self, state: &TState, arg: &'args str) -> bool {
        self.as_ref().map_or(true, |reducer| reducer.check(state, arg))
    }
}

pub trait DeriveState<'args, TState> {
    fn derive(&self, state: &mut TState, target_id: usize, arg: Arg<'args>) -> () where TState: RunnerState;
}

impl<'args, T, TState> DeriveState<'args, TState> for Option<T> where T: DeriveState<'args, TState> {
    fn derive(&self, state: &mut TState, target_id: usize, arg: Arg<'args>) -> () where TState: RunnerState {
        if let Some(reducer) = self {
            reducer.derive(state, target_id, arg)
        }

        state.set_node_id(target_id);
    }
}

pub struct Runner<'machine, 'cmds, TCheck, TReducer, TFallback, TState> {
    machine: &'machine Machine<'cmds, TCheck, TReducer>,
    fallback: TFallback,

    states: Vec<TState>,
    next_states: Vec<TState>,

    // Colors are used to avoid infinite loops.
    node_colors: Vec<usize>,
    current_color: usize,
}

impl<'machine, 'cmds, TCheck, TReducer, TFallback, TState> Runner<'machine, 'cmds, TCheck, TReducer, TFallback, TState> {
    /**
     * Run the state machine with the given arguments.
     */
    pub fn run<'args>(machine: &'machine Machine<'cmds, TCheck, TReducer>, fallback: TFallback, args: &[&'args str]) -> Vec<TState>
    where
        TCheck: ValidateTransition<'args, TState>,
        TReducer: DeriveState<'args, TState> + Debug,
        TFallback: Fn(TState, Arg<'args>) -> TState,
        TState: Clone + RunnerState + Debug + Default
    {
        let mut runner
            = Runner::<'machine, 'cmds, TCheck, TReducer, TFallback, TState>::new(machine, fallback);

        runner.send(args);

        runner.update(Arg::EndOfInput);
        runner.digest()
    }

    /**
     * Run the state machine with the given arguments. Unlike `run`, this method will mark
     * all states that are not in an error state as successful. This can be useful when you
     * want to obtain information about a partial command line (for example in a documentation
     * you often want to reference a command but not provide all the arguments).
     */
    pub fn run_partial<'args>(machine: &'machine Machine<'cmds, TCheck, TReducer>, fallback: TFallback, args: &[&'args str]) -> Vec<TState>
    where
        TCheck: ValidateTransition<'args, TState>,
        TReducer: DeriveState<'args, TState> + Debug,
        TFallback: Fn(TState, Arg<'args>) -> TState,
        TState: Clone + RunnerState + Debug + Default
    {
        let mut runner
            = Runner::<'machine, 'cmds, TCheck, TReducer, TFallback, TState>::new(machine, fallback);

        runner.send(args);

        runner.states.retain(|state| {
            state.get_node_id() != ERROR_NODE_ID
        });

        runner.digest()
    }

    pub fn new<'args>(machine: &'machine Machine<'cmds, TCheck, TReducer>, fallback: TFallback) -> Self
    where
        TCheck: ValidateTransition<'args, TState>,
        TReducer: DeriveState<'args, TState> + Debug,
        TState: Clone + RunnerState + Debug + Default
    {
        let mut runner = Runner {
            states: vec![],
            next_states: vec![],
            machine,
            fallback,
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

        runner.states = runner.next_states.into_iter().collect();
        runner.next_states = vec![];

        runner
    }

    fn send<'args>(&mut self, args: &[&'args str])
    where
        TCheck: ValidateTransition<'args, TState>,
        TReducer: DeriveState<'args, TState> + Debug,
        TFallback: Fn(TState, Arg<'args>) -> TState,
        TState: Clone + RunnerState,
        TState: Default + std::fmt::Debug
    {
        if std::env::var("CLIPANION_DEBUG").is_ok() {
            println!("========== Parsing ==========");
        }

        self.update(Arg::StartOfInput);

        for state in self.states.iter_mut() {
            let node
                = &self.machine.nodes[state.get_node_id()];

            state.set_context_id(node.context);
        }

        for (arg_index, arg) in args.iter().enumerate() {
            self.update(Arg::User(UserArg { value: arg, index: arg_index }));
        }
    }

    fn transition_to<'args>(&mut self, from_state: &TState, transition: &Transition<TReducer>, token: Arg<'args>) -> ()
    where
        TCheck: ValidateTransition<'args, TState>,
        TReducer: DeriveState<'args, TState> + Debug,
        TState: Clone + RunnerState + Debug
    {
        self.current_color = self.current_color.wrapping_add(1);
        self.transition_to_color(from_state, transition, token, self.current_color);
    }

    fn transition_to_color<'args>(&mut self, from_state: &TState, transition: &Transition<TReducer>, token: Arg<'args>, color: usize) -> ()
    where
        TCheck: ValidateTransition<'args, TState>,
        TReducer: DeriveState<'args, TState> + Debug,
        TState: Clone + RunnerState + Debug
    {
        let mut next_state
            = from_state.clone();

        transition.reducer.derive(&mut next_state, transition.to, token);

        if std::env::var("CLIPANION_DEBUG").is_ok() {
            println!("  [{}] {} -> {} (reducer: {:?})", from_state.get_context_id(), from_state.get_node_id(), transition.to, transition.reducer);
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

    pub fn update<'args>(&mut self, token: Arg<'args>) -> ()
    where
        TCheck: ValidateTransition<'args, TState>,
        TReducer: DeriveState<'args, TState> + Debug,
        TFallback: Fn(TState, Arg<'args>) -> TState,
        TState: Clone + RunnerState + Debug
    {
        if std::env::var("CLIPANION_DEBUG").is_ok() {
            println!("{:?}", token);
        }

        let states
            = std::mem::take(&mut self.states);

        for state in states {
            if state.get_node_id() == ERROR_NODE_ID {
                self.next_states.push(state);
                continue;
            }

            let current_node
                = self.machine.nodes.get(state.get_node_id())
                    .unwrap();

            let mut transitioned
                = false;

            let transitions
                = current_node.statics.get(&token.into());

            if let Some(transitions) = transitions {
                for transition in transitions {
                    self.transition_to(&state, transition, token);
                    transitioned = true;
                }
            }

            if let Arg::User(UserArg { value: raw, .. }) = &token {
                for (check, transition) in &current_node.dynamics {
                    if check.check(&state, raw) {
                        self.transition_to(&state, transition, token);
                        transitioned = true;
                    }
                }
            }

            if !transitioned {
                self.next_states.push((self.fallback)(state, token));
            }
        }

        let next_states
            = std::mem::take(&mut self.next_states);

        self.states
            = next_states;

        self.trim_shortest_branches();
    }

    fn trim_shortest_branches(&mut self) -> ()
    where
        TState: RunnerState
    {
        let max_keyword_count
            = self.states.iter()
                .map(|state| state.get_keyword_count())
                .max();

        if let Some(max_keyword_count) = max_keyword_count {
            self.states.retain(|state| state.get_keyword_count() == max_keyword_count);
        }
    }

    pub fn digest(self) -> Vec<TState> {
        self.states
    }
}
