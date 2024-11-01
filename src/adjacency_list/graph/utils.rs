/// Internally used utilities for the adjacency list graph.
use crate::GraphError;

use super::{AdjListGraph, Edge, EdgeID, NodeID};
pub type EdgeRefAndID<'a> = (EdgeID, &'a Edge);
pub type EdgeAndID = (EdgeID, Edge);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeCopyResult {
    pub new_edge_id: EdgeID,
    pub node_a: Option<(NodeID, NodeID)>,
    pub node_b: Option<(NodeID, NodeID)>,
}
#[derive(Debug, Clone)]
pub(crate) enum SingleEdgeOrManyEdges {
    Single(EdgeID, Edge),
    Many(Vec<(EdgeID, Edge)>),
}
impl From<EdgeAndID> for SingleEdgeOrManyEdges {
    fn from((id, edge): (EdgeID, Edge)) -> Self {
        SingleEdgeOrManyEdges::Single(id, edge)
    }
}
impl SingleEdgeOrManyEdges {
    fn weight(&self) -> u32 {
        match self {
            SingleEdgeOrManyEdges::Single(_, edge) => edge.weight(),
            SingleEdgeOrManyEdges::Many(edges) => edges.first().unwrap().1.weight(),
        }
    }
    fn push_weight(&mut self, new_id: EdgeID, new_edge: Edge) {
        match self {
            SingleEdgeOrManyEdges::Single { .. } => {
                let a = match self {
                    SingleEdgeOrManyEdges::Single(id, edge) => (*id, edge.clone()),
                    _ => unreachable!(),
                };

                let edges = vec![a, (new_id, new_edge)];
                *self = SingleEdgeOrManyEdges::Many(edges);
            }
            SingleEdgeOrManyEdges::Many(edges) => edges.push((new_id, new_edge)),
        }
    }
}

impl<T> AdjListGraph<T> {
    /// Copies the referenced edge and the nodes it connects to the target graph.
    ///
    /// To check if a node has been copied, use the `node_if_already_copied` closure.
    ///
    /// The return contains the new edge ID and the new node IDs if they were copied.
    pub(crate) fn copy_edge_and_referenced_nodes<F>(
        &self,
        target: &mut Self,
        edge: EdgeID,
        node_if_already_copied: F,
    ) -> Result<EdgeCopyResult, GraphError>
    where
        F: Fn(NodeID) -> Option<NodeID>,
        T: Clone,
    {
        let edge = &self.edges[edge.0];
        let (target_node_a_id, did_create_new_a_node) =
            self.target_node_or_copy(target, edge.node_a, &node_if_already_copied);
        let (target_node_b_id, did_create_new_b_node) =
            self.target_node_or_copy(target, edge.node_b, &node_if_already_copied);
        let node_a_return = if did_create_new_a_node {
            Some((edge.node_a, target_node_a_id))
        } else {
            None
        };
        let node_b_return = if did_create_new_b_node {
            Some((edge.node_b, target_node_b_id))
        } else {
            None
        };
        let edge =
            target.connect_nodes_with_weight(target_node_a_id, target_node_b_id, edge.weight())?;

        Ok(EdgeCopyResult {
            new_edge_id: edge,
            node_a: node_a_return,
            node_b: node_b_return,
        })
    }
    /// Copies the referenced node to the target graph.
    fn target_node_or_copy<F>(
        &self,
        target: &mut Self,
        node: NodeID,
        node_if_already_copied: &F,
    ) -> (NodeID, bool)
    where
        T: Clone,
        F: Fn(NodeID) -> Option<NodeID>,
    {
        if let Some(updated_node_id) = node_if_already_copied(node) {
            return (updated_node_id, false);
        }
        let from_node_value_cloned = self[node].value().clone();
        let new_node = target.add_node(from_node_value_cloned);

        (new_node, true)
    }
    /// Returns a list of edges sorted by weight.
    ///
    /// This is a tuple of the edge's ID and a reference to the edge.
    ///
    /// Note calling enumerate on the iterator will not give the correct id as the edges from this function are sorted.
    pub(crate) fn get_edges_sorted_by_weight(&self) -> Vec<(EdgeID, &Edge)> {
        let mut edges = self
            .edges
            .iter()
            .enumerate()
            .map(|(index, edge)| (EdgeID(index), edge))
            .collect::<Vec<_>>();
        edges.sort_by_key(|(_, edge)| edge.weight());
        edges
    }
    pub(crate) fn group_same_weights_and_sort(&self) -> Vec<SingleEdgeOrManyEdges> {
        let mut target: Vec<SingleEdgeOrManyEdges> = Vec::with_capacity(self.edges.len());

        for (index, edge) in self.edges.iter().enumerate() {
            if self.empty_edge_slots.contains(&EdgeID(index)) {
                continue;
            }
            let find_item = target
                .iter_mut()
                .find(|item| item.weight() == edge.weight());

            if let Some(item) = find_item {
                item.push_weight(EdgeID(index), edge.clone());
            } else {
                target.push((EdgeID(index), edge.clone()).into());
            }
        }
        target.sort_by_key(|item| item.weight());
        target
    }
    pub(crate) fn is_node_empty(&self, node_id: usize) -> bool {
        self.empty_node_slots.contains(&NodeID(node_id))
    }
}
