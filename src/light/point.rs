use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniform {
    pub position: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    constant: f32,
    linear: f32,
    quadratic: f32,
    // padding for 16 bytes align
    _padding1: u32,
    _padding2: u32
}
impl Default for PointLightUniform {
    fn default() -> Self {
        Self { position: Default::default(), intensity: Default::default(), color: Default::default(), constant: 1.0, linear: 0.09, quadratic: 0.032, _padding1: 0, _padding2: 0 }
    }
}

impl PointLightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32) -> PointLightUniform {
        PointLightUniform {
            position,
            intensity,
            color,
            ..Default::default()
        }
    }
}

pub struct PointLight {
    uniform: PointLightUniform,
    buffer: Option<wgpu::Buffer>
}
impl PointLight {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32) -> PointLight {
        PointLight { 
            uniform: PointLightUniform::new(position, color, intensity), 
            buffer: None
        }
    }
    pub fn init(self, device: wgpu::Device) {
        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[self.uniform]),
                // use copy_dst to update light position
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );
        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }]
        });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: None, 
            layout: &light_bind_group_layout, 
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding()
            }] 
        });
    }
}