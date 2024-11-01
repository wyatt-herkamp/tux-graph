use proc_macro::TokenStream;
pub(crate) mod graph;
#[proc_macro]
pub fn graph_no_import(item: TokenStream) -> TokenStream {
    let parse_content = syn::parse_macro_input!(item as graph::GraphInput);
    let output = graph::expand_no_inputs(parse_content);
    match output {
        Ok(output) => output.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn graph(item: TokenStream) -> TokenStream {
    let parse_content = syn::parse_macro_input!(item as graph::GraphInput);
    let output = graph::expand(parse_content);
    match output {
        Ok(output) => output.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
