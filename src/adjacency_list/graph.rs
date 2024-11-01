use std::{collections::VecDeque, mem};

mod check;
mod equality;
mod mst;
mod search;
mod utils;
pub(crate) use utils::*;

use crate::utils::ExtendedVec;
use crate::{adjacency_list::*, GraphError};

/// A graph is a collection of nodes and edges.
///
/// Nodes are identified by their index in the graph.
///
/// Each node will reference the edges it connects to. They are identified by their index in the graph.
///
/// The graph is undirected, meaning that if node A is connected to node B, then node B is connected to node A.
///
/// The graph is weighted, meaning that each edge has a weight. However, the weight can be zero.
///
/// ## Serde Note
///
/// Serialize is manually implemented to prevent serializing the empty slots.
#[derive(Debug, Clone)]
pub struct AdjListGraph<T> {
    pub(crate) nodes: Vec<Node<T>>,
    pub(crate) edges: Vec<Edge>,

    // Stores a Queue of empty slots in the edges and nodes arrays.
    // This will prevent having to update each node and edge index when removing a node or edge.
    empty_edge_slots: VecDeque<EdgeID>,
    empty_node_slots: VecDeque<NodeID>,
}
mod _serde {
    use super::*;
    use serde::Deserialize;
    use serde::{de::Visitor, ser::SerializeStruct, Serialize};
    const NODES: &str = "nodes";
    const EDGES: &str = "edges";
    impl<T> Serialize for AdjListGraph<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            if self.has_dead_edges() || self.has_dead_nodes() {
                return Err(serde::ser::Error::custom("Graph has dead nodes or edges. Please call remove_dead_values before serializing."));
            }
            let mut state = serializer.serialize_struct("AdjListGraph", 2)?;
            state.serialize_field(NODES, &self.nodes)?;
            state.serialize_field(EDGES, &self.edges)?;
            state.end()
        }
    }
    #[derive(Default)]
    struct AdjGraphVisitor<T>(std::marker::PhantomData<T>);
    impl<'de, T> Visitor<'de> for AdjGraphVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = AdjListGraph<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("Expecting a struct with nodes and edges fields.")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut nodes = None;
            let mut edges = None;
            while let Some(key) = map.next_key::<&str>()? {
                match key {
                    NODES => {
                        if nodes.is_some() {
                            return Err(serde::de::Error::duplicate_field(NODES));
                        }
                        nodes = Some(map.next_value()?);
                    }
                    EDGES => {
                        if edges.is_some() {
                            return Err(serde::de::Error::duplicate_field(EDGES));
                        }
                        edges = Some(map.next_value()?);
                    }
                    _ => {
                        return Err(serde::de::Error::unknown_field(key, &["nodes", "edges"]));
                    }
                }
            }

            let nodes = nodes.ok_or_else(|| serde::de::Error::missing_field(NODES))?;
            let edges = edges.ok_or_else(|| serde::de::Error::missing_field(EDGES))?;
            Ok(AdjListGraph {
                nodes,
                edges,
                empty_edge_slots: Default::default(),
                empty_node_slots: Default::default(),
            })
        }
    }
    impl<'de, T> Deserialize<'de> for AdjListGraph<T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            // TODO: Validate that the graph is valid.
            deserializer.deserialize_struct(
                "AdjListGraph",
                &["nodes", "edges"],
                AdjGraphVisitor(Default::default()),
            )
        }
    }
}

impl<T> Default for AdjListGraph<T> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            empty_edge_slots: VecDeque::new(),
            empty_node_slots: VecDeque::new(),
        }
    }
}
macro_rules! index {
    (
        $ty:ty => $array:ident => $output:ty
    ) => {
        impl<T> std::ops::Index<$ty> for AdjListGraph<T> {
            type Output = $output;

            fn index(&self, index: $ty) -> &Self::Output {
                &self.$array[index.0]
            }
        }
        impl<T> std::ops::Index<&$ty> for AdjListGraph<T> {
            type Output = $output;

            fn index(&self, index: &$ty) -> &Self::Output {
                &self.$array[index.0]
            }
        }
        impl<T> std::ops::IndexMut<$ty> for AdjListGraph<T> {
            fn index_mut(&mut self, index: $ty) -> &mut Self::Output {
                &mut self.$array[index.0]
            }
        }
        impl<T> std::ops::IndexMut<&$ty> for AdjListGraph<T> {
            fn index_mut(&mut self, index: &$ty) -> &mut Self::Output {
                &mut self.$array[index.0]
            }
        }
    };
}
index!(NodeID => nodes => Node<T>);
index!(EdgeID => edges => Edge);

impl<T> AdjListGraph<T> {
    /// Adds a node to the graph.
    ///
    /// # Arguments
    /// * `name` - The name of the node.
    /// # Returns
    /// The ID of the node.
    pub fn add_node(&mut self, value: T) -> NodeID {
        if let Some(empty_node) = self.empty_node_slots.pop_front() {
            self.nodes[empty_node.0].clear_and_set(value);
            empty_node
        } else {
            self.nodes.push_with_wrapped_id(Node::new(value))
        }
    }

    /// Adds a node to the graph.
    ///
    /// Returns the node IDs of the nodes added.
    pub fn add_nodes_from_iterator(&mut self, values: impl Iterator<Item = T>) -> Vec<NodeID> {
        values.map(|value| self.add_node(value)).collect()
    }

    /// Adds N nodes from an array.
    ///
    /// Returns the node IDs of the nodes added.
    pub fn add_nodes_from_sized_array<const N: usize>(&mut self, values: [T; N]) -> [NodeID; N] {
        let mut nodes = [NodeID(usize::MAX); N];
        for (i, value) in values.into_iter().enumerate() {
            nodes[i] = self.add_node(value);
        }
        nodes
    }

