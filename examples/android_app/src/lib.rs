use corsola::{
    new_surface,
    winit::{
        event::Event,
        event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    },
    Surface,
};

#[cfg(target_os = "android")]
use corsola::winit::platform::android::activity::AndroidApp;

#[cfg(not(target_os = "android"))]
struct AndroidApp();

fn _main(event_loop: EventLoop<()>) {
    let mut surface: Option<Surface> = None;
    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::Resumed => match surface {
                None => {
                    // surface = Some({
                    //     let mut surf =
                    //         new_surface(event_loop, "Hello Android", 1920, 1080).unwrap();
                    //     surf.request_redraw();
                    //     surf
                    // });
                }
                Some(ref mut surf) => {
                    surf.request_redraw();
                }
            },
            Event::Suspended => {
                surface = None;
            }
            Event::RedrawRequested(_) => {
                if let Some(surf) = &mut surface {
                    surf.fill(corsola::tiny_skia::Color::from_rgba8(0, 0, 0, 255));
                    surf.text(
                        "Hello, Android?",
                        20.0,
                        20.0,
                        60.0,
                        corsola::glyphon::cosmic_text::Color::rgb(255, 255, 255),
                    );
                    surf.request_redraw();
                }
            }
            _ => {}
        }
    });
}

// #[cfg(target_os = "android")]
// fn init_logging() {
//     android_logger::init_once(
//         android_logger::Config::default()
//             .with_min_level(log::Level::Info)
//             .with_tag("android"),
//     );
// }

// #[cfg(not(target_os = "android"))]
// fn init_logging() {
//     // wgpu_subscriber::initialize_default_subscriber(None);
// }

#[cfg(any(target_os = "ios", target_os = "android"))]
fn stop_unwind<F: FnOnce() -> T, T>(f: F) -> T {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
            std::process::abort();
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
    let event_loop = EventLoop::new();
    // event_loop.set_control_flow(ControlFlow::Poll);
    _main(event_loop);
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android_app: AndroidApp) {
    use corsola::winit::platform::android::EventLoopBuilderExtAndroid;
    let event_loop = EventLoopBuilder::with_user_event()
        .with_android_app(android_app.clone())
        .build();
    // event_loop.set_control_flow(ControlFlow::Poll);
    stop_unwind(|| _main(event_loop))
}
