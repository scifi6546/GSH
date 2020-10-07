#[cfg(feature = "dx11")]
extern crate gfx_backend_dx11 as back;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(not(any(
    feature = "vulkan",
    feature = "dx11",
    feature = "dx12",
    feature = "metal",
    feature = "gl",
)))]
extern crate gfx_backend_empty as back;
#[cfg(all(unix, feature = "gl"))]
extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use nalgebra::{Vector2, Vector3};
use winit::event::KeyboardInput;
mod front_end;
use front_end::{DrawCall, Engine, Event};
pub use front_end::{Model, Texture};
mod gpu;
use gfx_hal::{
    buffer, command, format as f,
    format::ChannelType,
    image as i, memory as m, pass,
    pass::Subpass,
    pool,
    prelude::*,
    pso,
    pso::{ShaderStageFlags, VertexInputRate},
    queue::{QueueGroup, Submission},
    window,
};
use gpu::{ModelAllocation, TextureAllocation, DEFAULT_SIZE, GPU};
use std::{
    borrow::Borrow,
    iter,
    mem::{self, ManuallyDrop},
    ptr,
};
#[derive(Clone,Debug)]
pub struct ModelId {
    id: usize,
}
#[derive(Clone,Debug)]
pub struct TextureId {
    id: usize,
}
struct Context<B: gfx_hal::Backend> {
    front_end: Engine,
    mesh_allocation: Vec<ModelAllocation<B>>,
    texture_allocation: Vec<TextureAllocation<B>>,
    gpu: GPU<B>,
}
impl<B: gfx_hal::Backend> Context<B> {
    fn new(
        instance: Option<B::Instance>,
        mut surface: B::Surface,
        adapter: gfx_hal::adapter::Adapter<B>,
    ) -> Self {
        let mut gpu = GPU::new(instance, surface, adapter);
        let (mut models, mut textures, engine_ctor) = Engine::new();
        let mesh_allocation: Vec<ModelAllocation<B>> = models
            .iter_mut()
            .map(|model| gpu.load_verticies(&mut model.mesh))
            .collect();
        let texture_allocation: Vec<TextureAllocation<B>> = textures
            .iter_mut()
            .map(|texture| gpu.load_textures(&mut texture.image))
            .collect();
        //TODO get model id
        let model_ids = (0..mesh_allocation.len())
            .map(|x| ModelId { id: x })
            .collect();
        let texture_ids = (0..texture_allocation.len())
            .map(|x| TextureId { id: x })
            .collect();
        Context {
            front_end: engine_ctor(model_ids, texture_ids),
            mesh_allocation,
            texture_allocation,
            gpu,
        }
    }
    fn process_event(&mut self, event: Event) {
        unimplemented!()
    }
    fn event_loop(&mut self, event: Vec<Event>) {}
    fn get_events(&self) -> Vec<Event> {
        unimplemented!()
    }
    fn resize(&mut self, new_dimensions: window::Extent2D) {
        unimplemented!()
    }
    fn draw(&mut self) {
        let draw_calls = self.front_end.get_draw_calls();
        let models = draw_calls.iter().map(|draw|
            match draw{
                DrawCall::DrawModel{
                    model,
                    texture,
                    ..
                }=>(& self.mesh_allocation[model.id] as *const ModelAllocation<B>,&self.texture_allocation[texture.id] as *const TextureAllocation<B>)
            }
        ).collect();

        
        unsafe{
            self.gpu.draw_models(models);
        }

    }
}
fn to_event(event: KeyboardInput) -> Event {
    unimplemented!()
}
pub fn build_vulkan_context() {
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Debug).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    #[cfg(not(any(
        feature = "vulkan",
        feature = "dx11",
        feature = "dx12",
        feature = "metal",
        feature = "gl",
    )))]
    eprintln!(
        "You are running the example with the empty backend, no graphical output is to be expected"
    );

    let event_loop = winit::event_loop::EventLoop::new();

    let wb = winit::window::WindowBuilder::new()
        .with_min_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            64.0, 64.0,
        )))
        .with_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(
            DEFAULT_SIZE.width,
            DEFAULT_SIZE.height,
        )))
        .with_title("quad".to_string());

    // instantiate backend
    #[cfg(not(target_arch = "wasm32"))]
    let (_window, instance, mut adapters, surface) = {
        let window = wb.build(&event_loop).unwrap();
        let instance =
            back::Instance::create("gfx-rs quad", 1).expect("Failed to create an instance!");
        let adapters = instance.enumerate_adapters();
        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("Failed to create a surface!")
        };
        // Return `window` so it is not dropped: dropping it invalidates `surface`.
        (window, Some(instance), adapters, surface)
    };

    #[cfg(target_arch = "wasm32")]
    let (_window, instance, mut adapters, surface) = {
        let (window, surface) = {
            let window = wb.build(&event_loop).unwrap();
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .body()
                .unwrap()
                .append_child(&winit::platform::web::WindowExtWebSys::canvas(&window))
                .unwrap();
            let surface = back::Surface::from_raw_handle(&window);
            (window, surface)
        };

        let adapters = surface.enumerate_adapters();
        (window, None, adapters, surface)
    };

    for adapter in &adapters {
        println!("{:?}", adapter.info);
    }

    let adapter = adapters.remove(0);

    let mut context = Context::new(instance, surface, adapter);

    //renderer.event_loop();

    // It is important that the closure move captures the Renderer,
    // otherwise it will not be dropped when the event loop exits.
    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    context.process_event(to_event(input));
                }

                winit::event::WindowEvent::Resized(dims) => {
                    println!("resized to {:?}", dims);
                    context.resize(window::Extent2D {
                        width: dims.width,
                        height: dims.height,
                    });
                }
                _ => {}
            },
            winit::event::Event::RedrawEventsCleared => {
                context.draw();
            }
            _ => {}
        }
    });
}
