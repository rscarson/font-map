use crate::{
    codegen::to_ident::ToIdentExt,
    font::{Font, Glyph, StringKind},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::str::FromStr;
use syn::Ident;

mod to_ident;

/// Trait used to perform code generation for a font
pub trait FontCodegenExt {
    /// Generates an enum representing the glyphs in the font
    fn gen_enum(&self, name: &str) -> TokenStream;

    /// Generates a single entry in the enum for a glyph
    fn gen_enum_entry(glyph: &Glyph) -> TokenStream;

    /// Generates the comments for the enum
    fn gen_enum_comments(&self) -> Vec<String>;
}
impl FontCodegenExt for Font {
    fn gen_enum(&self, name: &str) -> TokenStream {
        let identifier = Ident::new(name, Span::call_site());
        let comments = self.gen_enum_comments();
        let variants: Vec<_> = self.glyphs().iter().map(Self::gen_enum_entry).collect();

        let codepoints: Vec<_> = self.glyphs().iter().map(Glyph::codepoint).collect();
        let names: Vec<_> = self.glyphs().iter().map(Glyph::name).collect();

        let font_family = self.string(StringKind::FontFamily);
        let font_family: Vec<_> = font_family.iter().collect();

        quote! {
            #( #[doc = #comments] )*
            #[derive(Debug, Clone, Copy)]
            #[repr(u32)]
            pub enum #identifier {
                #( #variants )*
            }

            impl #identifier {
                #(
                    /// The font family of the glyph
                    #[allow(dead_code)]
                    const FONT_FAMILY: &str = #font_family;
                )*

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
            impl std::fmt::Display for #identifier {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", char::from(*self))
                }
            }
        }
    }

    fn gen_enum_comments(&self) -> Vec<String> {
        let name = self.string(StringKind::FullFontName);
        let copyright = self.string(StringKind::CopyrightNotice);
        let description = self.string(StringKind::Description);

        let mut comments = Vec::new();

        if let Some(name) = name {
            comments.push(format!("{name}  "));
        }

        if let Some(copy) = copyright {
            comments.push(format!("{copy}  "));
        }

        if let Some(desc) = description {
            comments.push(format!("{desc}  "));
        }

        if !comments.is_empty() {
            comments.push(String::new());
        }

        comments.push(format!(
            "Contains the complete set of {} named glyphs for this font  ",
            self.glyphs().len()
        ));
        comments.push("Glyphs can be converted to their respective codepoints using `as u32`, or to `char` and `String` using `.into()`  ".to_string());
        comments
            .push("The postscript name for each glyph can be accessed using `.name()`".to_string());

        comments
    }

    fn gen_enum_entry(glyph: &Glyph) -> TokenStream {
        let identifier = glyph.name().to_identifier();
        let identifier = Ident::new(&identifier, Span::call_site());

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
            if let Ok(url) = glyph.datalink_outline(false) {
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
