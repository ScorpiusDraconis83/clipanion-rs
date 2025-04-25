use std::collections::HashMap;

use crate::{shared::Arg, transition::Transition};

#[derive(Debug, Clone)]
pub struct Node<'a, TCheck, TReducer> {
    pub context: usize,
    pub dynamics: Vec<(TCheck, Transition<TReducer>)>,
    pub shortcuts: Vec<Transition<TReducer>>,
    pub statics: HashMap<Arg<'a>, Vec<Transition<TReducer>>>,
}

impl<'a, TCheck, TReducer> Node<'a, TCheck, TReducer> {
    pub fn new() -> Self {
        Self {
            context: 0,
            dynamics: vec![],
            shortcuts: vec![],
            statics: HashMap::new(),
        }
    }
}

impl<'a, TCheck, TReducer> Default for Node<'a, TCheck, TReducer> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, TCheck, TReducer> Node<'a, TCheck, TReducer> {
    pub fn clone_to_offset(&self, offset: usize) -> Self where TCheck: Clone, TReducer: Clone {
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
