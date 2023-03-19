mod parse;
mod result;

use indexmap::IndexMap;
use parse::{parse_dir, ResourceMap};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{ParseStream, Parser},
    Error, ItemStruct, LitStr, Result, Token,
};

#[proc_macro_attribute]
pub fn i18n(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    i18n_impl(attr.into(), item.into()).into()
}

fn i18n_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let resources = match parse_attr.parse2(attr) {
        Ok(resources_fn) => resources_fn,
        Err(err) => return Error::into_compile_error(err),
    };

    let mut item_struct = match syn::parse2::<ItemStruct>(item) {
        Ok(item_struct) => item_struct,
        Err(err) => return Error::into_compile_error(err),
    };

    let i18n_struct_name = &item_struct.ident;
    let i18n_resources_name = Ident::new(
        &(item_struct.ident.to_string() + "Resources"),
        Span::call_site(),
    );
    let i18n_resource_name = Ident::new(
        &(item_struct.ident.to_string() + "Resource"),
        Span::call_site(),
    );
    let i18n_message_name = Ident::new(
        &(item_struct.ident.to_string() + "Message"),
        Span::call_site(),
    );

    // ロケールのリソースを保持するフィールドを追加
    match item_struct.fields {
        syn::Fields::Named(ref mut fields_named) => {
            fields_named.named.push(
                syn::Field::parse_named
                    .parse2(quote! { pub resources: &'static #i18n_resources_name })
                    .unwrap(),
            );
        }
        _ => {}
    }

    // リソースの初期化処理
    let resources_insert = resources.iter().map(|(locale, resources)| {
        let resource_insert = resources.iter().map(|(key, messages)| {
            let message_new = messages.iter().map(|(message, ranges)| {
                let message_range = ranges.iter().map(|(i, range)| {
                    let start = range.start;
                    let end = range.end;
                    let dotdot = Token![..](Span::call_site());
                    quote! { #i => #start #dotdot #end }
                });

                quote! {
                    #i18n_message_name::new(#message, indexmap::indexmap! { #(#message_range),* })
                }
            });

            quote! {
                locale_map.insert(
                    #key,
                    #i18n_resource_name::new(vec![
                        #(#message_new),*
                    ]),
                );
            }
        });

        quote! {
            let mut locale_map: #i18n_resources_name = indexmap::IndexMap::new();
            #(#resource_insert)*
            map.insert(#locale, locale_map);
        }
    });

    let resources_init = quote! {
        let mut map: indexmap::IndexMap<&'static str, #i18n_resources_name> = indexmap::IndexMap::new();

        #(#resources_insert)*

        map
    };

    quote! {
        #item_struct

        pub type #i18n_resources_name = indexmap::IndexMap<&'static str, #i18n_resource_name>;

        #[derive(Debug)]
        pub struct #i18n_message_name {
            message: &'static str,
            ranges: indexmap::IndexMap<usize, std::ops::Range<usize>>,
        }

        impl #i18n_message_name {
            pub fn new(message: &'static str, ranges: indexmap::IndexMap<usize, std::ops::Range<usize>>) -> Self {
                Self { message, ranges }
            }

            pub fn get(&self) -> &'static str {
                self.message
            }

            pub fn replace(&self, reps: &Vec<&str>) -> String {
                let mut message = self.message.to_string();

                for (i, range) in self.ranges.iter() {
                    message.replace_range(range.clone(), reps[*i]);
                }

                message
            }
        }

        #[derive(Debug)]
        pub struct #i18n_resource_name {
            messages: Vec<#i18n_message_name>,
            messages_len: usize,
        }

        impl #i18n_resource_name {
            pub fn new(messages: Vec<#i18n_message_name>) -> Self {
                let messages_len = messages.len();
                Self {
                    messages,
                    messages_len,
                }
            }

            pub fn get(&self) -> &'static str {
                self.messages[0].get()
            }

            pub fn replace(&self, reps: &Vec<&str>) -> String {
                self.messages[0].replace(reps)
            }

            pub fn plural(&self, num: usize) -> &#i18n_message_name {
                let i;
                if self.messages_len == 1 {
                    i = 0;
                } else if self.messages_len == 2 {
                    if num == 1 {
                        i = 0;
                    } else {
                        i = 1;
                    }
                } else {
                    i = std::cmp::min(self.messages_len - 1, num)
                }

                &self.messages[i]
            }
        }

        impl #i18n_struct_name {
            pub fn new(locale: &str) -> Self {
                Self {
                    resources: Self::resources().get(locale).unwrap(),
                }
            }

            pub fn resources() -> &'static indexmap::IndexMap<&'static str, #i18n_resources_name> {
                static R: once_cell::sync::OnceCell<indexmap::IndexMap<&'static str, #i18n_resources_name>> = once_cell::sync::OnceCell::new();
                R.get_or_init(|| {
                    #resources_init
                })
            }

            pub fn t(&self, key: &str) -> &'static #i18n_resource_name {
                match self.resources.get(key) {
                    Some(v) => v,
                    None => {
                        panic!("Key `{}` not found", key)
                    }
                }
            }
        }

    }
}

fn parse_attr(input: ParseStream) -> Result<IndexMap<String, ResourceMap>> {
    let lit: LitStr = input.parse()?;
    let dir = lit.value();
    input.parse::<Token![,]>()?;
    let lit: LitStr = input.parse()?;
    let source_locale = lit.value();

    parse_dir(&dir, &source_locale).or_else(|err| Err(Error::new(input.span(), err)))
}
