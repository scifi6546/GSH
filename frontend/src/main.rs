#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    main();
}
mod front_end_api;
mod renderer;
mod text_render;
use front_end_api::build_vulkan_context;
use gfx_hal::{prelude::*, window};
use renderer::Renderer;
pub use renderer::Updater;

fn main() {
    build_vulkan_context();
}
