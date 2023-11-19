use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniform {
    pub position: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    // padding for 16 bytes align
    _padding: u32
}

impl PointLightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32) -> PointLightUniform {
        PointLightUniform {
            position,
            intensity,
            color,
            _padding: 0,
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