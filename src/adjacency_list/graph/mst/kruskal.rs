use std::fmt::Debug;

use ahash::{HashMap, HashMapExt};
use itertools::Itertools;

use crate::adjacency_list::{
    AdjListGraph, Edge, EdgeCopyResult, EdgeID, NodeID, SingleEdgeOrManyEdges,
};

impl<T> AdjListGraph<T> {
    pub fn find_all_msts(&self, remove_duplicates: bool) -> Vec<AdjListGraph<T>>
    where
        T: Clone + PartialEq + Eq + Debug,
    {
        let edges = self.group_same_weights_and_sort();
        let mut result = Vec::new();
        self.recursive_find_all_msts(
            AdjListGraph::default(),
            HashMap::default(),
            &edges,
            remove_duplicates,
            &mut result,
        );

        result
    }
    fn recursive_find_all_msts(
        &self,
        mut mst: AdjListGraph<T>,
        mut updated_nodes_id: HashMap<NodeID, NodeID>,
        edges: &[SingleEdgeOrManyEdges],
        remove_duplicates: bool,
        msts: &mut Vec<AdjListGraph<T>>,
    ) where
        T: Clone + PartialEq + Eq,
    {
        for (how_far, edge) in edges.iter().enumerate() {
            match edge {
                SingleEdgeOrManyEdges::Single(id, edge) => {
                    maybe_copy_edge(self, &mut mst, *id, &mut updated_nodes_id, edge);
                }
                SingleEdgeOrManyEdges::Many(vec) => {
                    for possible_orderings in vec.iter().permutations(vec.len()) {
                        let mut mst_variant = mst.clone();
                        let mut updated_nodes_id = updated_nodes_id.clone();
                        for (id, edge) in possible_orderings {
                            maybe_copy_edge(
                                self,
                                &mut mst_variant,
                                *id,
                                &mut updated_nodes_id,
                                edge,
                            );
                        }
                        self.recursive_find_all_msts(
                            mst_variant,
                            updated_nodes_id,
                            &edges[how_far + 1..],
                            remove_duplicates,
                            msts,
                        );
                    }
                    // Skips the current iteration as we had to diverge into multiple paths.
                    return;
                }
            }
        }
        if mst.number_of_nodes() != 0 {
            if remove_duplicates {
                if msts.contains(&mst) {
                    return;
                }
                msts.push(mst);
            } else {
                msts.push(mst);
            }
        }
    }
    /// Only works if the graphs data are unique.
    pub fn kruskal_find_mst(&self) -> Option<AdjListGraph<T>>
    where
        T: Clone + PartialEq + Eq + Debug,
    {
        let mut mst = AdjListGraph::default();
        let mut updated_node_ids = HashMap::<NodeID, NodeID>::new();
        let mut edges = self.get_edges_sorted_by_weight();

        edges.sort_by_key(|(_, edge)| edge.weight());

        for (og_index, edge) in edges {
            maybe_copy_edge(self, &mut mst, og_index, &mut updated_node_ids, edge);
        }

        if mst.number_of_nodes() == 0 {
            None
        } else {
            Some(mst)
        }
    }
}
fn maybe_copy_edge<T>(
    from: &AdjListGraph<T>,
    mst: &mut AdjListGraph<T>,
    og_index: EdgeID,
    updated_node_ids: &mut HashMap<NodeID, NodeID>,
    edge: &Edge,
) -> bool
where
    T: Clone,
{
    if mst.is_empty() {
        copy_edge_and_nodes(from, mst, og_index, updated_node_ids);
        return true;
    }
    if !updated_node_ids.contains_key(&edge.node_a) || !updated_node_ids.contains_key(&edge.node_b)
    {
        copy_edge_and_nodes(from, mst, og_index, updated_node_ids);
        return true;
    }
    let node_a = updated_node_ids[&edge.node_a];
    let node_b = updated_node_ids[&edge.node_b];
    if cycle::would_adding_edge_cause_cycle(mst, node_a.0, node_b.0) {
        return false;
    }
    copy_edge_and_nodes(from, mst, og_index, updated_node_ids);
    true
}
/// Copies the edge and nodes from the `from` graph to the `target` graph.
///
/// If a node already exists in the `target` graph, it will not be copied. Instead, the existing node will be used.
fn copy_edge_and_nodes<T>(
    from: &AdjListGraph<T>,
    target: &mut AdjListGraph<T>,
    edge: EdgeID,
    updated_node_ids: &mut HashMap<NodeID, NodeID>,
) where
    T: Clone,
{
    let EdgeCopyResult { node_a, node_b, .. } = from
        .copy_edge_and_referenced_nodes(target, edge, |node| {
            if let Some(updated_node_id) = updated_node_ids.get(&node) {
                return Some(*updated_node_id);
            }
            None
        })
        .unwrap();

    if let Some((og_node_a, new_node_a)) = node_a {
        updated_node_ids.insert(og_node_a, new_node_a);
    }
    if let Some((og_node_b, new_node_b)) = node_b {
        updated_node_ids.insert(og_node_b, new_node_b);
    }
}

