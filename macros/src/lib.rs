mod command;
mod slash_choice_parameter;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn command(args: TokenStream, function: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Vec<syn::NestedMeta>);
    let args = match <command::CommandAttrArgs as darling::FromMeta>::from_list(&args) {
        Ok(x) => x,
        Err(e) => return e.write_errors().into(),
    };

    let function = syn::parse_macro_input!(function as syn::ItemFn);

    match command::command(args, function) {
        Ok(x) => x,
        Err(e) => e.write_errors().into(),
    }
}

#[proc_macro_derive(SlashChoiceParameter, attributes(name))]
pub fn slash_choice_parameter(input: TokenStream) -> TokenStream {
    let enum_ = syn::parse_macro_input!(input as syn::DeriveInput);

    match slash_choice_parameter::slash_choice_parameter(enum_) {
        Ok(x) => x,
        Err(e) => e.write_errors().into(),
    }
}
