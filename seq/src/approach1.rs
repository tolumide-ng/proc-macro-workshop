

#[derive(Debug)]
pub struct SeqMacroInput {
    from: usize,
    to: usize,
    tt: proc_macro2::TokenStream,
    ident: syn::Ident
}


impl syn::parse::Parse for SeqMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = syn::Ident::parse(input)?;
        let _in = <syn::Token![in]>::parse(input)?;
        let from_litint = syn::LitInt::parse(input)?;
        let from = from_litint.base10_parse::<usize>()?;
        let _dots = <syn::Token![..]>::parse(input)?;
        let to_litint = syn::LitInt::parse(input)?;
        let to = to_litint.base10_parse::<usize>()?;

        let content;
        syn::braced!(content in input);
        let tt = proc_macro2::TokenStream::parse(&content)?;
        Ok(SeqMacroInput { from, to, tt, ident })
    }
}


impl Into<proc_macro::TokenStream> for SeqMacroInput {
    fn into(self) -> proc_macro::TokenStream {
        (self.from..self.to).map(|i| -> proc_macro::TokenStream {
            self.expand_stream(self.tt.clone(), i).into()
        }
        )
            .collect::<proc_macro::TokenStream>()
    }
}


impl SeqMacroInput {
    fn expand_stream(&self, stream: proc_macro2::TokenStream, i: usize) -> proc_macro2::TokenStream {

        let mut tst = stream.into_iter();
        let mut output = proc_macro2::TokenStream::new();

        while let Some(tt) = tst.next() {
            let result: proc_macro2::TokenStream = self.expand_tt(tt, &mut tst, i).into();
            output.extend(result);
        }

        output
    }
    
    /// Expands the token tree
    fn expand_tt(&self, tt: proc_macro2::TokenTree, rest: &mut proc_macro2::token_stream::IntoIter, i: usize) -> proc_macro2::TokenTree {
        match tt {
         proc_macro2::TokenTree::Group(g) => {
            let mut expanded = proc_macro2::Group::new(g.delimiter(), self.expand_stream(g.stream(), i));
            expanded.set_span(g.span());

            proc_macro2::TokenTree::Group(expanded)
         }
         
         proc_macro2::TokenTree::Ident(ref ident) if ident == &self.ident => {
            let mut lit = proc_macro2::Literal::usize_unsuffixed(i);
            lit.set_span(ident.span());

            let t = syn::parse2(quote::quote_spanned! {ident.span()=> #lit}).unwrap();
            proc_macro2::TokenTree::Literal(t)
         }


         proc_macro2::TokenTree::Ident(mut ident) => {
            let mut peek = rest.clone();

            match (peek.next(), peek.next()) {
                (Some(proc_macro2::TokenTree::Punct(ref punct)), Some(proc_macro2::TokenTree::Ident(ref ident2)))
                if punct.as_char() == '~' && ident2 == &self.ident => {
                    let value = format!("{}{}", ident, i);

                    ident = proc_macro2::Ident::new(value.as_str(), ident.span());
                    *rest = peek.clone();


                    match (peek.next(), peek.next()) {
                        (Some(proc_macro2::TokenTree::Punct(_)), Some(proc_macro2::TokenTree::Ident(ref ident3))) 
                        if punct.as_char() == '~' => {
                            let value = format!("{}{}", ident, ident3);
                            ident = proc_macro2::Ident::new(value.as_str(), ident.span());

                            *rest = peek;

                        }
                         _ => {}
                    }
                }
                _ => {}
            }

            proc_macro2::TokenTree::Ident(ident)
         }
         tt => tt
        }
    }
}

