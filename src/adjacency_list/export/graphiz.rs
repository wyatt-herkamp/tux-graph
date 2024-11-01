use crate::adjacency_list::AdjListGraph;

use super::FormattedStringBuilder;
#[derive(Debug, Clone)]
pub struct GraphizSettings {
    pub layout: String,
    pub overlap: bool,
    pub node_layout: String,
    pub graph_name: String,
}
impl Default for GraphizSettings {
    fn default() -> Self {
        Self {
            layout: "neato".to_string(),
            overlap: false,
            node_layout: "circle".to_string(),
            graph_name: "G".to_string(),
        }
    }
}

pub fn export_graphiz<T>(graph: &AdjListGraph<T>, settings: &GraphizSettings) -> String
where
    T: std::fmt::Display,
{
    let mut graphiz = FormattedStringBuilder::new(format!("graph {} {{\n", settings.graph_name), 4);
    graphiz.push(format!("layout={}", settings.layout));
    graphiz.push(format!("overlap={}", settings.overlap));
    graphiz.push(format!("node [shape={}]", settings.node_layout));
    graphiz.push("//  Nodes");
    for (index, node) in graph.nodes.iter().enumerate() {
        if let Some(value) = node.optional_value() {
            graphiz.push(format!("{{node [label=\"{value}\"] {index}}};"))
        }
    }
    graphiz.push("//  Edges");
    for edge in &graph.edges {
        graphiz.push(format!(
            "{node_a} -- {node_b};",
            node_a = edge.node_a.0,
            node_b = edge.node_b.0
        ));
    }
    graphiz.push_no_indent("}");
    graphiz.finish()
}
