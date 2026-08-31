mod app;
mod app_physics;
mod demos;
mod render;
mod render_analysis;
mod render_left;
mod render_right;
mod theme;

use app::DemoApp;
use gpui::{AppContext, Application, WindowOptions};

fn main() {
    let platform = gpui_platform::current_platform(false);
    Application::with_platform(platform).run(|cx: &mut gpui::App| {
        gpui_component::init(cx);
        cx.open_window(
            WindowOptions {
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Graphene-RS Interactive Visualizer".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(gpui::point(gpui::px(8.0), gpui::px(8.0))),
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| DemoApp::new(window, cx));
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            },
        )
        .expect("Failed to start application");
    });
}
