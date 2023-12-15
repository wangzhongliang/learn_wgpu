
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightUniform {
    pub direction: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    // padding for 16 bytes align
    _padding: u32,
}
impl Default for DirectionalLightUniform {
    fn default() -> Self {
        Self { direction: Default::default(), intensity: Default::default(), color: Default::default(), _padding: 0 }
    }
}

impl DirectionalLightUniform {
    pub fn new(direction: [f32; 3], color: [f32; 3], intensity: f32) -> DirectionalLightUniform {
        DirectionalLightUniform {
            direction,
            intensity,
            color,
            ..Default::default()
        }
    }
}