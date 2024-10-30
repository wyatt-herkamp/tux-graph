//! The functions defined in this module are used to check if the graph is in a valid state.
//!
//! These checks check for things that shouldn't happen in a graph. However, they are great for testing the graph's integrity.
use super::Graph;
use crate::utils::IdType;
use crate::{Edge, EdgeID, Node, NodeID};

macro_rules! valid_values {
    (
        $(#[$is_valid_fn_docs:meta])*
        is_valid_fn: $is_valid_fn:ident,
        $(#[$does_id_exist_docs:meta])*
        does_id_exist: $does_id_exist:ident,
        $(#[$get_fn_docs:meta])*
        get_fn: $fn_name:ident,
        $(#[$has_fn_docs:meta])*
        has_fn: $has_fn_name:ident,
        id_type: $id_ty:ty,
        values: $values:ident,
        empty_slots: $empty_slots:ident,
        check_fn: $check_fn:ident
    ) => {
        $(#[$is_valid_fn_docs])*
        pub fn $is_valid_fn(&self, edge_id: $id_ty) -> bool {
            if !self.$does_id_exist(edge_id) {
                return false;
            }
            self.$check_fn(&self[edge_id])
        }
        $(#[$does_id_exist_docs])*
        pub fn $does_id_exist(&self, id: $id_ty) -> bool {
            if self.$empty_slots.contains(&id) {
                return false;
            }

            self.$values.get(id.0).is_some()
        }
        $(#[$get_fn_docs])*
        pub fn $fn_name(&self) -> Vec<$id_ty> {
            let mut invalid_values = Vec::new();
            let Self {
                $values,
                $empty_slots,
                ..
            } = self;
            for (id, value) in $values
                .iter()
                .enumerate()
                .map(|(i, n)| (<$id_ty>::from_usize(i), n))
            {
                if $empty_slots.contains(&id) {
                    // Value is dead. Why check it?
                    continue;
                }
                if !self.$check_fn(value) {
                    invalid_values.push(id);
                }
            }
            invalid_values
        }
        $(#[$has_fn_docs])*
        pub fn $has_fn_name(&self) -> bool {
            for (id, value) in self
                .$values
                .iter()
                .enumerate()
                .map(|(i, n)| (<$id_ty>::from_usize(i), n))
            {
                if self.$empty_slots.contains(&id) {
                    // Value is dead. Why check it?
                    continue;
                }
                if !self.$check_fn(value) {
                    return true;
                }
            }
            false
        }
    };
}
impl Graph {
    valid_values! {
        /// Checks if the edge is valid.
        /// Checks if the id exists and if the nodes associated with the edge exist.
        is_valid_fn: is_valid_edge,
        /// Checks if the edge id exists.
        does_id_exist: does_edge_id_exist,
        /// Gets all the invalid edges.
        get_fn: invalid_edges,
        /// Checks if there are any invalid edges.
        has_fn: has_invalid_edges,
        id_type: EdgeID,
        values: edges,
        empty_slots: empty_edge_slots,
        check_fn: is_valid_edge_inner
    }
    valid_values! {
        /// Checks if the node is valid.
        is_valid_fn: is_valid_node,
        /// Checks if the node id exists.
        does_id_exist: does_node_id_exist,
        /// Gets all the invalid nodes.
        get_fn: invalid_nodes,
        /// Checks if there are any invalid nodes.
        has_fn: has_invalid_nodes,
        id_type: NodeID,
        values: nodes,
        empty_slots: empty_node_slots,
        check_fn: is_valid_node_inner
    }

    /// Checks if all the nodes edges exist
    #[inline]
    fn is_valid_node_inner(&self, node: &Node) -> bool {
        return node.edges.iter().any(|edge_id| {
            let value = self.does_edge_id_exist(*edge_id);
            print!("{node:?} {} ", value);
            value
        });
    }
    /// Checks if the nodes associated with the edge exist
    #[inline]
    fn is_valid_edge_inner(&self, edge: &Edge) -> bool {
        self.does_node_id_exist(edge.node_a) && self.does_node_id_exist(edge.node_b)
    }
}

#[cfg(test)]
mod tests {
    use crate::{EdgeID, Graph, NodeID};
    #[test]
    pub fn test_graph_with_invalid_node() {
        let mut graph = Graph::default();
        let a = graph.add_node("Node 1".to_string());
        graph[a].edges.insert(EdgeID(2));
        println!("{:?}", graph);
        assert!(graph.has_invalid_nodes());
    }
    #[test]
    pub fn test_valid_graph() {
        let mut graph = Graph::default();
        let a = graph.add_node("Node 1".to_string());
        let b = graph.add_node("Node 2".to_string());
        let _ = graph.connect_nodes(a, b);
        println!("{:?}", graph);
        assert!(!graph.has_invalid_nodes());
        assert!(!graph.has_invalid_edges());
    }

    #[test]
    pub fn test_graph_with_invalid_edge() {
        let mut graph = Graph::default();
        let a = graph.add_node("Node 1".to_string());
        let b = graph.add_node("Node 2".to_string());
        let edge = graph.connect_nodes(a, b);
        graph[edge].node_a = NodeID(2);
        println!("{:?}", graph);
        assert!(graph.has_invalid_edges());
    }
}
