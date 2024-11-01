use crate::adjacency_list::*;
use crate::utils::macros::id_type;
use ahash::{HashSet, HashSetExt};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node<T> {
    value: Option<T>,
    pub(crate) edges: HashSet<EdgeID>,
}
impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
            edges: HashSet::new(),
        }
    }
    /// Removes an edge from the node.
    pub(crate) fn remove_edge(&mut self, edge: EdgeID) {
        self.edges.remove(&edge);
    }
    /// Removes data within the node.
    ///
    /// This is used to clear the node's data when the node is removed from the graph.
    pub(crate) fn clear(&mut self) -> Option<T> {
        self.edges.clear();
        self.value.take()
    }
    pub(crate) fn clear_and_set(&mut self, value: T) {
        self.clear();
        self.value = Some(value);
    }
    pub fn value(&self) -> &T {
        self.value.as_ref().unwrap()
    }
    pub fn optional_value(&self) -> Option<&T> {
        self.value.as_ref()
    }
    pub fn has_edge(&self, edge: EdgeID) -> bool {
        self.edges.contains(&edge)
    }
    /// Checks if the node has an equivalent value to another node.
    ///
    /// If either one has none, it returns false.
    pub(crate) fn node_value_eq(&self, other: &Node<T>) -> bool
    where
        T: PartialEq,
    {
        if self.optional_value().is_none() || other.optional_value().is_none() {
            return false;
        }
        self.optional_value() == other.optional_value()
    }
    /// Checks if this node is truly equal to another node.
    ///
    /// Meaning that the each edge in this node exists in the other graph.
    pub(crate) fn are_nodes_truly_equal<'a, 'b>(
        &'a self,
        self_graph: &'a AdjListGraph<T>,
        other_node: &'b Node<T>,
        other_graph: &'b AdjListGraph<T>,
    ) -> bool
    where
        T: PartialEq,
    {
        if self.edges.len() != other_node.edges.len() {
            // Different number of edges. Can't be equal.
            return false;
        }
        // Loop through all edges in a and check if they are in b.
        self.edges.iter().all(|edge| {
            // Node. No need to check if these edges are valid as they exist in the node meaning they are valid.
            // If this is not the case, the graph is invalid.
            let edge = &self_graph.edges[edge.0];
            let (self_graph_node_a, self_graph_node_b) = edge.node_values(self_graph);
            // Find an equivalent edge in b.
            other_node.edges.iter().any(|edge_id| {
                let edge = &other_graph.edges[edge_id.0];
                if edge.weight() != edge.weight() {
                    // Different weight. Can't be equal.
                    return false;
                }
                let (other_graph_node_a, other_graph_node_b) = edge.node_values(other_graph);
                self_graph_node_a.node_value_eq(other_graph_node_a)
                    && self_graph_node_b.node_value_eq(other_graph_node_b)
                    || self_graph_node_a.node_value_eq(other_graph_node_a)
                        && self_graph_node_b.node_value_eq(other_graph_node_b)
            })
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeID(pub usize);
id_type!(NodeID);
