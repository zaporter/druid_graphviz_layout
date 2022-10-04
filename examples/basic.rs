
use druid::kurbo::BezPath;
use druid::piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget::prelude::*;
use druid::{
    Affine, AppLauncher, Color, FontDescriptor, LocalizedString, Point, Rect, TextLayout,
    WindowDesc,
};

use druid_graphviz_layout::backends::druid::{DruidCtxWriter, GraphvizWidget, VisualGraphData};
use druid_graphviz_layout::backends::svg::SVGWriter;
use druid_graphviz_layout::core::base::Orientation;
use druid_graphviz_layout::core::format::Renderable;
use druid_graphviz_layout::core::style::*;
use druid_graphviz_layout::core::utils::save_to_file;
use druid_graphviz_layout::std_shapes::shapes::*;
use druid_graphviz_layout::topo::layout::VisualGraph;
use druid_graphviz_layout::topo::placer::Placer;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, Ref};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;




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
    let window = WindowDesc::new(GraphvizWidget {}).title(LocalizedString::new("Fancy Colors"));
    vg.prepare_render(false, false);
    let vgd = VisualGraphData::new(vg);

    AppLauncher::with_window(window)
        .log_to_console()
        .launch(vgd)
        .expect("launch failed");
}

