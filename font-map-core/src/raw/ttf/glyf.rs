#![allow(clippy::cast_sign_loss)]
use crate::reader::{BinaryReader, Parse};
use crate::error::ParseResult;

mod simple;
pub use simple::SimpleGlyf;

mod compound;
pub use compound::CompoundGlyf;

/// The outline features of a glyph
#[derive(Debug, Clone)]
pub enum GlyfOutline {
    /// A simple glyph outline
    Simple(SimpleGlyf),

    /// A compound glyph outline
    Compound(CompoundGlyf),
}
impl Default for GlyfOutline {
    fn default() -> Self {
        GlyfOutline::Simple(SimpleGlyf {
            contours: vec![],
            num_contours: 0,
            x: (0, 0),
            y: (0, 0),
        })
    }
}
impl GlyfOutline {
    /// Returns true if the outline is a simple glyph
    #[must_use]
    pub fn is_simple(&self) -> bool {
        matches!(self, GlyfOutline::Simple(_))
    }

    /// Returns true if the outline is a compound glyph
    #[must_use]
    pub fn is_compound(&self) -> bool {
        matches!(self, GlyfOutline::Compound(_))
    }
}
impl Parse for GlyfOutline {
    fn parse<'a>(reader: &'a mut BinaryReader<'a>) -> ParseResult<Self> {
        let num_contours = reader.read_i16()?;
        let xmin = reader.read_i16()?;
        let ymin = reader.read_i16()?;
        let xmax = reader.read_i16()?;
        let ymax = reader.read_i16()?;

        if num_contours >= 0 {
            //
            // Simple glyph
            let mut glyph = SimpleGlyf {
                contours: Vec::with_capacity(num_contours as usize),
                num_contours,
                x: (xmin, xmax),
                y: (ymin, ymax),
            };

            glyph.parse_with(reader)?;
            Ok(GlyfOutline::Simple(glyph))
        } else {
            //
            // Compound glyf
            let glyph = CompoundGlyf::parse(reader)?;
            Ok(GlyfOutline::Compound(glyph))
        }
    }
}
