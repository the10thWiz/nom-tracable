#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::ToTokens;
use syn::{self, parse_macro_input, parse_quote, AttributeArgs, FnArg, ItemFn, NestedMeta, Stmt};

#[proc_macro_attribute]
pub fn tracable_parser(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let item = parse_macro_input!(item as ItemFn);
    impl_tracable_parser(&attr, &item)
}

fn impl_tracable_parser(attr: &AttributeArgs, item: &ItemFn) -> TokenStream {
    let ident = &item.sig.ident;
    let name = match attr.first() {
        Some(NestedMeta::Lit(name)) => name.to_token_stream(),
        _ => parse_quote!(stringify!(#ident)),
    };

    let default = impl_tracable_parser_default(&item);
    let trace = impl_tracable_parser_trace(&item, name);

    let mut item = item.clone();

    item.block.stmts.clear();
    item.block.stmts.push(default);
    item.block.stmts.push(trace);

    item.into_token_stream().into()
}

fn impl_tracable_parser_default(item: &ItemFn) -> Stmt {
    let body = item.block.as_ref();
    parse_quote! {
        #[cfg(not(feature = "trace"))]
        {
            #body
        }
    }
}

fn impl_tracable_parser_trace(item: &ItemFn, name: impl ToTokens) -> Stmt {
    let input = if let Some(x) = &item.sig.inputs.first() {
        match x {
            FnArg::Typed(arg) => &arg.pat,
            _ => panic!("function with #[tracable_parser] must have an argument"),
        }
    } else {
        panic!("function with #[tracable_parser] must have an argument");
    };

    let body = item.block.as_ref();

    parse_quote! {
        #[cfg(feature = "trace")]
        {
            let (depth, #input) = nom_tracable::forward_trace(#input, #name);

            let body_ret = {
                let body = || { #body };
                body()
            };

            nom_tracable::backward_trace(body_ret, #name, depth)
        }
    }
}
