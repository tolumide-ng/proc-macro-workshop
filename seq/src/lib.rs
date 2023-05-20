mod approach0;
mod approach1;

use approach1::SeqMacroInput;
use syn::{parse_macro_input};


#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as SeqMacroInput);
    input.into()
}





// #[proc_macro]
// pub fn seq(input: TokenStream) -> TokenStream {
//     let input = syn::parse_macro_input!(input as Seq);
//     input.into()
// }

