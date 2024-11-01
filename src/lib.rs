use adjacency_list::EdgeID;
use thiserror::Error;

pub mod adjacency_list;
pub(crate) mod utils;
/// Graph creation macro.
///
/// ```rust
/// use tux_graph::graph;
///
/// let graph = graph! {
///   a [value='a'];
///   b [value='b'];
///   c [value='c'];
///
///   a -- b [weight=1];
///   b -- c [weight=2];
///   a -- c;
/// };
/// ```
pub use tux_graph_macros::graph;
/// Graph creation macro without importing the graph types.
///
/// This is mainly used inside the actual crate for testing purposes.
#[doc(hidden)]
pub use tux_graph_macros::graph_no_import;
#[derive(Debug, Error)]
pub enum GraphError {
    #[error("Nodes already have a connected edge. Edge ID: {0:?}")]
    NodesAlreadyConnected(EdgeID),
}

#[cfg(test)]

mod macro_tests {

    use tux_graph_macros::graph_no_import;

    use crate::adjacency_list::AdjListGraph;

    #[test]
    fn test_graph_creation() {
        let graph: AdjListGraph<char> = graph_no_import! {
            a [value='a'];
            b [value='b'];
            c [value='c'];
            a -- b [weight=1];
            b -- c [weight=2];
            a -- c;
        };
        assert_eq!(graph.number_of_nodes(), 3);
        assert_eq!(graph.number_of_edges(), 3);
    }
}
