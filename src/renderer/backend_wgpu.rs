use std::{
    mem,
    borrow::Cow,
};
use image::GenericImageView;
use winit::window::Window;
use wgpu::*;
use crate::textures::{self, TexCoords, TexAnchor};
use crate::Vec2;
use super::Layer;


// TODO: remove light stuffs


#[repr(C)]
#[derive(Copy, Clone)]
struct SpriteInstance {
    pos: Vec2,
    size: Vec2,
    uv_center: Vec2,
    uv_size: Vec2,
    layer: f32,
    rotation: f32
} 
unsafe impl bytemuck::Pod for SpriteInstance {}
unsafe impl bytemuck::Zeroable for SpriteInstance {}

#[repr(C)]
#[derive(Copy, Clone)]
struct LightInstance {
    pos: Vec2,
    size: Vec2,
    uv_center: Vec2,
    uv_size: Vec2,
    color: (f32, f32, f32)
} 
unsafe impl bytemuck::Pod for LightInstance {}
unsafe impl bytemuck::Zeroable for LightInstance {}


pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    tex_bind_group: BindGroup,
    tex_light_bind_group_layout: BindGroupLayout,
    tex_light_bind_group: BindGroup,
    render_pipeline_lights: RenderPipeline,
    render_pipeline_world: RenderPipeline,
    light_tex_view: TextureView,
    depth_tex_view: TextureView,
    swap_chain: SwapChain,
    swap_chain_desc: SwapChainDescriptor,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,

    vertex_buffer: Buffer,
    light_instances: Vec<LightInstance>,
    sprite_instances: Vec<SpriteInstance>,
}


impl Renderer {
    pub fn create(window: &Window) -> Self {
        futures::executor::block_on(Self::create_async(window))
    }

    async fn create_async(window: &Window) -> Self {
        /* Set up device */

        let size = window.inner_size();
        let instance = Instance::new(BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) }; // Window handle guaranteed valid -> safe