mod cycle {
    use crate::adjacency_list::AdjListGraph;

    pub fn would_adding_edge_cause_cycle<T>(
        graph: &AdjListGraph<T>,
        node_a: usize,
        node_b: usize,
    ) -> bool {
        let mut visited = vec![false; graph.number_of_nodes()];
        would_adding_edge_cause_cycle_inner(graph, node_a, node_b, &mut visited)
    }
    pub fn would_adding_edge_cause_cycle_inner<T>(
        graph: &AdjListGraph<T>,
        node: usize,
        target: usize,
        visited: &mut Vec<bool>,
    ) -> bool {
        if visited[node] {
            return false;
        }
        visited[node] = true;
        if node == target {
            return true;
        }
        for &edge in &graph.nodes[node].edges {
            let next = if graph.edges[edge.0].node_a == node {
                graph.edges[edge.0].node_b.0
            } else {
                graph.edges[edge.0].node_a.0
            };
            if would_adding_edge_cause_cycle_inner(graph, next, target, visited) {
                return true;
            }
        }
        false
    }
    // TODO: Add tests
}
#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use anyhow::Context;
    use tux_graph_macros::graph_no_import;

    use crate::adjacency_list::{
        export::graphiz::{export_graphiz, GraphizSettings},
        AdjListGraph,
    };
    // Test is based on the example found on this video https://www.youtube.com/watch?v=71UQH7Pr9kU
    fn example_from_video() -> AdjListGraph<char> {
        graph_no_import! {
            a [value='A'];
            b [value='B'];
            c [value='C'];
            d [value='D'];
            e [value='E'];
            f [value='F'];
            g [value='G'];
            // Edges
            // A Connections
            a -- b [weight=2];
            a -- c [weight=3];
            a -- d [weight=3];
            // B Connections
            b -- c [weight=4];
            b -- e [weight=3];
            // C Connections
            c -- d [weight=5];
            c -- e [weight=1];
            // D Connections
            d -- f [weight=7];
            // E Connections
            e -- f [weight=8];
            // F Connections
            f -- g [weight=9];
        }
    }
    #[test]
    pub fn test_from_video_create() -> anyhow::Result<()> {
        let example_graph = example_from_video();
        assert_eq!(example_graph.number_of_nodes(), 7);
        assert_eq!(example_graph.number_of_edges(), 10);

        save_graph(&example_graph, "mst_test_from_video_create").context("Failed to save graph")
    }
    #[test]
    pub fn test_find_all() -> anyhow::Result<()> {
        let example_graph = example_from_video();

        let msts = example_graph.find_all_msts(true);
        println!("Found {} msts", msts.len());

        for (index, mst) in msts.iter().enumerate() {
            save_graph(mst, &format!("mst_test_find_all_{}", index))?;
        }
        Ok(())
    }
    #[test]
    pub fn test_one() -> anyhow::Result<()> {
        let example_graph = example_from_video();

        let mst = example_graph.kruskal_find_mst().unwrap();

        save_graph(&mst, "mst_test_one")?;
        Ok(())
    }

    fn save_graph(graph: &AdjListGraph<char>, file_name: &str) -> anyhow::Result<()> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("output")
            .join("mst")
            .join("kruskal");
        if !path.exists() {
            std::fs::create_dir_all(&path).context("Failed to create directory")?;
        }
        println!("Saving graph to {:?} with name {file_name}", path);
        {
            let graphiz_file = format!("{}.dot", file_name);
            let file_name = path.join(graphiz_file);
            let graphiz = export_graphiz(graph, &GraphizSettings::default());
            std::fs::write(file_name, graphiz).context("Failed to write file")?
        }
        {
            let json_file = format!("{}.json", file_name);
            let file_name = path.join(json_file);
            let json = serde_json::to_string_pretty(graph).context("Failed to serialize graph")?;
            std::fs::write(file_name, json).context("Failed to write file")?
        }
        Ok(())
    }
    /// This was a graph that was used in one of my class assignments.
    ///
    /// It asks how many MSTs can be found in the graph.
    ///
    /// You can see a picture of the graph in the `assets/graphs/class_assignment_9`.
    ///
    /// The answer comes out to be 6.
    #[test]
    pub fn find_for_class_assignment_9() {
        let graph = graph_no_import! {
            a [value='A'];
            b [value='B'];
            c [value='C'];
            d [value='D'];
            e [value='E'];
            f [value='F'];

            c -- b [weight=1];
            a -- b [weight=2];
            a -- d [weight=1];
            d -- c [weight=2];

            c -- e [weight=3];
            e -- f [weight=3];
            f -- c [weight=3];
        };

        let msts = graph.find_all_msts(true);

        for (index, mst) in msts.iter().enumerate() {
            save_graph(
                mst,
                &format!("mst_test_find_all_class_assignment_9_{}", index),
            )
            .unwrap();
        }
        println!("Found {} msts", msts.len());

        assert_eq!(msts.len(), 6, "Only 6 MSTs can be created from this graph");
    }
}
