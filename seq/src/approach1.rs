use std::result;

use quote::ToTokens;



#[derive(Debug)]
pub struct SeqMacroInput {
    from: usize,
    to: usize,
    tt: proc_macro2::TokenStream,
    ident: syn::Ident
}


#[derive(Debug, Copy, Clone)]
enum Mode {
    RepalceIdent(usize),
    ReplaceSequence,
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
        self.expand(self.tt.clone()).into()
    }
}


impl SeqMacroInput {
   fn expand(&self, stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let (out, mutated) = self.expand_stream(stream.clone(), Mode::ReplaceSequence);
        if mutated {
            return out
        }

        (self.from..self.to).map(|i| {
            self.expand_stream(stream.clone(), Mode::RepalceIdent(i)).0
        }).collect::<proc_macro2::TokenStream>()
    }


    fn expand_stream(&self, stream: proc_macro2::TokenStream, mode: Mode
        ) -> (proc_macro2::TokenStream, bool) {
        let mut output = proc_macro2::TokenStream::new();
        let mut mutated = false;
        let mut tst = stream.into_iter();

        while let Some(tt) = tst.next() {
            let result: proc_macro2::TokenStream = self.expand_tt(tt, &mut tst, &mut mutated, mode);
            output.extend(result);
        }

        (output, mutated)
    }
    
    /// Expands the token tree
    fn expand_tt(
        &self, tt: proc_macro2::TokenTree, rest: &mut proc_macro2::token_stream::IntoIter, 
        mutated: &mut bool, mode: Mode,
    ) -> proc_macro2::TokenStream {
        let tt = match tt {
         proc_macro2::TokenTree::Group(g) => {
            let (expanded, g_mutated) = self.expand_stream(g.stream(), mode);
            let mut expanded = proc_macro2::Group::new(g.delimiter(), expanded);
            *mutated |= g_mutated;

            expanded.set_span(g.span());
            proc_macro2::TokenTree::Group(expanded).into()
         }        
         proc_macro2::TokenTree::Ident(ident) if ident == self.ident => {
            if let Mode::RepalceIdent(i) = mode {
                let mut lit = proc_macro2::Literal::usize_unsuffixed(i);
                lit.set_span(ident.span());
                *mutated = true;
    
                let t = syn::parse2(quote::quote_spanned! {ident.span()=> #lit}).unwrap();
                proc_macro2::TokenTree::Literal(t)
            } else {
                proc_macro2::TokenTree::Ident(ident)
            }
         }
         
         proc_macro2::TokenTree::Ident(mut ident) => {
            let mut peek = rest.clone();

            match (mode, peek.next(), peek.next()) {
                (
                    Mode::RepalceIdent(i), 
                    Some(proc_macro2::TokenTree::Punct(ref punct)), 
                    Some(proc_macro2::TokenTree::Ident(ref ident2))
                ) if punct.as_char() == '~' && ident2 == &self.ident => {
                    let value = format!("{}{}", ident, i);
                    ident = proc_macro2::Ident::new(value.as_str(), ident.span());
                    *rest = peek.clone();
                    *mutated = true;


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
        
        // this expands scenarios such as on test5: where the par to be repeated is wrapped as a group
         proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '#' => {
             
             if let Mode::ReplaceSequence = mode {
                let mut peek = rest.clone();
                match (peek.next(), peek.next()) {
                    (   
                        Some(proc_macro2::TokenTree::Group(ref g)), 
                        Some(proc_macro2::TokenTree::Punct(ref star))
                    )
                    if g.delimiter() == proc_macro2::Delimiter::Parenthesis && star.as_char() == '*' => 
                    {
                        *mutated = true;
                        *rest = peek;


                        return (self.from..self.to).map(|i| {
                            self.expand_stream(g.stream(), Mode::RepalceIdent(i))
                        }).map(|(ts, _)| ts).collect::<proc_macro2::TokenStream>()
                    }
                    _ => {}
                }
            }
            proc_macro2::TokenTree::Punct(punct)
         }
         tt => tt
        };

        std::iter::once(tt).collect()
    }
}

