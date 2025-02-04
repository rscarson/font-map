#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use crate::error::ParseResult;
use crate::reader::{BinaryReader, Parse};

use super::{
    simple::{Contour, Point},
    GlyfOutline, SimpleGlyf,
};

const ARG_1_AND_2_ARE_WORDS: u16 = 0x0001;
const ARGS_ARE_XY_VALUES: u16 = 0x0002;
const WE_HAVE_A_SCALE: u16 = 0x0008;
const MORE_COMPONENTS: u16 = 0x0020;
const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 0x0040;
const WE_HAVE_A_TWO_BY_TWO: u16 = 0x0080;

/// A compound glyph outline
#[derive(Debug, Clone, Default)]
pub struct CompoundGlyf {
    /// The components of the compound glyph
    pub components: Vec<Component>,
}

impl CompoundGlyf {
    /// Converts the compound glyph to a simple glyph by resolving the components
    #[must_use]
    pub fn as_simple(&self, glyf_table: &[GlyfOutline]) -> SimpleGlyf {
        let mut contours = Vec::new();
        let (mut min_x, mut max_x) = (i16::MAX, i16::MIN);
        let (mut min_y, mut max_y) = (i16::MAX, i16::MIN);

        debug_msg!("Glyph has {} components", self.components.len());
        for component in &self.components {
            let glyph = &glyf_table[component.glyph_id as usize];
            match glyph {
                GlyfOutline::Simple(glyph) => {
                    let glyph = component.apply_to_glyf(glyph, &contours);
                    contours.extend_from_slice(&glyph.contours);

                    min_x = min_x.min(glyph.x.0);
                    max_x = max_x.max(glyph.x.1);
                    min_y = min_y.min(glyph.y.0);
                    max_y = max_y.max(glyph.y.1);
                }

                GlyfOutline::Compound(glyph) => {
                    let glyph = glyph.as_simple(glyf_table);
                    contours.extend_from_slice(&glyph.contours);

                    min_x = min_x.min(glyph.x.0);
                    max_x = max_x.max(glyph.x.1);
                    min_y = min_y.min(glyph.y.0);
                    max_y = max_y.max(glyph.y.1);
                }
            }
        }

        SimpleGlyf {
            num_contours: contours.len() as i16,
            contours,
            x: (min_x, max_x),
            y: (min_y, max_y),
        }
    }
}

impl Parse for CompoundGlyf {
    fn parse(reader: &mut BinaryReader) -> ParseResult<Self> {
        let mut flags;
        let mut components = Vec::new();
        loop {
            flags = reader.read_u16()?;
            let glyph_id = reader.read_u16()?;

            //
            // Get the arguments
            let is_words = flags & ARG_1_AND_2_ARE_WORDS != 0;
            let is_xy = flags & ARGS_ARE_XY_VALUES != 0;
            let args = match (is_words, is_xy) {
                (true, true) => {
                    let arg1 = reader.read_i16()?;
                    let arg2 = reader.read_i16()?;
                    ComponentArguments::ShortCoordinates(arg1, arg2)
                }

                (true, false) => {
                    let arg1 = reader.read_u16()?;
                    let arg2 = reader.read_u16()?;
                    ComponentArguments::ShortIndex(arg1, arg2)
                }

                (false, true) => {
                    let arg1 = reader.read_i8()?;
                    let arg2 = reader.read_i8()?;
                    ComponentArguments::ByteCoordinates(arg1, arg2)
                }

                (false, false) => {
                    let arg1 = reader.read_u8()?;
                    let arg2 = reader.read_u8()?;
                    ComponentArguments::ByteIndex(arg1, arg2)
                }
            };

            //
            // Get the scale
            let scale = if flags & WE_HAVE_A_SCALE != 0 {
                let scale = reader.read_f2dot14()?;
                ComponentScale::Scale(scale)
            } else if flags & WE_HAVE_AN_X_AND_Y_SCALE != 0 {
                let x_scale = reader.read_f2dot14()?;
                let y_scale = reader.read_f2dot14()?;
                ComponentScale::XYScale(x_scale, y_scale)
            } else if flags & WE_HAVE_A_TWO_BY_TWO != 0 {
                let x_scale = reader.read_f2dot14()?;
                let scale01 = reader.read_f2dot14()?;
                let scale10 = reader.read_f2dot14()?;
                let y_scale = reader.read_f2dot14()?;
                ComponentScale::TwoByTwo(x_scale, scale01, scale10, y_scale)
            } else {
                ComponentScale::None
            };

            components.push(Component {
                glyph_id,
                flags,
                args,
                scale,
            });

            if flags & MORE_COMPONENTS == 0 {
                break;
            }
        }

        Ok(Self { components })
    }
}

