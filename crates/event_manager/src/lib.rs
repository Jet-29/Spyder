use std::any::Any;
use std::collections::{HashMap, VecDeque};

#[derive(Eq, PartialEq, Hash)]
struct EventID(std::any::TypeId);

#[derive(Default)]
pub struct EventManager {
    events: HashMap<EventID, VecDeque<Box<dyn Any>>>,
}

impl EventManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<T: Any>(&mut self, event: T) {
        let id = self.get_event_id::<T>();

        self.events
            .entry(id)
            .or_default()
            .push_back(Box::new(event));
    }

    pub fn return_event<T: Any>(&mut self, event: T) {
        let id = self.get_event_id::<T>();

        self.events
            .entry(id)
            .or_default()
            .push_front(Box::new(event));
    }

    pub fn get_event_count<T: Any>(&self) -> usize {
        let id = self.get_event_id::<T>();
        self.events
            .get(&id)
            .map(|events| events.len())
            .unwrap_or_default()
    }

    pub fn get_event<T: Any>(&mut self) -> Option<T> {
        let id = self.get_event_id::<T>();
        self.events
            .get_mut(&id)
            .and_then(|events| events.pop_front())
            .map(|event| *event.downcast().unwrap())
    }

    pub fn take_all_of_type<T: Any>(&self) -> Vec<&T> {
        let id = self.get_event_id::<T>();
        self.events
            .get(&id)
            .map(|events| {
                events
                    .iter()
                    .map(|event| event.downcast_ref().unwrap())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_event_id<T: Any>(&self) -> EventID {
        EventID(std::any::TypeId::of::<T>())
    }
}
