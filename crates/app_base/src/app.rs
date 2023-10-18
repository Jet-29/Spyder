use crate::plugin::Plugin;
use logger::{debug, trace};
use resource_manager::ResourceManager;

pub struct App {
    resources: ResourceManager,
    run_function: Box<dyn FnOnce(Self)>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            resources: ResourceManager::default(),
            run_function: Box::new(run_once),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes by reference as it allows method chaining without taking ownership of a local variable.
    pub fn run(&mut self) {
        let mut app = std::mem::take(self);
        let run_function = std::mem::replace(&mut app.run_function, Box::new(run_once));
        run_function(app);
    }

    pub fn set_run_function(&mut self, run_function: Box<dyn FnOnce(Self)>) -> &mut Self {
        debug!("Replaced the run function.");
        self.run_function = run_function;
        self
    }

    pub fn add_plugin<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        plugin.init(self);
        self
    }

    pub fn get_resource_manager(&self) -> &ResourceManager {
        &self.resources
    }

    pub fn get_resource_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resources
    }
}

fn run_once(_app: App) {
    trace!("Default run function.")
}
