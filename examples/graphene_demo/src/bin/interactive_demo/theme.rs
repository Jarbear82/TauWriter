pub use graphene_gpui::style_bridge::color_value_to_rgba as color_value_to_gpui_color;
pub use graphene_gpui::UiTheme as Theme;
use gpui_kit::{Pixels, Point};

pub const LAYOUT_NAMES: &[&str] = graphene_layout::LayoutCommand::ALL_NAMES;

pub fn distance_to_segment(p: Point<Pixels>, a: Point<Pixels>, b: Point<Pixels>) -> f32 {
    graphene_gpui::distance_to_segment(
        gpui_kit::point(f32::from(p.x), f32::from(p.y)),
        gpui_kit::point(f32::from(a.x), f32::from(a.y)),
        gpui_kit::point(f32::from(b.x), f32::from(b.y)),
    )
}
