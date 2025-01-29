use crate::error::ParseResult;
use crate::reader::{BinaryReader, Parse};

use super::PlatformType;

/// A name record in a TrueType font
#[derive(Debug)]
pub struct NameRecord {
    pub platform_id: PlatformType,
    pub encoding_id: u16,
    pub language_id: u16,

    pub name_id: NameKind,
    pub name: String,
}

/// The name table of a TrueType font
#[derive(Debug, Default)]
pub struct NameTable {
    /// The name records in the table
    pub records: Vec<NameRecord>,
}

impl Parse for NameTable {
    fn parse(reader: &mut BinaryReader) -> ParseResult<Self> {
        let mut table = Self::default();

        //
        // Table header
        reader.skip_u16()?; // format
        let num_records = reader.read_u16()?;

        //
        // Name records
        let string_offset = reader.read_u16()?;

        //
        // Records
        table.records.reserve(num_records as usize);
        for _ in 0..num_records {
            let platform_id = reader.read_u16()?.into();
            let encoding_id = reader.read_u16()?;
            let language_id = reader.read_u16()?;
            let name_id = reader.read_u16()?.into();
            let length = reader.read_u16()?;
            let offset = reader.read_u16()?;

            let mut name_reader = reader.clone();
            name_reader.advance_to(string_offset as usize + offset as usize)?;
            let name = name_reader.read(length as usize)?;

            // Decode
            let name = name.decode(platform_id, encoding_id);
            table.records.push(NameRecord {
                platform_id,
                encoding_id,
                language_id,
                name_id,
                name,
            });
        }

        Ok(table)
    }
}

/// The strings supported by the name table
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u16)]
pub enum NameKind {
    CopyrightNotice = 0,
    FontFamily = 1,
    FontSubfamily = 2,
    UniqueIdentifier = 3,
    FullFontName = 4,
    NameTableVersion = 5,
    PostscriptName = 6,
    Trademark = 7,
    Manufacturer = 8,
    Designer = 9,
    Description = 10,
    VendorUrl = 11,
    DesignerUrl = 12,
    LicenseDescription = 13,
    LicenseInfoUrl = 14,

    PreferredFamily = 16,
    PreferredSubfamily = 17,
    CompatibleFull = 18,

    SampleText = 19,

    PostscriptCid = 20,
    WwsFamily = 21,
    WwsSubfamily = 22,
    LightBackgroundPalette = 23,
    DarkBackgroundPalette = 24,
    VariationsPostscriptNamePrefix = 25,

    Other = 0xFFFF,
}
impl From<u16> for NameKind {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::CopyrightNotice,
            1 => Self::FontFamily,
            2 => Self::FontSubfamily,
            3 => Self::UniqueIdentifier,
            4 => Self::FullFontName,
            5 => Self::NameTableVersion,
            6 => Self::PostscriptName,
            7 => Self::Trademark,
            8 => Self::Manufacturer,
            9 => Self::Designer,
            10 => Self::Description,
            11 => Self::VendorUrl,
            12 => Self::DesignerUrl,
            13 => Self::LicenseDescription,
            14 => Self::LicenseInfoUrl,
            16 => Self::PreferredFamily,
            17 => Self::PreferredSubfamily,
            18 => Self::CompatibleFull,
            19 => Self::SampleText,
            20 => Self::PostscriptCid,
            21 => Self::WwsFamily,
            22 => Self::WwsSubfamily,
            23 => Self::LightBackgroundPalette,
            24 => Self::DarkBackgroundPalette,
            25 => Self::VariationsPostscriptNamePrefix,
            _ => Self::Other,
        }
    }
}

/// Extension trait to decode a string from a byte array
pub trait StringDecoderExt {
    /// Decode a string from a byte array
    fn decode(&self, platform_id: PlatformType, encoding_id: u16) -> String;
}
impl StringDecoderExt for [u8] {
    fn decode(&self, platform_id: PlatformType, encoding_id: u16) -> String {
        match (platform_id, encoding_id) {
            (PlatformType::Unicode, _) | (PlatformType::Microsoft, 1 | 10) => {
                let words = self
                    .chunks_exact(2)
                    .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]));
                String::from_utf16_lossy(&words.collect::<Vec<u16>>())
            }

            _ => format!("Encoding type {platform_id:?}::{encoding_id} not supported"),
        }
    }
}
