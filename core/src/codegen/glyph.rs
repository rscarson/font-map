use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::font::Glyph;

/// Describes a glyph within a font
#[derive(Debug, Clone)]
pub struct GlyphDesc {
    identifier: String,
    name: String,
    codepoint: u32,
    comments: Vec<String>,
}
impl GlyphDesc {
    /// Create a new glyph description from an identifier and a glyph
    #[must_use]
    pub fn new(identifier: &str, glyph: &Glyph) -> Self {
        let identifier = identifier.to_string();
        let name = glyph.name().to_string();
        let codepoint = glyph.codepoint();
        let uni_range = glyph.unicode_range();

        let comments = vec![
            format!("`{name} (U+{codepoint:04X})`  "),
            format!("Unicode range: {uni_range}"),
            #[cfg(feature = "extended-svg")]
            format!(
                "\n\n![Preview Glyph]({})",
                glyph.svg_dataimage_url().unwrap_or_default()
            ),
        ];

        Self {
            identifier,
            name,
            codepoint,
            comments,
        }
    }

    /// Get the name of the glyph
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the codepoint of the glyph
    #[must_use]
    pub fn codepoint(&self) -> u32 {
        self.codepoint
    }

    /// Get the identifier of the glyph
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Set the identifier of the glyph
    pub fn set_identifier(&mut self, identifier: String) {
        self.identifier = identifier;
    }

    /// Generate code for the glyph
    #[must_use]
    pub fn codegen(&self) -> TokenStream {
        let identifier = format_ident!("{}", &self.identifier);
        let comments = &self.comments;
        let codepoint = self.codepoint;

        quote! {
            #( #[doc = #comments] )*
            #identifier = #codepoint,
        }
    }
}

impl Eq for GlyphDesc {}
impl PartialEq for GlyphDesc {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Ord for GlyphDesc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.identifier.cmp(&other.identifier)
    }
}

impl PartialOrd for GlyphDesc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
