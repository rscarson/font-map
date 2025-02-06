#![allow(clippy::cast_possible_wrap)]
use super::PlatformType;
use crate::error::ParseResult;
use crate::reader::{BinaryReader, Parse};

/// CMAP table data  
/// Contains only the subset of the table needed for mapping unicode codepoints to glyph indices
#[derive(Debug, Default)]
pub struct CmapTable {
    /// Mapping from glyph indices to unicode codepoints
    pub mappings: Vec<u32>,

    /// Raw Subtables
    pub tables: Vec<CmapSubtable>,
}

impl CmapTable {
    /// Returns the unicode codepoint for the given glyph index
    #[must_use]
    pub fn get_codepoint(&self, index: u16) -> Option<u32> {
        if index as usize >= self.mappings.len() {
            None
        } else {
            Some(self.mappings[index as usize])
        }
    }
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

            debug_msg!(
                "  CMAP subtable: platform={}, encoding={}, offset={}",
                platform_id,
                encoding_id,
                offset
            );

            let mut subtable_reader = reader.clone();
            subtable_reader.advance_to(offset as usize)?;
            let mut subtable = CmapSubtable::parse(&mut subtable_reader)?;
            subtable.platform = platform_id.into();
            subtable.encoding = encoding_id;

            for (idx, cde) in &subtable.mappings {
                let idx = *idx as usize;
                if table.mappings.len() <= idx {
                    table.mappings.resize(idx + 1, 0xFFFF);
                }
                table.mappings[idx] = *cde;
            }
            table.tables.push(subtable);
        }

        Ok(table)
    }
}

/// An individual CMAP subtable
#[derive(Debug, Default)]
pub struct CmapSubtable {
    /// Platform ID
    pub platform: PlatformType,

    /// Encoding type
    pub encoding: u16,

    /// Mappings from glyph indices to unicode codepoints
    pub mappings: Vec<(u16, u32)>,
}

impl Parse for CmapSubtable {
    #[allow(clippy::too_many_lines)]
    fn parse(reader: &mut BinaryReader) -> ParseResult<Self> {
        let fmt = reader.read_u16()?;

        let mut subtable = Self::default();
        debug_msg!("  CMAP format: {}", fmt);

        match fmt {
            0 => {
                //
                // Format 0 CMAP tables are simple 1:1 mappings
                reader.skip_u16()?; // length
                reader.skip_u16()?; // language

                for codepoint in 0u32..=0xFF {
                    let glyph_index = u16::from(reader.read_u8()?);
                    subtable.mappings.push((glyph_index, codepoint));
                }
            }

            4 => {
                //
                // Format 4 CMAP tables are segmented mappings
                reader.skip_u16()?; // length
                reader.skip_u16()?; // language

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
                            subtable.mappings.push((0, 0xFFFF));
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

                        subtable.mappings.push((glyph_index, u32::from(codepoint)));
                    }
                }
            }

            6 => {
                reader.skip_u16()?; // len
                reader.skip_u16()?; // lang

                let first_code = reader.read_u16()?;
                let entry_count = reader.read_u16()?;

                debug_msg!(
                    "  CMAP format 6: first_code={}, entry_count={}",
                    first_code,
                    entry_count
                );

                for i in 0..u32::from(entry_count) {
                    let glyph_index = reader.read_u16()?;
                    let codepoint = u32::from(first_code) + i;
                    subtable.mappings.push((glyph_index, codepoint));
                }
            }

            12 => {
                //
                // Format 12 CMAP tables are segmented mappings
                reader.skip_u16()?; // reserved
                reader.skip_u32()?; // len
                reader.skip_u32()?; // lang
                let num_groups = reader.read_u32()?;

                debug_msg!("  CMAP format 12: num_groups={}", num_groups);

                for _ in 0..num_groups {
                    let start = reader.read_u32()?;
                    let end = reader.read_u32()?;
                    let start_glyph = reader.read_u32()?; // Glyph index corresponding to the starting character code

                    debug_msg!(
                        "  CMAP group: start={}, end={}, start_glyph={}",
                        start,
                        end,
                        start_glyph
                    );

                    let adj = if start < end { 1 } else { -1 };

                    let n = start.abs_diff(end);
                    let mut codepoint = start;
                    for i in 0..n {
                        let index = u16::try_from(start_glyph + i).unwrap_or_default();
                        subtable.mappings.push((index, codepoint));
                        codepoint = codepoint.wrapping_add_signed(adj);
                    }
                }
            }

            _ => return Err(reader.err(&format!("Unsupported CMAP format: {fmt}"))),
        }

        debug_msg!("  Found {} mappings", subtable.mappings.len());
        Ok(subtable)
    }
}
