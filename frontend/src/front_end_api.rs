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
const SHADER_ENTRY_NAME: &str = "main";

use nalgebra::{Vector2, Vector3};
use winit::event::KeyboardInput;
mod front_end;
use front_end::{DrawCall, Engine, Event, Model, Texture};
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
use std::{
    borrow::Borrow,
    iter,
    mem::{self, ManuallyDrop},
    ptr,
};
struct GPU<B: gfx_hal::Backend> {
    instance: Option<B::Instance>,
    device: B::Device,
    queue_group: QueueGroup<B>,
    desc_pool: ManuallyDrop<B::DescriptorPool>,
    surface: ManuallyDrop<B::Surface>,
    adapter: gfx_hal::adapter::Adapter<B>,
    format: gfx_hal::format::Format,
    pub dimensions: window::Extent2D,
    viewport: pso::Viewport,
    render_pass: ManuallyDrop<B::RenderPass>,
    pipeline: ManuallyDrop<B::GraphicsPipeline>,
    pipeline_layout: ManuallyDrop<B::PipelineLayout>,
    desc_set: B::DescriptorSet,
    set_layout: ManuallyDrop<B::DescriptorSetLayout>,
    submission_complete_semaphores: Vec<B::Semaphore>,
    submission_complete_fences: Vec<B::Fence>,
    cmd_pools: Vec<B::CommandPool>,
    cmd_buffers: Vec<B::CommandBuffer>,
    //image_upload_buffer: ManuallyDrop<B::Buffer>,
    //image_logo: ManuallyDrop<B::Image>,
    //image_srv: ManuallyDrop<B::ImageView>,
    //image_memory: ManuallyDrop<B::Memory>,
    //image_upload_memory: ManuallyDrop<B::Memory>,
    //sampler: ManuallyDrop<B::Sampler>,
    frames_in_flight: usize,
    frame: u64,
}
const DEFAULT_SIZE: window::Extent2D = window::Extent2D {
    width: 1024,
    height: 700,
};
struct ModelAllocation {}
struct TextureAllocation {}
struct Context<B: gfx_hal::Backend> {
    front_end: Engine,
    gpu: GPU<B>,
}

