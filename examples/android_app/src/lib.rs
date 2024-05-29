#[cfg(target_os = "android")]
use corsola::winit::platform::android::activity::AndroidApp;

use corsola::{
    anyhow::Result,
    glyphon::TextBounds,
    new_surface_ex,
    tiny_skia::{Pixmap, PixmapPaint},
    winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::WindowId,
    },
    Surface, TextParams,
};

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

#[derive(Default)]
struct App {
    surface: Option<Surface>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.surface = Some(
            new_surface_ex(event_loop, "Hello Android", 1280.0, 720.0, |attrs| {
                attrs.with_resizable(true)
            })
            .unwrap(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(surf) = &mut self.surface {
                    surf.fill(corsola::tiny_skia::Color::from_rgba8(0, 0, 0, 255));
                    surf.text_ex(
                        "Hello, Android?",
                        20.0,
                        20.0,
                        60.0,
                        TextParams {
                            colour: corsola::glyphon::cosmic_text::Color::rgb(255, 255, 255),
                            ..Default::default()
                        },
                    );
                    surf.update().unwrap();
                    surf.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn _main(event_loop: EventLoop<()>) {
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn stop_unwind<F: FnOnce() -> T, T>(f: F) -> T {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
            std::process::abort()
        }
    }
}

#[cfg(target_os = "ios")]
fn _start_app() {
    stop_unwind(|| main());
}

#[no_mangle]
#[inline(never)]
#[cfg(target_os = "ios")]
pub extern "C" fn start_app() {
    _start_app();
}

#[cfg(not(target_os = "android"))]
pub fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .parse_default_env()
        .init();

    let event_loop = EventLoop::with_user_event().build().unwrap();
    _main(event_loop);
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Warn),
    );

    let event_loop = EventLoop::with_user_event()
        .with_android_app(app)
        .build()
        .unwrap();
    stop_unwind(|| _main(event_loop));
}