#[derive(Debug, Clone)]
pub enum ComponentArguments {
    ByteCoordinates(i8, i8),
    ShortCoordinates(i16, i16),
    ByteIndex(u8, u8),
    ShortIndex(u16, u16),
}

#[derive(Debug, Clone)]
pub enum ComponentScale {
    None,
    Scale(f64),
    XYScale(f64, f64),
    TwoByTwo(f64, f64, f64, f64),
}

#[derive(Debug, Clone)]
pub struct Component {
    pub glyph_id: u16,
    pub flags: u16,
    pub args: ComponentArguments,
    pub scale: ComponentScale,
}
impl Component {
    #[allow(clippy::many_single_char_names)]
    pub fn apply_to_point(&self, point: &mut Point, parent: &Vec<Contour>, child: &Vec<Contour>) {
        //
        // Get the first set of parameters
        let (a, b, c, d) = match self.scale {
            ComponentScale::None => (1.0, 0.0, 0.0, 1.0),
            ComponentScale::Scale(scale) => (scale, 0.0, 0.0, scale),
            ComponentScale::XYScale(x_scale, y_scale) => (x_scale, 0.0, 0.0, y_scale),
            ComponentScale::TwoByTwo(x_scale, scale01, scale10, y_scale) => {
                (x_scale, scale01, scale10, y_scale)
            }
        };

        //
        // Get the 2nd set
        let (e, f) = match self.args {
            ComponentArguments::ShortCoordinates(e, f) => {
                let e = f64::from(e);
                let f = f64::from(f);
                let e = a * e + b * f;
                let f = c * e + d * f;
                (e, f)
            }
            ComponentArguments::ByteCoordinates(e, f) => {
                let e = f64::from(e);
                let f = f64::from(f);
                let e = a * e + b * f;
                let f = c * e + d * f;
                (e, f)
            }

            ComponentArguments::ShortIndex(compound_i, component_i) => {
                let mut index = compound_i;
                let mut point1 = Point::default();
                for contour in parent {
                    for point in &contour.points {
                        if index == 0 {
                            point1 = *point;
                            break;
                        }
                        index -= 1;
                    }
                }

                index = component_i;
                let mut point2 = Point::default();
                for contour in child {
                    for point in &contour.points {
                        if index == 0 {
                            point2 = *point;
                            break;
                        }
                        index -= 1;
                    }
                }

                let e = f64::from(point1.x) - f64::from(point2.x);
                let f = f64::from(point1.y) - f64::from(point2.y);
                (e, f)
            }

            ComponentArguments::ByteIndex(compound_i, component_i) => {
                let mut index = compound_i;
                let mut point1 = Point::default();
                for contour in parent {
                    for point in &contour.points {
                        if index == 0 {
                            point1 = *point;
                            break;
                        }
                        index -= 1;
                    }
                }

                index = component_i;
                let mut point2 = Point::default();
                for contour in child {
                    for point in &contour.points {
                        if index == 0 {
                            point2 = *point;
                            break;
                        }
                        index -= 1;
                    }
                }

                let e = f64::from(point1.x) - f64::from(point2.x);
                let f = f64::from(point1.y) - f64::from(point2.y);
                (e, f)
            }
        };

        //
        // Calculate the last set of parameters
        let m0 = a.abs().max(b.abs());
        let n0 = c.abs().max(d.abs());
        let m = if (a.abs() - c.abs()) <= 33.0 / 65536.0 {
            2.0 * m0
        } else {
            m0
        };
        let n = if (b.abs() - d.abs()) <= 33.0 / 65536.0 {
            2.0 * n0
        } else {
            n0
        };

        //
        // Perform linear transformation
        let x = m * ((a / m) * f64::from(point.x) + (c / m) * f64::from(point.y) + e);
        let y = n * ((b / n) * f64::from(point.x) + (d / n) * f64::from(point.y) + f);

        point.x = x.round() as i16;
        point.y = y.round() as i16;
    }

    pub fn apply_to_glyf(&self, glyf: &SimpleGlyf, parent: &Vec<Contour>) -> SimpleGlyf {
        let mut new_glyf = glyf.clone();

        for contour in &mut new_glyf.contours {
            for point in &mut contour.points {
                self.apply_to_point(point, parent, &glyf.contours);
            }
        }

        //
        // Apply to bounds too
        let mut min_pt = Point {
            x: glyf.x.0,
            y: glyf.y.0,
            on_curve: false,
        };
        let mut max_pt = Point {
            x: glyf.x.1,
            y: glyf.y.1,
            on_curve: false,
        };
        self.apply_to_point(&mut min_pt, parent, &glyf.contours);
        self.apply_to_point(&mut max_pt, parent, &glyf.contours);
        new_glyf.x = (min_pt.x, max_pt.x);
        new_glyf.y = (min_pt.y, max_pt.y);

        new_glyf
    }
}
