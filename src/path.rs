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
                // dbg!(&operation, &(x, y));
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
    transform: Transform,
) {
    if let Some(clip_path) = path_id.and_then(|id| ctx.tree.defs_by_id(id)) {
        if let NodeKind::ClipPath(ref path) = *clip_path.borrow() {
            apply_clip_path(path.clip_path.as_ref(), content, ctx, transform);
            for child in clip_path.children() {
                match *child.borrow() {
                    NodeKind::Path(ref path) => {
                        #[allow(unused_mut)]
                        let mut trafo = path.transform;
                        trafo.prepend(&transform);
                        #[allow(unused_mut)]
                        let mut c = ctx.c.clone();
                        // c.transform(transform.to_arr());
                        draw_path(&path.data.0, trafo, content, &c);
                        content.clip_nonzero();
                        // content.stroke();
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
        let path_data = match *child.borrow() {
            NodeKind::Path(ref path) => path.data.0.clone(),
            ref node => panic!("expected path, found {node:?}"),
        };
        let render = |tr| {
            let mut content = Content::new();
            draw_path(&path_data, tr, &mut content, &coord);
            String::from_utf8(content.finish()).unwrap()
        };
        let (x1, y1) = (-10, 500 - (-10));
        let (dx, dy) = (400, -400);
        {
            let trafo = Transform::default();
            let pdf = format!("{x1} {y1} m\n{} {} l", x1 + dx, y1 + dy);
            assert_eq!(&pdf, "-10 510 m\n390 110 l");
            assert_eq!(render(trafo), pdf);
        }
        {
            // translate(130, 80) as in tests/clip_line.svg
            let (tx, ty) = (130, 80);
            let (xt, yt) = (x1 + tx, y1 - ty);
            let mut trafo = Transform::default();
            trafo.translate(tx as _, ty as _);
            let pdf = format!("{xt} {yt} m\n{} {} l", xt + dx, yt + dy);
            assert_eq!(&pdf, "120 430 m\n520 30 l");
            assert_eq!(render(trafo), pdf);
        }
        {
            // translate(130, -80)
            let (tx, ty) = (130, -80);
            let (xt, yt) = (x1 + tx, y1 - ty);
            let mut trafo = Transform::default();
            trafo.translate(tx as _, ty as _);
            let pdf = format!("{xt} {yt} m\n{} {} l", xt + dx, yt + dy);
            assert_eq!(&pdf, "120 590 m\n520 190 l");
            assert_eq!(render(trafo), pdf);
        }
        Ok(())
    }
}
