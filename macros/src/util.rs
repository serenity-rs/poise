// ngl this is ugly
// transforms a type of form `OuterType<T>` into `T`
pub fn extract_type_parameter<'a>(outer_type: &str, t: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(path) = t {
        if path.path.segments.len() == 1 {
            let path = &path.path.segments[0];
            if path.ident == outer_type {
                if let syn::PathArguments::AngleBracketed(generics) = &path.arguments {
                    if generics.args.len() == 1 {
                        if let syn::GenericArgument::Type(t) = &generics.args[0] {
                            return Some(t);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Converts None => `None` and Some(x) => `Some(#x)`
pub fn wrap_option<T: quote::ToTokens>(literal: Option<T>) -> syn::Expr {
    match literal {
        Some(literal) => syn::parse_quote! { Some(#literal) },
        None => syn::parse_quote! { None },
    }
}

/// Converts None => `None` and Some(x) => `Some(#map_path(#x))`
pub fn wrap_option_and_map<T: quote::ToTokens>(
    literal: Option<T>,
    map_path: impl quote::ToTokens,
) -> syn::Expr {
    match literal {
        Some(literal) => syn::parse_quote! { Some(#map_path(#literal)) },
        None => syn::parse_quote! { None },
    }
}

pub fn wrap_option_to_string<T: quote::ToTokens>(literal: Option<T>) -> syn::Expr {
    let to_string_path = quote::quote!(::std::string::ToString::to_string);
    wrap_option_and_map(literal, to_string_path)
}

/// Syn Fold to make all lifetimes 'static. Used to access trait items of a type without having its
/// concrete lifetime available
pub struct AllLifetimesToStatic;
impl syn::fold::Fold for AllLifetimesToStatic {
    fn fold_lifetime(&mut self, _: syn::Lifetime) -> syn::Lifetime {
        syn::parse_quote! { 'static }
    }
}

/// Darling utility type that accepts a list of things, e.g. `#[attr(thing1, thing2...)]`
#[derive(Debug, Clone)]
pub struct List<T>(pub Vec<T>);
impl<T: darling::FromMeta> darling::FromMeta for List<T> {
    fn from_list(items: &[::darling::ast::NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(|item| T::from_nested_meta(item))
            .collect::<darling::Result<Vec<T>>>()
            .map(Self)
    }
}
impl<T> Default for List<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Darling utility type that accepts a 2-tuple list of things, e.g. `#[attr(thing1, thing2)]`
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple2<T>(pub T, pub T);
impl<T: darling::FromMeta> darling::FromMeta for Tuple2<T> {
    fn from_list(items: &[::darling::ast::NestedMeta]) -> darling::Result<Self> {
        Ok(match items {
            [a, b] => Self(T::from_nested_meta(a)?, T::from_nested_meta(b)?),
            _ => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "expected two items `(\"a\", \"b\")`",
                )
                .into())
            }
        })
    }
}

pub fn tuple_2_iter_deref<'a, I: 'a, T: 'a, D: ?Sized + 'a>(
    iter: I,
) -> impl ExactSizeIterator<Item = Tuple2<&'a D>>
where
    I: IntoIterator<Item = &'a Tuple2<T>>,
    I::IntoIter: ExactSizeIterator,
    T: std::ops::Deref<Target = D>,
{
    iter.into_iter()
        .map(|Tuple2(t, v)| Tuple2(t.deref(), v.deref()))
}

pub fn iter_tuple_2_to_hash_map<I, T>(v: I) -> proc_macro2::TokenStream
where
    I: ExactSizeIterator<Item = Tuple2<T>>,
    T: quote::ToTokens,
{
    if v.len() == 0 {
        return quote::quote!(std::collections::HashMap::new());
    }

    let (keys, values) = v
        .into_iter()
        .map(|x| (x.0, x.1))
        .unzip::<_, _, Vec<_>, Vec<_>>();

    quote::quote! {
        std::collections::HashMap::from([
            #( (#keys.to_string(), #values.to_string()) ),*
        ])
    }
}
