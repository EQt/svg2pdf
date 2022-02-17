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
                if !transform.is_default() {
                    dbg!(transform, &(x, y), transform.apply(x, y));
                }
                let (x, y) = c.point(transform.apply(x, y));
                if !transform.is_default() {
                    dbg!(&(x, y));
                }
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
    transform: Transform,
) {
    if let Some(clip_path) = path_id.and_then(|id| ctx.tree.defs_by_id(id)) {
        if let NodeKind::ClipPath(ref path) = *clip_path.borrow() {
            apply_clip_path(path.clip_path.as_ref(), content, ctx, transform);
            for child in clip_path.children() {
                match *child.borrow() {
                    NodeKind::Path(ref path) => {
                        let mut trafo = path.transform.clone();
                        trafo.append(&transform);
                        dbg!(trafo);
                        draw_path(&path.data.0, trafo, content, &ctx.c);
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

#[cfg(test)]
mod tests {
    use super::*;
    use pdf_writer::Rect;

    type BoxErr = Box<dyn std::error::Error>;
    type Result<T, E = BoxErr> = std::result::Result<T, E>;

    fn tree_from_str(xml: &str) -> Result<usvg::Tree> {
        let opt = usvg::Options::default();
        Ok(usvg::Tree::from_str(xml, &opt.to_ref())?)
    }

    fn tree_coord(xml: &str) -> Result<(usvg::Tree, CoordToPdf, Rect)> {
        use super::super::{get_sizings, Options};

        let tree = tree_from_str(xml)?;
        let options = Options::default();
        let (coord, rect) = get_sizings(&tree, &options);
        Ok((tree, coord, rect))
    }

    #[allow(unused)]
    #[test]
    fn draw_path_line() -> Result<()> {
        let (svg, coord, _rect) = tree_coord(
            r##"
            <svg width="400" height="500" xmlns="http://www.w3.org/2000/svg">
              <path d="m -10 -10 400 400" stroke="#000" stroke-width="3"/>
            </svg>
            "##,
        )?;
        let child = svg.root().children().last().unwrap();
        match *child.borrow() {
            NodeKind::Path(ref path) => {
                let mut content = Content::new();
                let path_data = &path.data.0;
                let transform = Transform::default();
                draw_path(&path_data, transform, &mut content, &coord);
                let pdf = String::from_utf8(content.finish())?;
                assert_eq!(pdf, "-10 510 m\n390 110 l");
            }
            ref node => panic!("expected path, found {node:?}"),
        }
        Ok(())
    }
}
