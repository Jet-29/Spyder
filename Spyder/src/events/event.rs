// TODO: move somewhere nicer
macro_rules! BIT{
    ($a:expr) => { 1 << $a }
}

pub enum EventType {
    None = 0,
    WindowClose, WindowMinimize, WindowResize, WindowFocus, WindowLostFocus, WindowMoved, WindowTitleBarHitTest,
    AppTick, AppUpdate, AppRender,
    KeyPressed, KeyReleased, KeyTyped,
    MouseButtonPressed, MouseButtonReleased, MouseMoved, MouseScrolled,
    ScenePreStart, ScenePostStart, ScenePreStop, ScenePostStop,
    EditorExitPlayMode,
    SelectionChanged
}

pub enum EventCategory {
    None = 0,
    EventCategoryApplication = BIT!(0),
    EventCategoryInput = BIT!(1),
    EventCategoryKeyboard = BIT!(2),
    EventCategoryMouse = BIT!(3),
    EventCategoryMouseButton = BIT!(4),
    EventCategoryScene = BIT!(5),
    EventCategoryEditor = BIT!(6)
}

pub trait Event {
    fn has_been_handled(&self) -> bool;
    fn set_handled(&mut self);

    fn get_event_type(&self) -> EventType;
    fn get_name(&self) -> &str;
    fn get_category_flags(&self) -> u32;
    fn to_string(&self) -> String { String::from(self.get_name()) }

    fn is_in_category(&self, category: EventCategory) -> bool {
        (self.get_category_flags() & category as u32) == 0
    }
}

struct EventDispatcher {
    event: Box<dyn Event>
}

impl EventDispatcher {
    pub fn new(event: Box<dyn Event>) -> EventDispatcher {
        EventDispatcher{event}
    }

    pub fn dispatch(&mut self, event_func: fn(&Box<dyn Event>) -> bool) {
        if !self.event.has_been_handled() {
            if event_func(&self.event) {
                self.event.set_handled();
            }
        }
    }
}