use std::{collections::VecDeque, mem};

use ahash::{HashSet, HashSetExt};
mod check;
use crate::{utils::ExtendedVec, Edge, EdgeID, Node, NodeID};

/// A graph is a collection of nodes and edges.
///
/// Nodes are identified by their index in the graph.
///
/// Each node will reference the edges it connects to. They are identified by their index in the graph.
///
/// The graph is undirected, meaning that if node A is connected to node B, then node B is connected to node A.
///
/// The graph is weighted, meaning that each edge has a weight. However, the weight can be zero.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,

    // Stores a Queue of empty slots in the edges and nodes arrays.
    // This will prevent having to update each node and edge index when removing a node or edge.
    empty_edge_slots: VecDeque<EdgeID>,
    empty_node_slots: VecDeque<NodeID>,
}
macro_rules! index {
    (
        $ty:ty => $array:ident => $output:ty
    ) => {
        impl std::ops::Index<$ty> for Graph {
            type Output = $output;

            fn index(&self, index: $ty) -> &Self::Output {
                &self.$array[index.0]
            }
        }
        impl std::ops::Index<&$ty> for Graph {
            type Output = $output;

            fn index(&self, index: &$ty) -> &Self::Output {
                &self.$array[index.0]
            }
        }
        impl std::ops::IndexMut<$ty> for Graph {
            fn index_mut(&mut self, index: $ty) -> &mut Self::Output {
                &mut self.$array[index.0]
            }
        }
        impl std::ops::IndexMut<&$ty> for Graph {
            fn index_mut(&mut self, index: &$ty) -> &mut Self::Output {
                &mut self.$array[index.0]
            }
        }
    };
}
index!(NodeID => nodes => Node);
index!(EdgeID => edges => Edge);

impl Graph {
    /// Adds a node to the graph.
    ///
    /// # Arguments
    /// * `name` - The name of the node.
    /// # Returns
    /// The ID of the node.
    pub fn add_node(&mut self, name: String) -> NodeID {
        if let Some(empty_node) = self.empty_node_slots.pop_front() {
            self.nodes[empty_node.0] = Node {
                name,
                edges: HashSet::new(),
            };
            empty_node
        } else {
            self.nodes.push_with_wrapped_id(Node {
                name,
                edges: HashSet::new(),
            })
        }
    }

    pub fn connect_nodes(&mut self, a: NodeID, b: NodeID) -> EdgeID {
        self.connect_nodes_with_weight(a, b, 0)
    }
    pub fn connect_nodes_with_weight(&mut self, a: NodeID, b: NodeID, weight: u32) -> EdgeID {
        let id = if let Some(empty_edge) = self.empty_edge_slots.pop_front() {
            self.edges[empty_edge.0] = Edge {
                weight,
                node_a: a,
                node_b: b,
            };
            empty_edge
        } else {
            self.edges.push_with_wrapped_id(Edge {
                weight,
                node_a: a,
                node_b: b,
            })
        };
        self.nodes[a.0].edges.insert(id);
        self.nodes[b.0].edges.insert(id);
        id
    }
    ///
    /// Returns the nodes connected to the given node.
    ///
    /// # Arguments
    /// * `node` - The node to get the connected nodes for.
    /// # Returns
    /// A vector of the nodes connected to the given node.
    ///
    /// ```rust
    /// use tux_graph::Graph;
    ///
    /// let mut graph = Graph::default();
    /// let a = graph.add_node("A".to_string());
    /// let b = graph.add_node("B".to_string());
    /// let c = graph.add_node("C".to_string());
    ///
    /// graph.connect_nodes(a, b);
    /// graph.connect_nodes(b, c);
    /// graph.connect_nodes(c, a);
    ///
    ///
    /// let connected_nodes = graph.connected_nodes(a);
    /// assert_eq!(connected_nodes.len(), 2);
    /// ```
    pub fn connected_nodes(&self, node: NodeID) -> Vec<NodeID> {
        self[node]
            .edges
            .iter()
            .map(|edge_id| {
                let edge = &self.edges[edge_id.0];
                if edge.node_a == node {
                    edge.node_b
                } else {
                    edge.node_a
                }
            })
            .collect()
    }
    /// Returns true if the given node is connected to itself.
    /// ```rust
    /// use tux_graph::Graph;
    ///
    /// let mut graph = Graph::default();
    /// let a = graph.add_node("A".to_string());
    /// let b = graph.add_node("B".to_string());
    ///
    /// graph.connect_nodes(a, b);
    /// graph.connect_nodes(a, a);
    ///
    /// assert_eq!(graph.is_node_connected_to_itself(a), true);
    /// ```
    pub fn is_node_connected_to_itself(&self, node: NodeID) -> bool {
        self[node].edges.iter().any(|edge_id| {
            let edge = &self[*edge_id];
            edge.node_a == edge.node_b
        })
    }

    pub fn remove_edge(&mut self, edge: EdgeID) {
        let (node_a, node_b) = {
            let edge_value = &self.edges[edge.0];
            (edge_value.node_a, edge_value.node_b)
        };
        self[node_a].remove_edge(edge);
        self[node_b].remove_edge(edge);

        self.edges[edge.0].clear();

        self.empty_edge_slots.push_back(edge);
    }

