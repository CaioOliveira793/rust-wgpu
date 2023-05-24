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
