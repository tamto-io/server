use std::{net::SocketAddr, collections::{HashMap, BTreeMap}, time::Instant};

use tui::widgets::{ListState, ListItem};


#[derive(Clone, Debug)]
pub(crate) enum NodeStatus {
    New(Instant),
    Active,
    Inactive(u8),
}

impl NodeStatus {
    pub fn new() -> Self {
        Self::New(Instant::now())
    }

    pub fn is_new(&self) -> bool {
        match self {
            Self::New(instant) if (Instant::now().duration_since(*instant).as_secs()) < 5 => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeInfo {
    pub(crate) id: u64,
    pub(crate) addr: SocketAddr,
}

impl From<chord_rs::Node> for NodeInfo {
    fn from(node: chord_rs::Node) -> Self {
        Self { id: node.id().into(), addr: node.addr() }
    }
}

#[derive(Clone)]
pub(crate) struct NodeDetail {
    pub(crate) predecessor: Option<NodeInfo>,
    pub(crate) successor: NodeInfo,
    pub(crate) last_refresh: Instant,
}

impl NodeDetail {
    pub fn new(predecessor: Option<NodeInfo>, successor: NodeInfo) -> Self {
        Self { predecessor, successor, last_refresh: Instant::now() }
    }

    /// Returns true if the last refresh was more than 1 second ago
    pub fn should_refresh(&self) -> bool {
        (Instant::now().duration_since(self.last_refresh).as_secs()) > 1
    }
}

#[derive(Clone)]
pub(crate) struct NodeElement {
    pub(crate) id: u64,
    pub(crate) addr: SocketAddr,
    pub(crate) state: NodeStatus,
    pub(crate) detail: Option<NodeDetail>,
}

impl NodeElement {
    pub fn new(id: u64, addr: SocketAddr) -> Self {
        Self { id, addr, state: NodeStatus::new(), detail: None }
    }
}

pub struct NodeList {
    items: BTreeMap<u64, NodeElement>,

    pub state: ListState,
}

impl Default for NodeList {
    fn default() -> Self {
        Self { items: BTreeMap::new(), state: ListState::default() }
    }
}

impl NodeList {
    pub fn exists(&self, id: u64) -> bool {
        self.items.contains_key(&id)
    }

    pub fn add(&mut self, id: u64, addr: SocketAddr) {
        self.items.insert(id, NodeElement::new(id, addr));
    }

    pub(crate) fn get(&self, id: u64) -> Option<&NodeElement> {
        self.items.get(&id)
    }

    pub(crate) fn update(&mut self, id: u64, detail: NodeDetail) {
        if let Some(node) = self.items.get_mut(&id) {
            node.detail = Some(detail);
        }
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

    pub(crate) fn ids(&self) -> Vec<NodeElement> {
        self.items.values().cloned().collect()
    }
}
