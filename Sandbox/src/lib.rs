use spyder::core::layer_stack::*;

pub struct EditorLayer {}

impl EditorLayer {
    pub fn new() -> EditorLayer {
        EditorLayer {}
    }
}

impl Layer for EditorLayer {
    fn on_attach(&mut self) {
        println!("Heyyy im attached!");
    }
}