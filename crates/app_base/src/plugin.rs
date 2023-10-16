use crate::app::App;

pub trait Plugin {
    fn init(&self, app: &mut App);
}
