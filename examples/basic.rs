use spyder::{App, RasterizationRendererPlugin, WindowPlugin};

fn main() {
    App::new()
        .add_plugin(WindowPlugin)
        .add_plugin(RasterizationRendererPlugin)
        .run();
}
