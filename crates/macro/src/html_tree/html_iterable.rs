use crate::PeekValue;
use boolinator::Boolinator;
use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::spanned::Spanned;
use syn::{Expr, Token};

use proc_macro2::{Ident, Span};
use syn::visit_mut::{self, VisitMut};
use syn::Macro;

pub struct HtmlIterable(Expr);

impl PeekValue<()> for HtmlIterable {
    fn peek(cursor: Cursor) -> Option<()> {
        let (ident, _) = cursor.ident()?;
        (ident.to_string() == "for").as_option()
    }
}

struct HtmlInnerModifier;
impl VisitMut for HtmlInnerModifier {
    fn visit_macro_mut(&mut self, node: &mut Macro) {
        if node.path.is_ident("html") {
            let ident = &mut node.path.segments.last_mut().unwrap().ident;
            *ident = Ident::new("html_nested", Span::call_site());
        }

        // Delegate to the default impl to visit any nested functions.
        visit_mut::visit_macro_mut(self, node);
    }
}

impl Parse for HtmlIterable {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let for_token = input.parse::<Token![for]>()?;

        match input.parse() {
            Ok(mut expr) => {
                HtmlInnerModifier.visit_expr_mut(&mut expr);
                Ok(HtmlIterable(expr))
            }
            Err(err) => {
                if err.to_string().starts_with("unexpected end of input") {
                    Err(syn::Error::new_spanned(
                        for_token,
                        "expected expression after `for`",
                    ))
                } else {
                    Err(err)
                }
            }
        }
    }
}

impl ToTokens for HtmlIterable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.0;
        let new_tokens = quote_spanned! {expr.span()=> {
            let mut __yew_vlist = ::yew::virtual_dom::VList::default();
            let __yew_nodes: &mut ::std::iter::Iterator<Item = _> = &mut(#expr);
            for __yew_node in __yew_nodes.into_iter() {
                __yew_vlist.add_child(__yew_node.into());
            }
            ::yew::virtual_dom::VNode::from(__yew_vlist)
        }};

        tokens.extend(new_tokens);
    }
}
