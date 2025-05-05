use std::collections::BTreeSet;

use crate::{shared::SUCCESS_NODE_ID, BuiltinCommand, CommandSpec, Error, State};

pub enum SelectionResult<'cmds, T> {
    Builtin(BuiltinCommand<'cmds>),
    Command(T, &'cmds CommandSpec),
}

#[derive(Debug, Clone)]
pub struct Selector<'cmds, 'args> {
    pub commands: Vec<&'cmds CommandSpec>,
    pub states: Vec<State<'args>>,
}

impl<'cmds, 'args> Selector<'cmds, 'args> {
    pub fn new(commands: Vec<&'cmds CommandSpec>, states: Vec<State<'args>>) -> Self {
        Self {
            commands,
            states,
        }
    }

    pub fn prune_unsuccessful_nodes(&mut self) -> Result<(), Error<'cmds>> {
        self.states.retain(|state| {
            state.node_id == SUCCESS_NODE_ID
        });

        Ok(())
    }

    pub fn prune_by_keyword_count(&mut self) -> Result<(), Error<'cmds>> {
        let max_keyword_count = self.states.iter()
            .map(|state| state.keyword_count)
            .max();

        if let Some(max_keyword_count) = max_keyword_count {
            self.states.retain(|state| {
                state.keyword_count == max_keyword_count
            });
        }

        Ok(())
    }

    pub fn prune_missing_required_options(&mut self) -> Result<(), Error<'cmds>> {
        self.states.retain(|state| {
            let command
                = self.commands[state.context_id];

            command.required_options.iter().all(|option_id| {
                state.option_values.iter().any(|(id, _)| *id == *option_id)
            })
        });

        Ok(())
    }

    /**
     * This function is used to favour options that are more greedy than others
     * from the same command.
     * 
     * For example, if we have "foo bar baz" on a command "[arg] [...rest]", we
     * are going to have the two following options:
     * 
     * - arg = Some(foo), rest = vec!["bar", "baz"]
     * - arg = None, rest = vec!["foo", "bar", "baz"]
     * 
     * The first option is more greedy, so we remove the second one.
     */
    pub fn prune_by_greediness(&mut self) -> Result<(), Error<'cmds>> {
        let mut states = vec![];
        std::mem::swap(&mut self.states, &mut states);

        // First we convert the positional values into a list of sort
        // criteria: first by positional index, then by number of values
        // provided to the positional argument.
        //
        // - vec![(0, 1), (1, 2)] // for the first option
        // - vec![(1, 3)] // for the second option
        //
        // We have a small problem: we want to first favour the lowest
        // positional indexes (since they signal higher greediness), but
        // we want to then favour the HIGHEST number of values provided,
        // for the same reason.
        //
        // To do this, we apply `-x-1` on the positional index which, since
        // we're dealing with unsigned integers, ensures that the lowest
        // indexes are now the highest values, and allowing us to use the
        // default `max_by` function to sort on both indexes and value count.
        //
        let mut states_with_positional_tracks = states.into_iter().map(|state| {
            let positional_track = state.positional_values.iter().map(|(idx, values)| {
                (idx.wrapping_neg().wrapping_sub(1), values.len())
            }).collect::<Vec<_>>();

            (state, positional_track)
        }).collect::<Vec<_>>();

        states_with_positional_tracks.sort_by(|a, b| {
            b.1.first().unwrap().cmp(a.1.first().unwrap())
        });

        // We're now going to remove all the entries except for the first
        // one for each different command.
        let mut seen
            = BTreeSet::new();

        states_with_positional_tracks.retain(|(state, _)| {
            seen.insert(state.context_id)
        });

        self.states = states_with_positional_tracks.into_iter()
            .map(|(state, _)| state)
            .collect();

        Ok(())
    }

    pub fn get_best_hydrated_state<T>(&self, mut hydration_results: Vec<Result<T, Error<'args>>>) -> Result<SelectionResult<'cmds, T>, Error<'args>> {
        if self.states.is_empty() {
            return Err(Error::NotFound(vec![]));
        }

        let hydrated_states = self.states.iter()
            .enumerate()
            .filter(|(index, _)| hydration_results[*index].is_ok())
            .collect::<Vec<_>>();

        if hydrated_states.len() != 1 {
            if self.states.len() == 1 {
                let only_error = match hydration_results.remove(0) {
                    Ok(_) => unreachable!(),
                    Err(error) => error,
                };

                return Err(only_error);
            } else {
                return Err(Error::AmbiguousSyntax(vec![]));
            }
        }

        let (index, state)
            = hydrated_states.first().unwrap();

        let command_spec
            = self.commands[state.context_id];

        let hydration_result
            = hydration_results.remove(*index).unwrap();

        Ok(SelectionResult::Command(hydration_result, command_spec))
    }

    pub fn get_best_state(&self) -> Option<(&State<'args>, &'cmds CommandSpec)> {
        if self.states.len() != 1 {
            return None;
        }

        let state
            = self.states.first().unwrap();

        let command_spec
            = self.commands[state.context_id];

        Some((state, command_spec))
    }
}
