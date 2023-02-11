mod eight_fish_model;
use eight_fish_model::expand_eight_fish_model;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(EightFishModel)]
pub fn eight_fish_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_eight_fish_model(input).into()
}
