use darling::ast;
use proc_macro2::Span;
use quote::quote;

use super::FieldReceiver;

pub fn expand_format_string(
    attribute_name: &str,
    fmt: &str,
    fields: &ast::Fields<FieldReceiver>,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    match fields.style {
        ast::Style::Struct => {
            let field_args = fields.iter().filter_map(|field| {
                field.ident.as_ref().and_then(|field| {
                    if fmt.contains(&format!("{{{field}}}")) {
                        Some(quote! {
                            #field=self.#field
                        })
                    } else {
                        None
                    }
                })
            });

            Ok(quote! {
                format!(#fmt, #(#field_args),*)
            })
        }
        ast::Style::Tuple => {
            // A very naive and probably fragile way to get the number of arguments in the format string.
            // Should work for most cases, but could be improved someday.
            let num_arguments = fmt.replace("{{", "").replace("}}", "").matches('{').count();

            if num_arguments > fields.len() {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Too many arguments in #[{attribute_name}] format string."),
                ));
            }

            let field_args = (0..num_arguments).map(|i| {
                let index = syn::Index::from(i);
                quote! {
                    self.#index
                }
            });

            Ok(quote! {
                format!(#fmt, #(#field_args),*)
            })
        }
        ast::Style::Unit => Ok(quote! {format!(#fmt)}),
    }
}
