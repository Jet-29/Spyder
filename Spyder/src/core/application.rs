use crate::core::layer_stack::*;

pub struct Application {
    layer_stack: LayerStack
}

impl Application {
    pub fn new() -> Application {
        Application {
            layer_stack: LayerStack::new()
        }
    }

    pub fn add_layer(&mut self, layer: Box<dyn Layer>) {
        self.layer_stack.add_layer(layer);
    }
}