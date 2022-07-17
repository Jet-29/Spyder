
    pub trait Layer {
        fn on_attach(&mut self) {}
        fn on_update(&mut self, _dt: f32) {}
        fn on_detach(&mut self) {}
    }

    pub struct LayerStack {
        pub layers: Vec<Box<dyn Layer>>
    }

    impl LayerStack {
        pub fn new() -> LayerStack {
            LayerStack {
                layers: Vec::new()
            }
        }

        pub fn add_layer(&mut self, mut new_layer: Box<dyn Layer>) {
            new_layer.on_attach();
            self.layers.push(new_layer);
        }
    }
