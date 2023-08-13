#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;
use winit::event::{Event, WindowEvent};
#[cfg(target_family = "wasm")]
use winit::platform::web::WindowExtWebSys;

const NUM_RINGS: usize = 15;

mod app;
use app::AppState;
mod staged_buffer;
mod util;
pub(crate) mod platform;

mod square;

use crate::app::CreatedWindow;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    #[cfg(target_family = "wasm")]
    {
        console_error_panic_hook::set_once();
    }
    let CreatedWindow { window, event_loop } =
        app::create_window().expect("Could not create window");
    let elproxy = event_loop.create_proxy();
    #[cfg(target_family = "wasm")]
    {
        let browser_window = web_sys::window().expect("No browser window!");
        let canvas = window.canvas();
        let doc = browser_window.document().expect("No document!");
        let bod = doc.body().expect("No body!");
        bod.append_child(&canvas)
            .expect("Could not add canvas to document");
    }
    let primary_id = window.id();
    let mut app = AppState::setup(window, elproxy)
        .await
        .expect("Could not set up app");
    app.window.request_redraw();
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } => {
            if window_id == primary_id {
                match event {
                    WindowEvent::CloseRequested => {
                        control_flow.set_exit_with_code(0);
                    }
                    WindowEvent::Resized(new_size) => {
                        app.window.request_redraw();
                        app.resize(new_size);
                    }
                    _ => (),
                }
            }
        }
        Event::RedrawRequested(window_id) => {
            if window_id == primary_id {
                if let Err(error) = app.render() {
                    eprintln!("{error:?}");
                }
            }
        }
        _ => (),
    });
}
