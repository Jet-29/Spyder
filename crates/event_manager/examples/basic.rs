use event_manager::EventManager;

#[derive(Debug)]
struct KeyboardEvent {
    _pressed: bool,
}

#[derive(Debug)]
struct MouseMovedEvent;

fn main() {
    let mut ev = EventManager::new();

    ev.add(KeyboardEvent { _pressed: true });
    ev.add(KeyboardEvent { _pressed: false });

    println!(
        "KeyboardEvent count: {}",
        ev.get_event_count::<KeyboardEvent>()
    );
    println!(
        "MouseMovedEvent count: {}",
        ev.get_event_count::<MouseMovedEvent>()
    );

    println!("First KeyboardEvent: {:?}", ev.get_event::<KeyboardEvent>());
    println!(
        "First MouseMovedEvent: {:?}",
        ev.get_event::<MouseMovedEvent>()
    );

    println!(
        "All KeyboardEvents: {:?}",
        ev.take_all_of_type::<KeyboardEvent>()
    );
    println!(
        "All MouseMovedEvents: {:?}",
        ev.take_all_of_type::<MouseMovedEvent>()
    );
}
