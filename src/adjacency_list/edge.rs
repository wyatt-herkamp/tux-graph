use serde::{Deserialize, Serialize};

use crate::utils::macros::id_type;

use super::{AdjListGraph, Node, NodeID};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub(crate) weight: u32,
    pub(crate) node_a: NodeID,
    pub(crate) node_b: NodeID,
}
impl Edge {
    pub(crate) fn new(weight: u32, node_a: NodeID, node_b: NodeID) -> Self {
        Self {
            weight,
            node_a,
            node_b,
        }
    }
    /// Removes data within the edge.
    ///
    /// This is used to clear the edge's data when the edge is removed from the graph.
    pub(crate) fn clear(&mut self) {
        self.weight = 0;
        self.node_a = NodeID(usize::MAX);
        self.node_b = NodeID(usize::MAX);
    }
    pub fn weight(&self) -> u32 {
        self.weight
    }
    pub fn nodes(&self) -> (NodeID, NodeID) {
        (self.node_a, self.node_b)
    }
    pub fn node_values<'graph, T>(
        &self,
        graph: &'graph AdjListGraph<T>,
    ) -> (&'graph Node<T>, &'graph Node<T>) {
        (&graph[self.node_a], &graph[self.node_b])
    }
}
#[derive(Debug, Clone, Copy)]
pub struct EdgeID(pub(crate) usize);

id_type!(EdgeID);
