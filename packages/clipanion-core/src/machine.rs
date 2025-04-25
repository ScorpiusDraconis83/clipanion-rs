use std::{collections::HashSet, fmt::Debug};

use crate::{node::Node, shared::{is_terminal_node, Arg, CUSTOM_NODE_ID, ERROR_NODE_ID, INITIAL_NODE_ID, SUCCESS_NODE_ID}, transition::Transition};

pub struct Machine<'a, TCheck, TReducer> {
    pub contexts: Vec<usize>,
    pub nodes: Vec<Node<'a, TCheck, TReducer>>,
}

impl<'a, TCheck: Debug, TReducer: Debug> Debug for Machine<'a, TCheck, TReducer> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (id, node) in self.nodes.iter().enumerate() {
            writeln!(f, "Node {} ({}):", id, node.context)?;

            if id == ERROR_NODE_ID {
                writeln!(f, "  [Error]")?;
            } else if id == SUCCESS_NODE_ID {
                writeln!(f, "  [Success]")?;
            }

            for (check, transition) in node.dynamics.iter() {
                writeln!(f, "  Dynamic: {:?} -> {:?} -> {}", check, transition.reducer, transition.to)?;
            }

            for transition in node.shortcuts.iter() {
                writeln!(f, "  Shortcut -> {}", transition.to)?;
            }

            for (segment, transitions) in node.statics.iter() {
                for transition in transitions.iter() {
                    writeln!(f, "  Static: {:?} -> {:?} -> {}", segment, transition.reducer, transition.to)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a, TCheck, TReducer> Machine<'a, TCheck, TReducer> {
    pub fn new(context: usize) -> Self {
        let mut default = Self {
            contexts: vec![context],
            nodes: vec![],
        };

        for _ in 0..CUSTOM_NODE_ID {
            default.nodes.push(Node::new());
        }

        default
    }

    pub fn create_node(&mut self) -> usize {
        self.inject_node(Node::new())
    }

    pub fn inject_node(&mut self, node: Node<'a, TCheck, TReducer>) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn register_dynamic(&mut self, from: usize, check: TCheck, to: usize, reducer: TReducer) {
        self.nodes[from].dynamics.push((check, Transition::new(to, reducer)));
    }

    pub fn register_shortcut(&mut self, from: usize, to: usize) where TReducer: Default {
        self.nodes[from].shortcuts.push(Transition::new(to, Default::default()));
    }

    pub fn register_static(&mut self, from: usize, key: Arg<'a>, to: usize, reducer: TReducer) {
        self.nodes[from].statics.entry(key).or_default().push(Transition::new(to, reducer));
    }

    pub fn new_any_of<I>(machines: I) -> Self where TCheck: Clone, TReducer: Clone + Default, I: IntoIterator<Item = Self> {
        let mut out = Machine {
            contexts: vec![],
            nodes: vec![],
        };

        for _ in 0..CUSTOM_NODE_ID {
            out.nodes.push(Node::new());
        }

        for machine in machines {
            let context_offset
                = out.contexts.len();

            let node_offset
                = out.nodes.len();

            out.contexts.extend(machine.contexts);
            out.register_shortcut(INITIAL_NODE_ID, node_offset);

            for id in 0..machine.nodes.len() {
                if !is_terminal_node(id) {
                    let mut cloned_node = machine.nodes[id].clone_to_offset(node_offset);
                    cloned_node.context += context_offset;
                    out.nodes.push(cloned_node);
                }
            }
        }

        out
    }

    fn resolve_passthrough(&self, mut id: usize) -> usize {
        loop {
            let node
                = &self.nodes[id];

            if node.shortcuts.len() != 1 || !node.statics.is_empty() || !node.dynamics.is_empty() {
                break;
            }

            id = node.shortcuts[0].to;
        }

        id
    }

    pub fn simplify_machine(&mut self) where TCheck: Debug + Clone + PartialEq, TReducer: Debug, Node<'a, TCheck, TReducer>: Clone, Transition<TReducer>: Clone {
        let mut visited
            = HashSet::new();

        let mut queue
            = vec![INITIAL_NODE_ID];

        while let Some(node_id) = queue.pop() {
            let mut node
                = self.nodes[node_id].clone();

            for (_, transition) in node.dynamics.iter_mut() {
                transition.to = self.resolve_passthrough(transition.to);
                if visited.insert(transition.to) {
                    queue.push(transition.to);
                }
            }

            for (_, transitions) in node.statics.iter_mut() {
                for transition in transitions.iter_mut() {
                    transition.to = self.resolve_passthrough(transition.to);
                    if visited.insert(transition.to) {
                        queue.push(transition.to);
                    }
                }
            }

            for transition in node.shortcuts.iter_mut() {
                transition.to = self.resolve_passthrough(transition.to);
                if visited.insert(transition.to) {
                    queue.push(transition.to);
                }
            }

            std::mem::swap(
                &mut self.nodes[node_id],
                &mut node,
            );
        }

        let mut in_offset = 0;
        let mut out_offset = 0;

        let mut map_offset = Vec::with_capacity(self.nodes.len());
        map_offset.resize(self.nodes.len(), 99);

        self.nodes.retain(|_| {
            if in_offset < CUSTOM_NODE_ID || visited.contains(&in_offset) {
                map_offset[in_offset] = out_offset;

                in_offset += 1;
                out_offset += 1;

                true
            } else {
                in_offset += 1;

                false
            }
        });

        for node in self.nodes.iter_mut() {
            for (_, transition) in node.dynamics.iter_mut() {
                transition.to = map_offset[transition.to];
            }

            for transition in node.shortcuts.iter_mut() {
                transition.to = map_offset[transition.to];
            }

            for (_, transitions) in node.statics.iter_mut() {
                for transition in transitions.iter_mut() {
                    transition.to = map_offset[transition.to];
                }
            }
        }
    }
}