    pub fn connect_nodes(&mut self, a: NodeID, b: NodeID) -> Result<EdgeID, GraphError> {
        self.connect_nodes_with_weight(a, b, 0)
    }
    pub fn connect_nodes_with_weight(
        &mut self,
        a: NodeID,
        b: NodeID,
        weight: u32,
    ) -> Result<EdgeID, GraphError> {
        for edge_id in &self[a].edges {
            let edge = &self.edges[edge_id.0];
            let (node_a, node_b) = edge.nodes();
            if node_a == b || node_b == b {
                return Err(GraphError::NodesAlreadyConnected(*edge_id));
            }
        }

        let id = if let Some(empty_edge) = self.empty_edge_slots.pop_front() {
            self.edges[empty_edge.0] = Edge::new(weight, a, b);
            empty_edge
        } else {
            self.edges.push_with_wrapped_id(Edge::new(weight, a, b))
        };
        self.nodes[a.0].edges.insert(id);
        self.nodes[b.0].edges.insert(id);
        Ok(id)
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
    /// use tux_graph::adjacency_list::AdjListGraph;
    ///
    /// let mut graph = AdjListGraph::default();
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
                let (node_a, node_b) = edge.nodes();
                if node_a == node {
                    node_b
                } else {
                    node_a
                }
            })
            .collect()
    }
    /// Returns true if the given node is connected to itself.
    /// ```rust
    /// use tux_graph::adjacency_list::AdjListGraph;
    ///
    /// let mut graph = AdjListGraph::default();
    /// let a = graph.add_node("A".to_string());
    /// let b = graph.add_node("B".to_string());
    ///
    /// graph.connect_nodes(a, b);
    /// graph.connect_nodes(a, a);
    ///
    /// assert!(graph.is_node_connected_to_itself(a), "Node A is connected to itself.");
    /// ```
    pub fn is_node_connected_to_itself(&self, node: NodeID) -> bool {
        self.is_node_connected_to_node(node, node)
    }
    /// Returns true if the given node is connected to itself.
    /// ```rust
    /// use tux_graph::adjacency_list::AdjListGraph;
    ///
    /// let mut graph = AdjListGraph::default();
    /// let a = graph.add_node("A".to_string());
    /// let b = graph.add_node("B".to_string());
    ///
    /// graph.connect_nodes(a, b);
    /// graph.connect_nodes(a, a);
    ///
    /// assert!(graph.is_node_connected_to_node(a, a), "Node A is connected to itself.");
    /// assert!(graph.is_node_connected_to_node(a, b), "Node A is connected to Node B.");
    /// ```
    pub fn is_node_connected_to_node(&self, node_a: NodeID, node_b: NodeID) -> bool {
        self[node_a].edges.iter().any(|edge_id| {
            let edge = &self[*edge_id];
            let (edge_node_a, edge_node_b) = edge.nodes();
            edge_node_a == node_b || edge_node_b == node_b
        })
    }

    pub fn remove_edge(&mut self, edge: EdgeID) {
        let (node_a, node_b) = { &self.edges[edge.0].nodes() };
        self[node_a].remove_edge(edge);
        self[node_b].remove_edge(edge);

        self.edges[edge.0].clear();

        self.empty_edge_slots.push_back(edge);
    }
    /// Removes a node from the graph.
    ///
    /// Returns the value of the node if it exists.
    ///
    /// All edges connected to the node will be removed.
    ///
    /// Removed Node and connected edges will be pushed into the empty slots.
    pub fn remove_node(&mut self, node: NodeID) -> Option<T> {
        let node_value = mem::take(&mut self.nodes[node.0].edges);
        for edge in node_value {
            self.remove_edge(edge);
        }
        self.empty_node_slots.push_back(node);
        self.nodes[node.0].clear()
    }
    pub fn number_of_nodes(&self) -> usize {
        self.nodes.len() - self.empty_node_slots.len()
    }
    pub fn number_of_edges(&self) -> usize {
        self.edges.len() - self.empty_edge_slots.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
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
    pub fn remove_dead_values(&mut self)
    where
        T: Clone,
    {
        if !self.empty_edge_slots.is_empty() {
            self.remove_dead_edges();
        }
        if !self.empty_node_slots.is_empty() {
            self.remove_dead_nodes();
        }
    }
    fn remove_dead_nodes(&mut self)
    where
        T: Clone,
    {
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
            let (node_a, node_b) = edge.nodes();

            // Push the new edge.
            let new_index: EdgeID = new_edges.push_with_wrapped_id(edge.clone());
            // Update the nodes to reflect the new index.
            replace_node_edges(node_a, old_index_as_edge_id, new_index);
            replace_node_edges(node_b, old_index_as_edge_id, new_index);
        }
        *edges = new_edges;
    }

    pub fn get_node(&self, id: NodeID) -> Option<&Node<T>> {
        self.nodes.get(id.0)
    }
}

#[cfg(test)]
mod test {
    use crate::adjacency_list::*;

    #[test]
    pub fn basic_graph() {
        let mut graph = AdjListGraph::default();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());

        graph.connect_nodes(a, b).unwrap();
        graph.connect_nodes(b, c).unwrap();
        graph.connect_nodes(c, a).unwrap();

        assert_eq!(graph.number_of_nodes(), 3);
        assert_eq!(graph.number_of_edges(), 3);

        println!("{:#?}", graph);
    }
    #[test]
    pub fn cleanup_tests() {
        let mut graph = AdjListGraph::default();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());

        graph.connect_nodes(a, b).unwrap();
        graph.connect_nodes(b, c).unwrap();
        graph.connect_nodes(c, a).unwrap();

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
