use std::{process::Termination, time::SystemTime};

use wgpu::SurfaceError;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

#[derive(Debug)]
pub struct AppState {
    previous_time: SystemTime,
    elapsed_time: f32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            previous_time: SystemTime::now(),
            elapsed_time: 0.0,
        }
    }

    pub fn update(&mut self) {
        let current_time = SystemTime::now();
        let elapsed_time = current_time
            .duration_since(self.previous_time)
            .expect("Elapsed time calculation requires a monotonic clock")
            .as_secs_f32()
            / 1000.0;
        self.previous_time = current_time;
        self.elapsed_time = elapsed_time;
    }
}

pub struct Application<L: Layer + 'static> {
    layer: Option<L>,
    screen: Screen,
    state: AppState,
}

impl<L: Layer + 'static> Application<L> {
    pub fn new(screen: Screen) -> Self {
        Self {
            screen,
            layer: None,
            state: AppState::new(),
        }
    }

    fn run(
        &mut self,
        event: Event<()>,
        _event_loop: &EventLoopWindowTarget<()>,
        control_flow: &mut ControlFlow,
    ) {
        control_flow.set_wait();

        if let Some(layer) = self.layer.as_mut() {
            layer.process_event(&event, &mut self.screen);
        }

        match event {
            Event::NewEvents(StartCause::Init) => {
                self.layer = Some(L::start(&mut self.screen, &self.state));
            }
            Event::WindowEvent {
                window_id,
                ref event,
            } => match event {
                WindowEvent::CloseRequested if self.screen.window().id() == window_id => {
                    control_flow.set_exit_with_code(0);
                    let app_res = self
                        .layer
                        .as_mut()
                        .unwrap()
                        .shutdown(&self.state, &mut self.screen);
                    if let Some(_) = app_res.err() {
                        control_flow.set_exit_with_code(1);
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    self.screen.resize(*physical_size);
                    self.layer.as_mut().unwrap().resize(
                        *physical_size,
                        &self.state,
                        &mut self.screen,
                    );
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.screen.resize(**new_inner_size);
                    self.layer.as_mut().unwrap().resize(
                        **new_inner_size,
                        &self.state,
                        &mut self.screen,
                    );
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                self.state.update();
                self.screen.window().request_redraw();
            }
            Event::RedrawRequested(window_id) if self.screen.window().id() == window_id => {
                self.layer
                    .as_mut()
                    .unwrap()
                    .update(&self.state, &mut self.screen);

                match self
                    .layer
                    .as_mut()
                    .unwrap()
                    .render(&self.state, &mut self.screen)
                {
                    Ok(_) => {}
                    Err(SurfaceError::Lost) => self.screen.resize_to_current(),
                    Err(SurfaceError::OutOfMemory) => control_flow.set_exit_with_code(137),
                    Err(e) => tracing::error!("{:?}", e),
                }
            }
            _ => {}
        }
    }

    pub async fn init() {
        let event_loop = EventLoop::new();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let screen = Screen::new(&event_loop, &instance).await;
        let mut application = Self::new(screen);
        event_loop.run(move |event, event_loop, control_flow| {
            application.run(event, event_loop, control_flow);
        });
    }
}

pub struct Screen {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    window: Window,
}

impl Screen {
    pub async fn new(event_loop: &EventLoopWindowTarget<()>, instance: &wgpu::Instance) -> Self {
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        // SAFETY:
        // The surface needs to live as long as the window that created it.
        // Screen owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: adapter.features(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        Self {
            window,
            surface,
            device,
            queue,
            config,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Resize the screen to new window size.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Resize the screen to current window inner size.
    pub fn resize_to_current(&mut self) {
        self.resize(self.window.inner_size());
    }
}

pub trait Layer: Sized {
    type LayerErr: Termination + 'static;

    fn start(screen: &mut Screen, app: &AppState) -> Self;
    fn process_event(&mut self, event: &Event<()>, screen: &mut Screen);
    fn resize(&mut self, new_size: PhysicalSize<u32>, app: &AppState, screen: &mut Screen);
    fn update(&mut self, app: &AppState, screen: &mut Screen);
    fn render(&mut self, app: &AppState, screen: &mut Screen) -> Result<(), SurfaceError>;
    fn shutdown(&mut self, app: &AppState, screen: &mut Screen) -> Result<(), Self::LayerErr>;
}
