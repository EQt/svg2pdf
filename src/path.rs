use super::{Content, Context, NodeKind};
use crate::scale::CoordToPdf;
use usvg::{PathSegment, Transform};

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

/// Draw a clipping path into a content stream.
pub(crate) fn apply_clip_path(
    path_id: Option<&String>,
    content: &mut Content,
    ctx: &mut Context,
) {
    if let Some(clip_path) = path_id.and_then(|id| ctx.tree.defs_by_id(id)) {
        dbg!(&clip_path);
        if let NodeKind::ClipPath(ref path) = *clip_path.borrow() {
            apply_clip_path(path.clip_path.as_ref(), content, ctx);
            dbg!(clip_path.children().collect::<Vec<_>>());
            for child in clip_path.children() {
                match *child.borrow() {
                    NodeKind::Path(ref path) => {
                        draw_path(&path.data.0, path.transform, content, &ctx.c);
                        content.clip_nonzero();
                        content.end_path();
                    }
                    NodeKind::ClipPath(_) => {}
                    _ => unreachable!(),
                }
            }
        } else {
            unreachable!();
        }
    }
}