    pub fn remove_node(&mut self, node: NodeID) {
        let node_value = mem::take(&mut self.nodes[node.0].edges);
        for edge in node_value {
            self.remove_edge(edge);
        }

        self.nodes[node.0].clear();
        self.empty_node_slots.push_back(node);
    }
    pub fn number_of_nodes(&self) -> usize {
        self.nodes.len() - self.empty_node_slots.len()
    }
    pub fn number_of_edges(&self) -> usize {
        self.edges.len() - self.empty_edge_slots.len()
    }

    /// If the graph has dead nodes.
    ///
    /// A dead node is a node that has been removed from the graph. However, the node still exists in the graph to prevent updating the indexes of the edges.
    pub fn has_dead_nodes(&self) -> bool {
        !self.empty_node_slots.is_empty()
    }
    /// If the graph has dead edges.
    ///
    /// A dead edge is an edge that has been removed from the graph. However, the edge still exists in the graph to prevent updating the indexes of the nodes.
    pub fn has_dead_edges(&self) -> bool {
        !self.empty_edge_slots.is_empty()
    }
    /// Removes all nodes and edges that are in the unused slots.
    ///
    /// This will update the indexes of the nodes and edges.
    pub fn remove_dead_values(&mut self) {
        if !self.empty_edge_slots.is_empty() {
            self.remove_dead_edges();
        }
        if !self.empty_node_slots.is_empty() {
            self.remove_dead_nodes();
        }
    }
    fn remove_dead_nodes(&mut self) {
        let Self {
            nodes,
            empty_node_slots,
            edges,
            ..
        } = self;

        let mut empty_node_slots: Vec<_> = mem::take(empty_node_slots).into();
        empty_node_slots.sort();

        let first_index = empty_node_slots.first().map(|x| x.0).unwrap_or(usize::MAX);
        let mut new_nodes = Vec::with_capacity(nodes.len() - empty_node_slots.len());

        for (old_index, node) in nodes.iter().enumerate().map(|(i, x)| (NodeID(i), x)) {
            if old_index < first_index {
                // The node index did not change.
                new_nodes.push(node.clone());
                continue;
            }
            if empty_node_slots.binary_search_contains(&old_index) {
                // This is a dead node. So we skip it.
                continue;
            }
            // Alright this node is not dead.

            // First Update All the edges with the new index.
            let new_index = NodeID(new_nodes.len());
            for edge in &node.edges {
                let Edge { node_a, node_b, .. } = &mut edges[edge.0];
                if *node_a == old_index {
                    *node_a = new_index;
                }
                if *node_b == old_index {
                    *node_b = new_index;
                }
            }
            // Push the new node.
            new_nodes.push(node.clone());
        }
        *nodes = new_nodes;
    }
    fn remove_dead_edges(&mut self) {
        let Self {
            nodes,
            edges,
            empty_edge_slots,
            ..
        } = self;
        let mut replace_node_edges =
            |node: NodeID, old_index_as_edge_id: EdgeID, new_index: EdgeID| {
                let node = &mut nodes[node.0];
                if node.edges.remove(&old_index_as_edge_id) {
                    node.edges.insert(new_index);
                }
            };

        let mut empty_edge_slots: Vec<_> = mem::take(empty_edge_slots).into();
        empty_edge_slots.sort();

        let first_index = empty_edge_slots.first().map(|x| x.0).unwrap_or(usize::MAX);
        let mut new_edges = Vec::with_capacity(edges.len() - empty_edge_slots.len());
        // TODO: Optimize this by mutating the original edges array instead of creating a new one.
        for (old_index, edge) in edges.iter().enumerate() {
            let old_index_as_edge_id = EdgeID(old_index);
            if old_index < first_index {
                // The edge index did not change.
                new_edges.push(edge.clone());
                continue;
            }
            if empty_edge_slots.binary_search_contains(&old_index_as_edge_id) {
                // This is a dead edge. So we skip it.
                continue;
            }
            // Alright this edge is not dead.

            let Edge {
                node_a,
                node_b,
                weight,
            } = *edge;
            // Push the new edge.
            let new_index: EdgeID = new_edges.push_with_wrapped_id(Edge {
                node_a,
                node_b,
                weight,
            });
            // Update the nodes to reflect the new index.
            replace_node_edges(node_a, old_index_as_edge_id, new_index);
            replace_node_edges(node_b, old_index_as_edge_id, new_index);
        }
        *edges = new_edges;
    }
}

#[cfg(test)]
mod test {
    use crate::Graph;

    #[test]
    pub fn basic_graph() {
        let mut graph = Graph::default();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());

        graph.connect_nodes(a, b);
        graph.connect_nodes(b, c);
        graph.connect_nodes(c, a);

        assert_eq!(graph.number_of_nodes(), 3);
        assert_eq!(graph.number_of_edges(), 3);

        println!("{:#?}", graph);
    }
    #[test]
    pub fn cleanup_tests() {
        let mut graph = Graph::default();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());

        graph.connect_nodes(a, b);
        graph.connect_nodes(b, c);
        graph.connect_nodes(c, a);

        graph.remove_node(b);
        assert_eq!(graph.number_of_nodes(), 2);
        assert_eq!(graph.number_of_edges(), 1);
        // The actual number of nodes is 3, but one of them is dead.
        assert_eq!(graph.nodes.len(), 3);
        // The actual number of edges is 3, but two of them are dead.
        assert_eq!(graph.edges.len(), 3);

        graph.remove_dead_values();

        assert_eq!(graph.number_of_nodes(), 2);
        assert_eq!(graph.number_of_edges(), 1);

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }
}
