use std::collections::BTreeSet;

use itertools::Itertools;

use crate::{shared::SUCCESS_NODE_ID, BuiltinCommand, CommandError, CommandSpec, Component, Error, State};

pub enum SelectionResult<'cmds, 'args, T> {
    Builtin(BuiltinCommand<'cmds>),
    Command(&'cmds CommandSpec, State<'args>, T),
}

#[derive(Debug, Clone)]
pub struct Selector<'cmds, 'args> {
    pub commands: Vec<&'cmds CommandSpec>,
    pub states: Vec<State<'args>>,
    pub candidates: Vec<usize>,
}

impl<'cmds, 'args> Selector<'cmds, 'args> {
    pub fn new(commands: Vec<&'cmds CommandSpec>, states: Vec<State<'args>>) -> Self {
        let candidates = (0..states.len()).collect();

        Self {
            commands,
            states,
            candidates,
        }
    }

    fn prune_unsuccessful_nodes<'a>(&mut self) -> Result<(), Error<'cmds>> {
        let owned_candidates
            = std::mem::take(&mut self.candidates);

        let successful_candidates
            = owned_candidates.into_iter()
                .filter(|id| self.states[*id].node_id == SUCCESS_NODE_ID)
                .collect::<Vec<_>>();

        if successful_candidates.len() == 0 {
            return self.handle_everything_is_an_error();
        }
    
        self.candidates = successful_candidates;
        Ok(())
    }

    fn prune_by_keyword_count<'a>(&mut self) {
        let max_keyword_count = self.states.iter()
            .map(|state| state.keyword_count)
            .max();

        if let Some(max_keyword_count) = max_keyword_count {
            self.candidates.retain(|id| {
                self.states[*id].keyword_count == max_keyword_count
            });
        }
    }

    fn prune_missing_required_options<'a>(&mut self) -> Result<(), Error<'cmds>> {
        let owned_candidates
            = std::mem::take(&mut self.candidates);

        let (successful_candidates, unsuccessful_candidates)
            = owned_candidates.into_iter()
                .map(|id| {
                    let state
                        = &self.states[id];

                    let command
                        = self.commands[state.context_id];

                    let missing_options = command.required_options.iter().filter(|option_id| {
                        !state.option_values.iter().any(|(id, _)| *id == **option_id)
                    }).cloned().collect::<Vec<_>>();

                    (id, missing_options)
                }).partition::<Vec<_>, _>(|(_, required_options)| {
                    required_options.len() == 0
                });

        if successful_candidates.len() == 0 {
            if unsuccessful_candidates.len() == 1 {
                let (id, missing_option_indexes)
                    = unsuccessful_candidates.first().unwrap();

                let command_spec
                    = self.commands[*id];

                let missing_options = missing_option_indexes.iter()
                    .map(|index| &command_spec.components[*index])
                    .flat_map(|component| if let Component::Option(option) = component {Some(option.primary_name.clone())} else {None})
                    .collect::<Vec<_>>()
                    .join(", ");

                return Err(Error::CommandError(command_spec, CommandError::MissingOptionArguments(missing_options)));
            } else {
                let command_specs = unsuccessful_candidates.into_iter()
                    .map(|(id, _)| self.commands[id])
                    .collect::<Vec<_>>();

                return Err(Error::AmbiguousSyntax(command_specs));
            }
        }

        self.candidates = successful_candidates.into_iter()
            .map(|(id, _)| id)
            .collect();

        Ok(())
    }

    fn prune_by_hydration_results(&mut self, mut hydration_errors: Vec<(usize, CommandError)>) -> Result<(), Error<'cmds>> {
        let mut failed_hydrations
            = vec![false; self.states.len()];

        for (id, _) in hydration_errors.iter() {
            failed_hydrations[*id] = true;
        }

        self.candidates.retain(|id| {
            !failed_hydrations[*id]
        });

