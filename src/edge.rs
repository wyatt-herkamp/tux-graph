use crate::{utils::macros::id_type, NodeID};

#[derive(Debug, Clone)]
pub struct Edge {
    pub weight: u32,
    pub node_a: NodeID,
    pub node_b: NodeID,
}
impl Edge {
    /// Removes data within the edge.
    ///
    /// This is used to clear the edge's data when the edge is removed from the graph.
    pub(crate) fn clear(&mut self) {
        self.weight = 0;
        self.node_a = NodeID(usize::MAX);
        self.node_b = NodeID(usize::MAX);
    }
}
#[derive(Debug, Clone, Copy)]
pub struct EdgeID(pub(crate) usize);

id_type!(EdgeID);
