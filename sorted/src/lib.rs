use proc_macro::TokenStream;
use syn::{parse_macro_input};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut out = input.clone();

    let item = parse_macro_input!(input as syn::Item);
    assert!(args.is_empty());

    if let Err(e) = sort_variants(item) {
        out.extend(proc_macro::TokenStream::from(e.to_compile_error()));
    }

    out
}

fn sort_variants(input: syn::Item) -> Result<(), syn::Error> {

    if let syn::Item::Enum(item) = input {
        let value = &item.variants;
        let original_idents = value.into_iter().map(|x| x.ident.clone()).collect::<Vec<_>>();
        let mut sorted_idents = original_idents.clone(); 
        sorted_idents.sort();

        for (original, sorted) in original_idents.iter().zip(sorted_idents) {

            if original != &sorted {
                let err_msg = format!("{} should sort before {}", sorted, original);
                return Err(syn::Error::new(sorted.span(), err_msg))
            }
        }


        Ok(())
    } else {        
        Err(syn::Error::new(proc_macro2::Span::call_site(), "expected enum or match expression"))
    }
}



use syn::visit_mut::{VisitMut};

#[derive(Debug, Default)]
struct LexiographicMatching {
    errors: Vec<syn::Error>
}

impl syn::visit_mut::VisitMut for LexiographicMatching {
    fn visit_expr_match_mut(&mut self, m: &mut syn::ExprMatch) {
        if m.attrs.iter().any(|x| x.path().is_ident("sorted")) {
            m.attrs.retain(|a| !a.meta.path().is_ident("sorted"));

            let paths = m.arms.iter().filter_map(|a| get_arm_path(&a.pat)).collect::<Vec<_>>();
            let mut sorted_paths = paths.clone();
            sorted_paths.sort_by(|a, b| a.1.cmp(&b.1));
            // sorted_paths.sort_by(|a, b| a.get_ident().unwrap().cmp(&b.get_ident().unwrap()));
            
            for ((original, o_name), (sorted, s_name)) in paths.iter().zip(sorted_paths) {
                if original != &sorted {
                    let msg = syn::Error::new_spanned(
                        sorted, format!("{} should sort before {}", s_name, o_name)
                    );
                    self.errors.push(msg);
                    return
                }

            }
        }


        syn::visit_mut::visit_expr_match_mut(self, m);
    }
}

fn get_arm_path(arm: &syn::Pat) -> Option<(&syn::Path, String)> {
    match arm {
        syn::Pat::TupleStruct(ref t) => {Some((&t.path, get_arm_name(&t.path)))},
        syn::Pat::Path(ref p) => Some((&p.path, get_arm_name(&p.path))),
        syn::Pat::Struct(ref s) => Some((&s.path, get_arm_name(&s.path))),
        _ => None,
    }
}

fn get_arm_name(arm: &syn::Path) -> String {
    arm.segments.iter().map(|a| a.ident.to_string()).collect::<Vec<_>>().join("::")
}



#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(input as syn::ItemFn);

    let mut lm = LexiographicMatching::default();
    lm.visit_item_fn_mut(&mut f);

    let mut ts = quote::quote!(#f);
    ts.extend(lm.errors.into_iter().map(|e| e.to_compile_error()));

    
    ts.into()
}


// fn sort_match(input: syn::ItemFn) -> Result<(), syn::Error> {}