        if self.candidates.len() == 0 {
            if hydration_errors.len() == 1 {
                let (id, error)
                    = hydration_errors.remove(0);

                let command_spec
                    = self.commands[id];

                return Err(Error::CommandError(command_spec, error));
            } else {
                let suggested_commands = hydration_errors.into_iter()
                    .map(|(id, _)| self.commands[id])
                    .collect::<Vec<_>>();

                return Err(Error::NotFound(suggested_commands));
            }
        }

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
    fn prune_by_greediness<'a>(&mut self) {
        let owned_candidates
            = std::mem::take(&mut self.candidates);

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
        let mut states_with_positional_tracks = owned_candidates.into_iter().map(|id| {
            let positional_track = self.states[id].positional_values.iter().map(|(idx, values)| {
                (idx.wrapping_neg().wrapping_sub(1), values.len())
            }).collect::<Vec<_>>();

            (id, positional_track)
        }).collect::<Vec<_>>();

        states_with_positional_tracks.sort_by(|a, b| {
            b.1.first().unwrap().cmp(a.1.first().unwrap())
        });

        // We're now going to remove all the entries except for the first
        // one for each different command.
        let mut seen
            = vec![false; self.commands.len()];

        states_with_positional_tracks.retain(|(id, _)| {
            let context_id = self.states[*id].context_id;

            if seen[context_id] {
                false
            } else {
                seen[context_id] = true;
                true
            }
        });

        self.candidates = states_with_positional_tracks.into_iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
    }

    fn handle_everything_is_an_error(&mut self) -> Result<(), Error<'cmds>> {
        self.candidates = (0..self.states.len()).collect();

        self.prune_by_keyword_count();
        self.prune_by_greediness();

        let owned_candidates
            = std::mem::take(&mut self.candidates);

        let context_ids = owned_candidates.into_iter()
            .map(|id| self.states[id].context_id)
            .collect::<BTreeSet<_>>();

        let commands = context_ids.into_iter()
            .map(|id| self.commands[id])
            .collect::<Vec<_>>();

        Err(Error::NotFound(commands))
    }

    pub fn resolve_state<F: Fn(&State<'args>) -> Result<T, CommandError>, T>(&mut self, f: F) -> Result<SelectionResult<'cmds, 'args, T>, Error<'cmds>> {
        // println!("{:#?}", self.states);

        let help_contexts = self.states.iter()
            .filter(|state| state.is_help)
            .map(|state| state.context_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|id| self.commands[id])
            .collect::<Vec<_>>();

        if help_contexts.len() > 0 {
            return Ok(SelectionResult::Builtin(BuiltinCommand::Help(help_contexts)));
        }

        self.prune_unsuccessful_nodes()?;
        self.prune_missing_required_options()?;

        let hydration_results = self.candidates.iter()
            .map(|id| match f(&self.states[*id]) {
                Ok(result) => Ok((*id, result)),
                Err(err) => Err((*id, err)),
            })
            .collect::<Vec<_>>();

        let (successful_hydrations, unsuccessful_hydrations): (Vec<_>, Vec<_>)
            = hydration_results.into_iter()
                .partition_result();

        self.prune_by_hydration_results(unsuccessful_hydrations)?;
        self.prune_by_keyword_count();
        self.prune_by_greediness();

        let owned_candidates
            = std::mem::take(&mut self.candidates);

        if owned_candidates.len() > 1 {
            let context_ids = owned_candidates.into_iter()
                .map(|id| self.states[id].context_id)
                .collect::<BTreeSet<_>>();

            let commands = context_ids.into_iter()
                .map(|id| self.commands[id])
                .collect::<Vec<_>>();

            return Err(Error::AmbiguousSyntax(commands));
        }

        let index
            = owned_candidates.first().unwrap();

        let state
            = self.states.swap_remove(*index);
        let command_spec
            = self.commands[state.context_id];

        let (_, hydration_result)
            = successful_hydrations.into_iter()
                .find(|(id, _)| *id == *index)
                .unwrap();

        let res
            = SelectionResult::Command(command_spec, state, hydration_result);

        Ok(res)
    }
}
