#![allow(clippy::cast_possible_wrap)]
#![allow(dead_code)]

macro_rules! read_type {
    ($reader:expr, $kind:ty) => {
        $reader
            .read_array()
            .map(<$kind>::from_be_bytes)
            .map_err(|err| err.with_desc(stringify!($kind)))
    };
}

/// A simple parser for binary data
#[derive(Debug, Clone)]
pub struct BinaryReader<'a> {
    data: &'a [u8],
    pos: usize,
}
impl<'a> BinaryReader<'a> {
    pub fn new(data: &[u8]) -> BinaryReader {
        BinaryReader { data, pos: 0 }
    }

    /// Returns the current position of the reader
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns true if the reader is at the end of the data
    pub fn is_eof(&self) -> bool {
        self.pos == self.data.len()
    }

    /// Returns the length of the data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns an error at the current position with the given message
    pub fn err(&self, error: &impl ToString) -> ParseError {
        ParseError::Parse {
            pos: self.pos,
            message: error.to_string(),
        }
    }

    /// Set the current position of the reader
    pub fn advance_to(&mut self, offset: usize) -> ParseResult<()> {
        if offset > self.data.len() {
            return Err(ParseError::UnexpectedEof {
                pos: offset,
                size: 0,
                desc: None,
            });
        }

        self.pos = offset;
        Ok(())
    }

    /// Adjust the current position of the reader by the given offset
    pub fn advance_by(&mut self, offset: isize) -> ParseResult<()> {
        self.advance_to(self.pos.wrapping_add_signed(offset))
    }

    /// Read a slice of data from the given offset  
    /// Does not advance the reader's position
    pub fn read_from(&mut self, offset: usize, size: usize) -> ParseResult<&[u8]> {
        if offset + size > self.data.len() {
            return Err(ParseError::UnexpectedEof {
                pos: offset,
                size,
                desc: None,
            });
        }

        Ok(&self.data[offset..offset + size])
    }

    /// Read a slice of data from the current position, and advance the reader's position by the size
    pub fn read(&mut self, size: usize) -> ParseResult<&[u8]> {
        let offset = self.pos;
        if offset + size > self.data.len() {
            return Err(ParseError::UnexpectedEof {
                pos: offset,
                size,
                desc: None,
            });
        }

        self.pos += size;
        Ok(&self.data[offset..self.pos])
    }

    /// Read an array of bytes from the current position
    pub fn read_array<const N: usize>(&mut self) -> ParseResult<[u8; N]> {
        let data = self.read(N)?;
        let mut array = [0; N];
        array.copy_from_slice(data);
        Ok(array)
    }

    /// Skip the given number of bytes
    pub fn skip(&mut self, size: usize) -> ParseResult<()> {
        if self.pos + size > self.data.len() {
            return Err(ParseError::UnexpectedEof {
                pos: self.pos,
                size,
                desc: None,
            });
        }

        self.advance_by(size as isize)
    }

    pub fn skip_u8(&mut self) -> ParseResult<()> {
        self.skip(1).map_err(|err| err.with_desc("u8"))
    }

    pub fn skip_u16(&mut self) -> ParseResult<()> {
        self.skip(2).map_err(|err| err.with_desc("u16"))
    }

    pub fn skip_u24(&mut self) -> ParseResult<()> {
        self.skip(3).map_err(|err| err.with_desc("u24"))
    }

    pub fn skip_u32(&mut self) -> ParseResult<()> {
        self.skip(4).map_err(|err| err.with_desc("u32"))
    }

    pub fn skip_u64(&mut self) -> ParseResult<()> {
        self.skip(4).map_err(|err| err.with_desc("u64"))
    }

    pub fn read_u8(&mut self) -> ParseResult<u8> {
        self.read(1)
            .map(|data| data[0])
            .map_err(|err| err.with_desc("u8"))
    }

    pub fn read_i8(&mut self) -> ParseResult<i8> {
        self.read(1)
            .map(|data| data[0] as i8)
            .map_err(|err| err.with_desc("i8"))
    }

    pub fn read_u16(&mut self) -> ParseResult<u16> {
        read_type!(self, u16)
    }

    pub fn read_i16(&mut self) -> ParseResult<i16> {
        read_type!(self, i16)
    }

    pub fn read_u24(&mut self) -> ParseResult<u32> {
        self.read(3)
            .map(|data| (u32::from(data[0]) << 16) | (u32::from(data[1]) << 8) | u32::from(data[2]))
            .map_err(|err| err.with_desc("u24"))
    }

    pub fn read_u32(&mut self) -> ParseResult<u32> {
        read_type!(self, u32)
    }

    /// From the TTF docs; `16.16-bit signed fixed-point number`
    /// This is a 32-bit value, where the first 16 bits are the integer part, and the last 16 bits are the fractional part.
    pub fn read_fixed32(&mut self) -> ParseResult<(i16, u16)> {
        let int = self.read_i16()?;
        let frac = self.read_u16()?;
        Ok((int, frac))
    }

    pub fn read_f2dot14(&mut self) -> ParseResult<f64> {
        let value = self.read_i16()?;
        Ok(f64::from(value) / f64::from(1 << 14))
    }

    pub fn read_string(&mut self, size: usize) -> ParseResult<String> {
        let data = self.read(size)?;
        unsafe { Ok(String::from_utf8_unchecked(data.to_vec())) }
    }
}

pub trait Parse: Sized {
    fn from_data(data: &[u8]) -> ParseResult<Self> {
        let mut reader = BinaryReader::new(data);
        Self::parse(&mut reader)
    }

    fn parse<'a>(reader: &'a mut BinaryReader<'a>) -> ParseResult<Self>;

    fn parse_with<'a>(&mut self, reader: &'a mut BinaryReader<'a>) -> ParseResult<()> {
        *self = Self::parse(reader)?;
        Ok(())
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEof {
        pos: usize,
        size: usize,
        desc: Option<&'static str>,
    },
    InvalidValue {
        pos: usize,
        value: u32,
        name: &'static str,
    },
    Parse {
        pos: usize,
        message: String,
    },
    Io(std::io::Error),
}
impl ParseError {
    pub fn with_desc(self, desc: &'static str) -> ParseError {
        match self {
            ParseError::UnexpectedEof { pos, size, .. } => ParseError::UnexpectedEof {
                pos,
                size,
                desc: Some(desc),
            },
            other => other,
        }
    }
}
impl std::error::Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEof {
                pos,
                size,
                desc: Some(desc),
            } => {
                write!(
                    f,
                    "Unexpected EOF trying to read {size} bytes from {pos} while parsing {desc}"
                )
            }
            ParseError::UnexpectedEof { pos, size, .. } => {
                write!(f, "Unexpected EOF trying to read {size} bytes from {pos}")
            }
            ParseError::InvalidValue { pos, value, name } => {
                write!(f, "Invalid value {value:#0x} at {pos} while parsing {name}")
            }
            ParseError::Parse { pos, message } => {
                write!(f, "Error at {pos}: {message}")
            }
            ParseError::Io(err) => {
                write!(f, "IO Error: {err:#}")
            }
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> ParseError {
        ParseError::Io(err)
    }
}
