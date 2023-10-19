use spyder::prelude::*;

fn main() {
    App::new()
        .add_plugin(WindowPlugin)
        .add_plugin(RasterizationRendererPlugin)
        .run();
}
