use super::{simple::Contour, SimpleGlyf};
use crate::svg::{wrap_svg_component, PartialSvgExt, SvgExt, SvgPathComponent, SvgProperties};

impl PartialSvgExt for SimpleGlyf {
    /// Generate an SVG string representation of the glyph  
    /// If minify is on, the rendering function should perform a best-effort to reduce the size of the SVG output
    fn as_svg_component(&self) -> String {
        //
        // Draw all the contours
        let mut contours = Vec::with_capacity(self.contours.len());
        for contour in &self.contours {
            contours.push(contour.as_svg_component());
        }

        //
        // Collect inside a shape
        let shape = contours.join("");
        format!("<path fill-rule='evenodd' d='{shape}'/>")
    }
}
impl SvgExt for SimpleGlyf {
    fn to_svg(&self) -> String {
        //
        // Get viewbox properties
        let (xmin, xmax) = (self.x.0, self.x.1);
        let (ymin, ymax) = (-self.y.1, -self.y.0);
        let width = xmax - xmin;
        let height = ymax - ymin;
        let viewbox = SvgProperties {
            viewbox_position: (xmin.into(), ymin.into()),
            viewbox_size: (width.into(), height.into()),
            scale_to: Some(75.0),
            margin: Some(50.0),
        };

        //
        // Render SVG container
        let contours = self.as_svg_component();
        wrap_svg_component(&viewbox, &contours)
    }
}

impl PartialSvgExt for Contour {
    fn as_svg_component(&self) -> String {
        //let mut path = String::new();
        let mut path = Vec::with_capacity(self.points.len() * 2);

        // Prep the iterator
        let mut point_iter = self.points.iter();
        let mut first_point = match point_iter.next() {
            Some(pt) => *pt,
            None => return String::new(),
        };
        first_point.on_curve = true; // Prevent infinite loops later

        // Move to the first point
        let (x, y) = (first_point.x, -first_point.y);
        path.push(SvgPathComponent::MoveTo(x, y));

        //
        // Draw lines and curves
        // Each point is either on-curve or off-curve
        // On-curve points are interpreted as a line from the previous point, to the current point
        // Off-curve points are interpreted as a control point for a quadratic bezier curve
        // Multiple Off-curve points can appear in a row, in which case we must calculate 'virtual' on-curve points
        while let Some(point) = point_iter.next() {
            let (dx, dy) = (point.x, -point.y);

            if point.on_curve {
                //
                // Line
                path.push(SvgPathComponent::LineTo(dx, dy));
            } else {
                //
                // Quadratic (poly?)bezier curve
                // Collect a set of control/anchor point pairs
                let mut control_point = point;
                loop {
                    let curve_pt = match point_iter.next() {
                        Some(pt) => pt,
                        None => &first_point,
                    };

                    if curve_pt.on_curve {
                        // End curve
                        let (x1, y1) = (control_point.x, -control_point.y);
                        let (x2, y2) = (curve_pt.x, -curve_pt.y);
                        path.push(SvgPathComponent::QuadraticBezier(x1, y1, x2, y2));
                        break;
                    }

                    // 2 control points in a row. Calculate a virtual on-curve point midway between them
                    let (x1, y1) = (control_point.x, -control_point.y);
                    let (x2, y2) = (
                        (control_point.x + curve_pt.x) / 2,
                        -(control_point.y + curve_pt.y) / 2,
                    );
                    path.push(SvgPathComponent::QuadraticBezier(x1, y1, x2, y2));

                    control_point = curve_pt;
                }
            }
        }

        // Close the path
        path.push(SvgPathComponent::Close);

        SvgPathComponent::minify(&mut path);
        SvgPathComponent::render(&path)
    }
}
