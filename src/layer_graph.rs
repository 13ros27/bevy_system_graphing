use bevy::{ecs::schedule::NodeId, prelude::*, utils::HashMap};

#[derive(Debug)]
pub enum LayerNode {
    System(String),
    Set(String, LayerGraph),
}

#[derive(Debug, Default, Resource)]
pub struct LayerGraph {
    nodes: HashMap<NodeId, LayerNode>,
    pub layers: Vec<Vec<NodeId>>,
    edges: HashMap<NodeId, Vec<NodeId>>,
}

impl LayerGraph {
    pub fn add_node(&mut self, node_id: NodeId, node: LayerNode) {
        self.nodes.insert(node_id, node);
    }

    pub fn add_edges(&mut self, node_id: NodeId, edges: Vec<NodeId>) {
        self.edges.insert(node_id, edges);
    }

    pub fn node_name(&self, node_id: &NodeId) -> String {
        match &self.nodes[node_id] {
            LayerNode::System(name) => name.clone(),
            LayerNode::Set(name, _graph) => name.clone(),
        }
    }
}
