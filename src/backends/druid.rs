//! SVG rendering backend that accepts draw calls and saves the output to a file.


use crate::core::color::Color;
use crate::core::format::{ClipHandle, RenderBackend};
use crate::core::geometry::Point;
use crate::core::style::StyleAttr;
use crate::topo::layout::VisualGraph;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;
use std::rc::Rc;

use druid::kurbo::{Ellipse, BezPath};
use druid::{
    kurbo::{Circle, Shape, Line, RoundedRect},
    widget::Label,
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx,
    PaintCtx, RenderContext, Size, UpdateCtx, Widget, WidgetPod, Vec2, MouseButton, piet::{TextLayoutBuilder, Text, TextAttribute}, RadialGradient, LinearGradient, UnitPoint,
};

pub struct GraphvizWidget;

impl GraphvizWidget {

    pub fn new() -> Self{
        GraphvizWidget{}
    }
}

impl Widget<VisualGraphData> for GraphvizWidget {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut VisualGraphData, _env: &Env) {}

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &VisualGraphData,
        _env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event{
            data.graph.borrow_mut().prepare_render(false, false);
        }

    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &VisualGraphData, data: &VisualGraphData, _env: &Env) {

        data.graph.borrow_mut().prepare_render(false, false);
        //.into_inner().borrow_mut().prepare_render(false, false);
        ctx.request_paint();
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &VisualGraphData,
        _env: &Env,
    ) -> Size {
        if bc.is_width_bounded() && bc.is_height_bounded() {
            bc.max()
        } else {
            bc.constrain_aspect_ratio(1.0, 400.)
            // let size = Size::new(100.0, 100.0);
            // bc.constrain(size)
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &VisualGraphData, env: &Env) {
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.fill(rect, &druid::Color::WHITE);
        let mut writer = DruidCtxWriter::new();
        data.graph.borrow().render(false,&mut writer);
        writer.write(ctx, size);
    }
}

#[derive(Data,Clone)]
pub struct VisualGraphData{
    pub graph: Rc<RefCell<VisualGraph>>
}
impl VisualGraphData {
    pub fn new(graph : VisualGraph)->Self{
        Self{
            graph:Rc::new(RefCell::new(graph)) 
        }
    }
}

pub struct DruidCtxWriter {
    rects: Vec<DrawRectInfo>,
    circles: Vec<DrawCircleInfo>,
    texts: Vec<DrawTextInfo>,
    arrows: Vec<DrawArrowInfo>,
    lines: Vec<DrawLineInfo>,
    clips: HashMap<ClipHandle, ClipInfo>,
    view_size: Point,
}

struct DrawRectInfo {
    xy: Point,
    size: Point,
    look: StyleAttr,
    clip: Option<ClipHandle>,
}

struct DrawCircleInfo {
    xy: Point,
    size: Point,
    look: StyleAttr,
}

struct DrawTextInfo {
    xy: Point,
    text: String,
    look: StyleAttr,
}

struct DrawArrowInfo {
    paths: Vec<(Point, Point)>,
    dashed: bool,
    head: (bool, bool),
    look: StyleAttr,
    text: String,
}

struct DrawLineInfo {
    start: Point,
    stop: Point,
    look: StyleAttr,
}
struct ClipInfo {
    xy: Point,
    size: Point,
    rounded_px: usize,
}

impl DruidCtxWriter {
    pub fn new() -> Self {
        Self {
            rects: Vec::new(),
            circles: Vec::new(),
            texts: Vec::new(),
            arrows: Vec::new(),
            lines: Vec::new(),
            clips: HashMap::new(),
            view_size: Point { x: 0., y: 0. },
        }
    }
}

