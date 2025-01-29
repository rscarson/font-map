#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
use crate::error::ParseResult;
use crate::reader::{BinaryReader, Parse};

/// The Post table of a TrueType font  
/// Contains only the subset of the table needed for mapping glyph indices to glyph names
#[derive(Debug, Default)]
pub struct PostTable {
    /// True if the font is monospaced
    pub is_monospaced: bool,

    /// The glyph names in the table, by glyph index
    pub glyph_names: Vec<String>,
}

impl PostTable {
    /// Returns the name of the glyph at the specified index, if it exists
    #[must_use]
    pub fn get_glyph_name(&self, index: u16) -> Option<&str> {
        self.glyph_names.get(index as usize).map(String::as_str)
    }
}

impl Parse for PostTable {
    fn parse(reader: &mut BinaryReader) -> ParseResult<Self> {
        let mut table = Self::default();

        //
        // Table header
        let fmt = reader.read_fixed32()?;
        reader.skip_u32()?; // italic angle
        reader.skip_u16()?; // underline position
        reader.skip_u16()?; // underline thickness
        table.is_monospaced = reader.read_u32()? != 0; // is fixed pitch
        reader.skip_u32()?; // min memory t42
        reader.skip_u32()?; // max memory t42
        reader.skip_u32()?; // min memory t1
        reader.skip_u32()?; // max memory t1

        match fmt {
            (1, 0) => {
                //
                // Format 1.0 uses the standard Macintosh character set
                table.glyph_names = POST_MAC_NAMES.iter().map(ToString::to_string).collect();
            }

            (2, 0) => {
                //
                // Format 2.0 uses a 32-bit offset to the glyph name
                let num_glyphs = reader.read_u16()?;

                //
                // Read the glyph names first
                let mut names = Vec::with_capacity(num_glyphs as usize - POST_MAC_NAMES_LEN);
                let mut name_reader = reader.clone();
                name_reader.advance_by(num_glyphs as isize * 2)?;
                while !name_reader.is_eof() {
                    let len = name_reader.read_u8()?;
                    let name = name_reader.read_string(len as usize)?;
                    names.push(name);
                }

                for _ in 0..num_glyphs {
                    let ordinal = reader.read_u16()?;
                    if ordinal < POST_MAC_NAMES_LEN as u16 {
                        table
                            .glyph_names
                            .push(POST_MAC_NAMES[ordinal as usize].to_string());
                    } else {
                        let index = (ordinal - POST_MAC_NAMES_LEN as u16) as usize;
                        table.glyph_names.push(names[index].clone());
                    }
                }
            }

            (2, 5) => {
                //
                // Format 2.5 uses an 8-bit offset to the std glyph names
                let num_glyphs = reader.read_u16()?;

                let mut glyph_names = Vec::new();
                for i in 0..num_glyphs {
                    let offset = reader.read_i8()?;
                    let index = i.wrapping_add_signed(i16::from(offset));
                    glyph_names.push(POST_MAC_NAMES[index as usize].to_string());
                }
            }

            _ => {
                // Other formats are not useful to us here
            }
        }

        Ok(table)
    }
}

//
// Standard Macintosh glyph names
const POST_MAC_NAMES_LEN: usize = 258;
#[rustfmt::skip]
const POST_MAC_NAMES: [&str; POST_MAC_NAMES_LEN] = [
    ".notdef", ".null", "nonmarkingreturn", "space", "exclam", "quotedbl", "numbersign", "dollar", "percent", 
    "ampersand", "quotesingle","parenleft","parenright","asterisk","plus","comma", "hyphen", "period", "slash", 
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "colon", "semicolon", "less",
    "equal", "greater", "question", "at", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", 
    "O",  "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "bracketleft", "backslash", "bracketright", "asciicircum", 
    "underscore", "grave", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", 
    "s", "t", "u", "v", "w", "x", "y", "z", "braceleft", "bar", "braceright", "asciitilde", "Adieresis", "Aring", 
    "Ccedilla", "Eacute", "Ntilde", "Odieresis", "Udieresis", "aacute", "agrave", "acircumflex", "adieresis", 
    "atilde", "aring", "ccedilla", "eacute", "egrave", "ecircumflex", "edieresis", "iacute", "igrave",
    "icircumflex", "idieresis", "ntilde", "oacute", "ograve", "ocircumflex", "odieresis", "otilde", "uacute", "ugrave", 
    "ucircumflex", "udieresis", "dagger", "degree", "cent", "sterling", "section", "bullet", "paragraph", "germandbls", 
    "registered", "copyright", "trademark", "acute", "dieresis", "notequal", "AE", "Oslash", "infinity", "plusminus", 
    "lessequal", "greaterequal", "yen", "mu", "partialdiff", "summation", "product", "pi", "integral", "ordfeminine", 
    "ordmasculine", "Omega", "ae", "oslash", "questiondown", "exclamdown", "logicalnot", "radical", "florin", "approxequal", 
    "Delta", "guillemotleft", "guillemotright", "ellipsis", "nonbreakingspace", "Agrave", "Atilde", "Otilde", "OE", "oe", 
    "endash", "emdash", "quotedblleft", "quotedblright", "quoteleft", "quoteright", "divide", "lozenge", "ydieresis", 
    "Ydieresis", "fraction", "currency", "guilsinglleft", "guilsinglright", "fi", "fl", "daggerdbl", "periodcentered", 
    "quotesinglbase", "quotedblbase", "perthousand", "Acircumflex", "Ecircumflex", "Aacute", "Edieresis", "Egrave", 
    "Iacute", "Icircumflex", "Idieresis", "Igrave", "Oacute", "Ocircumflex", "apple", "Ograve", "Uacute", "Ucircumflex", 
    "Ugrave", "dotlessi", "circumflex", "tilde", "macron", "breve", "dotaccent", "ring", "cedilla", "hungarumlaut", 
    "ogonek", "caron", "Lslash", "lslash", "Scaron", "scaron", "Zcaron", "zcaron", "brokenbar", "Eth", "eth", "Yacute", 
    "yacute", "Thorn", "thorn", "minus", "multiply", "onesuperior", "twosuperior", "threesuperior", "onehalf", "onequarter", 
    "threequarters", "franc", "Gbreve", "gbreve", "Idotaccent", "Scedilla", "scedilla", "Cacute", "cacute", "Ccaron", 
    "ccaron", "dcroat"
];
