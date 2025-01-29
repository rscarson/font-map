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

            let mut component = Component {
                glyph_id,
                ..Default::default()
            };

            if flags & ARG_1_AND_2_ARE_WORDS != 0 {
                let arg1 = reader.read_i16()?;
                let arg2 = reader.read_i16()?;

                component.arg1 = i32::from(arg1);
                component.arg2 = i32::from(arg2);
            } else {
                let arg1 = reader.read_u8()?;
                let arg2 = reader.read_u8()?;

                component.arg1 = i32::from(arg1);
                component.arg2 = i32::from(arg2);
            }

            if flags & WE_HAVE_A_SCALE != 0 {
                let scale = reader.read_f2dot14()?;
                component.scale = Some(scale);
            } else if flags & WE_HAVE_AN_X_AND_Y_SCALE != 0 {
                let x_scale = reader.read_f2dot14()?;
                let y_scale = reader.read_f2dot14()?;

                component.xy_scale = Some((x_scale, y_scale));
            } else if flags & WE_HAVE_A_TWO_BY_TWO != 0 {
                let x_scale = reader.read_f2dot14()?;
                let scale01 = reader.read_f2dot14()?;
                let scale10 = reader.read_f2dot14()?;
                let y_scale = reader.read_f2dot14()?;

                component.two_by_two = Some((x_scale, scale01, scale10, y_scale));
            }

            components.push(component);

            if flags & MORE_COMPONENTS == 0 {
                break;
            }
        }

        Ok(Self { components })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Component {
    pub glyph_id: u16,
    pub flags: u16,
    pub arg1: i32,
    pub arg2: i32,
    pub scale: Option<f64>,
    pub xy_scale: Option<(f64, f64)>,
    pub two_by_two: Option<(f64, f64, f64, f64)>,
}
impl Component {
    pub fn scale_point(&self, point: &mut Point) {
        if let Some(scale) = self.scale {
            point.x = (f64::from(point.x) * scale) as i16;
            point.y = (f64::from(point.y) * scale) as i16;
        } else if let Some((x_scale, y_scale)) = self.xy_scale {
            point.x = (f64::from(point.x) * x_scale) as i16;
            point.y = (f64::from(point.y) * y_scale) as i16;
        } else if let Some((x_scale, scale01, scale10, y_scale)) = self.two_by_two {
            let x = f64::from(point.x);
            let y = f64::from(point.y);

            point.x = (x * x_scale + y * scale01) as i16;
            point.y = (x * scale10 + y * y_scale) as i16;
        }
    }

    pub fn apply_to_point(&self, point: &mut Point, parent: &Vec<Contour>, child: &Vec<Contour>) {
        //
        // First we get the scaling factors
        self.scale_point(point);

        //
        // Now we apply translation
        if self.flags & ARGS_ARE_XY_VALUES != 0 {
            // X-Y offset
            point.x += self.arg1 as i16;
            point.y += self.arg2 as i16;
        } else {
            // Point index
            let mut index = self.arg1;
            let mut point1 = Point::default();
            for contour in parent {
                for point in &contour.points {
                    if index == 0 {
                        point1 = point.clone();
                        break;
                    }
                    index -= 1;
                }
            }

            index = self.arg2;
            let mut point2 = Point::default();
            for contour in child {
                for point in &contour.points {
                    if index == 0 {
                        point2 = point.clone();
                        break;
                    }
                    index -= 1;
                }
            }

            self.scale_point(&mut point1);
            self.scale_point(&mut point2);

            point.x += point1.x - point2.x;
            point.y += point1.y - point2.y;
        }
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
