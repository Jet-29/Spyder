use std::any::Any;
use std::collections::HashMap;

pub mod prelude {
    pub use super::ResourceManager;
}

#[derive(Eq, PartialEq, Hash)]
struct ResourceID(std::any::TypeId);

#[derive(Default)]
pub struct ResourceManager {
    resources: HashMap<ResourceID, Box<dyn Any>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<T: Any>(&mut self, resource: T) {
        let id = self.get_resource_id::<T>();

        self.resources.insert(id, Box::new(resource));
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        let id = self.get_resource_id::<T>();
        self.resources
            .get(&id)
            .map(|resource| resource.as_ref().downcast_ref().unwrap())
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let id = self.get_resource_id::<T>();
        self.resources
            .get_mut(&id)
            .map(|resource| resource.as_mut().downcast_mut().unwrap())
    }

    pub fn get_unchecked<T: Any>(&self) -> &T {
        let id = self.get_resource_id::<T>();
        self.resources.get(&id).unwrap().downcast_ref().unwrap()
    }

    pub fn get_mut_unchecked<T: Any>(&mut self) -> &mut T {
        let id = self.get_resource_id::<T>();
        self.resources.get_mut(&id).unwrap().downcast_mut().unwrap()
    }

    pub fn remove<T: Any>(&mut self) -> Option<T> {
        let id = self.get_resource_id::<T>();
        self.resources
            .remove(&id)
            .map(|resource| *resource.downcast().unwrap())
    }

    pub fn remove_unchecked<T: Any>(&mut self) -> Box<T> {
        let id = self.get_resource_id::<T>();
        self.resources.remove(&id).unwrap().downcast::<T>().unwrap()
    }

    fn get_resource_id<T: Any>(&self) -> ResourceID {
        ResourceID(std::any::TypeId::of::<T>())
    }
}
