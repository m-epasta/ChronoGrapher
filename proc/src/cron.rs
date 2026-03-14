use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use chronographer_base::task::trigger::schedule::cron::{TaskScheduleCron, CronField};
use std::str::FromStr;

pub fn cron(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let s = lit.value();
    
    let cron = match TaskScheduleCron::from_str(&s) {
        Ok(c) => c,
        Err(e) => {
            return syn::Error::new(lit.span(), e.to_string()).to_compile_error().into();
        }
    };
    
    let sec = field_to_tokens(&cron.seconds);
    let min = field_to_tokens(&cron.minute);
    let hour = field_to_tokens(&cron.hour);
    let dom = field_to_tokens(&cron.day_of_month);
    let mon = field_to_tokens(&cron.month);
    let dow = field_to_tokens(&cron.day_of_week);

    let expanded = quote! {
        ::chronographer::task::trigger::schedule::cron::TaskScheduleCron::new([
            #sec, #min, #hour, #dom, #mon, #dow
        ])
    };

    expanded.into()
}

fn field_to_tokens(field: &CronField) -> TokenStream2 {
    match field {
        CronField::Wildcard => quote!(::chronographer::task::trigger::schedule::cron::CronField::Wildcard),
        CronField::Exact(v) => quote!(::chronographer::task::trigger::schedule::cron::CronField::Exact(#v)),
        CronField::Range(s, e) => quote!(::chronographer::task::trigger::schedule::cron::CronField::Range(#s, #e)),
        CronField::Step(base, step) => {
            let base_tokens = field_to_tokens(base);
            quote!(::chronographer::task::trigger::schedule::cron::CronField::Step(Box::new(#base_tokens), #step))
        }
        CronField::List(list) => {
            let items = list.iter().map(|f| field_to_tokens(f));
            quote!(::chronographer::task::trigger::schedule::cron::CronField::List(::std::boxed::Box::from([#(#items),*])))
        }
        CronField::Unspecified => quote!(::chronographer::task::trigger::schedule::cron::CronField::Unspecified),
        CronField::Last(opt) => {
            let opt_tokens = match opt {
                Some(v) => quote!(Some(#v)),
                None => quote!(None),
            };
            quote!(::chronographer::task::trigger::schedule::cron::CronField::Last(#opt_tokens))
        }
        CronField::NearestWeekday(v) => quote!(::chronographer::task::trigger::schedule::cron::CronField::NearestWeekday(#v)),
        CronField::NthWeekday(d, n) => quote!(::chronographer::task::trigger::schedule::cron::CronField::NthWeekday(#d, #n)),
    }
}
