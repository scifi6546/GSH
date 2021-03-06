const SHADER_ENTRY_NAME: &str = "main";
use gfx_hal::{
    buffer, command, format as f,
    format::{AsFormat, ChannelType, Rgba8Srgb as ColorFormat, Swizzle},
    image as i, memory as m, pass,
    pass::Subpass,
    pool,
    prelude::*,
    pso,
    pso::{PipelineStage, ShaderStageFlags, VertexInputRate},
    queue::{QueueGroup, Submission},
    window,
};
use handy::HandleMap;
use nalgebra::{Vector2, Vector3};
use std::{
    borrow::Borrow,
    iter,
    mem::{self, ManuallyDrop},
    ptr,
};
pub struct GPU<B: gfx_hal::Backend> {
    device: B::Device,
    queue_group: QueueGroup<B>,
    #[allow(dead_code)]
    desc_pool: ManuallyDrop<B::DescriptorPool>,
    surface: ManuallyDrop<B::Surface>,
    adapter: gfx_hal::adapter::Adapter<B>,
    format: gfx_hal::format::Format,
    dimensions: window::Extent2D,
    viewport: pso::Viewport,
    render_pass: ManuallyDrop<B::RenderPass>,
    pipeline: ManuallyDrop<B::GraphicsPipeline>,
    pipeline_layout: ManuallyDrop<B::PipelineLayout>,
    desc_set: B::DescriptorSet,
    #[allow(dead_code)]
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
pub const DEFAULT_SIZE: window::Extent2D = window::Extent2D {
    width: 1024,
    height: 700,
};
pub struct ModelAllocation<B: gfx_hal::Backend> {
    vertex_buffer: ManuallyDrop<B::Buffer>,
    #[allow(dead_code)]
    buffer_memory: ManuallyDrop<B::Memory>,
    vertex_count: u32,
}
pub struct TextureAllocation<B: gfx_hal::Backend> {
    freed: bool,
    image_upload_buffer: ManuallyDrop<B::Buffer>,
    image_logo: ManuallyDrop<B::Image>,
    image_memory: ManuallyDrop<B::Memory>,
    image_upload_memory: ManuallyDrop<B::Memory>,
    image_srv: ManuallyDrop<B::ImageView>,
    sampler: ManuallyDrop<B::Sampler>,
}

impl<B: gfx_hal::Backend> GPU<B> {
    pub fn new(
        _: Option<B::Instance>,
        mut surface: B::Surface,
        adapter: gfx_hal::adapter::Adapter<B>,
    ) -> Self {
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
        let queue_group = gpu.queue_groups.pop().unwrap();
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
                let glsl = include_str!("../data/quad.vert");
                let file =
                    glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Vertex).unwrap();
                let spirv = gfx_auxil::read_spirv(file).unwrap();
                unsafe { device.create_shader_module(&spirv) }.unwrap()
            };
            let fs_module = {
                let glsl = include_str!("../data/quad.frag");
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
                    stride: mem::size_of::<f32>() as u32 * 5,
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
                            offset: std::mem::size_of::<f32>() as u32 * 3,
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
    pub fn load_verticies(
        &mut self,
        verticies: &mut Vec<(Vector3<f32>, Vector2<f32>)>,
    ) -> ModelAllocation<B> {
        assert_eq!(
            std::mem::size_of::<(Vector3<f32>, Vector2<f32>)>(),
            std::mem::size_of::<f32>() * 5
        );
        for (m, u) in verticies.iter() {
            println!(
                "loading verticies: ({} {} {}) ({} {})",
                m.x, m.y, m.z, u.x, u.y
            );
        }
        let memory_types = self
            .adapter
            .physical_device
            .memory_properties()
            .memory_types;
        let limits = self.adapter.physical_device.limits();
        println!("Memory types: {:?}", memory_types);

        let non_coherent_alignment = limits.non_coherent_atom_size as u64;

        let buffer_stride = mem::size_of::<(Vector3<f32>, Vector2<f32>)>() as u64;
        let buffer_len = verticies.len() as u64 * buffer_stride;
        assert_ne!(buffer_len, 0);
        let padded_buffer_len = ((buffer_len + non_coherent_alignment - 1)
            / non_coherent_alignment)
            * non_coherent_alignment;

        let mut vertex_buffer = ManuallyDrop::new(
            unsafe {
                self.device
                    .create_buffer(padded_buffer_len, buffer::Usage::VERTEX)
            }
            .unwrap(),
        );

        let buffer_req = unsafe { self.device.get_buffer_requirements(&vertex_buffer) };

        let upload_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, mem_type)| {
                // type_mask is a bit field where each bit represents a memory type. If the bit is set
                // to 1 it means we can use that type for our buffer. So this code finds the first
                // memory type that has a `1` (or, is allowed), and is visible to the CPU.
                buffer_req.type_mask & (1 << id) != 0
                    && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
            })
            .unwrap()
            .into();
        // TODO: check transitions: read/write mapping and vertex buffer read
        let buffer_memory = unsafe {
            let memory = self
                .device
                .allocate_memory(upload_type, buffer_req.size)
                .unwrap();
            self.device
                .bind_buffer_memory(&memory, 0, &mut vertex_buffer)
                .unwrap();
            let mapping = self.device.map_memory(&memory, m::Segment::ALL).unwrap();
            ptr::copy_nonoverlapping(
                verticies.as_ptr() as *const u8,
                mapping,
                buffer_len as usize,
            );
            self.device
                .flush_mapped_memory_ranges(iter::once((&memory, m::Segment::ALL)))
                .unwrap();
            self.device.unmap_memory(&memory);
            ManuallyDrop::new(memory)
        };

