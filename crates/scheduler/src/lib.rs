use event_manager::EventManager;
use resource_manager::ResourceManager;

type System = dyn FnMut(&mut EventManager, &mut ResourceManager);

#[derive(Default)]
pub struct Scheduler {
    systems: Vec<Box<System>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system<F>(&mut self, system: F)
    where
        F: FnMut(&mut EventManager, &mut ResourceManager) + 'static,
    {
        self.systems.push(Box::new(system));
    }

    pub fn update(
        &mut self,
        event_manager: &mut EventManager,
        resource_manager: &mut ResourceManager,
    ) {
        for system in &mut self.systems {
            system(event_manager, resource_manager);
        }
    }
}
