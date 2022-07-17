use sandbox::EditorLayer;
use spyder::core::application::*;

fn main() {
    let mut app = Application::new();

    app.add_layer(Box::new(EditorLayer::new()));
}
