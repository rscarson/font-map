//! This module contains the TTF parser underlying the crate
//!
//! The parser is designed to be fast, and minimal. Supporting only a subset of the TTF spec
//!
use crate::reader::{BinaryReader, Parse, ParseResult};

mod post;
pub use post::PostTable;

mod cmap;
pub use cmap::CmapTable;

mod glyf;
pub use glyf::*;

mod name;
pub use name::NameKind;
pub use name::NameTable;

/// The raw data from a TrueType font  
/// Contains only the subset of the table needed for mapping unicode:
/// - Codepoints
/// - Glyph indices
/// - Glyph names
/// - Glyph outlines
#[derive(Debug)]
pub struct TrueTypeFont {
    /// The glyph outlines in the font, indexed by `glyph_id`
    pub glyf_table: Vec<GlyfOutline>,

    /// The CMAP table of the font
    pub cmap_table: CmapTable,

    /// The Post table of the font
    pub post_table: PostTable,

    /// The Name table of the font
    pub name_table: NameTable,
}

impl TrueTypeFont {
    /// Creates a new TrueType font from the given font data
    ///
    /// # Errors
    /// Returns an error if the font data is invalid or cannot be parsed
    pub fn new(font_data: &[u8]) -> ParseResult<Self> {
        Self::from_data(font_data)
    }
}

impl Parse for TrueTypeFont {
    fn parse(reader: &mut BinaryReader) -> ParseResult<Self> {
        let mut cmap = None;
        let mut post = None;
        let mut name = None;

        //
        // Offset Table
        reader.skip_u32()?; // Scaler type
        let num_tables = reader.read_u16()?;
        reader.skip_u16()?; // Search range
        reader.skip_u16()?; // Entry selector
        reader.skip_u16()?; // Range shift

        let mut loca_is_long = false;
        let mut glyf_offsets = vec![];
        let mut glyf_table: Vec<_> = vec![];

        //
        // Table directory
        for _ in 0..num_tables {
            let tag = reader.read_string(4)?;
            reader.skip_u32()?; // checksum
            let offset = reader.read_u32()?;
            let length = reader.read_u32()?;

            match tag.as_str() {
                "cmap" => {
                    let table = reader.read_from(offset as usize, length as usize)?;
                    let mut table_reader = BinaryReader::new(table);
                    let table = CmapTable::parse(&mut table_reader)?;
                    cmap = Some(table);
                }

                "post" => {
                    let table = reader.read_from(offset as usize, length as usize)?;
                    let mut table_reader = BinaryReader::new(table);
                    let table = PostTable::parse(&mut table_reader)?;
                    post = Some(table);
                }

                "name" => {
                    let table = reader.read_from(offset as usize, length as usize)?;
                    let mut table_reader = BinaryReader::new(table);
                    let table = NameTable::parse(&mut table_reader)?;
                    name = Some(table);
                }

                "glyf" => {
                    let table = reader.read_from(offset as usize, length as usize)?;
                    glyf_table = table.to_vec();
                }

                "head" => {
                    let table = reader.read_from(offset as usize, length as usize)?;
                    let mut table_reader = BinaryReader::new(table);

                    table_reader.skip_u32()?; // version
                    table_reader.skip_u32()?; // font_revision
                    table_reader.skip_u32()?; // checksum_adjustment
                    table_reader.skip_u32()?; // magic_number
                    table_reader.skip_u16()?; // flags
                    table_reader.skip_u16()?; // units_per_em
                    table_reader.skip_u64()?; // created
                    table_reader.skip_u64()?; // modified
                    table_reader.skip_u64()?; // x_min-ymax
                    table_reader.skip_u16()?; // mac_style
                    table_reader.skip_u16()?; // lowest_rec_ppem
                    table_reader.skip_u16()?; // font_direction_hint

                    loca_is_long = table_reader.read_u16()? != 0;
                }

                "loca" => {
                    let table = reader.read_from(offset as usize, length as usize)?;
                    let mut table_reader = BinaryReader::new(table);

                    while !table_reader.is_eof() {
                        let offset = if loca_is_long {
                            table_reader.read_u32()?
                        } else {
                            u32::from(table_reader.read_u16()?) * 2
                        };

                        glyf_offsets.push(offset);
                    }
                }

                _ => (),
            }
        }

        //
        // Grab completed tables
        let cmap = cmap.unwrap_or_default();
        let post = post.unwrap_or_default();
        let name = name.unwrap_or_default();

        //
        // Parse glyf table
        let mut glyphs = vec![];
        let mut glyf_offsets = glyf_offsets.into_iter().peekable();
        while let Some(offset) = glyf_offsets.next() {
            let Some(next_offset) = glyf_offsets.peek().copied().map(|o| o as usize) else {
                break;
            };

            let length = next_offset - offset as usize;
            let data = &glyf_table[offset as usize..next_offset];

            if length > 0 {
                let mut glyf_reader = BinaryReader::new(data);
                let glyph = GlyfOutline::parse(&mut glyf_reader)?;
                glyphs.push(glyph);
            } else {
                let glyph = GlyfOutline::default();
                glyphs.push(glyph);
            }
        }

        Ok(Self {
            cmap_table: cmap,
            post_table: post,
            glyf_table: glyphs,
            name_table: name,
        })
    }
}

/// The platform types supported by some tables
#[derive(Debug, Clone, Copy, Default)]
#[repr(u16)]
pub enum PlatformType {
    /// Unicode platform
    Unicode = 0,

    /// Macintosh platform
    Macintosh = 1,

    /// ISO platform
    Iso = 2,

    /// Microsoft platform
    Microsoft = 3,

    /// Invalid platform
    #[default]
    Invalid = 0xFFFF,
}
impl From<u16> for PlatformType {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Unicode,
            1 => Self::Macintosh,
            2 => Self::Iso,
            3 => Self::Microsoft,
            _ => Self::Invalid,
        }
    }
}
