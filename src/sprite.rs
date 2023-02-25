use wgpu::util::DeviceExt;
use wgpu::*;

const STARTING_LENGTH: u16 = 16;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WindowUnifrom {
    pub size: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

fn indices(sprites: u16) -> Vec<u16> {
    let mut indicies = Vec::<u16>::new();

    for i in 0..sprites {
        let offset = i * 4;
        indicies.push(offset);
        indicies.push(1 + offset);
        indicies.push(2 + offset);
        indicies.push(2 + offset);
        indicies.push(3 + offset);
        indicies.push(offset);
    }

    indicies
}

pub struct SpriteRenderer {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    window_buffer: Buffer,
    pipeline: RenderPipeline,
    window_bind_group: BindGroup,
    texture_bind_group_layout: BindGroupLayout,
    length: u16,
}

pub struct Sprite {
    pub position: [f32; 2],
    pub size: [f32; 2],
}

impl Sprite {
    fn vertices(&self) -> [Vertex; 4] {
        let half_width = self.size[0] / 2.0;
        let half_height = self.size[1] / 2.0;

        let left = self.position[0] - half_width;
        let right = self.position[0] + half_width;

        let bottom = self.position[1] - half_height;
        let top = self.position[1] + half_height;

        [
            Vertex {
                position: [left, bottom],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [right, bottom],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [right, top],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [left, top],
                tex_coords: [0.0, 1.0],
            },
        ]
    }
}

pub struct SpriteBatch<'a> {
    pub sprites: Vec<Sprite>,
    pub texture_bind_group: &'a BindGroup,
}

impl SpriteRenderer {
    pub fn new(
        config: &SurfaceConfiguration,
        device: &Device,
        window_width: f32,
        window_height: f32,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("sprite.wgsl"));

        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: &[0u8; std::mem::size_of::<Vertex>() * STARTING_LENGTH as usize],
            usage: wgpu::BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
            contents: bytemuck::cast_slice(&indices(STARTING_LENGTH)),
            usage: wgpu::BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let window_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("window_bind_group_layout"),
            });

        // window size
        let window_uniform = WindowUnifrom {
            size: [window_width, window_height],
        };

        let window_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Window Buffer"),
            contents: bytemuck::cast_slice(&[window_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let window_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &window_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: window_buffer.as_entire_binding(),
            }],
            label: Some("window_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &window_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::descriptor()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            texture_bind_group_layout,
            length: STARTING_LENGTH,
            vertex_buffer,
            index_buffer,
            pipeline,
            window_bind_group,
            window_buffer,
        }
    }

    pub fn create_texture_bind_group(
        &self,
        device: &Device,
        sampler: &Sampler,
        view: &TextureView,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("world_bind_group"),
        })
    }

    pub fn draw(
        &mut self,
        sprite_batches: &Vec<SpriteBatch>,
        device: &Device,
        queue: &Queue,
        view: &TextureView,
        window_size: [f32; 2],
    ) {
        // this doesn't need to write every frame, but I don't want to overcomplicate things
        queue.write_buffer(
            &self.window_buffer,
            0,
            bytemuck::cast_slice(&[WindowUnifrom { size: window_size }]),
        );

        for batch in sprite_batches.iter() {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("wgpu-text Command Encoder"),
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(1, &self.window_bind_group, &[]);

            let sprites = &batch.sprites;
            let num_sprites = sprites.len() as u32;
            if self.length < num_sprites as u16 {
                todo!();
            }

            let mut vertices = Vec::<Vertex>::new();

            for sprite in sprites.iter() {
                let sprite_vertices = sprite.vertices();
                vertices.push(sprite_vertices[0]);
                vertices.push(sprite_vertices[1]);
                vertices.push(sprite_vertices[2]);
                vertices.push(sprite_vertices[3]);
            }

            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

            render_pass.set_bind_group(0, batch.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_sprites * 6, 0, 0..1);
            drop(render_pass);
            queue.submit(vec![encoder.finish()]);
        }
    }
}