impl DruidCtxWriter {
    // Grow the viewable svg window to include the point \p point plus some
    // offset \p size.
    fn grow_window(&mut self, point: Point, size: Point) {
        self.view_size.x = self.view_size.x.max(point.x + size.x + 5.);
        self.view_size.y = self.view_size.y.max(point.y + size.y + 5.);
    }
    fn scale_pt(p:Point, max_vs:f64, window_size: &Size)->Point{
        Point{
            x: (p.x/max_vs)*window_size.width,
            y: (p.y/max_vs)*window_size.height,
        }
    }
    pub fn write(&self, ctx: &mut PaintCtx, window_size: Size) {
        let max_vs = self.view_size.x.max(self.view_size.y);

        for elem in &self.rects {
            let sx = (elem.xy.x / max_vs) * window_size.width;
            let ex = ((elem.xy.x + elem.size.x) / max_vs) * window_size.width;
            let sy = (elem.xy.y / max_vs) * window_size.height;
            let ey = ((elem.xy.y + elem.size.y) / max_vs) * window_size.height;
            ctx.fill(RoundedRect::new(sx, sy, ex, ey, 10.0), &druid::Color::WHITE);
            ctx.stroke(RoundedRect::new(sx, sy, ex, ey, 10.0), &druid::Color::BLACK, 1.0);
        }

        for elem in &self.circles {
            let sx = (elem.xy.x / max_vs) * window_size.width;
            let sizex = ((elem.size.x) / max_vs) * window_size.width;
            let sy = (elem.xy.y / max_vs) * window_size.height;
            let sizey = ((elem.size.y) / max_vs) * window_size.height;
            ctx.fill(
                Ellipse::new(
                    druid::Point::new(sx, sy),
                    druid::Vec2::new(sizex, sizey),
                    0.,
                ),
                &druid::Color::RED,
            );
        }
        for elem in &self.lines {
            let sx = (elem.start.x / max_vs) * window_size.width;
            let sy = (elem.start.y / max_vs) * window_size.height;
            let ex = (elem.stop.x / max_vs) * window_size.width;
            let ey = (elem.stop.y / max_vs) * window_size.height;
            ctx.stroke(
                Line::new(druid::Point::new(sx, sy), druid::Point::new(ex, ey)),
                &druid::Color::GREEN,
                10.0,
            );
        }
        for elem in &self.arrows {
            let mut paths = elem.paths.iter();
            let mut bpath = BezPath::new();
            let first = paths.next().unwrap();
            let second = paths.next().unwrap();
            let m = Self::scale_pt(first.0, max_vs, &window_size);
            let c1 = Self::scale_pt(first.1, max_vs, &window_size);
            let c2 = Self::scale_pt(second.0, max_vs, &window_size);
            let c3 = Self::scale_pt(second.1, max_vs, &window_size);
            bpath.move_to(druid::Point::new(m.x, m.y));
            // TODO
            //https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/Paths#curve_c
            //https://docs.rs/druid/latest/druid/piet/kurbo/struct.BezPath.html
            // THIS IS NOT CORRECT
            bpath.line_to(druid::Point::new(c3.x,c3.y));
            // bpath.curve_to(
            //     druid::Point::new(c1.x, c1.x),
            //     druid::Point::new(c2.x, c2.y),
            //     druid::Point::new(c3.x, c3.y),
            // );
            while let Some((p1,p2)) = paths.next(){
                let p1 = Self::scale_pt(*p1, max_vs, &window_size);
                let p2 = Self::scale_pt(*p2, max_vs, &window_size);
                bpath.quad_to(druid::Point::new(p1.x,p1.y), druid::Point::new(p2.x,p2.y));
            }
            ctx.stroke(bpath, &druid::Color::GREEN, 10.0);
        }
        for elem in &self.texts {
            let p = Self::scale_pt(elem.xy, max_vs, &window_size);

            let text = ctx.text();
            let to_draw = text.new_text_layout(elem.text.clone())
                .default_attribute(TextAttribute::FontSize(14.0))
                .build().unwrap();
            
            ctx.draw_text(&to_draw, druid::Point::new(p.x,p.y));
        }
    }
}
impl RenderBackend for DruidCtxWriter {
    fn draw_rect(&mut self, xy: Point, size: Point, look: &StyleAttr, clip: Option<ClipHandle>) {
        self.grow_window(xy, size);
        self.rects.push(DrawRectInfo {
            xy,
            size,
            look: look.clone(),
            clip,
        });
    }

    fn draw_circle(&mut self, xy: Point, size: Point, look: &StyleAttr) {
        self.grow_window(xy, size);
        self.circles.push(DrawCircleInfo {
            xy,
            size,
            look: look.clone(),
        });
    }

    fn draw_text(&mut self, xy: Point, text: &str, look: &StyleAttr) {
        // TODO grow window?
        self.texts.push(DrawTextInfo {
            xy,
            text: text.to_owned(),
            look: look.clone(),
        });
    }

    fn draw_arrow(
        &mut self,
        // This is a list of vectors. The first vector is the "exit" vector
        // from the first point, and the rest of the vectors are "entry" vectors
        // into the following points.
        path: &[(Point, Point)],
        dashed: bool,
        head: (bool, bool),
        look: &StyleAttr,
        text: &str,
    ) {
        for point in path {
            self.grow_window(point.0, Point::zero());
            self.grow_window(point.1, Point::zero());
        }
        self.arrows.push(DrawArrowInfo {
            paths: path.to_owned(),
            dashed,
            head,
            look: look.clone(),
            text: text.to_owned(),
        });
    }

    fn draw_line(&mut self, start: Point, stop: Point, look: &StyleAttr) {
        self.lines.push(DrawLineInfo {
            start,
            stop,
            look: look.clone(),
        });
    }

    fn create_clip(&mut self, xy: Point, size: Point, rounded_px: usize) -> ClipHandle {
        let handle = self.clips.len();
        self.clips.insert(
            handle,
            ClipInfo {
                xy,
                size,
                rounded_px,
            },
        );

        handle
    }
}
