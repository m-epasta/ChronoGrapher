use chronographer_base::task::trigger::schedule::{CronField, TaskScheduleCron};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::str::FromStr;
use syn::{
    LitStr,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct Cron {
    schedule: TaskScheduleCron,
}

fn tokenize(field: &CronField) -> TokenStream2 {
    match field {
        CronField::Wildcard => quote! { ::chronographer::task::CronField::Wildcard },
        CronField::Exact(v) => quote! { ::chronographer::task::CronField::Exact(#v) },
        CronField::Range(start, end) => {
            quote! { ::chronographer::task::CronField::Range(#start, #end) }
        }
        CronField::Step(base, step) => {
            let base_tokens = tokenize(base);
            quote! { ::chronographer::task::CronField::Step(Box::new(#base_tokens), #step) }
        }
        CronField::List(fields) => {
            let fields_tokens: Vec<_> = fields.iter().map(tokenize).collect();
            quote! { ::chronographer::task::CronField::List(vec![#(#fields_tokens),*]) }
        }
        CronField::Unspecified => quote! { ::chronographer::task::CronField::Unspecified },
        CronField::Last(val) => {
            let val_tokens = if let Some(v) = val {
                quote! { Some(#v) }
            } else {
                quote! { None }
            };
            quote! { ::chronographer::task::CronField::Last(#val_tokens) }
        }
        CronField::NearestWeekday(v) => {
            quote! { ::chronographer::task::CronField::NearestWeekday(#v) }
        }
        CronField::NthWeekday(v1, v2) => {
            quote! { ::chronographer::task::CronField::NthWeekday(#v1, #v2) }
        }
    }
}

impl Parse for Cron {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cron_str = if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            lit.value()
        } else {
            let mut s = String::new();
            while !input.is_empty() {
                let tt: proc_macro2::TokenTree = input.parse()?;
                s.push_str(&tt.to_string());
                s.push(' ');
            }
            s.trim()
                .replace(" / ", "/")
                .replace(" , ", ",")
                .replace(" - ", "-")
                .replace(" # ", "#")
        };

        match TaskScheduleCron::from_str(&cron_str) {
            Ok(schedule) => Ok(Cron { schedule }),
            Err(e) => Err(syn::Error::new(
                input.span(),
                format!("Invalid CRON expression: {}", e),
            )),
        }
    }
}

pub fn cron(input: TokenStream) -> TokenStream {
    let cron_input = parse_macro_input!(input as Cron);
    let schedule = cron_input.schedule;
    let fields = schedule.fields();

    let fields_tokens: Vec<_> = fields.iter().map(|f| tokenize(f)).collect();

    let expanded = quote! {
        ::chronographer::task::TaskScheduleCron::new([
            #(#fields_tokens),*
        ])
    };

    TokenStream::from(expanded)
}
