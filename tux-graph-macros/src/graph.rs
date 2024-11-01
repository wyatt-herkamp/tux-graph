use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, Error, Expr, Ident, LitInt, Result};
mod kw {
    syn::custom_keyword!(weight);
    syn::custom_keyword!(value);
}
/// The input for the graph macro
///
/// ```ignore
///graph! {
///   node_1 [value=1];
///   node_2 [value=2];
///   node_3 [value=3];
///   node_4 [value=4];
///
///   node_1 -- node_2 [weight=1];
///   node_1 -- node_3 [weight=2];
///   node_2 -- node_4 [weight=3];
///}
/// ```
pub struct GraphInput {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}
impl Parse for GraphInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            // If the next is is a `-` then we have an edge
            if input.peek(syn::Token![-]) {
                edges.push(parse_edge(input, key)?);
            } else {
                // Parse `,` separated key value pairs
                let content;
                syn::bracketed!(content in input);
                let NodeAttributes { value } = content.parse()?;
                nodes.push(Node { key, value });
            }
            input.parse::<syn::Token![;]>()?;
        }
        Ok(Self { nodes, edges })
    }
}

fn parse_edge(input: &syn::parse::ParseBuffer<'_>, node_a: Ident) -> Result<Edge> {
    input.parse::<syn::Token![-]>()?;
    input.parse::<syn::Token![-]>()?;
    let node_b: Ident = input.parse()?;
    if input.peek(syn::Token![;]) {
        Ok(Edge {
            weight: None,
            node_a,
            node_b,
        })
    } else {
        // Parse `,` separated key value pairs
        let content;
        syn::bracketed!(content in input);
        let EdgeAttributes { weight } = content.parse()?;
        Ok(Edge {
            weight,
            node_a,
            node_b,
        })
    }
}
/// Nodes are defined like
///
/// ```ignore
/// {{key}} [value={{value}}]
pub struct Node {
    key: Ident,
    value: Expr,
}
struct NodeAttributes {
    value: Expr,
}
impl Parse for NodeAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut value = None;
        while !input.is_empty() {
            let _ = input.parse::<kw::value>()?;
            input.parse::<syn::Token![=]>()?;
            let value_parse: Expr = input.parse()?;
            value = Some(value_parse);
        }
        let Some(value) = value else {
            return Err(Error::new(
                input.span(),
                "Expected value attribute for node",
            ));
        };
        Ok(Self { value })
    }
}
/// Edges are defined like
///
/// ```ignore
/// {{node_a}} - {{node_b}} [weight={{weight}}]
/// ```
pub struct Edge {
    weight: Option<LitInt>,
    node_a: Ident,
    node_b: Ident,
}

struct EdgeAttributes {
    weight: Option<LitInt>,
}
impl Parse for EdgeAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut weight = None;
        while !input.is_empty() {
            let _ = input.parse::<kw::weight>()?;
            input.parse::<syn::Token![=]>()?;
            let value: LitInt = input.parse()?;
            weight = Some(value);
        }
        Ok(Self { weight })
    }
}
fn expand_nodes(nodes: &[Node]) -> Vec<TokenStream> {
    nodes
        .iter()
        .map(|node| {
            let key = &node.key;
            let value = &node.value;
            quote! {
               let #key = graph.add_node(#value);
            }
        })
        .collect()
}
fn expand_edges(edges: &[Edge]) -> Vec<TokenStream> {
    edges
        .iter()
        .map(|edge| {
            let node_a = &edge.node_a;
            let node_b = &edge.node_b;
            if let Some(weight) = &edge.weight {
                quote! {
                    graph.connect_nodes_with_weight(#node_a, #node_b, #weight).unwrap();
                }
            } else {
                quote! {
                    graph.connect_nodes(#node_a, #node_b).unwrap();
                }
            }
        })
        .collect()
}
pub fn expand_no_inputs(input: GraphInput) -> Result<TokenStream> {
    let GraphInput { nodes, edges } = input;
    let expanded_nodes: Vec<_> = expand_nodes(&nodes);
    let expanded_edges: Vec<_> = expand_edges(&edges);
    // TODO: Ensure no duplicate edges
    let result = quote! {
        {
            let mut graph = AdjListGraph::default();
            #(#expanded_nodes)*
            #(#expanded_edges)*
            graph
        }
    };

    Ok(result)
}

pub fn expand(input: GraphInput) -> Result<TokenStream> {
    let GraphInput { nodes, edges } = input;
    let expanded_nodes: Vec<_> = expand_nodes(&nodes);
    let expanded_edges: Vec<_> = expand_edges(&edges);
    // TODO: Ensure no duplicate edges
    let result = quote! {
        {
            use tux_graph::adjacency_list::AdjListGraph;
            let mut graph = AdjListGraph::default();
            #(#expanded_nodes)*
            #(#expanded_edges)*
            graph
        }
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use quote::quote;
    #[test]
    pub fn test_graph_input_parse() {
        let input = quote! {
            a [value=1];
            b [value=2];
            c [value=3];
            a -- b [weight=1];
            b -- c [weight=2];
            a -- c;
        };
        let parsed = syn::parse2::<super::GraphInput>(input).unwrap();
        assert_eq!(parsed.nodes.len(), 3);
        assert_eq!(parsed.edges.len(), 3);
    }

    #[test]
    pub fn test_invalid_graph_input_parse() {
        let input = quote! {
            a [value=1];
            b [value=2];
            c [value=3];
            a -- b [weight=1];
            b -- c [weight=2];
            a -- c
        };
        let parsed = syn::parse2::<super::GraphInput>(input);
        assert!(parsed.is_err());
    }
}