impl<B: gfx_hal::Backend> GPU<B> {
    fn new(
        instance: Option<B::Instance>,
        mut surface: B::Surface,
        adapter: gfx_hal::adapter::Adapter<B>,
    ) -> Self {
        let memory_types = adapter.physical_device.memory_properties().memory_types;
        let limits = adapter.physical_device.limits();

        // Build a new device and associated command queues
        let family = adapter
            .queue_families
            .iter()
            .find(|family| {
                surface.supports_queue_family(family) && family.queue_type().supports_graphics()
            })
            .unwrap();
        let mut gpu = unsafe {
            adapter
                .physical_device
                .open(&[(family, &[1.0])], gfx_hal::Features::empty())
                .unwrap()
        };
        let mut queue_group = gpu.queue_groups.pop().unwrap();
        let device = gpu.device;

        let command_pool = unsafe {
            device.create_command_pool(queue_group.family, pool::CommandPoolCreateFlags::empty())
        }
        .expect("Can't create command pool");

        // Setup renderpass and pipeline
        let set_layout = ManuallyDrop::new(
            unsafe {
                device.create_descriptor_set_layout(
                    &[
                        pso::DescriptorSetLayoutBinding {
                            binding: 0,
                            ty: pso::DescriptorType::Image {
                                ty: pso::ImageDescriptorType::Sampled {
                                    with_sampler: false,
                                },
                            },
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                        pso::DescriptorSetLayoutBinding {
                            binding: 1,
                            ty: pso::DescriptorType::Sampler,
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                    ],
                    &[],
                )
            }
            .expect("Can't create descriptor set layout"),
        );

        // Descriptors
        let mut desc_pool = ManuallyDrop::new(
            unsafe {
                device.create_descriptor_pool(
                    1, // sets
                    &[
                        pso::DescriptorRangeDesc {
                            ty: pso::DescriptorType::Image {
                                ty: pso::ImageDescriptorType::Sampled {
                                    with_sampler: false,
                                },
                            },
                            count: 1,
                        },
                        pso::DescriptorRangeDesc {
                            ty: pso::DescriptorType::Sampler,
                            count: 1,
                        },
                    ],
                    pso::DescriptorPoolCreateFlags::empty(),
                )
            }
            .expect("Can't create descriptor pool"),
        );
        let desc_set = unsafe { desc_pool.allocate_set(&set_layout) }.unwrap();

        let caps = surface.capabilities(&adapter.physical_device);
        let formats = surface.supported_formats(&adapter.physical_device);
        println!("formats: {:?}", formats);
        let format = formats.map_or(f::Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        });

        let swap_config = window::SwapchainConfig::from_caps(&caps, format, DEFAULT_SIZE);
        println!("{:?}", swap_config);
        let extent = swap_config.extent;
        unsafe {
            surface
                .configure_swapchain(&device, swap_config)
                .expect("Can't configure swapchain");
        };

        let render_pass = {
            let attachment = pass::Attachment {
                format: Some(format),
                samples: 1,
                ops: pass::AttachmentOps::new(
                    pass::AttachmentLoadOp::Clear,
                    pass::AttachmentStoreOp::Store,
                ),
                stencil_ops: pass::AttachmentOps::DONT_CARE,
                layouts: i::Layout::Undefined..i::Layout::Present,
            };

            let subpass = pass::SubpassDesc {
                colors: &[(0, i::Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            ManuallyDrop::new(
                unsafe { device.create_render_pass(&[attachment], &[subpass], &[]) }
                    .expect("Can't create render pass"),
            )
        };

        // Define maximum number of frames we want to be able to be "in flight" (being computed
        // simultaneously) at once
        let frames_in_flight = 3;

        // The number of the rest of the resources is based on the frames in flight.
        let mut submission_complete_semaphores = Vec::with_capacity(frames_in_flight);
        let mut submission_complete_fences = Vec::with_capacity(frames_in_flight);
        // Note: We don't really need a different command pool per frame in such a simple demo like this,
        // but in a more 'real' application, it's generally seen as optimal to have one command pool per
        // thread per frame. There is a flag that lets a command pool reset individual command buffers
        // which are created from it, but by default the whole pool (and therefore all buffers in it)
        // must be reset at once. Furthermore, it is often the case that resetting a whole pool is actually
        // faster and more efficient for the hardware than resetting individual command buffers, so it's
        // usually best to just make a command pool for each set of buffers which need to be reset at the
        // same time (each frame). In our case, each pool will only have one command buffer created from it,
        // though.
        let mut cmd_pools = Vec::with_capacity(frames_in_flight);
        let mut cmd_buffers = Vec::with_capacity(frames_in_flight);

        cmd_pools.push(command_pool);
        for _ in 1..frames_in_flight {
            unsafe {
                cmd_pools.push(
                    device
                        .create_command_pool(
                            queue_group.family,
                            pool::CommandPoolCreateFlags::empty(),
                        )
                        .expect("Can't create command pool"),
                );
            }
        }

        for i in 0..frames_in_flight {
            submission_complete_semaphores.push(
                device
                    .create_semaphore()
                    .expect("Could not create semaphore"),
            );
            submission_complete_fences
                .push(device.create_fence(true).expect("Could not create fence"));
            cmd_buffers.push(unsafe { cmd_pools[i].allocate_one(command::Level::Primary) });
        }

        let pipeline_layout = ManuallyDrop::new(
            unsafe {
                device.create_pipeline_layout(
                    iter::once(&*set_layout),
                    &[(pso::ShaderStageFlags::VERTEX, 0..8)],
                )
            }
            .expect("Can't create pipeline layout"),
        );
        let pipeline = {
            let vs_module = {
                let glsl = include_str!("data/quad.vert");
                let file =
                    glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Vertex).unwrap();
                let spirv = gfx_auxil::read_spirv(file).unwrap();
                unsafe { device.create_shader_module(&spirv) }.unwrap()
            };
            let fs_module = {
                let glsl = include_str!("data/quad.frag");
                let file =
                    glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Fragment).unwrap();
                let spirv = gfx_auxil::read_spirv(file).unwrap();
                unsafe { device.create_shader_module(&spirv) }.unwrap()
            };

            let pipeline = {
                let (vs_entry, fs_entry) = (
                    pso::EntryPoint {
                        entry: SHADER_ENTRY_NAME,
                        module: &vs_module,
                        specialization: gfx_hal::spec_const_list![0.8f32],
                    },
                    pso::EntryPoint {
                        entry: SHADER_ENTRY_NAME,
                        module: &fs_module,
                        specialization: pso::Specialization::default(),
                    },
                );

                let subpass = Subpass {
                    index: 0,
                    main_pass: &*render_pass,
                };

                let vertex_buffers = vec![pso::VertexBufferDesc {
                    binding: 0,
                    stride: mem::size_of::<f32>() as u32 * 3,
                    rate: VertexInputRate::Vertex,
                }];

                let attributes = vec![
                    pso::AttributeDesc {
                        location: 0,
                        binding: 0,
                        element: pso::Element {
                            format: f::Format::Rg32Sfloat,
                            offset: 0,
                        },
                    },
                    pso::AttributeDesc {
                        location: 1,
                        binding: 0,
                        element: pso::Element {
                            format: f::Format::Rg32Sfloat,
                            offset: 8,
                        },
                    },
                ];

                let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
                    pso::PrimitiveAssemblerDesc::Vertex {
                        buffers: &vertex_buffers,
                        attributes: &attributes,
                        input_assembler: pso::InputAssemblerDesc {
                            primitive: pso::Primitive::TriangleList,
                            with_adjacency: false,
                            restart_index: None,
                        },
                        vertex: vs_entry,
                        geometry: None,
                        tessellation: None,
                    },
                    pso::Rasterizer::FILL,
                    Some(fs_entry),
                    &*pipeline_layout,
                    subpass,
                );

                pipeline_desc.blender.targets.push(pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: Some(pso::BlendState::ALPHA),
                });

                unsafe { device.create_graphics_pipeline(&pipeline_desc, None) }
            };

            unsafe {
                device.destroy_shader_module(vs_module);
            }
            unsafe {
                device.destroy_shader_module(fs_module);
            }

            ManuallyDrop::new(pipeline.unwrap())
        };

        // Rendering setup
        let viewport = pso::Viewport {
            rect: pso::Rect {
                x: 0,
                y: 0,
                w: extent.width as _,
                h: extent.height as _,
            },
            depth: 0.0..1.0,
        };

        GPU {
            instance,
            device,
            queue_group,
            desc_pool,
            surface: ManuallyDrop::new(surface),
            adapter,
            format,
            dimensions: DEFAULT_SIZE,
            viewport,
            render_pass,
            pipeline,
            pipeline_layout,
            desc_set,
            set_layout,
            submission_complete_semaphores,
            submission_complete_fences,
            cmd_pools,
            cmd_buffers,

            //image_upload_buffer,
            //image_logo,
            //image_srv,

            //image_memory,
            //image_upload_memory,
            //sampler,
            frames_in_flight,
            frame: 0,
        }
    }
    fn load_models(&mut self, models: Vec<Model>) -> ModelAllocation {
        unimplemented!()
    }
    fn load_textures(&mut self, textures: Vec<Texture>) -> TextureAllocation {
        unimplemented!()
    }
}
impl<B: gfx_hal::Backend> Context<B> {
    fn new(
        instance: Option<B::Instance>,
        mut surface: B::Surface,
        adapter: gfx_hal::adapter::Adapter<B>,
    ) -> Self {
        let mut gpu = GPU::new(instance, surface, adapter);
        let (models, textures, engine_ctor) = Engine::new();
        let model_allocation = gpu.load_models(models);
        let texture_allocation = gpu.load_textures(textures);
        //TODO get model id

        Context {
            front_end: engine_ctor(vec![], vec![]),
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
        unimplemented!()
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
