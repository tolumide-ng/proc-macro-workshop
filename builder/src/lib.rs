#![recursion_limit = "128"]

use proc_macro::TokenStream;
use syn::{self, parse_macro_input, DeriveInput};
use quote::{quote, format_ident, ToTokens};

fn ty_inner_wrapper<'a>(wrapper: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        let is_expected_wrapper = p.path.segments.len() == 1 && p.path.segments[0].ident == wrapper;

        if !is_expected_wrapper {
            return std::option::Option::None
        }

        if let syn::PathArguments::AngleBracketed(ref inner_type) = p.path.segments[0].arguments {
            let inner_arg = inner_type.args.first().unwrap();

            if let syn::GenericArgument::Type(ref t) = inner_arg {
                return std::option::Option::Some(t)
            }
        }
    }
    std::option::Option::None
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let bident = format_ident!("{}Builder", name);
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data {
        named
    } else {
        unimplemented!()
    };

    let optionized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;                      
        if ty_inner_wrapper("Option", &ty).is_some() || has_builder(&f).is_some() {
            return quote! { #name: #ty }
        }

        quote! {
            #name: std::option::Option<#ty>
        }
    });   

    let methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        let set_method = if let std::option::Option::Some(inner_ty) = ty_inner_wrapper("Option", &ty) {
            quote! {
                pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                    self.#name = std::option::Option::Some(#name);
                    self
                }
            }
        } else if has_builder(&f).is_some() {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = #name;
                    self
                }
            }
        } else {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = std::option::Option::Some(#name);
                    self
                }
            }
        };
        
        match extend_method(&f) {
            std::option::Option::None => set_method.into(),
            std::option::Option::Some((true, extend_method)) => extend_method,
            std::option::Option::Some((false, extend_method)) => quote! {
                #set_method
                #extend_method
            }.into()
        }

    });

    fn has_builder(f: &syn::Field) -> Option<&syn::MetaList> {
        for attr in &f.attrs {
            if let syn::Meta::List(meta) = &attr.meta {
                return std::option::Option::Some(meta)
            }
        }
        std::option::Option::None
    }

    fn extend_method(f: &syn::Field) -> Option<(bool, proc_macro2::TokenStream)> {
        let name = f.ident.as_ref().unwrap();

        let meta = has_builder(&f)?;
        let segments = &meta.path.segments;
        let tokens = &meta.tokens;

        let err = std::option::Option::Some((false, syn::Error::new_spanned(&meta, "expected `builder(each = \"...\")`").to_compile_error()));


        
        if segments.len() == 1 && &segments[0].ident.to_string() == "builder"  {
            let token_stream2 = proc_macro2::TokenStream::from(tokens.to_token_stream());
            let mut token_tree = token_stream2.to_token_stream().into_iter();

            let token_tree_ident = token_tree.next().unwrap();
            match token_tree_ident {
                proc_macro2::TokenTree::Ident(i) => {
                    if i != "each" {
                        return err
                    }

                    assert_eq!(i, "each")
                },
                tt => panic!("expected string, but found {}", tt)
            }

            let token_tree_punct = token_tree.next().unwrap();
            match token_tree_punct {
                proc_macro2::TokenTree::Punct(p) => {
                    if p.as_char() != '=' {
                        return err
                    }
                    assert_eq!(p.as_char(), '=')
                },
                tt => panic!("expected '=', but found {}", tt),
            }

            let token_tree_literal = token_tree.next().unwrap();
            match token_tree_literal {
                proc_macro2::TokenTree::Literal(l) => {
                    let value = l.to_string().replace("\"", "");
                    let arg = syn::Ident::new(&value, l.span());

                    let inner_ty = ty_inner_wrapper("Vec", &f.ty).unwrap();

                    let method = quote! {
                        pub fn #arg(&mut self, #arg: #inner_ty) -> &mut Self {
                            self.#name.push(#arg);
                            self
                        }
                    };

                    return std::option::Option::Some((arg == *name, method))
                }
                lit => panic!("expected string, but found {:?}", lit)
            }
        }
        std::option::Option::None
    }

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;

        if ty_inner_wrapper("Option", &f.ty).is_some() || has_builder(&f).is_some(){
            return quote! {
                #name: self.#name.clone()
            };
        }

        quote! {
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), "is not self"))?
        }
    });

    let build_empty = fields.iter().map(|f| {
        let name = &f.ident;

        if has_builder(&f).is_some() {
            return quote! { #name: Vec::new() }
        }

        quote! {
            #name: std::option::Option::None
        }
    });


    let expanded = quote!{
        pub struct #bident {
            #(#optionized,)*
        }

        impl #bident {
            #(#methods)*

            // #(#extend_methods)*

            pub fn build(&mut self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })
            }
        }


        impl #name {
            fn builder() -> #bident {
                #bident {
                    #(#build_empty,)*
                }
            }
        }
    };
    
    expanded.into()
}
