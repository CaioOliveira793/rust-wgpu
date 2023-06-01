use glam::*;
use image::{Rgba, RgbaImage};
use ray::Ray;
use rust_wgpu_lib::{
    application::{AppState, Application, Layer, Screen},
    camera::{Camera, CameraController},
    renderer::{IndexBuffer, Vertex, VertexBuffer, QUAD_INDICES, QUAD_VERTICES},
    texture::Texture,
};
use scene::{Scene, Sphere};
use wgpu::{
    include_wgsl, util::DeviceExt, CommandEncoderDescriptor, PipelineLayoutDescriptor,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event::Event};

mod ray;
mod scene;

struct RayTracingCPU {
    camera: Camera,
    camera_controller: CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    texture: Texture,
    img_texture: RgbaImage,
    scene: Scene,
    diffuse_bind_group: wgpu::BindGroup,
}

fn create_target_texture(screen: &Screen) -> (RgbaImage, Texture) {
    let mut img_data = Vec::with_capacity((IMG_WIDTH * IMG_HEIGHT * 4) as usize);
    for _ in 0..(IMG_WIDTH * IMG_HEIGHT) {
        for i in [234, 65, 123, 255] {
            img_data.push(i);
        }
    }

    let img_texture = image::RgbaImage::from_raw(IMG_WIDTH, IMG_HEIGHT, img_data).unwrap();
    let texture = Texture::from_image(
        &screen.device,
        &screen.queue,
        &img_texture,
        IMG_WIDTH,
        IMG_HEIGHT,
        Some("Target texture"),
    );
    (img_texture, texture)
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

        let (img_texture, texture) = create_target_texture(screen);

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
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
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

        let scene = Scene {
            spheres: vec![
                Sphere {
                    albedo: Vec3::new(1.0, 0.0, 1.0),
                    radius: 0.5,
                    position: Vec3::ZERO,
                },
                Sphere {
                    albedo: Vec3::new(0.2, 0.3, 1.0),
                    radius: 1.5,
                    position: Vec3::new(1.0, 0.0, -5.0),
                },
            ],
        };

        Self {
            camera,
            camera_controller: CameraController::new(0.2),
            camera_buffer,
            camera_bind_group,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            texture,
            img_texture,
            scene,
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

        render_to_texture(
            &mut self.img_texture,
            &self.texture,
            &mut self.scene,
            &screen.queue,
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

const IMG_WIDTH: u32 = 800;
const IMG_HEIGHT: u32 = 800;

fn render_to_texture(img: &mut RgbaImage, texture: &Texture, scene: &Scene, queue: &wgpu::Queue) {
    let mut ray = Ray {
        origin: glam::Vec3::new(0.0, 0.0, 2.0),
        direction: glam::Vec3::ZERO,
    };
    for y in 0..IMG_HEIGHT {
        for x in 0..IMG_WIDTH {
            let coord = glam::Vec2::new(x as f32 / IMG_WIDTH as f32, y as f32 / IMG_HEIGHT as f32)
                * 2.0
                - 1.0;
            ray.direction = glam::Vec3::new(coord.x, coord.y, -1.0);
            let color = cast_ray(scene, &ray);
            img.put_pixel(x, y, Rgba(convert_rgba(color)));
        }
    }

    texture.update_data(queue, &img, IMG_WIDTH, IMG_HEIGHT);
}

fn convert_rgba(color: glam::Vec4) -> [u8; 4] {
    let r = (color.x * 255.0) as u8;
    let g = (color.y * 255.0) as u8;
    let b = (color.z * 255.0) as u8;
    let a = (color.w * 255.0) as u8;
    [r, g, b, a]
}

fn cast_ray(scene: &Scene, ray: &Ray) -> glam::Vec4 {
    // (bx^2 + by^2 + bz^2)t^2 + (2(axbx + ayby + azbz))t + (ax^2 + ay^2 + az^2 - r^2) = 0
    // where
    // a = ray origin
    // b = ray direction
    // r = radius
    // t = hit distance

    let clear_color = glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
    let light_direction = glam::Vec3::new(-1.0, -1.0, -1.0).normalize();

    if scene.spheres.is_empty() {
        return clear_color;
    }

    let mut closest_sphere: Option<&Sphere> = None;
    let mut hit_distance = std::f32::MAX;

    for sphere in &scene.spheres {
        let origin = ray.origin - sphere.position;

        let a = glam::Vec3::dot(ray.direction, ray.direction);
        let b = 2.0 * glam::Vec3::dot(origin, ray.direction);
        let c = glam::Vec3::dot(origin, origin) - sphere.radius * sphere.radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            continue;
        }

        let closest_t = (-b - discriminant.sqrt()) / (2.0 * a);

        if closest_t < hit_distance {
            hit_distance = closest_t;
            closest_sphere = Some(&sphere);
        }
    }

    if closest_sphere.is_none() {
        return clear_color;
    }

    let sphere = closest_sphere.unwrap();

    let origin = ray.origin - sphere.position;
    let hit_point = origin + ray.direction * hit_distance;
    let normal = hit_point.normalize();

    let intensity = normal.dot(-light_direction).max(0.0); // == cos(angle)

    let sphere_color = sphere.albedo * intensity;
    return glam::Vec4::new(sphere_color.x, sphere_color.y, sphere_color.z, 1.0);
}

fn main() {
    tracing_subscriber::fmt::init();
    pollster::block_on(Application::<RayTracingCPU>::init());
}
