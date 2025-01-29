#![allow(clippy::cast_possible_truncation)]
use crate::reader::{BinaryReader, Parse};
use crate::error::ParseResult;

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
    fn parse<'a>(_: &'a mut BinaryReader<'a>) -> ParseResult<Self> {
        unimplemented!("Use parse_with instead")
    }

    fn parse_with<'a>(&mut self, reader: &'a mut BinaryReader<'a>) -> ParseResult<()> {
        // Simple glyph
        let mut end_pts_of_contours = Vec::new();
        let mut last_pt = 0;

        for _ in 0..self.num_contours {
            last_pt = reader.read_u16()?;
            end_pts_of_contours.push(last_pt);
        }

        let instruction_length = reader.read_u16()?;
        let _instructions = reader.read(instruction_length as usize)?;

        let num_points = last_pt + 1;

        //
        // Parse instructions to get real point count
        let mut flags = Vec::new();
        let mut i = 0;
        while i < num_points {
            let flag = reader.read_u8()?;
            flags.push(flag);
            i += 1;

            if flag & REPEAT != 0 {
                // Repeat bit is set, read the repeat count
                let repeat = reader.read_u8()?;

                // Add the repeated flags
                for _ in 0..repeat {
                    flags.push(flag);
                }

                // Increment `i` for the repeated flags
                i += u16::from(repeat);
            }
        }

        //
        // Parse X coords into objective coords
        let mut last_x = 0;
        let mut x_coordinates = Vec::new();
        for flag in &flags {
            if flag & X_SHORT != 0 {
                let x = i16::from(reader.read_u8()?);
                let is_neg = flag & X_SAME == 0;
                last_x += if is_neg { -x } else { x };
            } else if flag & X_SAME != 0 {
                // Use previous x
            } else {
                let delta = reader.read_i16()?;
                last_x += delta;
            };

            x_coordinates.push(last_x);
        }

        //
        // Parse Y coords into objective coords
        let mut last_y = 0;
        let mut y_coordinates = Vec::new();
        for flag in &flags {
            if flag & Y_SHORT != 0 {
                let y = i16::from(reader.read_u8()?);
                let is_neg = flag & Y_SAME == 0;

                last_y += if is_neg { -y } else { y };
            } else if flag & Y_SAME != 0 {
                // Use previous y
            } else {
                let delta = reader.read_i16()?;
                last_y += delta;
            };

            y_coordinates.push(last_y);
        }

        //
        // Create points
        let mut points = Vec::new();
        for i in 0..num_points {
            let i = i as usize;
            let x = x_coordinates[i];
            let y = y_coordinates[i];
            let on_curve = flags[i] & ON_CURVE != 0;
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

impl SimpleGlyf {
    /// Generate an SVG string representation of the glyph
    #[must_use]
    pub fn as_svg(&self) -> String {
        let mut shape = String::new();

        // Draw all the contours
        for contour in &self.contours {
            shape.push_str(&contour.as_svg());
        }

        // Wrap in SVG container, using x/y min/max as viewBox
        let (xmin, xmax) = (self.x.0, self.x.1);
        let (ymin, ymax) = (-self.y.1, -self.y.0);
        let width = xmax - xmin;
        let height = ymax - ymin;

        // Add a margin, preserving aspect ratio
        let x_margin = 10;
        let aspect_ratio = f32::from(width) / f32::from(height);
        let y_margin = (f32::from(x_margin) / aspect_ratio) as i16;
        let (xmin, xmax) = (xmin - x_margin, xmax + x_margin);
        let (ymin, ymax) = (ymin - y_margin, ymax + y_margin);
        let (width, height) = (xmax - xmin, ymax - ymin);

        // Calculate a new set of sizes for the final display
        let width2 = 50i16;
        let height2 = (f32::from(width2) / aspect_ratio) as i16;

        [
            r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string(),
            format!(
                "<svg xmlns='http://www.w3.org/2000/svg' width='{width2}' height='{height2}' viewBox='{xmin} {ymin} {width} {height}'>{shape}</svg>"
            ),
        ].join("")
    }
}

#[derive(Debug, Clone)]
pub struct Contour {
    pub points: Vec<Point>,
}
impl Contour {
    fn as_svg(&self) -> String {
        let mut path = String::new();

        // Move to the first point
        let mut point_iter = self.points.iter();
        if let Some(first) = point_iter.next() {
            let (x, y) = (first.x, -first.y);
            path.push_str(&format!("M{x} {y}"));
        }

        // Draw lines and curves
        for point in point_iter {
            let (x, y, on_curve) = (point.x, -point.y, point.on_curve);
            let ctrl = if on_curve { 'L' } else { 'T' };
            path.push_str(&format!("{ctrl}{x} {y}"));
        }

        // Close the path
        path.push('Z');

        format!("<path d='{path}' fill='none' stroke='red' stroke-width='5' />")
    }
}

#[derive(Debug, Default, Clone)]
pub struct Point {
    pub x: i16,
    pub y: i16,
    pub on_curve: bool,
}

const ON_CURVE: u8 = 0x01;
const X_SHORT: u8 = 0x02;
const Y_SHORT: u8 = 0x04;
const REPEAT: u8 = 0x08;
const X_SAME: u8 = 0x10;
const Y_SAME: u8 = 0x20;
