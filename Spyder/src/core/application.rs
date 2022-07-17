use crate::core::layer_stack::*;
use crate::core::window::*;

enum ApplicationStatus {
    None,
    Initialized,
    Running,
    HotReleoding,
    Closed,
}

pub struct Application {
    application_state: ApplicationStatus,
    window: Window,
    layer_stack: LayerStack,
}

impl Application {
    pub fn new() -> Application {
        Application {
            application_state: ApplicationStatus::None,
            window: Window::new(),
            layer_stack: LayerStack::new(),
        }
    }

    pub fn init(&mut self) {
        self.application_state = ApplicationStatus::Initialized;
    }

    pub fn run(&mut self) {
        self.application_state = ApplicationStatus::Running;
        while matches!(self.application_state, ApplicationStatus::Running) {
            self.layer_stack.update_layers(0.5);
            std::thread::sleep(std::time::Duration::from_secs(1));
            self.window.process_events();
            if self.window.should_close() {
                self.shutdown();
            }
        }
    }

    pub fn hot_reload(&mut self) {
        self.application_state = ApplicationStatus::HotReleoding;
    }

    pub fn shutdown(&mut self) {
        self.application_state = ApplicationStatus::Closed;
    }

    pub fn add_layer(&mut self, layer: Box<dyn Layer>) {
        self.layer_stack.add_layer(layer);
    }
}
