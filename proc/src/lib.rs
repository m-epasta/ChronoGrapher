mod cron;
mod every;
mod utils;

use proc_macro::TokenStream;

#[proc_macro]
pub fn every(input: TokenStream) -> TokenStream {
    every::every(input)
}

#[proc_macro]
pub fn cron(input: TokenStream) -> TokenStream {
    cron::cron(input)
}
