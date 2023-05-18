mod approach0;
mod approach1;

use approach0::{Seq};
use approach1::SeqMacroInput;

use proc_macro::TokenStream;
use syn::{Visibility, parse_macro_input, Token};




#[proc_macro]
pub fn seq_macro_input(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as SeqMacroInput);
    input.into()
}





#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Seq);
    input.into()
}

