#![allow(clippy::cast_possible_wrap)]
use super::PlatformType;
use crate::reader::{BinaryReader, Parse, ParseResult};

/// CMAP table data  
/// Contains only the subset of the table needed for mapping unicode codepoints to glyph indices
#[derive(Debug, Default)]
pub struct CmapTable {
    /// The unicode subtable of the CMAP table
    pub unicode_subtable: CmapSubtable,
}

impl CmapTable {
    const UNICODE_PLATFORM_ID: u16 = 0;
}

impl Parse for CmapTable {
    fn parse(reader: &mut BinaryReader) -> ParseResult<Self> {
        let mut table = Self::default();

        //
        // Table header
        reader.skip_u16()?; // version
        let num_tables = reader.read_u16()?;

        //
        // Subtables
        for _ in 0..num_tables {
            let platform_id = reader.read_u16()?;
            let encoding_id = reader.read_u16()?;
            let offset = reader.read_u32()?;

            // Skip non-unicode subtables
            if platform_id != Self::UNICODE_PLATFORM_ID {
                continue;
            }

            let mut subtable_reader = reader.clone();
            subtable_reader.advance_to(offset as usize)?;
            let mut subtable = CmapSubtable::parse(&mut subtable_reader)?;
            subtable.platform = platform_id.into();
            subtable.encoding = encoding_id;

            table.unicode_subtable = subtable;
            break;
        }

        Ok(table)
    }
}

#[derive(Debug, Default)]
pub struct CmapSubtable {
    pub platform: PlatformType,
    pub encoding: u16,
    pub mappings: Vec<u32>,
}

impl CmapSubtable {
    pub fn get_codepoint(&self, glyph_index: u16) -> Option<u32> {
        self.mappings.get(glyph_index as usize).copied()
    }
}

impl Parse for CmapSubtable {
    fn parse(reader: &mut BinaryReader) -> ParseResult<Self> {
        let fmt = reader.read_u16()?;
        reader.skip_u16()?; // length
        reader.skip_u16()?; // language

        let mut subtable = Self::default();

        match fmt {
            0 => {
                //
                // Format 0 CMAP tables are simple 1:1 mappings
                for codepoint in 0u32..=0xFF {
                    let glyph_index = usize::from(reader.read_u8()?);
                    subtable.mappings.insert(glyph_index, codepoint);
                }
            }

            2 => todo!("Format 2 CMAP tables not yet implemented"),

            4 => {
                //
                // Format 4 CMAP tables are segmented mappings
                let mut seg_count = reader.read_u16()?;
                seg_count /= 2;

                reader.skip_u16()?; // search range
                reader.skip_u16()?; // entry selector
                reader.skip_u16()?; // range shift

                let mut end_code = Vec::with_capacity(seg_count as usize);
                for _ in 0..seg_count {
                    end_code.push(reader.read_u16()?);
                }

                reader.skip_u16()?; // reserved pad

                let mut start_code = Vec::with_capacity(seg_count as usize);
                for _ in 0..seg_count {
                    start_code.push(reader.read_u16()?);
                }

                let mut id_delta = Vec::with_capacity(seg_count as usize);
                for _ in 0..seg_count {
                    id_delta.push(reader.read_u16()?);
                }

                for i in 0..seg_count as usize {
                    let id_range_offset = reader.read_u16()?;

                    for codepoint in start_code[i]..=end_code[i] {
                        if codepoint == 0xFFFF {
                            subtable.mappings[0] = u32::from(codepoint);
                            break;
                        }

                        let glyph_index = if id_range_offset == 0 {
                            //
                            // Simple mapping
                            codepoint.wrapping_add(id_delta[i])
                        } else {
                            //
                            // Indexed mapping
                            //  let index_offset = id_range_offset / 2 + (codepoint - start_code[i]);

                            let index_offset =
                                id_range_offset + 2 * (codepoint - start_code[i]) - 2;

                            let mut glyph_reader = reader.clone();
                            glyph_reader.advance_by(index_offset as isize)?;

                            let glyph_index = glyph_reader.read_u16()?;
                            if glyph_index != 0 {
                                glyph_index.wrapping_add(id_delta[i])
                            } else {
                                glyph_index
                            }
                        };

                        if subtable.mappings.len() <= glyph_index as usize {
                            subtable.mappings.resize(glyph_index as usize + 1, 0xFFFF);
                        }
                        subtable.mappings[glyph_index as usize] = u32::from(codepoint);
                    }
                }
            }

            _ => todo!("Unsupported CMAP format: {}", fmt),
        }

        Ok(subtable)
    }
}
