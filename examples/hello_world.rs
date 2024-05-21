use corsola::{
    anyhow::Result,
    glyphon::TextBounds,
    new_surface,
    tiny_skia::Pixmap,
    // new_window,
    winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::WindowId,
    },
    // Renderer,
    Surface,
    TextParams,
};
use std::collections::HashMap;
use tiny_skia::PixmapPaint;

#[derive(Default)]
struct App {
    // window: Option<Window>,
    // renderer: Option<Renderer<'a>>,
    surface: Option<Surface>,
    textures: HashMap<String, Pixmap>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.surface = Some(new_surface(event_loop, "Hello, world!", 1280.0, 720.0).unwrap());
        // self.surface = Some(Surface::new(self.window.as_ref().unwrap()).unwrap());
        // self.window = Some(new_window(event_loop, "Hello, world!", 1280.0, 720.0).unwrap());
        // self.renderer = Some(Renderer::new(self.window.as_ref().unwrap()).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(surf) = &mut self.surface {
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

                    surf.text_ex(
                        "Hello, world!",
                        20.0,
                        120.0,
                        60.0,
                        TextParams {
                            colour: glyphon::cosmic_text::Color::rgb(255, 255, 255),
                            ..Default::default()
                        },
                    )
                    .unwrap();

                    surf.update().unwrap();
                    surf.request_redraw();
                }
                // if let Some(win) = &mut self.window {
                // win.request_redraw();
                // }
            }
            _ => {}
        }
    }
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
