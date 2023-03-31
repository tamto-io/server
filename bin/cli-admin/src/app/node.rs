use std::{net::SocketAddr, collections::HashMap};

use tui::widgets::{ListState, ListItem};


enum NodeStatus {
    New(u8),
    Active,
    Inactive(u8),
}


struct NodeDetail {
}

struct NodeElement {
    id: u64,
    addr: SocketAddr,
    state: NodeStatus,
    detail: Option<NodeDetail>
}

pub struct NodeList {
    items: HashMap<u64, NodeElement>,

    pub state: ListState,
}

impl Default for NodeList {
    fn default() -> Self {
        Self { items: HashMap::new(), state: ListState::default() }
    }
}

impl NodeList {
    pub fn exists(&self, id: u64) -> bool {
        self.items.contains_key(&id)
    }

    pub fn add(&mut self, id: u64, addr: SocketAddr) {
        self.items.insert(id, NodeElement { id, addr, state: NodeStatus::New(0), detail: None });
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn ids(&self) -> Vec<u64> {
        self.items.keys().cloned().collect()
    }
}
