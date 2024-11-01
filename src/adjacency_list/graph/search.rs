use tracing::trace;

use crate::adjacency_list::*;

use super::AdjListGraph;
impl<T> AdjListGraph<T> {
    /// Depth First Search
    pub fn dfs<F>(&self, f: F) -> Option<Vec<NodeID>>
    where
        F: Fn(&T) -> bool,
    {
        let mut visited = vec![false; self.nodes.len()];
        let mut path = vec![];
        if self.dfs_inner(0, &mut visited, &mut path, &f) {
            Some(path)
        } else {
            None
        }
    }
    fn dfs_inner<F>(
        &self,
        node: usize,
        visited: &mut Vec<bool>,
        path: &mut Vec<NodeID>,
        f: &F,
    ) -> bool
    where
        F: Fn(&T) -> bool,
    {
        if visited[node] {
            return false;
        }
        let node_id = NodeID(node);
        if self.empty_node_slots.contains(&node_id) {
            // Doesn't exist
            return false;
        }
        visited[node] = true;
        path.push(node_id);
        if f(self.nodes[node].value()) {
            return true;
        }
        for &edge in &self.nodes[node].edges {
            let next = if self.edges[edge.0].node_a == node {
                self.edges[edge.0].node_b.0
            } else {
                self.edges[edge.0].node_a.0
            };
            trace!(?next, ?visited, ?path, "DFS inner");
            if self.dfs_inner(next, visited, path, f) {
                return true;
            }
        }
        path.pop();
        false
    }

    pub fn find_node<F>(&self, f: F) -> Option<NodeID>
    where
        F: Fn(&T) -> bool,
    {
        for (index, node) in self.nodes.iter().enumerate() {
            if let Some(value) = node.optional_value() {
                if f(value) {
                    return Some(NodeID(index));
                }
            }
        }
        None
    }

    pub fn find_node_with_that_equals(&self, value: &T) -> Option<NodeID>
    where
        T: PartialEq + Eq,
    {
        self.find_node(|x| x == value)
    }

    /// Finds a node in the graph. If the node is not found, a new node is created with the given value.
    ///
    ///
    pub fn find_equivalent_node_value<'a>(&'a self, node: &Node<T>) -> Option<&'a Node<T>>
    where
        T: PartialEq,
    {
        self.nodes.iter().find(|b| node.node_value_eq(b))
    }
    /// Finds all nodes in the graph that are equivalent to the given node.
    pub fn find_all_equivalent_nodes_values<'a>(&'a self, node: &Node<T>) -> Vec<&'a Node<T>>
    where
        T: PartialEq,
    {
        self.nodes
            .iter()
            .filter(|b| node.node_value_eq(b))
            .collect()
    }
}

#[cfg(test)]

mod tests {
    use tux_graph_macros::graph_no_import;

    use crate::adjacency_list::*;

    #[test]
    pub fn test_searches() {
        let graph = graph_no_import! {
            data_1 [value = "Data 1"];
            data_2 [value = "Data 2"];
            data_3 [value = "Data 3"];
            data_4 [value = "Data 4"];
            data_5 [value = "Data 5"];
            data_6 [value = "Data 6"];
            data_7 [value = "Data 7"];
            data_8 [value = "Data 8"];
            data_9 [value = "Data 9"];

            data_1 -- data_2;
            data_1 -- data_3;
            data_2 -- data_4;
            data_2 -- data_5;
            data_3 -- data_6;
            data_3 -- data_7;
            data_4 -- data_8;
            data_4 -- data_9;
        };

        let path = graph.dfs(|x| *x == "Data 9").unwrap();
        assert_eq!(path, vec![0, 1, 3, 8]);
    }
}
