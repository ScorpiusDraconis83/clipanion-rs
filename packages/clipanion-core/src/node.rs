use std::collections::HashMap;

use crate::{actions::Check, shared::Arg, transition::Transition};

#[derive(Debug, Default, Clone)]
pub struct Node {
    pub context: usize,
    pub dynamics: Vec<(Check, Transition)>,
    pub shortcuts: Vec<Transition>,
    pub statics: HashMap<Arg, Vec<Transition>>,
}

impl Node {
    pub fn new() -> Node {
        Default::default()
    }

    pub fn clone_to_offset(&self, offset: usize) -> Node {
        let mut out = Node::new();

        for (check, transition) in self.dynamics.iter() {
            out.dynamics.push((check.clone(), transition.clone_to_offset(offset)));
        }

        for transition in self.shortcuts.iter() {
            out.shortcuts.push(transition.clone_to_offset(offset));
        }

        for (key, transitions) in self.statics.iter() {
            out.statics.insert(key.clone(), transitions.iter().map(|t| t.clone_to_offset(offset)).collect::<Vec<_>>());
        }

        out
    }
}
