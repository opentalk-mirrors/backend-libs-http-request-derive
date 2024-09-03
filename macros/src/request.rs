// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use darling::{ast, util, FromDeriveInput, FromField};
use proc_macro2::Span;
use quote::quote;

use self::format::expand_format_string;
use crate::http_method::HttpMethod;

mod format;

pub const ATTRIBUTE_NAME: &str = "http_request";

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(http_request))]
struct HttpRequestParameters {
    method: HttpMethod,
    response: syn::Path,
    path: syn::LitStr,
    data: ast::Data<util::Ignored, FieldReceiver>,
}

#[derive(Debug, FromField)]
#[darling(attributes(http_request))]
struct FieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    #[darling(default)]
    body: bool,

    #[darling(default)]
    query: bool,

    #[darling(default)]
    header: bool,
}

enum FieldIdentifier {
    Index(syn::Index),
    Named(syn::Ident),
}

impl quote::ToTokens for FieldIdentifier {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldIdentifier::Index(index) => index.to_tokens(tokens),
            FieldIdentifier::Named(ident) => ident.to_tokens(tokens),
        };
    }
}

impl From<syn::Ident> for FieldIdentifier {
    fn from(value: syn::Ident) -> Self {
        FieldIdentifier::Named(value)
    }
}

impl From<usize> for FieldIdentifier {
    fn from(value: usize) -> Self {
        FieldIdentifier::Index(value.into())
    }
}

struct Fields {
    body: Option<(FieldIdentifier, syn::Type)>,
    query: Option<(FieldIdentifier, syn::Type)>,
    header: Option<(FieldIdentifier, syn::Type)>,
}

fn extract_fields(fields: &ast::Fields<FieldReceiver>) -> Result<Fields, syn::Error> {
    let mut body = None;
    let mut query = None;
    let mut header = None;

    fn update_item(
        extracted: &mut Option<(FieldIdentifier, syn::Type)>,
        (index, field): (usize, &FieldReceiver),
        name: &str,
    ) -> Result<(), syn::Error> {
        if extracted.is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("#[{ATTRIBUTE_NAME}({name})] found on multiple struct fields"),
            ));
        }
        let identifier = field
            .ident
            .clone()
            .map(FieldIdentifier::from)
            .unwrap_or_else(|| FieldIdentifier::from(index));
        *extracted = Some((identifier, field.ty.clone()));
        Ok(())
    }

    for (index, field) in fields.iter().enumerate() {
        if field.body {
            update_item(&mut body, (index, field), "body")?;
        }

        if field.query {
            update_item(&mut query, (index, field), "query")?;
        }

        if field.header {
            update_item(&mut header, (index, field), "header")?;
        }
    }

    Ok(Fields {
        body,
        query,
        header,
    })
}

pub(crate) fn request(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match try_to_http_request(ast) {
        Ok(k) => k,
        Err(e) => proc_macro::TokenStream::from(e.to_compile_error()),
    }
}

fn try_to_http_request(ast: syn::DeriveInput) -> Result<proc_macro::TokenStream, syn::Error> {
    let parameters = HttpRequestParameters::from_derive_input(&ast)?;

    impl_request(&ast, parameters)
}

fn impl_request(
    input: &syn::DeriveInput,
    HttpRequestParameters {
        method,
        response,
        path,
        data,
    }: HttpRequestParameters,
) -> Result<proc_macro::TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let Some(fields) = data.take_struct() else {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("#[{ATTRIBUTE_NAME}(...)] can only be used with structs"),
        ));
    };

    let path = expand_format_string(ATTRIBUTE_NAME, &path.value(), &fields)?;
    let fields = extract_fields(&fields)?;

    let (query_fn, query_type) = match fields.query {
        Some((name, ty)) => (
            quote! {
                fn query(&self) -> Option<&Self::Query> {
                    Some(&self.#name)
                }
            },
            quote! { #ty },
        ),
        None => (
            quote! {
                fn query(&self) -> Option<&Self::Query> {
                    None
                }
            },
            quote! { () },
        ),
    };

    let (body_fn, body_type) = match fields.body {
        Some((name, ty)) => (
            quote! {
                fn body(&self) -> Option<&Self::Body> {
                    Some(&self.#name)
                }
            },
            quote! { #ty },
        ),
        None => (
            quote! {
                fn body(&self) -> Option<&Self::Body> {
                    None
                }
            },
            quote! { () },
        ),
    };

    let header_fn = match fields.header {
        Some((name, ty)) => {
            let syn::Type::Path(ref type_path) = ty else {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Attribute #[{ATTRIBUTE_NAME}(header)] must be applied to valid type",),
                ));
            };

            if !type_path
                .path
                .segments
                .iter()
                .any(|segment| segment.ident == "HeaderMap")
            {
                return Err(syn::Error::new(Span::call_site(), format!("Attribute #[{ATTRIBUTE_NAME}(header)] must be applied to field of type http::HeaderMap",)));
            }

            quote! {
                fn apply_headers(&self, headers: &mut http::HeaderMap) {
                    use ::http_request_derive::HttpRequestBody as _;

                    headers.extend(self.#name.clone());
                    if let Some(body) = self.body() {
                        body.apply_headers(headers);
                    }
                }
            }
        }
        None => quote! {
            fn apply_headers(&self, headers: &mut http::HeaderMap) {
                use ::http_request_derive::HttpRequestBody as _;

                if let Some(body) = self.body() {
                    body.apply_headers(headers);
                }
            }
        },
    };

    let expanded = quote! {
        impl #generics ::http_request_derive::HttpRequest for #ident #generics {
            type Response = #response;
            type Query = #query_type;
            type Body = #body_type;

            const METHOD: ::http_request_derive::__exports::http::Method = ::http_request_derive::__exports::http::Method::#method;

            fn path(&self) -> std::string::String {
                #path.into()
             }

            #query_fn

            #body_fn

            #header_fn
        }
    };

    Ok(proc_macro::TokenStream::from(expanded))
}
