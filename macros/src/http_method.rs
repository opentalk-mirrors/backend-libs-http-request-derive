use std::str::FromStr;

use darling::FromMeta;
use proc_macro2::Span;
use quote::TokenStreamExt;
use strum::{EnumString, IntoStaticStr, VariantNames};

#[derive(Debug, Clone, Copy, EnumString, VariantNames, IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

impl FromMeta for HttpMethod {
    fn from_string(value: &str) -> darling::Result<Self> {
        HttpMethod::from_str(value.to_uppercase().as_str())
            .map_err(|_| darling::Error::unknown_value(value))
    }
}

impl quote::ToTokens for HttpMethod {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let word: &'static str = self.into();

        tokens.append(proc_macro2::Ident::new(word, Span::call_site()));
    }
}
