use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
use syn::{FnArg, ItemFn, Pat};

pub struct ContextMethods {
    methods: Vec<ItemFn>,
}

impl Parse for ContextMethods {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut methods = Vec::new();
        while !input.is_empty() {
            methods.push(input.parse()?);
        }
        Ok(Self { methods })
    }
}

impl ContextMethods {
    pub fn generate(self) -> TokenStream {
        let (prefix_methods, application_methods): (Vec<_>, Vec<_>) = self
            .methods
            .iter()
            .map(|method| {
                let attrs = &method.attrs;
                let vis = &method.vis;
                let sig = &method.sig;

                let fn_name = &method.sig.ident;
                let args = method.sig.inputs.iter().filter_map(|arg| match arg {
                    FnArg::Typed(pat) if let Pat::Ident(ident) = &*pat.pat => Some(&ident.ident),
                    _ => None,
                });
                let r#await = method.sig.asyncness.map(|_| quote::quote! { .await });
                let fn_call = quote::quote! {
                    #fn_name(#(#args),*) #r#await
                };

                (
                    quote::quote! {
                        #(#attrs)*
                        #vis #sig { crate::Context::Prefix(self).#fn_call }
                    },
                    quote::quote! {
                        #(#attrs)*
                        #vis #sig { crate::Context::Application(self).#fn_call }
                    },
                )
            })
            .unzip();

        let methods = self.methods;
        TokenStream::from(quote::quote! {
            impl<'a, U: Send + Sync + 'static, E> Context<'a, U, E> {
                #(#methods)*
            }

            impl<'a, U: Send + Sync + 'static, E> crate::PrefixContext<'a, U, E> {
                #(#prefix_methods)*
            }

            impl<'a, U: Send + Sync + 'static, E> crate::ApplicationContext<'a, U, E> {
                #(#application_methods)*
            }
        })
    }
}
