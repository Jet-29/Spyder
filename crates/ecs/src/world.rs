use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::entity::Entity;

#[derive(Default)]
pub struct World {
    next_uuid: u64,
    entities: Vec<Entity>,
    components: HashMap<(Entity, TypeId), Box<dyn Any>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
}
