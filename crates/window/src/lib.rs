use app_base::prelude::{App, Plugin};
use logger::trace;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn init(&self, app: &mut App) {
        trace!("Window plugin initializing");
        app.set_run_function(Box::new(run_window));

        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("Spyder")
            .build(&event_loop)
            .unwrap();

        app.get_resource_manager_mut()
            .add(WindowResource { window, event_loop });
    }
}

struct WindowResource {
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
}

fn run_window(mut app: App) {
    let WindowResource { window, event_loop } = app
        .get_resource_manager_mut()
        .remove_unchecked::<WindowResource>();

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => {
                control_flow.set_exit();
            }
            _ => (),
        },
        winit::event::Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => (),
    });
}
