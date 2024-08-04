use std::collections::{HashMap, HashSet};

use crate::{actions::{Check, Reducer}, node::Node, shared::{is_terminal_node, Arg, CUSTOM_NODE_ID, ERROR_NODE_ID, INITIAL_NODE_ID, SUCCESS_NODE_ID}, transition::Transition};

#[derive(Debug, Default)]
pub struct MachineContext {
    pub command_index: usize,
    pub command_usage: String,
    pub preferred_names: HashMap<String, String>,
    pub valid_bindings: HashSet<String>,
}

pub struct Machine {
    pub contexts: Vec<MachineContext>,
    pub nodes: Vec<Node>,
}

impl std::fmt::Debug for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (id, node) in self.nodes.iter().enumerate() {
            writeln!(f, "Node {}:", id)?;

            if id == ERROR_NODE_ID {
                writeln!(f, "  [Error]")?;
            } else if id == SUCCESS_NODE_ID {
                writeln!(f, "  [Success]")?;
            }

            for (check, transition) in node.dynamics.iter() {
                writeln!(f, "  Dynamic: {:?} -> {}", check, transition.to)?;
            }

            for transition in node.shortcuts.iter() {
                writeln!(f, "  Shortcut -> {}", transition.to)?;
            }

            for (segment, transitions) in node.statics.iter() {
                for transition in transitions.iter() {
                    writeln!(f, "  Static: {:?} -> {}", segment, transition.to)?;
                }
            }
        }

        Ok(())
    }
}

impl Default for Machine {
    fn default() -> Machine {
        let mut default = Machine {
            contexts: vec![MachineContext::default()],
            nodes: vec![],
        };

        for _ in 0..CUSTOM_NODE_ID {
            default.nodes.push(Node::new());
        }

        default
    }
}

impl Machine {
    pub fn new() -> Machine {
        Default::default()
    }

    pub fn new_any_of<I>(machines: I) -> Machine where I: IntoIterator<Item = Machine> {
        let mut out = Machine::new();

        for machine in machines {
            let context_offset = out.contexts.len();
            let node_offset = out.nodes.len();

            out.contexts.extend(machine.contexts);
            out.register_shortcut(INITIAL_NODE_ID, node_offset, Reducer::None);

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

    pub fn inject_node(&mut self, node: Node) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn register_dynamic(&mut self, from: usize, check: Check, to: usize, reducer: Reducer) {
        self.nodes[from].dynamics.push((check, Transition::new(to, reducer)));
    }

    pub fn register_shortcut(&mut self, from: usize, to: usize, reducer: Reducer) {
        self.nodes[from].shortcuts.push(Transition::new(to, reducer));
    }

    pub fn register_static(&mut self, from: usize, key: Arg, to: usize, reducer: Reducer) {
        self.nodes[from].statics.entry(key).or_default().push(Transition::new(to, reducer));
    }

    pub fn simplify_machine(&mut self) {
        let mut visited = HashSet::new();
        let mut queue = vec![INITIAL_NODE_ID];

        while let Some(node) = queue.pop() {
            if !visited.insert(node) {
                continue;
            }

            let mut node_def = std::mem::take(&mut self.nodes[node]);

            for (_, transition) in node_def.dynamics.iter() {
                queue.push(transition.to);
            }

            for transition in node_def.shortcuts.iter() {
                queue.push(transition.to);
            }

            for (_, transitions) in node_def.statics.iter() {
                for transition in transitions.iter() {
                    queue.push(transition.to);
                }
            }

            let mut shortcuts: HashSet<usize>
                = HashSet::from_iter(node_def.shortcuts.iter().map(|t| t.to));

            while let Some(Transition {to, ..}) = node_def.shortcuts.pop() {
                let to_def = self.nodes[to].clone();

                for (segment, transitions) in to_def.statics.iter() {
                    let store
                        = node_def.statics.entry(segment.clone()).or_default();

                    for transition in transitions {
                        if !store.iter().any(|t| t.to == transition.to) {
                            store.push(transition.clone());
                        }
                    }
                }

                for (check, transition) in to_def.dynamics.iter() {
                    if !node_def.dynamics.iter().any(|(c, t)| c == check && t.to == transition.to) {
                        node_def.dynamics.push((check.clone(), transition.clone()));
                    }
                }

                for transition in to_def.shortcuts.iter() {
                    if !shortcuts.contains(&transition.to) {
                        node_def.shortcuts.push(transition.clone());
                        shortcuts.insert(transition.to);
                    }
                }
            }

            self.nodes[node] = std::mem::take(&mut node_def);
        }
    }
}
