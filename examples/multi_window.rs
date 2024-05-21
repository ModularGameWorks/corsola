use corsola::{
    anyhow::Result,
    new_surface, new_surface_ex,
    tiny_skia::Pixmap,
    winit::{
        application::ApplicationHandler,
        event::{ElementState, KeyEvent, WindowEvent},
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        raw_window_handle::HasRawWindowHandle,
        window::{Window, WindowId},
    },
    Surface, TextParams,
};
use std::collections::HashMap;
use tiny_skia::PixmapPaint;

#[derive(Default)]
struct App {
    parent: Option<WindowId>,
    surfaces: HashMap<WindowId, Surface>,
    textures: HashMap<String, Pixmap>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let surface = new_surface(event_loop, "Hello, world!", 1280.0, 720.0).unwrap();
        let window_id = surface.window().id();
        self.parent = Some(window_id);
        self.surfaces.insert(window_id, surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(surf) = &mut self.surfaces.get_mut(&id) {
                    surf.background(
                        self.textures.get("background").unwrap(),
                        &PixmapPaint::default(),
                    );
                    surf.text_ex(
                        "Hello, world!",
                        20.0,
                        20.0,
                        60.0,
                        TextParams {
                            colour: glyphon::cosmic_text::Color::rgb(0, 0, 0),
                            ..Default::default()
                        },
                    )
                    .unwrap();

                    surf.update().unwrap();
                    surf.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let Some(parent_id) = &self.parent {
                    let parent = self.surfaces.get(parent_id).unwrap().window();
                    let child = spawn_child(parent, event_loop).unwrap();
                    let child_id = child.window().id();
                    self.surfaces.insert(child_id, child);
                }
            }
            _ => {}
        }
    }
}

fn spawn_child(parent: &Window, event_loop: &ActiveEventLoop) -> Result<Surface> {
    let parent = parent.raw_window_handle()?;
    new_surface_ex(event_loop, "Child Window", 100.0, 100.0, |attrs| unsafe {
        attrs.with_parent_window(Some(parent))
    })
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    app.textures.insert(
        "background".to_owned(),
        Pixmap::load_png("examples/undermine_cloth.png")?,
    );
    event_loop.run_app(&mut app)?;
    Ok(())
}
