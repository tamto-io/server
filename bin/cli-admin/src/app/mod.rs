use std::{sync::{mpsc, Mutex, Arc}, collections::HashMap, net::SocketAddr, time::Duration};

use chord_rs::Client;
use tamto_grpc::client::ChordGrpcClient;
use tui::widgets::ListState;

use crate::IoEvent;

use self::node::{NodeList, NodeDetail};

mod node;

enum UpdateEvent {
    NodeAdd(SocketAddr),
    NodeRemove(u64),
    FetchNodeDetail(u64),
}

pub struct App {
    /// The channel used to send events to the main thread
    tx: mpsc::Sender<IoEvent>,

    /// The channel used to send state update events
    update_tx: mpsc::Sender<UpdateEvent>,

    /// The shared state of the application
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<State>,
}

impl Shared {
    pub fn new() -> Self {
        Self { state: Mutex::new(State::new()) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiWidget {
    Search,
    Debug
}

pub enum ScrollEvent {
    Up,
    Down,
}

struct State {
    // state: HashMap<u64, NodeElement>,

    /// List of active widgets
    /// These are the optional widgets that can be enabled or disabled
    /// e.g. the search widget, or the debug widget
    active_widgets: Vec<UiWidget>,

    /// List of debug messages
    /// They are displayed in the debug widget, which can be enabled with the 'd' key
    debug: Vec<String>,

    /// The state of the list widget containing the nodes
    node_list: NodeList,
}

impl State {
    pub fn new() -> Self {
        let node_list = NodeList::default();

        Self { active_widgets: Vec::new(), debug: Vec::new(), node_list }
    }
}

impl App {
    pub const DEBUG_SIZE: u16 = 5;


    pub fn new(tx: mpsc::Sender<IoEvent>) -> Self {
        let data = Arc::new(Shared::new());
        let (update_tx, update_rx) = mpsc::channel();


        Self::run(update_tx.clone(), update_rx, data.clone());

        Self { tx, update_tx, shared: data }
    }

    /// Run the background task
    fn run(tx: mpsc::Sender<UpdateEvent>, rx: mpsc::Receiver<UpdateEvent>, shared: Arc<Shared>) {
        let shared_refresh = shared.clone();
        tokio::spawn(async move {
            let shared = shared_refresh.clone();
            loop {
                tokio::time::sleep(Duration::from_millis(1000)).await;

                let mut refreshed: u64 = 0;
                let nodes = {
                    let state = shared.state.lock().unwrap();
                    state.node_list.ids()
                };
                for node in nodes {
                    let id = node.id;
                    log::debug!("Refreshing node {}", id);
                    if id == refreshed {
                        continue;
                    }
                    tx.send(UpdateEvent::NodeAdd(node.addr)).unwrap();
                    refreshed = id;
                }
            }
        });
        tokio::spawn(async move {
            loop {
                let shared = shared.clone();
                tokio::time::sleep(Duration::from_millis(100)).await;

                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(event) => {
                        match event {
                            UpdateEvent::NodeAdd(addr) => {
                                tokio::spawn(async move {
                                    let client = ChordGrpcClient::init(addr);
                                    let finger_table = client.get_finger_table().await.unwrap();

                                    let shared = shared.clone();
                                    let mut state = shared.state.lock().unwrap();

                                    for finger in finger_table {
                                        if state.node_list.exists(finger.id().into()) {
                                            continue;
                                        }
                                        log::debug!("Adding node {} to list", finger.id());

                                        state.node_list.add(finger.id().into(), finger.addr());
                                    }
                                    drop(state);
                                });
                                
                                // let finger_table = client.get_finger_table().await.unwrap();

                                
                                // let mut state = shared.state.lock().unwrap();
                            }
                            UpdateEvent::NodeRemove(id) => {
                                // let mut state = shared.state.lock().unwrap();
                                // state.node_list.remove(id);
                            }
                            UpdateEvent::FetchNodeDetail(id) => {
                                let shared = shared.clone();
                                tokio::spawn(async move {
                                    let node = {
                                        let state = shared.state.lock().unwrap();
                                        state.node_list.get(id).unwrap().clone()
                                    };
                                    let client = ChordGrpcClient::init(node.addr);
                                    let predecessor = client.predecessor().await.unwrap();
                                    let successor = client.successor().await.unwrap().into();

                                    let mut state = shared.state.lock().unwrap();
                                    let detail = NodeDetail::new(predecessor.map(|p| p.into()), successor);

                                    state.node_list.update(id, detail);
                                });
                            },
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }
        });
    }

    pub fn active_widgets(&self) -> Vec<UiWidget> {
        let widgets = &self.shared.state.lock().unwrap().active_widgets;

        widgets.clone()
    }

    pub fn widget_enabled(&self, widget: UiWidget) -> bool {
        let widgets = &self.shared.state.lock().unwrap().active_widgets;

        widgets.contains(&widget)
    }

    pub fn enable_widget(&self, widget: UiWidget) {
        let mut state = self.shared.state.lock().unwrap();
        state.active_widgets.push(widget);
    }

    pub fn disable_widget(&self, widget: UiWidget) {
        let mut state = self.shared.state.lock().unwrap();
        state.active_widgets.retain(|w| *w != widget);
    }

    pub fn get_debug(&self) -> Vec<String> {
        let state = self.shared.state.lock().unwrap();

        state.debug.clone()
    }

    pub fn add_node(&self, id: u64, addr: SocketAddr) {
        self.update_tx.send(UpdateEvent::NodeAdd(addr)).unwrap();
    }

    pub(crate) fn get_node_detail(&self, id: u64) -> Option<node::NodeDetail> {
        let state = self.shared.state.lock().unwrap();

        let detail = state.node_list.get(id)
            .map(|node| node.detail.clone())
            .flatten();

        if detail.is_none() || detail.as_ref().unwrap().should_refresh() {
            self.update_tx.send(UpdateEvent::FetchNodeDetail(id)).unwrap();
        }

        detail
    }
    
    /// Returns the list of node ids and the state of the list widget
    pub(crate) fn node_ids(&self) -> (ListState, Vec<node::NodeElement>) {
        let state = self.shared.state.lock().unwrap();

        (state.node_list.state.clone(), state.node_list.ids())
    }

    pub fn scroll(&self, event: ScrollEvent) {
        let mut state = self.shared.state.lock().unwrap();

        match event {
            ScrollEvent::Up => state.node_list.previous(),
            ScrollEvent::Down => state.node_list.next(),
        }
    }
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self { tx: self.tx.clone(), update_tx: self.update_tx.clone(), shared: self.shared.clone() }
    }
}
