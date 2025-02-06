//! This module contains the font enumeration and glyph data structures
//!
//! The `Font` struct contains all the glyphs in a font, along with any stored strings
//!
//! The `Glyph` struct contains information about a single glyph in a font:
//! - Unicode codepoint
//! - Postscript name
//! - Outline data
//!
#![allow(clippy::match_on_vec_items)]
#![allow(clippy::cast_possible_truncation)]
pub use crate::raw::ttf::NameKind as StringKind;
use crate::{
    error::ParseResult,
    raw::ttf::{GlyfOutline, SimpleGlyf, TrueTypeFont},
    svg::SvgExt,
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

/// A parsed font, with access to its glyphs and stored strings
#[derive(Debug, Clone)]
pub struct Font {
    glyphs: Vec<Glyph>,
    strings: HashMap<StringKind, String>,
}
impl Font {
    /// Creates a new font from the given font data
    ///
    /// # Errors
    /// Returns an error if the font data is invalid or cannot be parsed
    pub fn new(font_data: &[u8]) -> ParseResult<Self> {
        let font = TrueTypeFont::new(font_data)?;
        Ok(font.into())
    }

    /// Creates a new font from the font file at the specified path
    ///
    /// # Errors
    /// Returns an error if the font data is invalid or cannot be parsed
    pub fn from_file(path: impl AsRef<std::path::Path>) -> ParseResult<Self> {
        let font_data = std::fs::read(path)?;
        Self::new(&font_data)
    }

    /// Returns the string with the specified kind, if it exists
    #[must_use]
    pub fn string(&self, kind: StringKind) -> Option<&str> {
        self.strings.get(&kind).map(String::as_str)
    }

    /// Returns all the strings in the font
    #[must_use]
    pub fn strings(&self) -> &HashMap<StringKind, String> {
        &self.strings
    }

    /// Returns the glyph with the specified unicode codepoint, if it exists
    #[must_use]
    pub fn glyph(&self, codepoint: u32) -> Option<&Glyph> {
        self.glyphs.iter().find(|g| g.codepoint == codepoint)
    }

    /// Returns the glyph with the specified postscript name, if it exists
    #[must_use]
    pub fn glyph_named(&self, name: &str) -> Option<&Glyph> {
        self.glyphs.iter().find(|g| g.name == name)
    }

    /// Returns the glyphs in the font
    #[must_use]
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }
}

impl From<TrueTypeFont> for Font {
    fn from(value: TrueTypeFont) -> Self {
        let cmap = value.cmap_table;
        let post = value.post_table;
        let name = value.name_table;
        let glyf = value.glyf_table;

        let mut strings = HashMap::new();
        for record in name.records {
            strings.insert(record.name_id, record.name);
        }

        let mut glyphs = Vec::new();
        let mut codepoint_hash = HashSet::new();
        for (glyph_index, name) in post.glyph_names.into_iter().enumerate() {
            let name = Cow::Owned(name);
            let glyph_index = glyph_index as u16;

            // Find unicode codepoint, skipping unmapped glyphs
            let codepoint = cmap.get_codepoint(glyph_index);
            let codepoint = match codepoint {
                Some(c) if glyph_index == 0 => c,
                Some(c) if c != 0xFFFF => c,
                _ => continue,
            };

            // Skip duplicate codepoints
            if !codepoint_hash.insert(codepoint) {
                continue;
            }

            // Get the glyph outline data
            let outline = match glyf[glyph_index as usize] {
                GlyfOutline::Simple(ref outline) => outline.clone(),
                GlyfOutline::Compound(ref outline) => outline.as_simple(&glyf),
            };
            let preview = GlyphPreview::Ttf(outline);

            glyphs.push(Glyph {
                codepoint,
                name,
                preview,
            });
        }

        Self { glyphs, strings }
    }
}

/// A preview of a glyph, either as a TTF outline or SVG image
#[derive(Debug, Clone)]
pub enum GlyphPreview {
    /// TTF formatted glyph data - converted to simple fmt if needed
    Ttf(SimpleGlyf),

    /// SVG formatted glyph data, as a string
    Svg(Cow<'static, str>),
}
impl SvgExt for GlyphPreview {
    fn to_svg(&self) -> String {
        match self {
            Self::Ttf(outline) => outline.to_svg(),
            Self::Svg(svg) => svg.to_string(),
        }
    }
}

/// A single glyph in a font
#[derive(Debug, Clone)]
pub struct Glyph {
    codepoint: u32,
    name: Cow<'static, str>,
    preview: GlyphPreview,
}
impl Glyph {
    /// Creates a new glyph with the specified codepoint, name, and preview data
    #[must_use]
    pub const fn new(codepoint: u32, name: &'static str, preview: GlyphPreview) -> Self {
        Self {
            codepoint,
            name: Cow::Borrowed(name),
            preview,
        }
    }

    /// Returns the unicode range for the glyph
    #[must_use]
    pub fn unicode_range(&self) -> &str {
        crate::unicode_range::unicode_range(self.codepoint)
    }

    /// Returns the unicode codepoint for the glyph
    #[must_use]
    pub fn codepoint(&self) -> u32 {
        self.codepoint
    }

    /// Returns the character for the glyph
    #[must_use]
    pub fn char(&self) -> char {
        std::char::from_u32(self.codepoint).unwrap_or(char::REPLACEMENT_CHARACTER)
    }

    /// Returns the postscript name of the glyph
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the raw visual data of this glyph  
    /// Compound glyphs will be simplified to a single outline
    #[must_use]
    pub fn outline(&self) -> &GlyphPreview {
        &self.preview
    }

    /// Returns the SVG data of this glyph's outline  
    #[must_use]
    pub fn svg_preview(&self) -> String {
        self.preview.to_svg()
    }

    /// Returns the gzip compressed SVGZ data of this glyph
    ///
    /// # Errors
    /// Returns an error if the data cannot be compressed
    #[cfg(feature = "extended-svg")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-svg")))]
    pub fn svgz_preview(&self) -> std::io::Result<Vec<u8>> {
        self.preview.to_svgz()
    }

    /// Generates a `data:image` link containing the svg data for this glyph  
    ///
    /// # Errors
    /// Returns an error if the data cannot be encoded properly
    #[cfg(feature = "extended-svg")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-svg")))]
    pub fn svg_dataimage_url(&self) -> std::io::Result<String> {
        self.preview.to_svg_dataimage_url()
    }
}

impl From<Glyph> for char {
    fn from(value: Glyph) -> Self {
        value.char()
    }
}

impl From<&Glyph> for char {
    fn from(value: &Glyph) -> Self {
        value.char()
    }
}

impl From<Glyph> for u32 {
    fn from(value: Glyph) -> Self {
        value.codepoint()
    }
}

impl From<&Glyph> for u32 {
    fn from(value: &Glyph) -> Self {
        value.codepoint()
    }
}

impl std::fmt::Display for Glyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char())
    }
}
