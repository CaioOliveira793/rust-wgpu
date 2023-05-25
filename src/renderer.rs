use wgpu::util::DeviceExt;

pub struct VertexBuffer(wgpu::Buffer);

impl VertexBuffer {
    pub fn init_immediate<'label>(
        device: &wgpu::Device,
        content: &[u8],
        label: Option<&'label str>,
    ) -> Self {
        let init_descriptor = wgpu::util::BufferInitDescriptor {
            label,
            contents: content,
            usage: wgpu::BufferUsages::VERTEX,
        };
        let buffer = device.create_buffer_init(&init_descriptor);
        Self(buffer)
    }

    pub fn init<'label>(device: &wgpu::Device, size: u64, label: Option<&'label str>) -> Self {
        let wgt_descriptor = wgpu::BufferDescriptor {
            label,
            size,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&wgt_descriptor);
        Self(buffer)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.0
    }
}

pub struct IndexBuffer {
    buffer: wgpu::Buffer,
    format: wgpu::IndexFormat,
}

macro_rules! index_buffer_init_immediate {
    ($device:expr, $content:expr, $label:expr, $ty:ident) => {{
        let init_descriptor = wgpu::util::BufferInitDescriptor {
            label: $label,
            contents: bytemuck::cast_slice($content),
            usage: wgpu::BufferUsages::INDEX,
        };
        let buffer = $device.create_buffer_init(&init_descriptor);
        IndexBuffer {
            buffer,
            format: wgpu::IndexFormat::$ty,
        }
    }};
}

impl IndexBuffer {
    pub fn init_immediate_u16<'label>(
        device: &wgpu::Device,
        content: &[u16],
        label: Option<&'label str>,
    ) -> Self {
        index_buffer_init_immediate!(device, content, label, Uint16)
    }

    pub fn init_immediate_u32<'label>(
        device: &wgpu::Device,
        content: &[u32],
        label: Option<&'label str>,
    ) -> Self {
        index_buffer_init_immediate!(device, content, label, Uint32)
    }

    pub fn init<'label>(
        device: &wgpu::Device,
        count: u32,
        format: wgpu::IndexFormat,
        label: Option<&'label str>,
    ) -> Self {
        let wgt_descriptor = wgpu::BufferDescriptor {
            label,
            size: Self::format_size(format) as u64 * count as u64,
            usage: wgpu::BufferUsages::INDEX,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&wgt_descriptor);
        Self { buffer, format }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn format(&self) -> wgpu::IndexFormat {
        self.format
    }

    pub fn count(&self) -> u32 {
        (self.buffer.size() / Self::format_size(self.format) as u64) as u32
    }

    /// Return the index byte size from the index format
    #[inline(always)]
    pub fn format_size(format: wgpu::IndexFormat) -> u8 {
        match format {
            wgpu::IndexFormat::Uint16 => 2,
            wgpu::IndexFormat::Uint32 => 4,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    texture_coord: [f32; 2],
}

impl Vertex {
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.0],
        texture_coord: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        texture_coord: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        texture_coord: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0],
        texture_coord: [0.0, 1.0],
    },
];

pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 3, 0, 2];
