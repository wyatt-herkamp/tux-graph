use ahash::HashSet;

use crate::{utils::macros::id_type, EdgeID};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub(crate) edges: HashSet<EdgeID>,
}
impl Node {
    /// Removes an edge from the node.
    pub(crate) fn remove_edge(&mut self, edge: EdgeID) {
        self.edges.remove(&edge);
    }
    /// Removes data within the node.
    ///
    /// This is used to clear the node's data when the node is removed from the graph.
    pub(crate) fn clear(&mut self) {
        self.edges.clear();
        self.name.clear();
    }

    pub fn has_edge(&self, edge: EdgeID) -> bool {
        self.edges.contains(&edge)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeID(pub usize);
id_type!(NodeID);
