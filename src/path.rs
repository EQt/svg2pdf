use usvg::{PathSegment, Transform};
use super::Content;
use crate::scale::CoordToPdf;

/// Draw a path into a content stream. Does close the path but not perform any
/// drawing operators.
pub fn draw_path(
    path_data: &[PathSegment],
    transform: Transform,
    content: &mut Content,
    c: &CoordToPdf,
) {
    for &operation in path_data {
        match operation {
            PathSegment::MoveTo { x, y } => {
                let (x, y) = c.point(transform.apply(x, y));
                content.move_to(x, y);
            }
            PathSegment::LineTo { x, y } => {
                let (x, y) = c.point(transform.apply(x, y));
                content.line_to(x, y);
            }
            PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                let (x1, y1) = c.point(transform.apply(x1, y1));
                let (x2, y2) = c.point(transform.apply(x2, y2));
                let (x, y) = c.point(transform.apply(x, y));
                content.cubic_to(x1, y1, x2, y2, x, y);
            }
            PathSegment::ClosePath => {
                content.close_path();
            }
        }
    }
}