        let adapter = instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: Some(&surface)
            }
        )
        .await
        .unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            features: Features::empty(),
            limits: Limits::default(),
            shader_validation: true // Will be removed later
        }, None).await.unwrap();

        let (tex_bind_group, tex_bind_group_layout) = Self::load_sprite_texture(&device, &queue);

        let vertex_shader_light = device.create_shader_module(
            include_spirv!(concat!(env!("OUT_DIR"), "/shaders/vertex_light.spv")));

        let fragment_shader_light = device.create_shader_module(
            include_spirv!(concat!(env!("OUT_DIR"), "/shaders/fragment_light.spv")));

        let vertex_shader_world = device.create_shader_module(
            include_spirv!(concat!(env!("OUT_DIR"), "/shaders/vertex_world.spv")));

        let fragment_shader_world = device.create_shader_module(
            include_spirv!(concat!(env!("OUT_DIR"), "/shaders/fragment_world.spv")));

        let tex_light_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ]),
            label: None
        });

        let (tex_light_bind_group, light_tex_view, depth_tex_view) 
            = Self::create_light_and_depth_texture(&device, &tex_light_bind_group_layout, size.width, size.height);

        let render_pipeline_lights = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &device.create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: Cow::Owned(vec![&tex_bind_group_layout]),
                push_constant_ranges: Cow::Owned(vec![])
            }),
            vertex_stage: ProgrammableStageDescriptor {
                module: &vertex_shader_light,
                entry_point: Cow::Borrowed("main"),
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fragment_shader_light,
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
                color_blend: BlendDescriptor { src_factor: BlendFactor::One, dst_factor: BlendFactor::One, operation: BlendOperation::Add},
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }]),
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: Cow::Owned(vec![
                    // Instance buffer
                    VertexBufferDescriptor {
                        stride: mem::size_of::<LightInstance>() as BufferAddress,
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
                            // UV-coordinates center
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 4,
                                shader_location: 3,
                                format: VertexFormat::Float2,
                            },
                            // UV-coordinates size
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 6,
                                shader_location: 4,
                                format: VertexFormat::Float2,
                            },
                            // Color
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 8,
                                shader_location: 5,
                                format: VertexFormat::Float3,
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
        
        let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: Cow::Owned(vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ]),
            label: None
        });

        let uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[0.0f32; 5]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let uniform_bind_group= device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: Cow::Owned(vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(uniform_buffer.slice(..))
                }
            ]),
            label: None
        });

        let render_pipeline_world = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &device.create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: Cow::Owned(vec![&tex_bind_group_layout, &tex_light_bind_group_layout, &uniform_bind_group_layout]),
                push_constant_ranges: Cow::Owned(vec![])
            }),
            vertex_stage: ProgrammableStageDescriptor {
                module: &vertex_shader_world,
                entry_point: Cow::Borrowed("main"),
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fragment_shader_world,
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
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
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
                            // UV-coordinates center
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 4,
                                shader_location: 3,
                                format: VertexFormat::Float2,
                            },
                            // UV-coordinates size
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 6,
                                shader_location: 4,
                                format: VertexFormat::Float2,
                            },
                            // Layer
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 8,
                                shader_location: 5,
                                format: VertexFormat::Float,
                            },
                            // Rotation
                            VertexAttributeDescriptor {
                                offset: std::mem::size_of::<f32>() as BufferAddress * 9,
                                shader_location: 6,
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
            tex_bind_group,
            tex_light_bind_group_layout,
            tex_light_bind_group,
            render_pipeline_lights,
            render_pipeline_world,
            light_tex_view: light_tex_view,
            depth_tex_view: depth_tex_view,
            swap_chain,
            swap_chain_desc,
            uniform_buffer,
            uniform_bind_group,

            vertex_buffer,
            light_instances: Vec::new(),
            sprite_instances: Vec::new(),
        }
    }


    fn load_sprite_texture(device: &Device, queue: &Queue) -> (BindGroup, BindGroupLayout) {
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
            label: None
        });

        let texture_buffer = device.create_buffer_with_data(
            &texture_image.as_rgba8().unwrap(),
            BufferUsage::COPY_SRC 
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: None
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
            label: None
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
            label: None
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
                label: None
            }),
            texture_bind_group_layout,
        )
    }

    fn create_light_and_depth_texture(device: &Device, layout: &BindGroupLayout, width: u32, height: u32) -> (BindGroup, TextureView, TextureView) {
        let light_texture= device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width,
                height,
                depth: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT,
            label: None
        });

        let light_texture_view = light_texture.create_default_view();

        let depth_texture= device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width,
                height,
                depth: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsage::SAMPLED | TextureUsage::OUTPUT_ATTACHMENT,
            label: None
        });

        let depth_texture_view = depth_texture.create_default_view();

        let light_texture_sampler = device.create_sampler(&SamplerDescriptor {
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
            label: None
        });

        let uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[width as f32, height as f32]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        (
            device.create_bind_group(&BindGroupDescriptor {
                layout: &layout,
                entries: Cow::Owned(vec![
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&light_texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&light_texture_sampler),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Buffer(uniform_buffer.slice(..))
                    }
                ]),
                label: None
            }),
            light_texture_view,
            depth_texture_view
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_desc.width = width;
        self.swap_chain_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
        let tex_resize = Self::create_light_and_depth_texture(
            &self.device, &self.tex_light_bind_group_layout, width, height);
        self.tex_light_bind_group = tex_resize.0;
        self.light_tex_view = tex_resize.1;
        self.depth_tex_view = tex_resize.2;
    }

    pub fn render(&mut self) {
        let frame = self.swap_chain
            .get_current_frame()
            .expect("Timeout when acquiring next swap chain texture");

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: None
        });

        // Light
        let instance_buffer_light = self.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.light_instances), 
            BufferUsage::VERTEX
        );
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: Cow::Owned(vec![RenderPassColorAttachmentDescriptor { // TODO: take vec out of loop
                attachment: &self.light_tex_view,
                //attachment: &frame.output.view,
                resolve_target: None,
                ops: Operations { load: LoadOp::Clear(Color::BLACK), store: true }
            }]),
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline_lights);
        render_pass.set_bind_group(0, &self.tex_bind_group, &[]);
        render_pass.set_vertex_buffer(0, instance_buffer_light.slice(..));
        render_pass.set_vertex_buffer(1, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..(self.light_instances.len() as u32));
        drop(render_pass);
        self.light_instances.clear();

        // World
        let instance_buffer_world = self.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.sprite_instances), 
            BufferUsage::VERTEX
        );
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: Cow::Owned(vec![RenderPassColorAttachmentDescriptor {
                attachment: &frame.output.view,
                resolve_target: None,
                ops: Operations {load: LoadOp::Clear(Color::BLACK), store: true }
            }]),
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_tex_view,
                depth_ops: Some(Operations {load: LoadOp::Clear(1.0), store: true} ),
                stencil_ops: None
            })
        });
        render_pass.set_pipeline(&self.render_pipeline_world);
        render_pass.set_bind_group(0, &self.tex_bind_group, &[]);
        render_pass.set_bind_group(1, &self.tex_light_bind_group, &[]);
        render_pass.set_bind_group(2, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, instance_buffer_world.slice(..));
        render_pass.set_vertex_buffer(1, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..(self.sprite_instances.len() as u32));
        drop(render_pass);
        self.sprite_instances.clear();


        self.queue.submit(Some(encoder.finish()));
    }

    pub fn set_transition(&mut self, camera: &crate::Camera, center: Vec2, distance: f32, victory: bool) {
        let aspect_ratio = self.swap_chain_desc.height as f32 / self.swap_chain_desc.width as f32;
        let screen_pos = ((center-camera.pos) * aspect_ratio/camera.size + Vec2(1.0, -1.0)) * Vec2(0.5, -0.5);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: None});

        let staging_buffer = self.device.create_buffer_with_data(
            bytemuck::cast_slice(&[screen_pos.0, screen_pos.1, distance, aspect_ratio, if victory {1.0} else {0.0} ]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.uniform_buffer, 0, 4*5 as wgpu::BufferAddress);

        self.queue.submit(Some(encoder.finish()));
    }

    pub fn draw(&mut self, camera: &crate::Camera, pos: Vec2, anchor: TexAnchor, tex: &TexCoords, layer: Layer, mirror: bool, rotation: u8) {
        let size_real = tex.size / textures::PIXELS_PER_TILE;
        let pos = Vec2(pos.0, pos.1 + match anchor {
            TexAnchor::Top    => -size_real.1/2.0,
            TexAnchor::Center => 0.0,
            TexAnchor::Bottom => size_real.1/2.0
        });
        let aspect_ratio = self.swap_chain_desc.height as f32 / self.swap_chain_desc.width as f32;
        let screen_pos = (pos-camera.pos) / camera.size * Vec2(aspect_ratio, 1.0);
        let screen_size = size_real / camera.size
            * if rotation % 2 == 0 {Vec2(aspect_ratio, 1.0)} else {Vec2(1.0, aspect_ratio)};
        if (screen_pos.0 + screen_size.0/2.0 > -1.0) &
           (screen_pos.1 + screen_size.1/2.0 > -1.0) &
           (screen_pos.0 - screen_size.0/2.0 <  1.0) &
           (screen_pos.1 - screen_size.1/2.0 <  1.0) 
        {
            self.sprite_instances.push(SpriteInstance {
                pos: screen_pos,
                size: screen_size,
                uv_center: tex.center * textures::UV_COORDS_FACTOR,
                uv_size: tex.size * if mirror {Vec2(-1.0, 1.0)} else {Vec2(1.0, 1.0)} * textures::UV_COORDS_FACTOR,
                layer: layer.into(),
                rotation: rotation as f32
            })
        }
    }

}
