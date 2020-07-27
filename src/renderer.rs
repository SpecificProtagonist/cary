use std::{
    mem,
    borrow::Cow,
};
use image::GenericImageView;
use winit::window::Window;
use wgpu::*;
use super::textures::{self, Sprite};
use super::Vec2;



#[repr(C)]
#[derive(Copy, Clone)]
struct SpriteInstance {
    position: Vec2,
    size: Vec2,
    uv_center: Vec2,
    layer: f32
} 
unsafe impl bytemuck::Pod for SpriteInstance {}
unsafe impl bytemuck::Zeroable for SpriteInstance {}




pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    texture_bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    swap_chain: SwapChain,
    swap_chain_desc: SwapChainDescriptor,

    vertex_buffer: Buffer,
    instances: Vec<SpriteInstance>,
}

impl Renderer {
    pub async fn create(window: &Window) -> Self {
        /* Set up device */

        let size = window.inner_size();
        let instance = Instance::new(BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) }; // Window handle guaranteed valid -> safe

        let adapter = instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: Some(&surface),
            }
        )
        .await
        .unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            features: Features::empty(),
            limits: Limits::default(),
            shader_validation: true // Will be removed later
        }, None).await.unwrap();

        let (texture_bind_group, texture_bind_group_layout) = Self::create_texture(&device, &queue);

        let vertex_shader_module = device.create_shader_module(
            include_spirv!(concat!(env!("OUT_DIR"), "/shaders/vertex.spv")));

        let fragment_shader_module = device.create_shader_module(
            include_spirv!(concat!(env!("OUT_DIR"), "/shaders/fragment.spv")));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: Cow::Owned(vec![&texture_bind_group_layout]),
            push_constant_ranges: Cow::Owned(vec![])
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vertex_shader_module,
                entry_point: Cow::Borrowed("main"),
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fragment_shader_module,
                entry_point: Cow::Borrowed("main"),
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: Cow::Owned(vec![ColorStateDescriptor {
                format: TextureFormat::Bgra8UnormSrgb,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }]),
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: Cow::Owned(vec![
                    // Instance buffer
                    VertexBufferDescriptor {
                        stride: mem::size_of::<SpriteInstance>() as BufferAddress,
                        step_mode: InputStepMode::Instance,
                        attributes: Cow::Owned(vec![
                            // Position
                            VertexAttributeDescriptor {
                                offset: 0,
                                shader_location: 1,
                                format: VertexFormat::Float2,
                            },
                            // Size
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 2,
                                shader_location: 2,
                                format: VertexFormat::Float2,
                            },
                            // UV-Coordinates center
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 4,
                                shader_location: 3,
                                format: VertexFormat::Float2,
                            },
                            // Layer
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 6,
                                shader_location: 4,
                                format: VertexFormat::Float,
                            }
                        ])
                    },
                    // Vertex buffer
                    VertexBufferDescriptor {
                        stride: mem::size_of::<[f32; 2]>() as BufferAddress,
                        step_mode: InputStepMode::Vertex,
                        attributes: Cow::Owned(vec![
                            VertexAttributeDescriptor {
                                offset: 0,
                                shader_location: 0,
                                format: VertexFormat::Float2,
                            }
                        ])
                    }
                ]),
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let swap_chain_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[
                [-0.5_f32,  0.5_f32],
                [-0.5_f32, -0.5_f32],
                [ 0.5_f32,  0.5_f32],
                [-0.5_f32, -0.5_f32],
                [ 0.5_f32, -0.5_f32],
                [ 0.5_f32,  0.5_f32]
            ]),
            BufferUsage::VERTEX
        );

        Renderer {
            surface,
            device,
            queue,
            texture_bind_group,
            render_pipeline,
            swap_chain,
            swap_chain_desc,

            vertex_buffer,
            instances: Vec::new()
        }
    }


    fn create_texture(device: &Device, queue: &Queue) -> (BindGroup, BindGroupLayout) {
        let texture_image = image::load_from_memory(include_bytes!(concat!(env!("OUT_DIR"), "/textures.png"))).unwrap();
        // All textures are stored as 3d, we represent our 2d texture by setting depth to 1.
        let texture_size = Extent3d {
            width: texture_image.dimensions().0,
            height: texture_image.dimensions().1,
            depth: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            // SAMPLED tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            label: Some(Cow::Borrowed("texture-atlas")),
        });

        let texture_buffer = device.create_buffer_with_data(
            &texture_image.as_rgba8().unwrap(),
            BufferUsage::COPY_SRC 
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some(Cow::Borrowed("texture-buffer-copy-encoder")),
        });

        encoder.copy_buffer_to_texture(
            BufferCopyView {
                buffer: &texture_buffer,
                layout: TextureDataLayout {
                    offset: 0,
                    bytes_per_row: 4 * texture_size.width,
                    rows_per_image: texture_size.height,
                }
            },
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            texture_size,
        );

        queue.submit(Some(encoder.finish()));

        let texture_view = texture.create_default_view();

        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            label: Some(Cow::Borrowed("texture-sampler"))
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: Cow::Owned(vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Uint,
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler {
                        comparison: false,
                    },
                    count: None
                },
            ]),
            label: Some(Cow::Borrowed("texture-bind-group-layout")),
        });

        (
            device.create_bind_group(&BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: Cow::Owned(vec![
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&texture_sampler),
                    }
                ]),
                label: Some(Cow::Borrowed("texture-bind-group")),
            }),
            texture_bind_group_layout,
        )
    }



    pub fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn render(&mut self) {
        // TEST
        self.draw(Vec2(0.3,0.3), textures::ITEM_CONSUMABLE_HEALTH_POTION, 0.5);

        let instance_buffer = self.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.instances), 
            BufferUsage::VERTEX
        );

        let frame = self.swap_chain
            .get_current_frame()
            .expect("Timeout when acquiring next swap chain texture");
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: None,
        });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: Cow::Owned(vec![RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLUE),
                        store: true
                    }
                }]),
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..(self.instances.len() as u32));
        }
        self.queue.submit(Some(encoder.finish()));
        
    }

    pub fn draw(&mut self, pos: Vec2, sprite: Sprite, layer: f32) {
        self.instances.push(SpriteInstance {position: pos, size: Vec2(1.0, 1.0), uv_center: sprite.center, layer})
    }
}