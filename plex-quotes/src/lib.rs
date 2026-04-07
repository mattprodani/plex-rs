use proc_macro::TokenStream;
use quote::quote;
use rand::seq::SliceRandom;

const QUOTES: &[&str] = &[
    "Booting up...",
    "To infinity and beyond!",
    "I'm afraid I can't do that, Dave.",
    "Hello, IT. Have you tried turning it off and on again?",
    "A computer let me down; how can I ever trust it?",
    "There are 10 types of people in the world...",
    "It works on my machine.",
    "404 Quote Not Found",
    "May the Force be with you.",
    "sudo make me a sandwich",
];

#[proc_macro]
pub fn random_quote(_item: TokenStream) -> TokenStream {
    let mut rng = rand::thread_rng();
    let quote_str = QUOTES.choose(&mut rng).unwrap();
    let expanded = quote! { #quote_str };
    expanded.into()
}
