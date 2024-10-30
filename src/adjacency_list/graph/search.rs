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
        visited[node] = true;
        path.push(NodeID(node));
        if f(self.nodes[node].value.as_ref().unwrap()) {
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
}

#[cfg(test)]

mod tests {
    use crate::adjacency_list::*;

    #[test]
    pub fn test_searches() {
        let mut graph = AdjListGraph::default();
        let node1 = graph.add_node("Data 1".to_owned());
        let node2 = graph.add_node("Data 2".to_owned());
        let node3 = graph.add_node("Data 3".to_owned());
        let node4 = graph.add_node("Data 4".to_owned());
        let node5 = graph.add_node("Data 5".to_owned());
        let node6 = graph.add_node("Data 6".to_owned());
        let node7 = graph.add_node("Data 7".to_owned());
        let node8 = graph.add_node("Data 8".to_owned());
        let node9 = graph.add_node("Data 9".to_owned());

        graph.connect_nodes(node1, node2);
        graph.connect_nodes(node1, node3);
        graph.connect_nodes(node2, node4);
        graph.connect_nodes(node2, node5);
        graph.connect_nodes(node3, node6);
        graph.connect_nodes(node3, node7);
        graph.connect_nodes(node4, node8);
        graph.connect_nodes(node4, node9);

        let path = graph.dfs(|x| *x == "Data 9").unwrap();
        assert_eq!(path, vec![0, 1, 3, 8]);
    }
}
