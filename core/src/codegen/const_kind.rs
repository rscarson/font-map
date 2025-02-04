use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::Ident;

use super::{docstring::DocstringExt, to_ident::to_categories};
use crate::font::{Font, Glyph, StringKind};

/// Generates code to represent the contents of this font
///
/// Structured as a module containing category modules, each with constants of the type `font_map::font::Glyph`  
/// Warning: `font_map` must be in scope in the module where this code is used
pub struct FontConst {
    name: Ident,
    family: Option<String>,
    comments: Vec<String>,
    categories: HashMap<String, HashMap<String, Glyph>>,
    all_glyphs: Vec<Glyph>,
    injected: Option<TokenStream>,
}
impl FontConst {
    /// Create a new `FontEnum` from a font file
    pub fn from_font(name: &str, font: &Font) -> Self {
        let name = Ident::new(name, Span::call_site());
        let family = font.string(StringKind::FontFamily).map(ToString::to_string);
        let comments = font.gen_docblock();
        let all_glyphs = font.glyphs().to_vec();
        let categories = to_categories(font.glyphs());

        Self {
            name,
            family,
            comments,
            categories,
            all_glyphs,
            injected: None,
        }
    }

    /// Inject additional code into the generated module
    ///
    /// Code will be in the body section of the outer module
    pub fn inject(&mut self, tokens: TokenStream) {
        self.injected = Some(tokens);
    }

    /// Generate the code for the enum
    #[must_use]
    pub fn codegen(&self) -> TokenStream {
        let identifier = &self.name;
        let comments = &self.comments;
        let font_family: Vec<_> = self.family.iter().collect();
        let codepoints_len = self.all_glyphs.len();

        let categories = self
            .categories
            .iter()
            .map(|(name, glyphs)| Self::codegen_category(name, glyphs));

        let injected = self.injected.iter();

        quote! {
            #[allow(rustdoc::bare_urls)]
            #[allow(dead_code)]
            #[allow(non_snake_case)]
            #( #[doc = #comments] )*
            #[rustfmt::skip]
            pub mod #identifier {
                use super::font_map;

                #(
                    /// Font family
                    #[allow(dead_code)]
                    pub const FONT_FAMILY: &str = #font_family;
                )*

                /// The number of glyphs in the font
                #[allow(dead_code)]
                pub const GLYPHS: usize = #codepoints_len;

                #(
                    #categories
                )*

                #(
                    #injected
                )*
            }
        }
    }

    fn codegen_category(category: &str, glyphs: &HashMap<String, Glyph>) -> TokenStream {
        let name = Ident::new(category, Span::call_site());
        let comment = format!(
            "Contains the {} glyphs in the `{}` category",
            glyphs.len(),
            category
        );

        let mut comments = vec![comment];

        for (ident, glyph) in glyphs {
            comments.push(format!("\n-----\n[`{name}::{ident}`]  "));
            comments.extend(Self::codegen_glyph_comment(glyph));
        }

        let glyphs = glyphs
            .iter()
            .map(|(name, glyph)| Self::codegen_glyph(name, glyph));

        quote! {
            #( #[doc = #comments] )*
            #[allow(clippy::module_name_repetitions)]
            #[rustfmt::skip]
            pub mod #name {
                use super::font_map;

                #(
                    #glyphs
                )*
            }
        }
    }

    fn codegen_glyph(name: &str, glyph: &Glyph) -> TokenStream {
        let ident = Ident::new(name, Span::call_site());
        let codepoint = glyph.codepoint();
        let name = glyph.name();
        let svg = glyph.svg_preview();

        let comments = Self::codegen_glyph_comment(glyph);

        quote! {
            #( #[doc = #comments] )*
            #[allow(non_upper_case_globals)]
            pub const #ident: font_map::font::Glyph = font_map::font::Glyph::new(
                #codepoint,
                #name,
                font_map::font::GlyphPreview::Svg(std::borrow::Cow::Borrowed(#svg))
            );
        }
    }

    #[allow(unused_mut)]
    fn codegen_glyph_comment(glyph: &Glyph) -> Vec<String> {
        let name = glyph.name();
        let codepoint = glyph.codepoint();
        let range = glyph.unicode_range();

        let mut comments = vec![
            format!("`{name} (U+{codepoint:04X})`  "),
            format!("Unicode range: {range}"),
        ];

        #[cfg(feature = "extended-svg")]
        {
            if let Ok(url) = glyph.svg_dataimage_url() {
                let link = format!("![Preview Glyph]({url})");
                comments.push(String::new());
                comments.push(link);
            }
        }

        comments
    }
}
