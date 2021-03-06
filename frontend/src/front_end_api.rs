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

use winit::event::KeyboardInput;
mod front_end;
use front_end::{DrawCall, Event, Scene, SceneCtor};
pub use front_end::{Model, Terminal, Texture};
mod gpu;
use gfx_hal::{prelude::*, window};
use gpu::{ModelAllocation, TextureAllocation, DEFAULT_SIZE, GPU};
use handy::{Handle, HandleMap};
#[derive(Clone, Debug)]
pub struct ModelId {
    id: Handle,
}
#[derive(Clone, Debug)]
pub struct TextureId {
    id: Handle,
}
struct Context<B: gfx_hal::Backend, E: Scene> {
    front_end: E,
    mesh_allocation: HandleMap<ModelAllocation<B>>,
    texture_allocation: HandleMap<TextureAllocation<B>>,
    gpu: GPU<B>,
}
impl<B: gfx_hal::Backend, E: Scene> Context<B, E> {
    fn new(
        instance: Option<B::Instance>,
        surface: B::Surface,
        adapter: gfx_hal::adapter::Adapter<B>,
        scene_ctor: Box<dyn Fn() -> SceneCtor<E>>,
    ) -> Self {
        let mut gpu = GPU::new(instance, surface, adapter);
        let (mut models, mut textures, engine_ctor) = scene_ctor();
        let mut mesh_allocation = HandleMap::new();
        let model_ids = models
            .iter_mut()
            .map(|model| ModelId {
                id: mesh_allocation.insert(gpu.load_verticies(&mut model.mesh)),
            })
            .collect();
        let mut texture_allocation = HandleMap::new();
        let texture_ids = textures
            .iter_mut()
            .map(|texture| TextureId {
                id: texture_allocation.insert(gpu.load_textures(&mut texture.image)),
            })
            .collect();
        Context {
            front_end: engine_ctor(model_ids, texture_ids),
            mesh_allocation,
            texture_allocation,
            gpu,
        }
    }
    fn process_event(&mut self, event: Event) {
        self.front_end.process_event(event);
    }
    fn resize(&mut self, new_size: window::Extent2D) {
        self.gpu.change_resolution(new_size);
    }
    fn draw(&mut self) {
        let draw_calls = self.front_end.get_draw_calls();
        let mut models = vec![];
        let mut texture_modifications = vec![];
        for draw in draw_calls.iter() {
            match draw {
                DrawCall::DrawModel { model, texture, .. } => models.push((
                    &self.mesh_allocation[model.id] as *const ModelAllocation<B>,
                    &self.texture_allocation[texture.id] as *const TextureAllocation<B>,
                )),
                DrawCall::UpdateTexture {
                    texture,
                    new_texture,
                } => texture_modifications.push((
                    &mut self.texture_allocation[texture.id] as *mut TextureAllocation<B>,
                    new_texture,
                )),
                DrawCall::NewModel { .. } => todo!("figure this crap out"),
            }
        }

        #[allow(unused_mut)]
        for (mut texture_alloc, mut new_texture) in texture_modifications.iter_mut() {
            unsafe {
                self.gpu.destroy_texture(&mut *texture_alloc);
                *texture_alloc = self.gpu.load_textures(new_texture);

                //texture_deref = self.gpu.load_textures(new_texture);
            }
        }
        self.gpu.draw_models(models);
    }
}
fn to_event(keyboard: KeyboardInput) -> Event {
    if let Some(event) = keyboard.virtual_keycode {
        use winit::event::VirtualKeyCode;
        match event {
            VirtualKeyCode::Key1 => Event::RegularKey('1'),
            VirtualKeyCode::Key2 => Event::RegularKey('2'),
            VirtualKeyCode::Key3 => Event::RegularKey('3'),
            VirtualKeyCode::Key4 => Event::RegularKey('4'),
            VirtualKeyCode::Key5 => Event::RegularKey('5'),
            VirtualKeyCode::Key6 => Event::RegularKey('6'),
            VirtualKeyCode::Key7 => Event::RegularKey('7'),
            VirtualKeyCode::Key8 => Event::RegularKey('8'),
            VirtualKeyCode::Key9 => Event::RegularKey('9'),
            VirtualKeyCode::Key0 => Event::RegularKey('0'),
            VirtualKeyCode::A => Event::RegularKey('a'),
            VirtualKeyCode::B => Event::RegularKey('b'),
            VirtualKeyCode::C => Event::RegularKey('c'),
            VirtualKeyCode::D => Event::RegularKey('d'),
            VirtualKeyCode::E => Event::RegularKey('e'),
            VirtualKeyCode::F => Event::RegularKey('f'),
            VirtualKeyCode::G => Event::RegularKey('g'),
            VirtualKeyCode::H => Event::RegularKey('h'),
            VirtualKeyCode::I => Event::RegularKey('i'),
            VirtualKeyCode::J => Event::RegularKey('j'),
            VirtualKeyCode::K => Event::RegularKey('k'),
            VirtualKeyCode::L => Event::RegularKey('l'),
            VirtualKeyCode::M => Event::RegularKey('m'),
            VirtualKeyCode::N => Event::RegularKey('n'),
            VirtualKeyCode::O => Event::RegularKey('o'),
            VirtualKeyCode::P => Event::RegularKey('p'),
            VirtualKeyCode::Q => Event::RegularKey('q'),
            VirtualKeyCode::R => Event::RegularKey('r'),
            VirtualKeyCode::S => Event::RegularKey('s'),
            VirtualKeyCode::T => Event::RegularKey('t'),
            VirtualKeyCode::U => Event::RegularKey('u'),
            VirtualKeyCode::V => Event::RegularKey('v'),
            VirtualKeyCode::W => Event::RegularKey('w'),
            VirtualKeyCode::X => Event::RegularKey('x'),
            VirtualKeyCode::Y => Event::RegularKey('y'),
            VirtualKeyCode::Z => Event::RegularKey('z'),
            _ => Event::Unknown,
        }
    } else {
        Event::Unknown
    }
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
        println!("num adaptors: {}", adapters.len());
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

    let mut context = Context::new(instance, surface, adapter, Box::new(Terminal::new));

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
