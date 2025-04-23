use crate::shared::{is_terminal_node, CUSTOM_NODE_ID};

#[derive(Debug, Default, Clone)]
pub struct Transition<TReducer> {
    pub to: usize,
    pub reducer: TReducer,
}

impl<TReducer> Transition<TReducer> {
    pub fn new(to: usize, reducer: TReducer) -> Transition<TReducer> {
        Transition {
            to,
            reducer,
        }
    }
}

impl<TReducer: Clone> Transition<TReducer> {
    pub fn clone_to_offset(&self, offset: usize) -> Transition<TReducer> {
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
