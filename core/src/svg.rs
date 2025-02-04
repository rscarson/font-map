#![allow(clippy::similar_names)]
use std::cmp::Ordering;

pub trait PartialSvgExt {
    /// Returns the outline of this glyph a set of svg objects, not wrapped in an svg container
    fn as_svg_component(&self) -> String;
}

/// Implements methods for converting a glyph to an SVG representation
pub trait SvgExt: PartialSvgExt {
    /// Returns the outline of this glyph as an SVG document
    #[must_use]
    fn to_svg(&self) -> String;

    /// Returns the gzip compressed SVGZ data of this glyph
    ///
    /// # Errors
    /// Returns an error if the data cannot be compressed
    #[cfg(feature = "extended-svg")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-svg")))]
    fn to_svgz(&self) -> std::io::Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut buffer = Vec::new();
        let outline = self.to_svg();
        let mut encoder = GzEncoder::new(&mut buffer, flate2::Compression::best());
        encoder.write_all(outline.as_bytes())?;
        encoder.finish()?;

        Ok(buffer)
    }

    /// Generates a `data:` link containing the outline svg data for this glyph  
    ///
    /// # Errors
    /// Returns an error if the data cannot be encoded properly
    #[cfg(feature = "extended-svg")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-svg")))]
    fn to_svg_dataimage_url(&self) -> std::io::Result<String> {
        use base64::{engine::general_purpose::STANDARD, write::EncoderStringWriter};
        use std::io::Write;

        let buffer = self.to_svg().into_bytes();

        let mut encoder = EncoderStringWriter::new(&STANDARD);
        encoder.write_all(&buffer)?;

        let data = encoder.into_inner();
        let url = format!("data:image/svg+xml;base64,{data}",);
        Ok(url)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SvgProperties {
    /// Top-left position of the viewbox
    pub viewbox_position: (f32, f32),

    /// Size of the viewbox
    pub viewbox_size: (f32, f32),

    /// If provided, represents the horizontal view size  
    /// A vertical size will be calculated based on the aspect ratio of the viewbox
    pub scale_to: Option<f32>,

    /// If provided, represents the horizontal margin to add to the viewbox  
    /// A vertical margin will be calculated based on the aspect ratio of the viewbox
    pub margin: Option<f32>,
}

pub enum SvgPathComponent {
    MoveTo(i16, i16),
    HorizontalTo(i16),
    VerticalTo(i16),
    LineTo(i16, i16),
    QuadraticBezier(i16, i16, i16, i16),
    RelativeLineTo(i16, i16),
    RelativeQuadraticBezier(i16, i16, i16, i16),
    RelativeSmoothQuadraticBezier(i16, i16),
    RelativeVerticalTo(i16),
    RelativeHorizontalTo(i16),
    Close,
}
impl SvgPathComponent {
    pub fn render(path: &[Self]) -> String {
        let mut out = String::with_capacity(path.len() * 12); // Estimate capacity
        let mut ctrl = ' ';
        for component in path {
            let (cmd, args) = component.components();

            let mut skip_next = false;
            if ctrl != cmd {
                out.push(cmd);
                ctrl = cmd;
                skip_next = true;
            }

            let mut buffer = itoa::Buffer::new();
            for c in args {
                if c >= 0 && !skip_next {
                    out.push(' ');
                }

                out.push_str(buffer.format(c)); // Convert without `format!`
                skip_next = false;
            }
        }

        out
    }

    pub fn minify(path: &mut [Self]) {
        if path.len() < 2 {
            return;
        }

        //
        // Remove redundancies
        let mut i = 1;
        while i < path.len() {
            let prev = &path[i - 1];
            let curr = &path[i];

            let (Some(prev_line), Some(curr_line)) =
                (prev.line_components(), curr.line_components())
            else {
                i += 1;
                continue;
            };

            let (dx, dy) = (prev_line.0.cmp(&curr_line.0), prev_line.1.cmp(&curr_line.1));
            match (dx, dy) {
                (Ordering::Equal, Ordering::Equal) => {
                    // New line is a No-Op
                    // But these are sometimes used for rendering fill
                }

                (Ordering::Equal, _) => {
                    // New line is vertical
                    path[i] = SvgPathComponent::VerticalTo(curr_line.1);
                }

                (_, Ordering::Equal) => {
                    // New line is horizontal
                    path[i] = SvgPathComponent::HorizontalTo(curr_line.0);
                }

                _ => {}
            }

            i += 1;
        }

        //
        // Convert LineTo and QuadraticBezier chains to relative coordinates
        let mut px = 0;
        let mut py = 0;
        let mut last_q = None; // Track last Q's endpoint
        for component in path.iter_mut() {
            match component {
                Self::MoveTo(x, y) => {
                    px = *x;
                    py = *y;
                }

                Self::LineTo(x, y) => {
                    let (x, y) = (*x, *y);
                    let (dx, dy) = (x - px, y - py);
                    *component = Self::RelativeLineTo(dx, dy);
                    px = x;
                    py = y;
                }

                Self::QuadraticBezier(x1, y1, x, y) => {
                    let (x, y) = (*x, *y);
                    let (dx1, dy1, dx, dy) = (*x1 - px, *y1 - py, x - px, y - py);
                    *component = Self::RelativeQuadraticBezier(dx1, dy1, dx, dy);
                    px = x;
                    py = y;
                }

                Self::HorizontalTo(x) => {
                    let x = *x;
                    let dx = x - px;
                    *component = Self::RelativeHorizontalTo(dx);
                    px = x;
                }

                Self::VerticalTo(y) => {
                    let y = *y;
                    let dy = y - py;
                    *component = Self::RelativeVerticalTo(dy);
                    py = y;
                }

                _ => {}
            }

            //
            // Detect smooth curves
            match component {
                Self::RelativeQuadraticBezier(x1, y1, x, y) => {
                    let (x1, y1, x, y) = (*x1, *y1, *x, *y);
                    // Is the ctrl point a reflection of the last Q's endpoint?
                    if let Some((_, _, px, py)) = last_q {
                        if x1 == px && y1 == py {
                            *component = Self::RelativeSmoothQuadraticBezier(x, y);
                        }
                    }

                    last_q = Some((x1, y1, x, y));
                }

                _ => {
                    last_q = None;
                }
            }
        }
    }

    pub fn line_components(&self) -> Option<(i16, i16)> {
        match self {
            Self::MoveTo(x, y) | Self::LineTo(x, y) => Some((*x, *y)),
            Self::HorizontalTo(x) => Some((*x, i16::MAX)),
            Self::VerticalTo(y) => Some((i16::MAX, *y)),
            _ => None,
        }
    }

    pub fn components(&self) -> (char, Vec<i16>) {
        match self {
            Self::MoveTo(x, y) => ('M', vec![*x, *y]),
            Self::HorizontalTo(x) => ('H', vec![*x]),
            Self::VerticalTo(y) => ('V', vec![*y]),
            Self::LineTo(x, y) => ('L', vec![*x, *y]),
            Self::QuadraticBezier(x1, y1, x2, y2) => ('Q', vec![*x1, *y1, *x2, *y2]),
            Self::RelativeLineTo(x, y) => ('l', vec![*x, *y]),
            Self::RelativeQuadraticBezier(x1, y1, x2, y2) => ('q', vec![*x1, *y1, *x2, *y2]),
            Self::RelativeSmoothQuadraticBezier(x, y) => ('t', vec![*x, *y]),
            Self::RelativeVerticalTo(y) => ('v', vec![*y]),
            Self::RelativeHorizontalTo(x) => ('h', vec![*x]),
            Self::Close => ('Z', vec![]),
        }
    }
}

/// Wrap a set of SVG components in an SVG container
pub fn wrap_svg_component(properties: &SvgProperties, component: &str) -> String {
    let (width, height) = properties.viewbox_size;
    let (xmin, ymin) = properties.viewbox_position;
    let aspect_ratio = width / height;

    //
    // Calculate margins
    let x_margin = properties.margin.unwrap_or_default();
    let y_margin = x_margin / aspect_ratio;

    //
    // Get new viewbox properties
    let (xmin, ymin) = (xmin - x_margin, ymin - y_margin);
    let (width, height) = (width + 2.0 * x_margin, height + 2.0 * y_margin);

    //
    // Calculate new height
    let vwidth = properties.scale_to.unwrap_or(width);
    let vheight = vwidth / aspect_ratio;

    //
    // Put the pieces together
    let vsize = format!("width='{vwidth}' height='{vheight}'");
    let viewbox = format!("viewBox='{xmin} {ymin} {width} {height}'",);
    format!("<svg xmlns='http://www.w3.org/2000/svg' style='background-color:#FFF' {vsize} {viewbox}>{component}</svg>")
}
