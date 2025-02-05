use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use std::str::FromStr;
use syn::Ident;

use crate::font::Glyph;

/// Describes a single category of glyphs in a font
#[derive(Debug, Clone)]
pub struct FontCategoryDesc {
    identifier: Ident,
    comments: Vec<String>,
    glyphs: HashMap<String, Glyph>,
}
impl FontCategoryDesc {
    /// Create a new category from a name and a list of glyphs
    pub fn new(name: &str, glyphs: HashMap<String, Glyph>) -> Self {
        let identifier = Ident::new(name, Span::call_site());
        let comment = format!(
            "Contains the {} glyphs in the `{}` category",
            glyphs.len(),
            name.to_lowercase(),
        );

        Self {
            identifier,
            comments: vec![comment],
            glyphs,
        }
    }

    /// Get the name of the category
    pub fn name(&self) -> &Ident {
        &self.identifier
    }

    pub fn set_name(&mut self, name: &str) {
        self.identifier = Ident::new(name, Span::call_site());
    }

    /// Get the comments of this category
    pub fn comments(&self) -> &[String] {
        &self.comments
    }

    /// Inject additional comments into the generated category
    pub fn set_comments(&mut self, comments: impl IntoIterator<Item = String>) {
        self.comments = comments.into_iter().collect();
    }

    /// Generates the code for this category
    ///
    /// Optionally, you can inject additional code into the generated category's impl
    #[allow(unused_mut)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn codegen(&self, extra_impl: Option<TokenStream>) -> TokenStream {
        let identifier = &self.identifier;
        let comments = &self.comments;
        let injection = extra_impl.iter();
        let n_glyphs = self.glyphs.len();

        let codepoints = self.glyphs.values().map(Glyph::codepoint);
        let names = self.glyphs.values().map(Glyph::name);

        let variants = self.glyphs.iter().map(|(name, glyph)| {
            let identifier = Ident::new(name, Span::call_site());
            let name = glyph.name();
            let codepoint = glyph.codepoint();

            let mut comments = vec![
                format!("`{name} (U+{codepoint:04X})`  "),
                format!("Unicode range: {}", glyph.unicode_range()),
            ];

            #[cfg(feature = "extended-svg")]
            {
                if let Ok(url) = glyph.svg_dataimage_url() {
                    let link = format!("\n![Preview Glyph]({url})");
                    comments.push(link);
                }
            }

            let codepoint = format!("{codepoint:#x}");
            let codepoint = TokenStream::from_str(&codepoint).unwrap();

            quote! {
                #( #[doc = #comments] )*
                #identifier = #codepoint,
            }
        });

        quote! {
            #[allow(rustdoc::bare_urls)]
            #( #[doc = #comments] )*
            #[derive(Debug, Clone, Copy)]
            #[rustfmt::skip]
            #[repr(u32)]
            pub enum #identifier {
                #( #variants )*
            }

            #[rustfmt::skip]
            #[allow(dead_code)]
            impl #identifier {
                /// The total number of glyphs in this enum
                pub const TOTAL_GLYPHS: usize = #n_glyphs;

                /// Returns the postscript name of the glyph
                #[allow(clippy::too_many_lines)]
                #[allow(clippy::match_same_arms)]
                #[must_use]
                pub fn name(&self) -> &'static str {
                    match *self as u32 {
                        #( #codepoints => #names, )*
                        _ => ".notdef",
                    }
                }

                #(
                    #injection
                )*
            }

            impl From<#identifier> for char {
                fn from(value: #identifier) -> Self {
                    std::char::from_u32(value as u32).unwrap_or(char::REPLACEMENT_CHARACTER)
                }
            }

            impl From<&#identifier> for char {
                fn from(value: &#identifier) -> Self {
                    (*value).into()
                }
            }

            impl From<#identifier> for u32 {
                fn from(value: #identifier) -> Self {
                    value as u32
                }
            }

            impl From<&#identifier> for u32 {
                fn from(value: &#identifier) -> Self {
                    *value as u32
                }
            }

            impl std::fmt::Display for #identifier {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", char::from(*self))
                }
            }
        }
    }
}

impl From<&FontCategoryDesc> for TokenStream {
    #[allow(unused_mut)]
    fn from(value: &FontCategoryDesc) -> Self {
        value.codegen(None)
    }
}
