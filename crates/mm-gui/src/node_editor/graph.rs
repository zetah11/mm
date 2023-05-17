use std::collections::{HashMap, HashSet};

use egui::{vec2, Pos2, Vec2};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(usize);

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub pos: Pos2,
    pub size: Vec2,
    pub focused: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Edge<N> {
    pub from: N,
    pub to: N,
}

#[derive(Debug, Default)]
pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    edges: HashSet<Edge<NodeId>>,
    count: usize,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashSet::new(),
            count: 0,
        }
    }

    pub fn add_node(&mut self, name: String, pos: Pos2) -> NodeId {
        let node = Node {
            name,
            pos,
            size: vec2(400.0, 200.0),
            focused: false,
        };

        let id = NodeId(self.count);
        self.count += 1;
        debug_assert!(self.nodes.insert(id, node).is_none());
        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.edges.insert(Edge { from, to });
    }

    /// Remove the node with the given id. Panics if this node has already been
    /// removed.
    fn remove_node(&mut self, id: NodeId) -> Node {
        let node = self
            .nodes
            .remove(&id)
            .expect("nodes are never removed twice");
        self.edges.retain(|edge| edge.from != id && edge.to != id);
        node
    }
}

/// Stores the UI state of the graph.
#[derive(Debug, Default)]
pub struct GraphEditor {
    graph: Graph,
    selected: Option<NodeId>,
}

impl GraphEditor {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            selected: None,
        }
    }

    pub fn add_node(&mut self, name: String, pos: Pos2) -> NodeId {
        self.graph.add_node(name, pos)
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.graph.add_edge(from, to);
    }

    /// Remove the node with the given id. Panics if this node has already been
    /// removed.
    pub fn remove_node(&mut self, id: NodeId) -> Node {
        if self.selected == Some(id) {
            self.selected = None;
        }

        self.graph.remove_node(id)
    }

    /// Get the node with the given id. Returns `None` if and only if that node
    /// has been removed.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.graph.nodes.get(&id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.graph.nodes.iter().map(|(id, node)| (*id, node))
    }

    pub fn nodes_mut(&mut self) -> impl Iterator<Item = (NodeId, &mut Node)> {
        self.graph.nodes.iter_mut().map(|(id, node)| (*id, node))
    }

    pub fn edges(&self) -> impl Iterator<Item = Edge<&Node>> {
        const NO_DANGLING_EDGES: &str = "edges to removed nodes are also removed";

        self.graph.edges.iter().map(|edge| {
            let from = self.graph.nodes.get(&edge.from).expect(NO_DANGLING_EDGES);
            let to = self.graph.nodes.get(&edge.to).expect(NO_DANGLING_EDGES);
            Edge { from, to }
        })
    }
}
