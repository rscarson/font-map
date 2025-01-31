#![allow(clippy::cast_possible_truncation)]
use crate::error::ParseResult;
use crate::reader::{BinaryReader, Parse};

/// The outline features of a simple-type glyph
#[derive(Debug, Clone)]
pub struct SimpleGlyf {
    /// The contours of the glyph
    pub contours: Vec<Contour>,

    /// The number of contours in the glyph
    /// This field is used to prime the parser
    pub num_contours: i16,

    /// Horizontal bounds of the glyph
    pub x: (i16, i16),

    /// Vertical bounds of the glyph
    pub y: (i16, i16),
}

impl Parse for SimpleGlyf {
    fn parse(_: &mut BinaryReader) -> ParseResult<Self> {
        unimplemented!("Use parse_with instead")
    }

    fn parse_with(&mut self, reader: &mut BinaryReader) -> ParseResult<()> {
        // Simple glyph
        let mut end_pts_of_contours = Vec::with_capacity(self.num_contours as usize);
        let mut last_pt = 0;

        for _ in 0..self.num_contours {
            last_pt = reader.read_u16()?;
            end_pts_of_contours.push(last_pt);
        }

        let instruction_length = reader.read_u16()?;
        let _instructions = reader.read(instruction_length as usize)?;

        let num_points = last_pt + 1;
        debug_msg!("  Num_points: {}", num_points);

        //
        // Parse instructions to get real point count
        let mut flags = Vec::with_capacity(num_points as usize);
        let mut remaining_pts = num_points;
        while remaining_pts > 0 {
            let flag = reader.read_u8()?;
            let mut flag = Flag::from_byte(flag);
            remaining_pts -= 1;

            // Repeat the flag
            if flag.repeats != 0 {
                let n_times = reader.read_u8()?;
                debug_msg!("  Repeats: {n_times}");
                flag.repeats = n_times;
                remaining_pts -= u16::from(n_times);
            }

            flags.push(flag);
            flags.reserve(usize::from(flag.repeats));
            for _ in 0..flag.repeats {
                flags.push(flag);
            }
        }

        //
        // Parse X coords into objective coords
        let mut x_coordinates = Vec::with_capacity(flags.len());
        let mut last_x = 0;
        for flag in &flags {
            let delta = match flag.x_kind {
                FlagCoordKind::NegShort => -i16::from(reader.read_u8()?),
                FlagCoordKind::PosShort => i16::from(reader.read_u8()?),
                FlagCoordKind::Long => reader.read_i16()?,
                FlagCoordKind::Same => 0,
            };

            last_x += delta;
            debug_msg!("  X: {delta}");
            x_coordinates.push(last_x);
        }

        //
        // Parse Y coords into objective coords
        let mut y_coordinates = Vec::with_capacity(flags.len());
        let mut last_y = 0;
        for flag in &flags {
            let delta = match flag.y_kind {
                FlagCoordKind::NegShort => -i16::from(reader.read_u8()?),
                FlagCoordKind::PosShort => i16::from(reader.read_u8()?),
                FlagCoordKind::Long => reader.read_i16()?,
                FlagCoordKind::Same => 0,
            };

            last_y += delta;
            debug_msg!("  Y: {delta}");
            y_coordinates.push(last_y);
        }

        //
        // Create points
        let mut points = Vec::with_capacity(flags.len());
        for i in 0..flags.len() {
            let x = x_coordinates[i];
            let y = y_coordinates[i];
            let on_curve = flags[i].on_curve;
            points.push(Point { x, y, on_curve });
        }

        //
        // Map points to contours
        let mut start = 0;
        for end in &end_pts_of_contours {
            let contour_points = points[start..=*end as usize].to_vec();
            start = *end as usize + 1;
            self.contours.push(Contour {
                points: contour_points,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum FlagCoordKind {
    NegShort,
    PosShort,
    Long,

    #[default]
    Same,
}

/// A flag describing a point in a glyph outline
#[derive(Debug, Default, Clone, Copy)]
pub struct Flag {
    pub repeats: u8,
    pub on_curve: bool,
    pub x_kind: FlagCoordKind,
    pub y_kind: FlagCoordKind,
}
impl Flag {
    pub fn from_byte(flag: u8) -> Self {
        //
        // Extract flag components
        let on_curve = (flag & 0x01) != 0;
        let x_short_vec = (flag & 0x02) != 0;
        let y_short_vec = (flag & 0x04) != 0;
        let repeats = flag & 0x08;
        let x_same_or_pos = (flag & 0x10) != 0;
        let y_same_or_pos = (flag & 0x20) != 0;

        //
        // Parse out the meanings
        let x_kind = match (x_short_vec, x_same_or_pos) {
            (true, false) => FlagCoordKind::NegShort, /* 1 byte coord - pos */
            (true, true) => FlagCoordKind::PosShort,  /* 1 byte coord - neg */
            (false, false) => FlagCoordKind::Long,    /* 2 byte short */
            _ => FlagCoordKind::Same,                 /* Same as previous */
        };
        let y_kind = match (y_short_vec, y_same_or_pos) {
            (true, false) => FlagCoordKind::NegShort, /* 1 byte coord - pos */
            (true, true) => FlagCoordKind::PosShort,  /* 1 byte coord - neg */
            (false, false) => FlagCoordKind::Long,    /* 2 byte short */
            _ => FlagCoordKind::Same,                 /* Same as previous */
        };

        Self {
            repeats,
            on_curve,
            x_kind,
            y_kind,
        }
    }
}

/// A point in a glyph outline
#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: i16,
    pub y: i16,
    pub on_curve: bool,
}

/// A set of points making up a contour in a glyph
#[derive(Debug, Clone)]
pub struct Contour {
    pub points: Vec<Point>,
}
