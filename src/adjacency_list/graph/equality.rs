use super::AdjListGraph;

impl<T> PartialEq for AdjListGraph<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // Equals ignoring location and empty slots.
        for (index, node_a) in self.nodes.iter().enumerate() {
            if self.is_node_empty(index) {
                // Node is marked as dead it doesn't need to be checked.
                continue;
            }
            // Finds a node with an equivalent value.
            let Some(equivalent_item) = other.find_equivalent_node_value(node_a) else {
                return false;
            };
            // Checks if the two nodes are equal.
            if !node_a.are_nodes_truly_equal(self, equivalent_item, other) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::adjacency_list::*;
    use tux_graph_macros::graph_no_import;
    #[test]
    pub fn cloned_equality() {
        let graph_a = graph_no_import! {
            a [value = "A"];
            b [value = "B"];
            c [value = "C"];

            a -- b [weight = 1];
            b -- c [weight = 2];
            a -- c;
        };
        let graph_b = graph_a.clone();

        assert_eq!(graph_a, graph_b);
    }
    #[test]
    pub fn basic_equality() {
        let graph_a = graph_no_import! {
            a [value = "A"];
            b [value = "B"];
            c [value = "C"];

            a -- b [weight = 1];
            b -- c [weight = 2];
            a -- c;
        };
        let graph_b = graph_no_import! {
            c [value = "C"];
            a [value = "A"];
            b [value = "B"];

            a -- b [weight = 1];
            b -- c [weight = 2];
            a -- c;
        };

        assert_eq!(graph_a, graph_b);
    }
}
