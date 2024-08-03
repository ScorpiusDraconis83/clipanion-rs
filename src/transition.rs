use crate::{actions::Reducer, shared::{is_terminal_node, CUSTOM_NODE_ID}};

#[derive(Debug, Default, Clone)]
pub struct Transition {
    pub to: usize,
    pub reducer: Reducer,
}

impl Transition {
    pub fn new(to: usize, reducer: Reducer) -> Transition {
        Transition {
            to,
            reducer,
        }
    }

    pub fn clone_to_offset(&self, offset: usize) -> Transition {
        let to = if is_terminal_node(self.to) {
            self.to
        } else if self.to >= CUSTOM_NODE_ID {
            self.to + offset - CUSTOM_NODE_ID + 1
        } else {
            self.to + offset
        };
    
        Transition {
            to,
            reducer: self.reducer.clone(),
        }
    }
}
