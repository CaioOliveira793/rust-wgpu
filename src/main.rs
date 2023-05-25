use rust_wgpu_lib::{
    application::{AppState, Application, Layer, Screen},
    camera::{Camera, CameraController},
    renderer::{IndexBuffer, Vertex, VertexBuffer, QUAD_INDICES, QUAD_VERTICES},
    texture::Texture,
};
use wgpu::{
    include_wgsl, util::DeviceExt, CommandEncoderDescriptor, PipelineLayoutDescriptor,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event::Event};

struct RayTracingCPU {
    camera: Camera,
    camera_controller: CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    diffuse_bind_group: wgpu::BindGroup,
}

impl Layer for RayTracingCPU {
    type LayerErr = ();

    fn start(screen: &mut Screen, _app: &AppState) -> Self {
        let shader = screen
            .device
            .create_shader_module(include_wgsl!("asset/shader/basic_shape.wgsl"));

        let vertex_buffer = VertexBuffer::init_immediate(
            &screen.device,
            bytemuck::cast_slice(QUAD_VERTICES),
            Some("Vertex Buffer"),
        );
        let index_buffer =
            IndexBuffer::init_immediate_u16(&screen.device, QUAD_INDICES, Some("Index Buffer"));

        let img_texture = image::RgbaImage::from_raw(1, 1, vec![234, 65, 123, 255]).unwrap();
        let diffuse_texture = Texture::from_image(
            &screen.device,
            &screen.queue,
            &image::DynamicImage::ImageRgba8(img_texture),
            Some("Solid pink texture"),
        );

        let texture_bind_group_layout =
            screen
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let diffuse_bind_group = screen.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera = Camera::default();

        let camera_buffer = screen
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera.view_projection()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout =
            screen
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    label: Some("camera_bind_group_layout"),
                });

        let camera_bind_group = screen.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            screen
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline = screen
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: screen.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        Self {
            camera,
            camera_controller: CameraController::new(0.2),
            camera_buffer,
            camera_bind_group,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            diffuse_bind_group,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>, _state: &AppState, _screen: &mut Screen) {
        self.camera.projection.aspect_ratio = new_size.width as f32 / new_size.height as f32;
    }

    fn process_event(&mut self, event: &Event<()>, _screen: &mut Screen) {
        match event {
            Event::WindowEvent { ref event, .. } => {
                self.camera_controller
                    .process_events(&mut self.camera, event, 1.0);
            }
            _ => {}
        }
    }

    fn update(&mut self, _app: &AppState, screen: &mut Screen) {
        screen.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera.view_projection()]),
        );
    }

    fn render(&mut self, _app: &AppState, screen: &mut Screen) -> Result<(), wgpu::SurfaceError> {
        let output = screen.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = screen
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.buffer().slice(..));
            render_pass.set_index_buffer(
                self.index_buffer.buffer().slice(..),
                self.index_buffer.format(),
            );
            render_pass.draw_indexed(0..self.index_buffer.count(), 0, 0..1);
        }

        screen.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn shutdown(&mut self, _app: &AppState, _screen: &mut Screen) -> Result<(), Self::LayerErr> {
        tracing::info!("exiting");
        Ok(())
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    pollster::block_on(Application::<RayTracingCPU>::init());
}