        ModelAllocation {
            vertex_buffer,
            buffer_memory,
            vertex_count: verticies.len() as u32,
        }
    }
    fn wait_fence(
        &mut self,
    ) -> (
        &mut B::CommandBuffer,
        &B::Fence,
        &mut gfx_hal::queue::family::QueueGroup<B>,
    ) {
        let frame_idx = self.frame as usize % self.frames_in_flight;
        let fence = &self.submission_complete_fences[frame_idx];
        unsafe {
            self.device
                .wait_for_fence(fence, !0)
                .expect("Failed to wait for fence");
            self.device
                .reset_fence(fence)
                .expect("Failed to reset fence");
            self.cmd_pools[frame_idx].reset(false);
        }

        (
            &mut self.cmd_buffers[frame_idx],
            fence,
            &mut self.queue_group,
        )
    }
    unsafe fn bind_verticies(
        model: *const ModelAllocation<B>,
        command_buffer: &mut B::CommandBuffer,
    ) {
        command_buffer.bind_vertex_buffers(
            0,
            iter::once((&*(*model).vertex_buffer, buffer::SubRange::WHOLE)),
        );
    }
    pub fn load_textures(&mut self, image: &image::RgbaImage) -> TextureAllocation<B> {
        println!("dimensions: {} {}", image.width(), image.height());
        let limits = self.adapter.physical_device.limits();
        let non_coherent_alignment = limits.non_coherent_atom_size as u64;
        let kind = i::Kind::D2(image.width() as i::Size, image.height() as i::Size, 1, 1);
        let row_alignment_mask = limits.optimal_buffer_copy_pitch_alignment as u32 - 1;
        let image_stride = 4usize;
        let row_pitch =
            (image.width() * image_stride as u32 + row_alignment_mask) & !row_alignment_mask;
        let upload_size = (image.height() * row_pitch) as u64;
        println!("row pitch: {}", row_pitch);
        let padded_upload_size = ((upload_size + non_coherent_alignment - 1)
            / non_coherent_alignment)
            * non_coherent_alignment;

        let mut image_upload_buffer = ManuallyDrop::new(
            unsafe {
                self.device
                    .create_buffer(padded_upload_size, buffer::Usage::TRANSFER_SRC)
            }
            .unwrap(),
        );
        let buffer_req = unsafe { self.device.get_buffer_requirements(&image_upload_buffer) };
        let memory_types = self
            .adapter
            .physical_device
            .memory_properties()
            .memory_types;
        let upload_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, mem_type)| {
                // type_mask is a bit field where each bit represents a memory type. If the bit is set
                // to 1 it means we can use that type for our buffer. So this code finds the first
                // memory type that has a `1` (or, is allowed), and is visible to the CPU.
                buffer_req.type_mask & (1 << id) != 0
                    && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
            })
            .unwrap()
            .into();
        let image_mem_reqs = unsafe { self.device.get_buffer_requirements(&image_upload_buffer) };
        // copy image data into staging buffer

        //checking preconditions
        let image_layout = image.as_flat_samples().layout;
        assert_eq!(image_layout.width_stride, 4);
        assert_eq!(image_layout.height_stride, 4 * image_layout.width as usize);
        let image_upload_memory = unsafe {
            let memory = self
                .device
                .allocate_memory(upload_type, image_mem_reqs.size)
                .unwrap();
            self.device
                .bind_buffer_memory(&memory, 0, &mut image_upload_buffer)
                .unwrap();
            let mapping = self.device.map_memory(&memory, m::Segment::ALL).unwrap();
            for y in 0..image.height() {
                ptr::copy_nonoverlapping(
                    image
                        .as_ptr()
                        .offset(y as isize * image_layout.height_stride as isize),
                    mapping.offset(y as isize * row_pitch as isize),
                    image.width() as usize * image_layout.width_stride as usize,
                )
            }
            //ptr::copy_nonoverlapping(image.as_ptr(), mapping, upload_size as usize);
            //for y in 0..image.height() as usize {
            //    let row = flat_sample.as_mut_slice();
            //    ptr::copy_nonoverlapping(
            //        row.as_ptr(),
            //        mapping.offset(y as isize * row_pitch as isize),
            //        image.width() as usize * image_stride,
            //    );
            //}
            self.device
                .flush_mapped_memory_ranges(iter::once((&memory, m::Segment::ALL)))
                .unwrap();
            self.device.unmap_memory(&memory);
            ManuallyDrop::new(memory)
        };
        let mut image_logo = ManuallyDrop::new(
            unsafe {
                self.device.create_image(
                    kind,
                    1,
                    ColorFormat::SELF,
                    i::Tiling::Optimal,
                    i::Usage::TRANSFER_DST | i::Usage::SAMPLED,
                    i::ViewCapabilities::empty(),
                )
            }
            .unwrap(),
        );
        let image_req = unsafe { self.device.get_image_requirements(&image_logo) };

        let device_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, memory_type)| {
                image_req.type_mask & (1 << id) != 0
                    && memory_type.properties.contains(m::Properties::DEVICE_LOCAL)
            })
            .unwrap()
            .into();
        let image_memory = ManuallyDrop::new(
            unsafe { self.device.allocate_memory(device_type, image_req.size) }.unwrap(),
        );

        unsafe {
            self.device
                .bind_image_memory(&image_memory, 0, &mut image_logo)
        }
        .unwrap();
        let image_srv = ManuallyDrop::new(
            unsafe {
                self.device.create_image_view(
                    &image_logo,
                    i::ViewKind::D2,
                    ColorFormat::SELF,
                    Swizzle::NO,
                    i::SubresourceRange {
                        aspects: f::Aspects::COLOR,
                        ..Default::default()
                    },
                )
            }
            .unwrap(),
        );

        let sampler = ManuallyDrop::new(
            unsafe {
                self.device
                    .create_sampler(&i::SamplerDesc::new(i::Filter::Linear, i::WrapMode::Clamp))
            }
            .expect("Can't create sampler"),
        );

        unsafe {
            self.device.write_descriptor_sets(vec![
                pso::DescriptorSetWrite {
                    set: &(self.desc_set),
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(pso::Descriptor::Image(
                        &*image_srv,
                        i::Layout::ShaderReadOnlyOptimal,
                    )),
                },
                pso::DescriptorSetWrite {
                    set: &(self.desc_set),
                    binding: 1,
                    array_offset: 0,
                    descriptors: Some(pso::Descriptor::Sampler(&*sampler)),
                },
            ]);
        }

        //buffering texture

        unsafe {
            let (cmd_buffer, fence, queue_group) = self.wait_fence();
            cmd_buffer.begin_primary(command::CommandBufferFlags::ONE_TIME_SUBMIT);

            let image_barrier = m::Barrier::Image {
                states: (i::Access::empty(), i::Layout::Undefined)
                    ..(i::Access::TRANSFER_WRITE, i::Layout::TransferDstOptimal),
                target: &*image_logo,
                families: None,
                range: i::SubresourceRange {
                    aspects: f::Aspects::COLOR,
                    ..Default::default()
                },
            };

            cmd_buffer.pipeline_barrier(
                PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
                m::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.copy_buffer_to_image(
                &image_upload_buffer,
                &image_logo,
                i::Layout::TransferDstOptimal,
                &[command::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_width: row_pitch / (image_stride as u32),
                    buffer_height: image.height() as u32,
                    image_layers: i::SubresourceLayers {
                        aspects: f::Aspects::COLOR,
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: i::Offset { x: 0, y: 0, z: 0 },
                    image_extent: i::Extent {
                        width: image.width(),
                        height: image.height(),
                        depth: 1,
                    },
                }],
            );

            let image_barrier = m::Barrier::Image {
                states: (i::Access::TRANSFER_WRITE, i::Layout::TransferDstOptimal)
                    ..(i::Access::SHADER_READ, i::Layout::ShaderReadOnlyOptimal),
                target: &*image_logo,
                families: None,
                range: i::SubresourceRange {
                    aspects: f::Aspects::COLOR,
                    ..Default::default()
                },
            };
            cmd_buffer.pipeline_barrier(
                PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
                m::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.finish();

            queue_group.queues[0].submit_without_semaphores(Some(&cmd_buffer), Some(&fence));

            //I need to wait for the fence somehow opps
            //todo!("Wait for fence. That might be a bug");
        }
        TextureAllocation {
            freed: false,
            image_upload_buffer,
            image_logo,
            image_memory,
            image_upload_memory,
            image_srv,
            sampler,
        }
    }
    pub unsafe fn destroy_texture(&mut self, texture: &mut TextureAllocation<B>) {
        assert_eq!(texture.freed, false);
        self.device
            .destroy_buffer(ManuallyDrop::take(&mut (texture).image_upload_buffer));
        self.device
            .destroy_image(ManuallyDrop::take(&mut (*texture).image_logo));
        self.device
            .free_memory(ManuallyDrop::take(&mut (*texture).image_memory));
        self.device
            .free_memory(ManuallyDrop::take(&mut (*texture).image_upload_memory));
        self.device
            .destroy_image_view(ManuallyDrop::take(&mut (*texture).image_srv));
        self.device
            .destroy_sampler(ManuallyDrop::take(&mut (*texture).sampler));
        texture.freed = true;
    }
    pub unsafe fn bind_texture(
        texture: *const TextureAllocation<B>,
        device: &B::Device,
        descriptor_set: &B::DescriptorSet,
    ) {
        device.write_descriptor_sets(vec![
            pso::DescriptorSetWrite {
                set: descriptor_set,
                binding: 0,
                array_offset: 0,
                descriptors: Some(pso::Descriptor::Image(
                    &*((*texture).image_srv),
                    i::Layout::ShaderReadOnlyOptimal,
                )),
            },
            pso::DescriptorSetWrite {
                set: descriptor_set,
                binding: 1,
                array_offset: 0,
                descriptors: Some(pso::Descriptor::Sampler(&*(*texture).sampler)),
            },
        ]);
    }
    fn recreate_swapchain(&mut self) {
        let caps = self.surface.capabilities(&self.adapter.physical_device);
        let swap_config = window::SwapchainConfig::from_caps(&caps, self.format, self.dimensions);
        println!("{:#?}", swap_config);
        let extent = swap_config.extent.to_extent();

        unsafe {
            self.surface
                .configure_swapchain(&self.device, swap_config)
                .expect("Can't create swapchain");
        }

        self.viewport.rect.w = extent.width as _;
        self.viewport.rect.h = extent.height as _;
    }
    pub fn change_resolution(&mut self, new_size: window::Extent2D) {
        self.dimensions = new_size;
        self.recreate_swapchain();
    }
    pub fn draw_models(
        &mut self,
        draw_calls: Vec<(*const ModelAllocation<B>, *const TextureAllocation<B>)>,
    ) {
        let surface_image = unsafe {
            match self.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.recreate_swapchain();
                    return;
                }
            }
        };
        let framebuffer = unsafe {
            self.device
                .create_framebuffer(
                    &self.render_pass,
                    iter::once(surface_image.borrow()),
                    i::Extent {
                        width: self.dimensions.width,
                        height: self.dimensions.height,
                        depth: 1,
                    },
                )
                .unwrap()
        };

        // Compute index into our resource ring buffers based on the frame number
        // and number of frames in flight. Pay close attention to where this index is needed
        // versus when the swapchain image index we got from acquire_image is needed.
        let frame_idx = self.frame as usize % self.frames_in_flight;
        // Wait for the fence of the previous submission of this frame and reset it; ensures we are
        // submitting only up to maximum number of frames_in_flight if we are submitting faster than
        // the gpu can keep up with. This would also guarantee that any resources which need to be
        // updated with a CPU->GPU data copy are not in use by the GPU, so we can perform those updates.
        // In this case there are none to be done, however.
        unsafe {
            let fence = &self.submission_complete_fences[frame_idx];
            self.device
                .wait_for_fence(fence, !0)
                .expect("Failed to wait for fence");
            self.device
                .reset_fence(fence)
                .expect("Failed to reset fence");
            self.cmd_pools[frame_idx].reset(false);
        }
        // Rendering
        let cmd_buffer = &mut self.cmd_buffers[frame_idx];
        unsafe {
            cmd_buffer.begin_primary(command::CommandBufferFlags::ONE_TIME_SUBMIT);

            cmd_buffer.set_viewports(0, &[self.viewport.clone()]);
            cmd_buffer.set_scissors(0, &[self.viewport.rect]);
            //cmd_buffer.set_scissors(0, &[self.viewport.rect]);
            cmd_buffer.bind_graphics_pipeline(&self.pipeline);
            cmd_buffer.bind_graphics_descriptor_sets(
                &self.pipeline_layout,
                0,
                iter::once(&self.desc_set),
                &[],
            );

            cmd_buffer.begin_render_pass(
                &self.render_pass,
                &framebuffer,
                self.viewport.rect,
                &[command::ClearValue {
                    color: command::ClearColor {
                        float32: [0.8, 0.8, 0.8, 1.0],
                    },
                }],
                command::SubpassContents::Inline,
            );
            let cmd_ptr = (cmd_buffer) as *mut B::CommandBuffer;
            for (m, t) in draw_calls.iter() {
                Self::bind_texture(*t, &self.device, &self.desc_set);
                Self::bind_verticies(*m, cmd_buffer);
                (*cmd_ptr).draw(0..(**m).vertex_count, 0..1);
            }
            cmd_buffer.end_render_pass();
            cmd_buffer.finish();

            let submission = Submission {
                command_buffers: iter::once(&*cmd_buffer),
                wait_semaphores: None,
                signal_semaphores: iter::once(&self.submission_complete_semaphores[frame_idx]),
            };
            self.queue_group.queues[0].submit(
                submission,
                Some(&self.submission_complete_fences[frame_idx]),
            );

            // present frame
            let result = self.queue_group.queues[0].present(
                &mut self.surface,
                surface_image,
                Some(&self.submission_complete_semaphores[frame_idx]),
            );

            self.device.destroy_framebuffer(framebuffer);

            if result.is_err() {
                self.recreate_swapchain();
            }
        }

        // Increment our frame
        self.frame += 1;
    }
}
