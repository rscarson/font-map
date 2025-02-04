use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use std::str::FromStr;
use syn::Ident;

use crate::font::{Font, Glyph, StringKind};

use super::docstring::DocstringExt;
use super::to_ident::to_identifiers;

/// Generates code to represent the contents of this font
///
/// Structured as a giant enum of a unique type
pub struct FontEnum {
    name: Ident,
    comments: Vec<String>,
    family: Option<String>,
    glyphs: HashMap<String, Glyph>,
    injected: Option<TokenStream>,
}
impl FontEnum {
    /// Create a new `FontEnum` from a font file
    pub fn from_font(name: &str, font: &Font) -> Self {
        let name = Ident::new(name, Span::call_site());
        let family = font.string(StringKind::FontFamily).map(ToString::to_string);
        let comments = font.gen_docblock();
        let glyphs = to_identifiers(font.glyphs());

        Self {
            name,
            comments,
            family,
            glyphs,
            injected: None,
        }
    }

    /// Inject additional code into the generated enum
    ///
    /// Code will be in the `impl` section of the new type
    pub fn inject(&mut self, tokens: TokenStream) {
        self.injected = Some(tokens);
    }

    /// Generate the code for the enum
    #[must_use]
    pub fn codegen(&self) -> TokenStream {
        let identifier = &self.name;
        let comments = &self.comments;
        let font_family: Vec<_> = self.family.iter().collect();
        let variants: Vec<_> = self
            .glyphs
            .iter()
            .map(|(name, glyph)| Self::codegen_entry(name, glyph))
            .collect();

        let codepoints: Vec<_> = self.glyphs.values().map(Glyph::codepoint).collect();
        let names: Vec<_> = self.glyphs.values().map(Glyph::name).collect();
        let codepoints_len = self.glyphs.len();

        let injection: Vec<_> = self.injected.iter().collect();

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
            impl #identifier {
                #(
                    /// The font family of the glyph
                    #[allow(dead_code)]
                    pub const FONT_FAMILY: &str = #font_family;
                )*

                /// The number of glyphs in the font
                #[allow(dead_code)]
                pub const GLYPHS: usize = #codepoints_len;

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

    fn codegen_entry(name: &str, glyph: &Glyph) -> TokenStream {
        let identifier = Ident::new(name, Span::call_site());

        let name = glyph.name();
        let codepoint = glyph.codepoint();

        let comments = [
            format!("`{name} (U+{codepoint:04X})`  "),
            format!("Unicode range: {}", glyph.unicode_range()),
        ];

        #[cfg(not(feature = "extended-svg"))]
        let extended_svg = quote! {};
        #[cfg(feature = "extended-svg")]
        let extended_svg = {
            if let Ok(url) = glyph.svg_dataimage_url() {
                let link = format!("![Preview Glyph]({url})");
                quote! {
                    #[doc = ""]
                    #[doc = #link]
                }
            } else {
                quote! {}
            }
        };

        let codepoint = format!("{codepoint:#x}");
        let codepoint = TokenStream::from_str(&codepoint).unwrap();

        quote! {
            #( #[doc = #comments] )*
            #extended_svg
            #identifier = #codepoint,
        }
    }
}
