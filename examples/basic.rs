
use druid::kurbo::BezPath;
use druid::piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget::prelude::*;
use druid::{
    Affine, AppLauncher, Color, FontDescriptor, LocalizedString, Point, Rect, TextLayout,
    WindowDesc,
};

use druid_graphviz_layout::backends::druid::DruidCtxWriter;
use druid_graphviz_layout::backends::svg::SVGWriter;
use druid_graphviz_layout::core::base::Orientation;
use druid_graphviz_layout::core::style::*;
use druid_graphviz_layout::core::utils::save_to_file;
use druid_graphviz_layout::std_shapes::shapes::*;
use druid_graphviz_layout::topo::layout::VisualGraph;
use druid_graphviz_layout::topo::placer::Placer;
use std::fs;
use std::rc::Rc;

struct BasicGraph;

// If this widget has any child widgets it should call its event, update and layout
// (and lifecycle) methods as well to make sure it works. Some things can be filtered,
// but a general rule is to just pass it through unless you really know you don't want it.
impl Widget<VGA> for BasicGraph {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut VGA, _env: &Env) {}

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &VGA,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &VGA, _data: &VGA, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &VGA,
        _env: &Env,
    ) -> Size {
        // BoxConstraints are passed by the parent widget.
        // This method can return any Size within those constraints:
        // bc.constrain(my_size)
        //
        // To check if a dimension is infinite or not (e.g. scrolling):
        // bc.is_width_bounded() / bc.is_height_bounded()
        //
        // bx.max() returns the maximum size of the widget. Be careful
        // using this, since always make sure the widget is bounded.
        // If bx.max() is used in a scrolling widget things will probably
        // not work correctly.
        if bc.is_width_bounded() && bc.is_height_bounded() {
            bc.max()
        } else {
            let size = Size::new(100.0, 100.0);
            bc.constrain(size)
        }
    }

    // The paint method gets called last, after an event flow.
    // It goes event -> update -> layout -> paint, and each method can influence the next.
    // Basically, anything that changes the appearance of a widget causes a paint.
    fn paint(&mut self, ctx: &mut PaintCtx, data: &VGA, env: &Env) {
        // Clear the whole widget with the color of your choice
        // (ctx.size() returns the size of the layout rect we're painting in)
        // Note: ctx also has a `clear` method, but that clears the whole context,
        // and we only want to clear this widget's area.
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.fill(rect, &Color::WHITE);


        let mut writer = DruidCtxWriter::new();

        data.g.render(false,&mut writer);
        writer.write(ctx, size);
    }
}

#[derive(Data,Clone)]
struct VGA {
    g:Rc<VisualGraph>,
}

pub fn main() {
    let mut vg = VisualGraph::new(Orientation::LeftToRight);

    // Define the node styles:
    let sp0 = ShapeKind::new_box("one");
    let sp1 = ShapeKind::new_box("two");
    let look0 = StyleAttr::simple();
    let look1 = StyleAttr::simple();
    let sz = druid_graphviz_layout::core::geometry::Point::new(100., 100.);
    // Create the nodes:
    let node0 = Element::create(sp0, look0, Orientation::LeftToRight, sz);
    let node1 = Element::create(sp1, look1, Orientation::LeftToRight, sz);

    // Add the nodes to the graph, and save a handle to each node.
    let handle0 = vg.add_node(node0);
    let handle1 = vg.add_node(node1);

    // Add an edge between the nodes.
    let arrow = Arrow::simple("123");
    vg.add_edge(arrow, handle0, handle1);
    let window = WindowDesc::new(BasicGraph {}).title(LocalizedString::new("Fancy Colors"));
    vg.prepare_render(false, false);
    AppLauncher::with_window(window)
        .log_to_console()
        .launch(VGA { g: Rc::new(vg) })
        .expect("launch failed");
}

fn make_image_data(width: usize, height: usize) -> Vec<u8> {
    let mut result = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let ix = (y * width + x) * 4;
            result[ix] = x as u8;
            result[ix + 1] = y as u8;
            result[ix + 2] = !(x as u8);
            result[ix + 3] = 127;
        }
    }
    result
}
