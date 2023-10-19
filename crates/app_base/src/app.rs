use crate::plugin::Plugin;
use event_manager::EventManager;
use logger::{debug, trace};
use resource_manager::ResourceManager;
use scheduler::Scheduler;

pub struct App {
    resources: ResourceManager,
    events: EventManager,
    scheduler: Scheduler,
    run_function: Box<dyn FnOnce(Self)>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            resources: ResourceManager::default(),
            events: EventManager::default(),
            scheduler: Scheduler::default(),
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

    pub fn update(&mut self) {
        self.scheduler.update(&mut self.events, &mut self.resources);
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

    pub fn get_event_manager(&self) -> &EventManager {
        &self.events
    }

    pub fn get_event_manager_mut(&mut self) -> &mut EventManager {
        &mut self.events
    }

    pub fn get_scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn get_scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }
}

fn run_once(_app: App) {
    trace!("Default run function.")
}
