use crate::events::event::*;

struct WindowResizeEvent {
    is_handled: bool,
    width: u32,
    height: u32
}

impl WindowResizeEvent {
    fn new(width: u32, height: u32) -> WindowResizeEvent {
        WindowResizeEvent {
            is_handled: false,
            width,
            height
        }
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }
}

impl Event for WindowResizeEvent {
    fn has_been_handled(&self) -> bool {
        self.is_handled
    }
    fn set_handled(&mut self) {
        self.is_handled = true;
    }

    fn get_event_type(&self) -> EventType {
        EventType::WindowResize
    }
    fn get_name(&self) -> &str {
        "WindowResize"
    }
    fn get_category_flags(&self) -> u32 {
        EventCategory::EventCategoryApplication as u32
    }
    fn to_string(&self) -> String {
        format!("WindowResizeEvent: ({0}, {1}", self.width, self.height)
    }
}