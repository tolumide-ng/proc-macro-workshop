use quote::ToTokens;



#[derive(Debug )]
pub struct Seq {
    ident: syn::Ident,
    from: usize,
    to: usize,
    stmts: Vec<syn::Stmt>,
    brace_token: syn::token::Brace,
}

impl syn::parse::Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![in]>()?;
        let from = input.parse::<syn::LitInt>()?;
        let from = from.base10_parse::<usize>()?;
        input.parse::<syn::Token![..]>()?;
        let to = input.parse::<syn::LitInt>()?;
        let to = to.base10_parse::<usize>()?;
        
        let content;
        // let body = input.parse::<syn::Block>()?;
        let brace_token = syn::braced!(content in input);
        // let inner_attrs = content.call(syn::Attribute::parse_inner)?;
        let stmts = content.call(syn::Block::parse_within)?;

        
        Ok(Self { ident, from, to, stmts, brace_token })
    }
    
}


impl Into<proc_macro::TokenStream> for Seq {
    fn into(self) -> proc_macro::TokenStream {
        let new_stmts = self.stmts.iter().enumerate().map(|(i, s)| -> proc_macro::TokenStream  {
            let stream: proc_macro2::TokenStream = proc_macro2::TokenStream::from(s.into_token_stream());
            self.expand_stream(stream, i).into()

        }).collect::<Vec<proc_macro::TokenStream>>();

        let mac = new_stmts[0].clone();
        mac
    }
}


impl Seq {
    pub fn expand_stream(&self, stream: proc_macro2::TokenStream, i: usize) -> proc_macro2::TokenStream {
        stream.into_iter().map(|tt| {self.expand_tt(tt, i)}).collect()
    }

    pub fn expand_tt(&self, tt: proc_macro2::TokenTree, i: usize) -> proc_macro2::TokenTree {
        match tt {
            proc_macro2::TokenTree::Group(g) => {
                let mut expanded = proc_macro2::Group::new(g.delimiter(), self.expand_stream(g.stream(), i));
                expanded.set_span(g.span());
                proc_macro2::TokenTree::from(expanded)
            },

            proc_macro2::TokenTree::Ident(ident) if ident == self.ident => {
                let id = quote::quote_spanned! {ident.span()=> #i};
                let t = syn::parse2(id).unwrap();
                proc_macro2::TokenTree::Literal(t)
            },
            tt => tt
        }
    }
}